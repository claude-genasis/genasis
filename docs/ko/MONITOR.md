> English: [../MONITOR.md](../MONITOR.md)

# `genasis monitor`

> Status: M9 placeholder. Authoritative spec lives in [`../blueprint.md` §11](../blueprint.md).

Ratatui dashboard with widgets for:
- Sprint (current Plane Cycle, todo / in-progress / review / done counts)
- Agents (last activity time, current ticket per role)
- Tokens (RTK savings via `rtk gain --json`, MCP/cache hits, Anthropic cache hits)
- Network (Plane / MM / GitHub byte counters)
- Deploy (dev URL + prod URL LED, manifest-hash REFRESHED badge, visited flag)
- Log tail (`logs/agent-launches/*`)

Actions: `b`uild, `d`eploy, `r`ollback, `o`pen URL, `v` mark visited, `q`uit.

Configuration: `genasis.toml [deploy]` — see template `crates/genasis-templates/templates/genasis.toml.tera`.

## 트러블슈팅

| 증상 | 원인 / 해결 |
|---|---|
| 모니터에서 드래그·더블클릭 텍스트 선택이 안 됨 | 0.1.x+에서 수정됨. 이전 빌드는 마우스 이벤트를 소비하는 위젯이 없는데도 `EnableMouseCapture`를 켜서 native selection이 막혔음. 수정이 포함된 빌드로 업데이트하면 별도 플래그 없이 동작. |
| tmux 안에서 `genasis init` / `attach` wizard 텍스트 선택이 안 됨 | tmux의 `set -g mouse on`이 클릭을 가로챔. **Shift**를 누른 채 드래그하면 tmux를 우회해 호스트 터미널의 native selection을 사용. (하단 힌트 바에도 표시됨) |
| iTerm2 에선 선택이 되는데 `screen` 안에서 안 됨 | `screen`은 tmux 같은 mouse-pass 모드를 제공하지 않음. `screen` 없이 실행하거나 `screen`의 copy-mode (`Ctrl-a [`) 로 복사. |
