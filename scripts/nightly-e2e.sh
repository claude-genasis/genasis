#!/usr/bin/env bash
# =============================================================================
# Genasis Nightly E2E — local pre-push gate (M20)
#
# Boots the full Plane + Mattermost docker-compose stack defined in
# `servers/docker-compose.yml`, waits for both stacks to become healthy,
# runs the workspace test suite in release mode, and probes the live
# stack via `genasis init --probe-only`. Tears the stack back down on
# exit.
#
# This script intentionally runs **locally**, not in GitHub Actions.
# GitHub free runners (7 GB RAM, 2 vCPU) cannot reliably host the
# full Plane stack (postgres + redis + minio + rabbitmq + plane-api +
# plane-web + plane-space + plane-admin + plane-live + caddy +
# Mattermost + mm-postgres). A local workstation handles it in ~10 min;
# GitHub typically spends ~30 min on health waits alone before timing
# out. Run this before pushing code that touches:
#   - crates/genasis-providers/   (real-server flavor logic)
#   - servers/docker-compose.yml  (server stack itself)
#   - crates/genasis-cli/         (init / attach lifecycle)
#
# Prerequisites:
#   - docker + docker-compose v2
#   - rustup with stable toolchain
#   - ~10 GB free disk for the docker images
#
# Usage:
#   scripts/nightly-e2e.sh             # full run
#   scripts/nightly-e2e.sh --skip-test # boot stack + probe only (~3 min)
#   scripts/nightly-e2e.sh --no-down   # leave stack running for inspection
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
SERVERS_DIR="$REPO_ROOT/servers"
PROJECT_DIR="$(mktemp -d -t genasis-nightly-XXXXXX)"

SKIP_TEST=0
NO_DOWN=0
for arg in "$@"; do
    case "$arg" in
        --skip-test) SKIP_TEST=1 ;;
        --no-down)   NO_DOWN=1 ;;
        --help|-h)   sed -n '3,30p' "$0"; exit 0 ;;
        *) echo "unknown flag: $arg (try --help)"; exit 2 ;;
    esac
done

# ── Colour helpers ──────────────────────────────────────────────────
if [[ -t 1 ]]; then
    GREEN=$'\033[32m'; RED=$'\033[31m'; YELLOW=$'\033[33m'; CYAN=$'\033[36m'; RESET=$'\033[0m'
else
    GREEN=""; RED=""; YELLOW=""; CYAN=""; RESET=""
fi
say()  { printf "%s[nightly]%s %s\n" "$CYAN" "$RESET" "$*"; }
ok()   { printf "%s[ ok ]%s %s\n" "$GREEN" "$RESET" "$*"; }
warn() { printf "%s[warn]%s %s\n" "$YELLOW" "$RESET" "$*"; }
fail() { printf "%s[FAIL]%s %s\n" "$RED" "$RESET" "$*"; exit 1; }

# ── Pre-flight ──────────────────────────────────────────────────────
command -v docker >/dev/null 2>&1 || fail "docker not found in PATH"
docker compose version >/dev/null 2>&1 || fail "'docker compose' v2 plugin not found"
command -v cargo >/dev/null 2>&1 || fail "cargo not found — install via rustup"
[[ -d "$SERVERS_DIR" ]] || fail "servers/ directory missing — run from repo root"

cd "$REPO_ROOT"

# ── Tear-down trap ──────────────────────────────────────────────────
cleanup() {
    local rc=$?
    if [[ "$NO_DOWN" -eq 1 ]]; then
        warn "leaving docker stack running (--no-down). Stop with: (cd servers && docker compose down -v)"
    else
        say "tearing down servers/docker-compose…"
        ( cd "$SERVERS_DIR" && docker compose down -v --remove-orphans ) || \
            warn "docker compose down returned non-zero (probably never came up)"
    fi
    rm -rf "$PROJECT_DIR"
    if [[ $rc -eq 0 ]]; then
        ok "Nightly E2E PASSED"
    else
        fail "Nightly E2E FAILED (exit $rc) — see output above"
    fi
}
trap cleanup EXIT

# ── 1. Build genasis (release) ──────────────────────────────────────
say "building genasis CLI (release)…"
cargo build --workspace --release
[[ -x target/release/genasis ]] || fail "target/release/genasis not produced"
ok "genasis $(./target/release/genasis version 2>/dev/null || echo '?') built"

# ── 2. Seed servers/.env ────────────────────────────────────────────
say "seeding servers/.env…"
if [[ ! -f "$SERVERS_DIR/.env" ]]; then
    cp "$SERVERS_DIR/.env.example" "$SERVERS_DIR/.env"
fi
sed -i.bak 's#^PLANE_DOMAIN=.*#PLANE_DOMAIN=localhost#g' "$SERVERS_DIR/.env" || true
sed -i.bak 's#^MM_DOMAIN=.*#MM_DOMAIN=localhost#g' "$SERVERS_DIR/.env" || true
rm -f "$SERVERS_DIR/.env.bak"
ok "PLANE_DOMAIN / MM_DOMAIN pinned to localhost"

# ── 3. Boot stack ───────────────────────────────────────────────────
say "booting docker-compose stack…"
( cd "$SERVERS_DIR" && docker compose up -d ) || fail "docker compose up failed"

# ── 4. Wait for health ──────────────────────────────────────────────
say "waiting for Mattermost (max 15 min)…"
for i in $(seq 1 90); do
    if curl -fsS http://localhost:8065/api/v4/system/ping >/dev/null 2>&1; then
        ok "Mattermost ready after ${i}0s"
        break
    fi
    sleep 10
    [[ $i -eq 90 ]] && fail "Mattermost health timeout"
done

say "waiting for Plane (max 15 min)…"
for i in $(seq 1 90); do
    if curl -fsS http://localhost/ -o /dev/null 2>&1; then
        ok "Plane web ready after ${i}0s"
        break
    fi
    sleep 10
    [[ $i -eq 90 ]] && fail "Plane web health timeout"
done

# ── 5. Workspace test (release) ─────────────────────────────────────
if [[ "$SKIP_TEST" -eq 0 ]]; then
    say "cargo test --workspace --release --no-fail-fast…"
    cargo test --workspace --release --no-fail-fast
    ok "release test suite passed"
else
    warn "skipping cargo test (--skip-test)"
fi

# ── 6. genasis init --probe-only ────────────────────────────────────
say "smoke `genasis init --probe-only` against live stack…"
cat > "$PROJECT_DIR/genasis.toml" <<TOML
[project]
name = "nightly"
domain = "localhost"

[plane]
url = "http://localhost"
workspace_slug = "nightly"
flavor = "auto"

[mattermost]
url = "http://localhost:8065"
team_name = "nightly"
flavor = "auto"
TOML

# Probe-only doesn't validate token contents (it hits Plane /health
# and MM /system/ping). Stub values let genasis's env-var guard pass.
PLANE_API_KEY=nightly-stub-no-auth-required-for-probe \
MM_ADMIN_TOKEN=nightly-stub-no-auth-required-for-probe \
    ./target/release/genasis --non-interactive --yes init \
        --project "$PROJECT_DIR" --probe-only
ok "probe-only init succeeded"

# Cleanup runs in trap.
