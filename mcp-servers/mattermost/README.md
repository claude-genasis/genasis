# mattermost MCP server (P5 — stub, beta 사이클에 채움)

genasis v0.6.0 beta 에서 채울 위치. real Mattermost flavor 일 때
`agents/overlays/*/`의 PM/agent prompt 가 `mcp__mattermost__*` tool 을
호출하면 이 server 가 Mattermost REST API 로 위임.

## 노출 예정 tools

| Tool | Mattermost API |
|---|---|
| `post_message(channel_id, root_id?, actor, text)` | `POST /api/v4/posts` (root_id = thread parent) |
| `list_posts(channel_id, since?)` | `GET /api/v4/channels/{id}/posts` |
| `list_channels(team_id)` | `GET /api/v4/teams/{id}/channels` |
| `update_post(id, message)` | `PUT /api/v4/posts/{id}/patch` |

## Env

- `MM_URL` (예: `https://mm.example.com`)
- `MM_ADMIN_TOKEN` (sysadmin PAT — agent 의 bot account 로 게시)
- `MM_TEAM_ID`
- `MM_DEFAULT_CHANNEL_ID` (scrum 채널)

## 구현 시 참고

`mcp-servers/trial-app/index.mjs` 와 같은 구조. fetch wrapper + tool schema +
JSON-RPC stdio. tool 인터페이스가 trial-app 과 동일하므로 overlay 본문 변경 없이 swap.
