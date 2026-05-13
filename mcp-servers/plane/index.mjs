#!/usr/bin/env node
// genasis plane MCP server (v0.6.0 beta).
//
// stdio MCP server exposing the Plane REST API as the same surface agents
// use in trial flavor — `mcp__plane__create_issue`, `transition_issue`,
// `list_issues`, `list_states`. Overlay text is identical between trial and
// real flavor; this server absorbs Plane-specific quirks (state UUIDs, page
// IDs, idempotency by name).
//
// Env (set by the daemon via build_mcp_config when flavor=real):
//   PLANE_URL              e.g. https://plane.example.com
//   PLANE_API_KEY          workspace-scoped API key (header X-API-Key)
//   PLANE_WORKSPACE_SLUG   e.g. acme
//   PLANE_PROJECT_ID       project UUID
//   PLANE_USER_ID_<ROLE>   per-role user UUID (assignee mapping). Optional —
//                          unset roles get an unassigned issue.
//
// State mapping: overlay calls send "todo"/"inprogress"/"inreview"/"done"
// (the trial-app SQLite enum). Plane stores state UUIDs and a `group`
// (backlog|unstarted|started|completed|cancelled). On first use we GET
// /states/ and build alias → UUID map.

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";

const BASE = (process.env.PLANE_URL || "").replace(/\/+$/, "");
const KEY = process.env.PLANE_API_KEY || "";
const WS = process.env.PLANE_WORKSPACE_SLUG || "";
const PROJ = process.env.PLANE_PROJECT_ID || "";

if (!BASE || !KEY || !WS || !PROJ) {
  console.error(
    "[mcp-plane] missing required env: PLANE_URL, PLANE_API_KEY, PLANE_WORKSPACE_SLUG, PLANE_PROJECT_ID",
  );
  process.exit(1);
}

const HEADERS = {
  "X-API-Key": KEY,
  "Content-Type": "application/json",
  Accept: "application/json",
};

async function planeFetch(path, init = {}) {
  const url = `${BASE}/api/v1${path}`;
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
      `plane ${init.method || "GET"} ${path} → ${res.status}: ${text.slice(0, 200)}`,
    );
  }
  return body;
}

let stateCache = null;
async function loadStates() {
  if (stateCache) return stateCache;
  const list = await planeFetch(`/workspaces/${WS}/projects/${PROJ}/states/`);
  const states = Array.isArray(list?.results)
    ? list.results
    : Array.isArray(list)
      ? list
      : [];
  const byName = {};
  const byGroup = {};
  for (const s of states) {
    const n = (s.name || "").toLowerCase().replace(/\s+/g, "");
    if (n) byName[n] = s.id;
    const g = (s.group || "").toLowerCase();
    if (g && !byGroup[g]) byGroup[g] = s.id;
  }
  // friendly aliases — match by name first, then by group semantics.
  const alias = {
    todo: byName["todo"] || byGroup["backlog"] || byGroup["unstarted"],
    inprogress: byName["inprogress"] || byGroup["started"],
    inreview: byName["inreview"] || byGroup["started"],
    done: byName["done"] || byGroup["completed"],
  };
  stateCache = { byName, byGroup, alias, states };
  return stateCache;
}

async function resolveStateUuid(state) {
  const cache = await loadStates();
  const s = (state || "").toLowerCase().replace(/\s+/g, "");
  const uuid = cache.alias[s] || cache.byName[s];
  if (!uuid) {
    const available = Object.keys(cache.byName).concat(Object.keys(cache.alias));
    throw new Error(
      `cannot resolve state '${state}' — known: ${available.join(", ")}`,
    );
  }
  return uuid;
}

function resolveAssigneeUuids(roleName) {
  if (!roleName) return [];
  const env = process.env[`PLANE_USER_ID_${roleName.toUpperCase()}`];
  if (env && env.trim().length > 0) return [env.trim()];
  return [];
}

async function findIssueByTitle(title) {
  const list = await planeFetch(`/workspaces/${WS}/projects/${PROJ}/issues/`);
  const issues = Array.isArray(list?.results)
    ? list.results
    : Array.isArray(list)
      ? list
      : [];
  return (
    issues.find((i) => (i.name || "").trim() === (title || "").trim()) || null
  );
}

const TOOLS = [
  {
    name: "create_issue",
    description:
      "Create a Plane issue. Idempotent on name — re-call updates state/assignee instead of creating a duplicate.",
    inputSchema: {
      type: "object",
      properties: {
        title: { type: "string" },
        assignee: {
          type: "string",
          description:
            "Role name (e.g. 'frontend', 'designer'). Mapped to PLANE_USER_ID_<ROLE> env. null/empty allowed.",
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
    description:
      "Move a Plane issue to a different state. Title-keyed (idempotent on no-op).",
    inputSchema: {
      type: "object",
      properties: {
        title: { type: "string" },
        state: {
          type: "string",
          enum: ["todo", "inprogress", "inreview", "done"],
        },
      },
      required: ["title", "state"],
    },
  },
  {
    name: "list_issues",
    description: "List all issues in PLANE_PROJECT_ID.",
    inputSchema: { type: "object", properties: {} },
  },
  {
    name: "list_states",
    description: "List the kanban states defined on the project.",
    inputSchema: { type: "object", properties: {} },
  },
];

const server = new Server(
  { name: "genasis-plane", version: "0.1.0" },
  { capabilities: { tools: {} } },
);

server.setRequestHandler(ListToolsRequestSchema, async () => ({ tools: TOOLS }));

server.setRequestHandler(CallToolRequestSchema, async (req) => {
  const { name, arguments: args = {} } = req.params;
  try {
    let result;
    switch (name) {
      case "create_issue": {
        const existing = await findIssueByTitle(args.title);
        if (existing) {
          const patch = {};
          if (args.state) patch.state = await resolveStateUuid(args.state);
          const assignees = resolveAssigneeUuids(args.assignee);
          if (assignees.length) patch.assignees = assignees;
          if (Object.keys(patch).length === 0) {
            result = existing;
          } else {
            result = await planeFetch(
              `/workspaces/${WS}/projects/${PROJ}/issues/${existing.id}/`,
              { method: "PATCH", body: JSON.stringify(patch) },
            );
          }
        } else {
          const body = { name: args.title };
          if (args.state) body.state = await resolveStateUuid(args.state);
          const assignees = resolveAssigneeUuids(args.assignee);
          if (assignees.length) body.assignees = assignees;
          result = await planeFetch(
            `/workspaces/${WS}/projects/${PROJ}/issues/`,
            { method: "POST", body: JSON.stringify(body) },
          );
        }
        break;
      }
      case "transition_issue": {
        const issue = await findIssueByTitle(args.title);
        if (!issue) {
          throw new Error(
            `issue '${args.title}' not found in workspace=${WS} project=${PROJ}`,
          );
        }
        const stateUuid = await resolveStateUuid(args.state);
        result = await planeFetch(
          `/workspaces/${WS}/projects/${PROJ}/issues/${issue.id}/`,
          { method: "PATCH", body: JSON.stringify({ state: stateUuid }) },
        );
        break;
      }
      case "list_issues": {
        result = await planeFetch(
          `/workspaces/${WS}/projects/${PROJ}/issues/`,
        );
        break;
      }
      case "list_states": {
        result = await planeFetch(
          `/workspaces/${WS}/projects/${PROJ}/states/`,
        );
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
  `[mcp-plane] ready — base=${BASE} ws=${WS} project=${PROJ}`,
);
