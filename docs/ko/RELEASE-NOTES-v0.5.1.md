# 릴리즈 노트 — v0.5.1

> English: [`../RELEASE-NOTES-v0.5.1.md`](../RELEASE-NOTES-v0.5.1.md)

**릴리즈일**: 2026-05-10

v0.5.0 위에 얹는 작은 UX 패치. 핵심 수정은 `genasis monitor` 안에서의
native 텍스트 선택(드래그·더블클릭·트리플클릭) 복구입니다. v0.5.0 은
crossterm 마우스 캡처를 켜두었지만 정작 마우스 이벤트를 소비하는 위젯은
없어서, 사용자가 대시보드를 읽을 수는 있어도 어떤 텍스트도 복사할 수 없는
상태였습니다.

---

## 수정된 항목

### `genasis monitor` — native 텍스트 선택 복구

v0.5.0 모니터는 시작 시 `EnableMouseCapture`를 호출했습니다. 그러나
실제로 마우스 이벤트를 처리하는 위젯이 없었기 때문에, 효과는 호스트
터미널의 selection 메커니즘을 가로막는 것뿐이었습니다.

v0.5.1 은 [`crates/genasis-monitor/src/app.rs`](../../crates/genasis-monitor/src/app.rs)에서
`EnableMouseCapture` 와 짝꿍인 `DisableMouseCapture` 를 모두 제거했습니다.
드래그 선택, 더블클릭 단어 선택, 트리플클릭 줄 선택이 native selection
지원 터미널(iTerm2, kitty, alacritty, gnome-terminal, Windows Terminal 등)
에서 정상 동작합니다. 추후 위젯이 클릭을 필요로 하면 `EnableMouseCapture`
를 전역으로 켜지 말고 opt-in 플래그 뒤로 게이팅하라는 지침이 코멘트로
남아있습니다.

### TUI wizard — tmux 안에서 Shift+drag 안내

wizard 는 이미 마우스 캡처를 켜지 않으므로 native selection 이 동작합니다.
다만 tmux 안에서 `set -g mouse on` 인 경우 tmux 가 클릭을 가로채기 때문에
호스트 터미널이 selection 을 못 봅니다. wizard 하단 힌트 바 끝에 dim 스타일로
`Shift+drag select text (in tmux)` 안내를 추가해 표준 우회법을 노출시켰습니다
([`crates/genasis-tui/src/wizard/widgets/key_hints.rs`](../../crates/genasis-tui/src/wizard/widgets/key_hints.rs)).

### `docs/MONITOR.md` — 트러블슈팅 표

[`MONITOR.md`](../MONITOR.md) 와 한국어 미러에 3행 표 추가:
- v0.5.0 selection 이슈 (수정됨) 안내
- tmux 안 wizard Shift+drag 우회
- `screen` 사용자는 copy-mode (`Ctrl-a [`) 사용

---

## 업그레이드

순수 패치 릴리즈. 설정 / 스키마 / 트레이트 표면 변화 없음. 새 바이너리만
받으면 끝:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/claude-genasis/genasis/main/install.sh | sh
```

`genasis upgrade --fence-version 1.0` 은 예상대로 no-op 유지.

---

## Breaking changes

없음.

---

## Coverage and tests

- 워크스페이스 10개 crate, **245 테스트** 모두 green
- 테스트 표면 추가 없음 — 회귀가 UX 한정이라 iTerm2 + alacritty + gnome-terminal
  수동 selection 으로 검증

---

## 감사의 말

v0.5.0 dogfooding 중 사용자가 발견·보고. 빠른 피드백에 감사드립니다.

전체 upstream / contributor 감사 명단은 [`docs/CREDITS.md`](../CREDITS.md) 참조.

---

## Commit list

```
fix(monitor): remove unused EnableMouseCapture so terminal selection works
docs(monitor): wizard Shift+drag hint + troubleshooting table (EN+KO)
chore(release): bump workspace version 0.5.0 → 0.5.1
```
