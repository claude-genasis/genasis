#!/usr/bin/env node
// genasis trial-app MCP server.
//
// stdio MCP server exposing the trial-app REST API as structured tools
// the long-running claude session can call. Replaces the v0.5.x marker
// parsing in routing.rs::parse_pm_routing — agents now manipulate the
// kanban/chat/showcase directly through tool calls.
//
// Env (set by the daemon when spawning):
//   GENASIS_TRIAL_URL    e.g. https://mmplane-trial.realstory.blog
//   GENASIS_TEAM_TOKEN   32-char team token
//   GENASIS_PROJECT_SLUG e.g. v516-final
//   GENASIS_PROJECT_NAME e.g. v516 Final
//   GENASIS_CHANNEL_NAME e.g. scrum-v516-final
//
// Auth: the trial-app's "token IS capability" model — no admin secret.

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";

const BASE = (process.env.GENASIS_TRIAL_URL || "").replace(/\/+$/, "");
const TEAM_TOKEN = process.env.GENASIS_TEAM_TOKEN || "";
const PROJECT_SLUG = process.env.GENASIS_PROJECT_SLUG || "";
const PROJECT_NAME = process.env.GENASIS_PROJECT_NAME || PROJECT_SLUG;
const CHANNEL_NAME = process.env.GENASIS_CHANNEL_NAME || `scrum-${PROJECT_SLUG}`;

if (!BASE || !TEAM_TOKEN || !PROJECT_SLUG) {
  console.error(
    "[mcp-trial-app] missing required env: GENASIS_TRIAL_URL, GENASIS_TEAM_TOKEN, GENASIS_PROJECT_SLUG",
  );
  process.exit(1);
}

const HEADERS = {
  "X-Genasis-Team-Token": TEAM_TOKEN,
  "Content-Type": "application/json",
  Accept: "application/json",
};

async function trialFetch(path, init = {}) {
  const url = `${BASE}${path}`;
  const res = await fetch(url, { ...init, headers: { ...HEADERS, ...(init.headers || {}) } });
  const text = await res.text();
  let body;
  try {
    body = text ? JSON.parse(text) : null;
  } catch {
    body = text;
  }
  if (!res.ok) {
    throw new Error(`trial-app ${init.method || "GET"} ${path} → ${res.status}: ${text.slice(0, 200)}`);
  }
  return body;
}

// D-055: per-channel id cache. post_message 의 channel param 으로
// scrum/design/release 같은 분리 채널 지원.
const channelIdCache = new Map();
async function resolveChannelId(channelName) {
  const name = channelName || CHANNEL_NAME;
  if (channelIdCache.has(name)) return channelIdCache.get(name);
  const body = await trialFetch(
    `/api/mattermost/posts?channel_name=${encodeURIComponent(name)}`,
  );
  const posts = body?.posts || [];
  if (posts.length === 0) {
    throw new Error(
      `cannot resolve channel_id for ${name}: no posts seeded yet — run genasis init --trial first or pick a channel that has at least one welcome post`,
    );
  }
  const id = posts[0].channel_id;
  channelIdCache.set(name, id);
  return id;
}

const TOOLS = [
  {
    name: "post_message",
    description:
      "Post a chat message to one of the team's channels. Default channel = scrum. Use `root_id` to thread under an existing message (typically the human's request).",
    inputSchema: {
      type: "object",
      properties: {
        actor: {
          type: "string",
          description:
            "Persona posting the message (e.g. 'pm', 'frontend', 'designer', 'qa', 'devops', 'deploy', 'cleanup', 'status'). Avoid 'human'.",
        },
        text: { type: "string", description: "Message body (markdown ok)." },
        root_id: {
          type: "integer",
          description:
            "Optional sim_posts.id of the human's request to thread under. Omit for top-level message.",
        },
        channel: {
          type: "string",
          description:
            "Channel name (D-055). Default = team scrum channel. Other channels must already exist in sim_channels.",
        },
      },
      required: ["actor", "text"],
    },
  },
  {
    name: "list_posts",
    description: "List recent chat posts in the scrum channel.",
    inputSchema: { type: "object", properties: {} },
  },
  {
    name: "create_issue",
    description: "Create a kanban card. Idempotent on title.",
    inputSchema: {
      type: "object",
      properties: {
        title: { type: "string" },
        assignee: {
          type: "string",
          description: "Role name (e.g. 'frontend', 'designer'). null/empty allowed.",
        },
        state: {
          type: "string",
          enum: ["todo", "inprogress", "inreview", "done"],
          default: "todo",
        },
      },
      required: ["title"],
    },
  },
  {
    name: "transition_issue",
    description: "Move a kanban card to a new state. Title-keyed (idempotent).",
    inputSchema: {
      type: "object",
      properties: {
        title: { type: "string" },
        state: { type: "string", enum: ["todo", "inprogress", "inreview", "done"] },
      },
      required: ["title", "state"],
    },
  },
  {
    name: "list_issues",
    description: "List kanban cards. Use to inspect current state before transitioning.",
    inputSchema: { type: "object", properties: {} },
  },
  {
    name: "set_app_features",
    description:
      "Mark which showcase features are active (e.g. ['accent-red', 'dark-mode']). LRU-append — most recently set wins for visual priority.",
    inputSchema: {
      type: "object",
      properties: {
        features: { type: "array", items: { type: "string" } },
      },
      required: ["features"],
    },
  },
  {
    name: "set_app_kind",
    description:
      "Pick which showcase app the panel renders ('quiz' default; 'todo', 'pomodoro', etc.).",
    inputSchema: {
      type: "object",
      properties: {
        kind: {
          type: "string",
          enum: ["quiz", "todo", "pomodoro", "markdown", "counter", "habit"],
        },
      },
      required: ["kind"],
    },
  },
  {
    name: "announce_dev_server_url",
    description:
      "D-056: devops 가 dev server 띄운 후 호출. 사용자 ShowcasePanel 의 LocalDevServerOrFallback 가 이 URL 을 자동 prefill 해서 iframe 으로 표시. URL 은 localhost:<port> 또는 외부 접근 가능한 주소.",
    inputSchema: {
      type: "object",
      properties: {
        url: {
          type: "string",
          description: "Dev server URL (예: http://localhost:5173).",
        },
      },
      required: ["url"],
    },
  },
];

const server = new Server(
  { name: "genasis-trial-app", version: "0.1.0" },
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
        const body = {
          channel_id,
          actor: args.actor,
          message: args.text,
        };
        if (args.root_id) body.root_id = args.root_id;
        result = await trialFetch("/api/mattermost/posts", {
          method: "POST",
          body: JSON.stringify(body),
        });
        break;
      }
      case "list_posts": {
        const name = args?.channel || CHANNEL_NAME;
        result = await trialFetch(
          `/api/mattermost/posts?channel_name=${encodeURIComponent(name)}`,
        );
        break;
      }
      case "create_issue": {
        // ensureIssue via /api/trial/bootstrap with one demo_issues entry
        result = await trialFetch("/api/trial/bootstrap", {
          method: "POST",
          body: JSON.stringify({
            team_token: TEAM_TOKEN,
            project: { slug: PROJECT_SLUG, name: PROJECT_NAME },
            channels: [
              { key: "scrum", name: CHANNEL_NAME, display_name: `${PROJECT_NAME} — Scrum` },
            ],
            demo_issues: [
              {
                title: args.title,
                state: args.state || "todo",
                assignee: args.assignee ?? null,
              },
            ],
          }),
        });
        break;
      }
      case "transition_issue": {
        // Same bootstrap path — ensureIssue dedups on title and applies state.
        result = await trialFetch("/api/trial/bootstrap", {
          method: "POST",
          body: JSON.stringify({
            team_token: TEAM_TOKEN,
            project: { slug: PROJECT_SLUG, name: PROJECT_NAME },
            channels: [
              { key: "scrum", name: CHANNEL_NAME, display_name: `${PROJECT_NAME} — Scrum` },
            ],
            demo_issues: [
              { title: args.title, state: args.state, assignee: "agent" },
            ],
          }),
        });
        break;
      }
      case "list_issues": {
        result = await trialFetch(
          `/api/plane/issues?project_slug=${encodeURIComponent(PROJECT_SLUG)}`,
        );
        break;
      }
      case "set_app_features": {
        result = await trialFetch("/api/trial/team-app/status", {
          method: "POST",
          body: JSON.stringify({
            team_token: TEAM_TOKEN,
            status: "complete",
            project: { slug: PROJECT_SLUG, name: PROJECT_NAME },
            app_features: args.features,
          }),
        });
        break;
      }
      case "set_app_kind": {
        result = await trialFetch("/api/trial/team-app/status", {
          method: "POST",
          body: JSON.stringify({
            team_token: TEAM_TOKEN,
            status: "complete",
            project: { slug: PROJECT_SLUG, name: PROJECT_NAME },
            app_kind: args.kind,
          }),
        });
        break;
      }
      case "announce_dev_server_url": {
        // D-056: trial-app 의 sim_teams.dev_server_url 컬럼 갱신.
        // ShowcasePanel 의 LocalDevServerOrFallback 가 GET 으로 prefill.
        result = await trialFetch("/api/trial/team-app/dev-server", {
          method: "POST",
          body: JSON.stringify({
            team_token: TEAM_TOKEN,
            url: args.url,
          }),
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
console.error(`[mcp-trial-app] ready — base=${BASE} team=${TEAM_TOKEN.slice(0, 8)} project=${PROJECT_SLUG} channel=${CHANNEL_NAME}`);
