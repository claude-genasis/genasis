# plane MCP server (P5 — stub, beta 사이클에 채움)

genasis v0.6.0 beta 에서 채울 위치. real Plane flavor 일 때
`agents/overlays/*/`의 PM/agent prompt 가 `mcp__plane__*` tool 을
호출하면 이 server 가 Plane REST API 로 위임.

## 노출 예정 tools

| Tool | Plane API |
|---|---|
| `create_issue(workspace_slug, project_id, title, assignees)` | `POST /api/v1/workspaces/{ws}/projects/{p}/issues/` |
| `transition_issue(id, state_uuid)` | `PATCH /api/v1/workspaces/{ws}/projects/{p}/issues/{id}/` |
| `list_issues(workspace_slug, project_id, state?)` | `GET /api/v1/workspaces/{ws}/projects/{p}/issues/` |
| `list_states(workspace_slug, project_id)` | `GET /api/v1/workspaces/{ws}/projects/{p}/states/` |

## Env

- `PLANE_URL`
- `PLANE_API_KEY` (또는 PAT — 각 agent 별 PAT 우선, fallback admin)
- `PLANE_WORKSPACE_SLUG`
- `PLANE_PROJECT_ID`
- `PLANE_USER_ID_<ROLE>` — 각 role 의 user UUID (assignees 매핑)

## 구현 시 참고

trial-app MCP 와 같은 구조 + state UUID 매핑 (Plane 은 state 가 string 이 아닌 UUID).
tool 인터페이스 (예 transition_issue 의 `state` param 이 trial 에선 "done" 문자열,
real 에선 UUID — server 가 내부 매핑 흡수). overlay 본문 변경 없이 swap.
