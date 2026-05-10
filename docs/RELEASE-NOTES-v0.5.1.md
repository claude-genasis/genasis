# Release Notes — v0.5.1

> 한국어: [`ko/RELEASE-NOTES-v0.5.1.md`](ko/RELEASE-NOTES-v0.5.1.md)

**Released**: 2026-05-10

A small UX patch on top of v0.5.0. The headline fix restores native
text selection (drag, double-click, triple-click) inside `genasis
monitor`, which the v0.5.0 build silently disabled by enabling
crossterm mouse capture without any widget that consumed mouse events.

---

## What's fixed

### `genasis monitor` — native text selection restored

The v0.5.0 monitor called `EnableMouseCapture` at startup. Because no
widget actually consumed mouse events, the only effect was that the
host terminal stopped routing clicks/drags to its own selection
machinery — users could read the dashboard but couldn't copy any of
it.

v0.5.1 removes both `EnableMouseCapture` and the matching
`DisableMouseCapture` in [`crates/genasis-monitor/src/app.rs`](../crates/genasis-monitor/src/app.rs).
Drag-select, double-click-word, and triple-click-line now work in
every terminal that supports them natively (iTerm2, kitty, alacritty,
gnome-terminal, Windows Terminal, etc.). When a future widget needs
click handling, the comment in `app.rs` instructs maintainers to gate
`EnableMouseCapture` behind an opt-in flag rather than turn it on
globally.

### TUI wizard — Shift+drag hint in tmux

The wizard already left mouse capture off, so selection works
natively. When run inside tmux with `set -g mouse on`, however, tmux
intercepts clicks before the host terminal sees them. The wizard's
bottom hint bar now ends with a dim `Shift+drag select text (in tmux)`
reminder so users discover the standard workaround
([`crates/genasis-tui/src/wizard/widgets/key_hints.rs`](../crates/genasis-tui/src/wizard/widgets/key_hints.rs)).

### `docs/MONITOR.md` — troubleshooting table

A new three-row table in [MONITOR.md](MONITOR.md) and its Korean
mirror documents:
- the v0.5.0 selection issue (now fixed),
- the tmux Shift+drag workaround for the wizard,
- guidance for `screen` users (use `Ctrl-a [` copy-mode).

---

## Upgrade

Pure patch. No config, schema, or trait surface changed. Drop in the
new binary:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/claude-genasis/genasis/main/install.sh | sh
```

`genasis upgrade --fence-version 1.0` remains a no-op as expected.

---

## Breaking changes

None.

---

## Coverage and tests

- **245 tests** across 10 workspace crates, all green
- No new test surface — the regression was UX-only and verified by
  manual selection in iTerm2 + alacritty + gnome-terminal

---

## Acknowledgements

Reported by the user during a v0.5.0 dogfooding session — thanks for
the catch.

See [`docs/CREDITS.md`](CREDITS.md) for the full upstream and
contributor acknowledgements.

---

## Commit list

```
fix(monitor): remove unused EnableMouseCapture so terminal selection works
docs(monitor): wizard Shift+drag hint + troubleshooting table (EN+KO)
chore(release): bump workspace version 0.5.0 → 0.5.1
```
