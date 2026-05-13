#!/usr/bin/env node
// genasis mattermost MCP server (v0.6.0 beta).
//
// stdio MCP server exposing the Mattermost REST API as the same surface
// agents use in trial flavor — agents call `mcp__mattermost__post_message`
// and the long-running team session routes it here, which delegates to
// `POST /api/v4/posts` on the real Mattermost instance.
//
// Env (set by the daemon via build_mcp_config when flavor=real):
//   MM_URL                 e.g. https://mm.example.com
//   MM_ADMIN_TOKEN         sysadmin / bot PAT (Bearer)
//   MM_TEAM_ID             team UUID this server is bound to
//   MM_DEFAULT_CHANNEL_ID  scrum channel UUID (used when `channel` arg absent)
//
// Channel resolution: tools accept a `channel` arg = channel NAME (no '#').
// The server resolves it to channel_id via /teams/{team}/channels/name/{n}
// and caches the result in-process.

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";

const BASE = (process.env.MM_URL || "").replace(/\/+$/, "");
const TOKEN = process.env.MM_ADMIN_TOKEN || "";
const TEAM_ID = process.env.MM_TEAM_ID || "";
const DEFAULT_CHANNEL_ID = process.env.MM_DEFAULT_CHANNEL_ID || "";

if (!BASE || !TOKEN || !TEAM_ID) {
  console.error(
    "[mcp-mattermost] missing required env: MM_URL, MM_ADMIN_TOKEN, MM_TEAM_ID",
  );
  process.exit(1);
}

const HEADERS = {
  Authorization: `Bearer ${TOKEN}`,
  "Content-Type": "application/json",
  Accept: "application/json",
};

async function mmFetch(path, init = {}) {
  const url = `${BASE}/api/v4${path}`;
  const res = await fetch(url, {
    ...init,
    headers: { ...HEADERS, ...(init.headers || {}) },
  });
  const text = await res.text();
  let body;
  try {
    body = text ? JSON.parse(text) : null;
  } catch {
    body = text;
  }
  if (!res.ok) {
    throw new Error(
      `mattermost ${init.method || "GET"} ${path} → ${res.status}: ${text.slice(0, 200)}`,
    );
  }
  return body;
}

const channelIdCache = new Map();
async function resolveChannelId(channelName) {
  if (!channelName) {
    if (!DEFAULT_CHANNEL_ID) {
      throw new Error(
        "no `channel` arg given and MM_DEFAULT_CHANNEL_ID env not set — pass channel:'scrum-<slug>' or set MM_DEFAULT_CHANNEL_ID at daemon start",
      );
    }
    return DEFAULT_CHANNEL_ID;
  }
  const normalized = channelName.toLowerCase().replace(/^#/, "");
  if (channelIdCache.has(normalized)) return channelIdCache.get(normalized);
  const channel = await mmFetch(
    `/teams/${TEAM_ID}/channels/name/${encodeURIComponent(normalized)}`,
  );
  const id = channel?.id;
  if (!id) {
    throw new Error(
      `channel '${channelName}' not found in team ${TEAM_ID}`,
    );
  }
  channelIdCache.set(normalized, id);
  return id;
}

const TOOLS = [
  {
    name: "post_message",
    description:
      "Post a chat message to a Mattermost channel. Default channel = MM_DEFAULT_CHANNEL_ID. Use `root_id` to thread under the human's request.",
    inputSchema: {
      type: "object",
      properties: {
        actor: {
          type: "string",
          description:
            "Persona label prefixed to the message body for visibility (Mattermost has no per-message bot identity beyond the token's user).",
        },
        text: { type: "string", description: "Message body (markdown ok)." },
        root_id: {
          type: "string",
          description: "Optional Mattermost post_id to thread under.",
        },
        channel: {
          type: "string",
          description:
            "Channel name (without `#`). Default = MM_DEFAULT_CHANNEL_ID.",
        },
      },
      required: ["text"],
    },
  },
  {
    name: "list_posts",
    description:
      "List recent posts in a Mattermost channel (default = MM_DEFAULT_CHANNEL_ID).",
    inputSchema: {
      type: "object",
      properties: {
        channel: { type: "string" },
        limit: { type: "integer", default: 50 },
      },
    },
  },
  {
    name: "list_channels",
    description: "List all channels visible to the bot in MM_TEAM_ID.",
    inputSchema: { type: "object", properties: {} },
  },
  {
    name: "update_post",
    description: "Edit a post the bot owns.",
    inputSchema: {
      type: "object",
      properties: {
        post_id: { type: "string" },
        text: { type: "string" },
      },
      required: ["post_id", "text"],
    },
  },
];

const server = new Server(
  { name: "genasis-mattermost", version: "0.1.0" },
  { capabilities: { tools: {} } },
);

server.setRequestHandler(ListToolsRequestSchema, async () => ({ tools: TOOLS }));

server.setRequestHandler(CallToolRequestSchema, async (req) => {
  const { name, arguments: args = {} } = req.params;
  try {
    let result;
    switch (name) {
      case "post_message": {
        const channel_id = await resolveChannelId(args.channel);
        let message = args.text || "";
        if (args.actor && args.actor.trim().length > 0) {
          message = `**[${args.actor}]** ${message}`;
        }
        const body = { channel_id, message };
        if (args.root_id) body.root_id = args.root_id;
        result = await mmFetch(`/posts`, {
          method: "POST",
          body: JSON.stringify(body),
        });
        break;
      }
      case "list_posts": {
        const channel_id = await resolveChannelId(args.channel);
        const limit = Math.max(1, Math.min(args.limit || 50, 200));
        result = await mmFetch(
          `/channels/${channel_id}/posts?per_page=${limit}`,
        );
        break;
      }
      case "list_channels": {
        result = await mmFetch(`/teams/${TEAM_ID}/channels`);
        break;
      }
      case "update_post": {
        result = await mmFetch(`/posts/${args.post_id}/patch`, {
          method: "PUT",
          body: JSON.stringify({ message: args.text }),
        });
        break;
      }
      default:
        throw new Error(`unknown tool: ${name}`);
    }
    return {
      content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
    };
  } catch (err) {
    return {
      content: [{ type: "text", text: `ERROR: ${err.message || err}` }],
      isError: true,
    };
  }
});

const transport = new StdioServerTransport();
await server.connect(transport);
console.error(
  `[mcp-mattermost] ready — base=${BASE} team=${TEAM_ID} default_channel=${DEFAULT_CHANNEL_ID || "(none)"}`,
);
