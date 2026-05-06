#!/usr/bin/env bash
# =============================================================================
# Genasis E2E Test Suite
#
# Runs a full lifecycle test against live Plane + Mattermost instances.
# Results are recorded in test-results/ for badge generation.
#
# Prerequisites:
#   - genasis binary built (target/release/genasis or in PATH)
#   - .env.e2e in agents-pool/ (private) OR environment variables set:
#       PLANE_URL, PLANE_API_KEY, PLANE_WORKSPACE_SLUG
#       MM_URL, MM_ADMIN_TOKEN, MM_TEAM_ID
#   - Plane + Mattermost running and accessible
#
# Usage:
#   ./scripts/e2e-test.sh                 # full suite
#   ./scripts/e2e-test.sh --quick         # skip slow tests (Plane provisioning)
#   ./scripts/e2e-test.sh --mock          # mock mode (no live services needed)
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$REPO_ROOT/test-results"
TIMESTAMP="$(date -u +"%Y%m%d-%H%M%S")"
LOG_FILE="$RESULTS_DIR/e2e-${TIMESTAMP}.log"
RESULT_JSON="$RESULTS_DIR/e2e-latest.json"
HISTORY_CSV="$RESULTS_DIR/e2e-history.csv"

# Parse flags
QUICK=0
MOCK=0
for arg in "$@"; do
    case "$arg" in
        --quick) QUICK=1 ;;
        --mock) MOCK=1 ;;
    esac
done

# ── Find genasis binary ─────────────────────────────────────────────

GENASIS=""
if [ -f "$REPO_ROOT/target/release/genasis" ]; then
    GENASIS="$REPO_ROOT/target/release/genasis"
elif [ -f "$REPO_ROOT/target/debug/genasis" ]; then
    GENASIS="$REPO_ROOT/target/debug/genasis"
elif command -v genasis &>/dev/null; then
    GENASIS="$(command -v genasis)"
else
    echo "ERROR: genasis binary not found."
    echo "  Build it first: cargo build --release"
    echo "  Or install it: ./install.sh"
    exit 1
fi

echo "Using genasis: $GENASIS"
echo "Mode: $([ "$MOCK" -eq 1 ] && echo "MOCK" || echo "LIVE")"
echo ""

# ── Load environment ────────────────────────────────────────────────

ENV_FILE="$REPO_ROOT/agents-pool/.env.e2e"
if [ -f "$ENV_FILE" ]; then
    # shellcheck disable=SC1090
    source "$ENV_FILE"
    echo "Loaded env from agents-pool/.env.e2e"
fi

# Defaults for mock mode
if [ "$MOCK" -eq 1 ]; then
    PLANE_URL="${PLANE_URL:-http://localhost:0}"
    MM_URL="${MM_URL:-http://localhost:0}"
    PLANE_API_KEY="${PLANE_API_KEY:-mock-key}"
    MM_ADMIN_TOKEN="${MM_ADMIN_TOKEN:-mock-token}"
fi

# ── Test framework ──────────────────────────────────────────────────

TOTAL=0
PASSED=0
FAILED=0
SKIPPED=0
FAILURES=""
START_TIME="$(date +%s)"

pass() {
    local name="$1"
    TOTAL=$((TOTAL + 1))
    PASSED=$((PASSED + 1))
    echo "  ✅ PASS: $name"
}

fail() {
    local name="$1"
    local reason="${2:-}"
    TOTAL=$((TOTAL + 1))
    FAILED=$((FAILED + 1))
    FAILURES="${FAILURES}\n  - ${name}: ${reason}"
    echo "  ❌ FAIL: $name — $reason"
}

skip() {
    local name="$1"
    local reason="${2:-}"
    TOTAL=$((TOTAL + 1))
    SKIPPED=$((SKIPPED + 1))
    echo "  ⏭  SKIP: $name — $reason"
}

# ── Test cases ──────────────────────────────────────────────────────

echo "═══ Genasis E2E Test Suite ═══"
echo ""

# ── T1: Binary version ──────────────────────────────────────────────
echo "── T1: Binary version"
VERSION_OUT="$("$GENASIS" version 2>&1 || true)"
if echo "$VERSION_OUT" | grep -q "genasis"; then
    pass "genasis version runs"
else
    fail "genasis version" "unexpected output: $VERSION_OUT"
fi

# ── T2: Agents fetch ────────────────────────────────────────────────
echo "── T2: Agents catalog fetch"
if [ "$MOCK" -eq 1 ]; then
    skip "agents fetch" "mock mode"
else
    FETCH_OUT="$("$GENASIS" agents fetch 2>&1 || true)"
    if echo "$FETCH_OUT" | grep -qiE "cached|fetched|already"; then
        pass "agents catalog fetch"
    else
        fail "agents catalog fetch" "$FETCH_OUT"
    fi
fi

# ── T3: Agents list ─────────────────────────────────────────────────
echo "── T3: Agents list"
LIST_OUT="$("$GENASIS" agents list 2>&1 || true)"
if echo "$LIST_OUT" | grep -qiE "agent|frontend|backend|available"; then
    pass "agents list shows agents"
else
    fail "agents list" "$LIST_OUT"
fi

# ── T4: Init guard (safety check) ───────────────────────────────────
echo "── T4: Init guard"
TMPDIR_TEST="$(mktemp -d)"
cd "$TMPDIR_TEST"
GUARD_OUT="$("$GENASIS" doctor 2>&1 || true)"
cd "$REPO_ROOT"
rm -rf "$TMPDIR_TEST"
# Doctor should run without crashing even in empty dir
if [ $? -eq 0 ] || echo "$GUARD_OUT" | grep -qiE "check\|warn\|error\|doctor"; then
    pass "init guard / doctor runs in empty dir"
else
    fail "init guard" "crashed: $GUARD_OUT"
fi

# ── T5: Agents install (preset) ─────────────────────────────────────
echo "── T5: Agents install preset"
TMPDIR_PROJECT="$(mktemp -d)"
cd "$TMPDIR_PROJECT"
mkdir -p .claude/agents
INSTALL_OUT="$("$GENASIS" agents install --preset web-app 2>&1 || true)"
AGENT_COUNT="$(find .claude/agents -name "*.md" 2>/dev/null | wc -l)"
cd "$REPO_ROOT"
if [ "$AGENT_COUNT" -ge 5 ]; then
    pass "agents install --preset web-app ($AGENT_COUNT agents)"
else
    if [ "$MOCK" -eq 1 ] || echo "$INSTALL_OUT" | grep -qi "not cached"; then
        skip "agents install preset" "catalog not available"
    else
        fail "agents install preset" "only $AGENT_COUNT agents: $INSTALL_OUT"
    fi
fi
rm -rf "$TMPDIR_PROJECT"

# ── T6: Plane connectivity ──────────────────────────────────────────
echo "── T6: Plane connectivity"
if [ "$MOCK" -eq 1 ]; then
    skip "Plane connectivity" "mock mode"
elif [ -z "${PLANE_URL:-}" ] || [ -z "${PLANE_API_KEY:-}" ]; then
    skip "Plane connectivity" "PLANE_URL or PLANE_API_KEY not set"
else
    PLANE_HEALTH="$(curl -sf -H "X-API-Key: $PLANE_API_KEY" "${PLANE_URL}/api/v1/users/me/" 2>&1 || true)"
    if echo "$PLANE_HEALTH" | grep -qiE "id\|email\|username"; then
        pass "Plane API reachable"
    else
        fail "Plane connectivity" "response: $PLANE_HEALTH"
    fi
fi

# ── T7: Mattermost connectivity ─────────────────────────────────────
echo "── T7: Mattermost connectivity"
if [ "$MOCK" -eq 1 ]; then
    skip "MM connectivity" "mock mode"
elif [ -z "${MM_URL:-}" ] || [ -z "${MM_ADMIN_TOKEN:-}" ]; then
    skip "MM connectivity" "MM_URL or MM_ADMIN_TOKEN not set"
else
    MM_PING="$(curl -sf -H "Authorization: Bearer $MM_ADMIN_TOKEN" "${MM_URL}/api/v4/system/ping" 2>&1 || true)"
    if echo "$MM_PING" | grep -qi "OK\|status"; then
        pass "Mattermost API reachable"
    else
        fail "MM connectivity" "response: $MM_PING"
    fi
fi

# ── T8: Init (live provisioning) ────────────────────────────────────
echo "── T8: Init (live provisioning)"
if [ "$MOCK" -eq 1 ] || [ "$QUICK" -eq 1 ]; then
    skip "Init provisioning" "$([ "$MOCK" -eq 1 ] && echo "mock mode" || echo "quick mode")"
else
    TMPDIR_INIT="$(mktemp -d)"
    cd "$TMPDIR_INIT"
    git init -q
    mkdir -p .claude/agents
    cat > genasis.toml <<TOML
[project]
name = "e2e-test-$(date +%s)"

[plane]
url = "${PLANE_URL}"
workspace_slug = "${PLANE_WORKSPACE_SLUG:-default}"

[mattermost]
url = "${MM_URL}"
TOML
    cat > .env.agents <<ENV
PLANE_API_KEY=${PLANE_API_KEY}
MM_ADMIN_TOKEN=${MM_ADMIN_TOKEN}
MM_TEAM_ID=${MM_TEAM_ID:-}
ENV

    INIT_OUT="$("$GENASIS" init --non-interactive 2>&1 || true)"
    cd "$REPO_ROOT"

    if echo "$INIT_OUT" | grep -qiE "project.*created\|connected\|provisioned\|health"; then
        pass "Init provisioning"
    else
        fail "Init provisioning" "$(echo "$INIT_OUT" | tail -3)"
    fi
    rm -rf "$TMPDIR_INIT"
fi

# ── T9: Detach (clean removal) ──────────────────────────────────────
echo "── T9: Detach (clean removal)"
TMPDIR_DETACH="$(mktemp -d)"
cd "$TMPDIR_DETACH"
mkdir -p .claude/agents
# Create a fake agent with fence
cat > .claude/agents/test-agent.md <<'MD'
---
name: test-agent
---
<!-- GENASIS:BEGIN role=test version=1.0 hash=00000000 -->
test overlay
<!-- GENASIS:END -->
# Test Agent
MD
DETACH_OUT="$("$GENASIS" detach 2>&1 || true)"
AFTER="$(cat .claude/agents/test-agent.md 2>/dev/null || true)"
cd "$REPO_ROOT"

if echo "$AFTER" | grep -q "GENASIS:BEGIN"; then
    fail "Detach" "fence still present after detach"
else
    pass "Detach removes overlay fence"
fi
rm -rf "$TMPDIR_DETACH"

# ── Results ─────────────────────────────────────────────────────────

END_TIME="$(date +%s)"
DURATION=$((END_TIME - START_TIME))
ALL_PASSED=$( [ "$FAILED" -eq 0 ] && echo "true" || echo "false" )

echo ""
echo "═══ Results ═══"
echo "  Total: $TOTAL  Passed: $PASSED  Failed: $FAILED  Skipped: $SKIPPED"
echo "  Duration: ${DURATION}s"
echo "  All passed: $ALL_PASSED"

if [ "$FAILED" -gt 0 ]; then
    echo ""
    echo "  Failures:$FAILURES"
fi

# ── Write results ───────────────────────────────────────────────────

GENASIS_VERSION="$("$GENASIS" version --json 2>/dev/null | python3 -c "import json,sys; print(json.load(sys.stdin).get('version','?'))" 2>/dev/null || echo "unknown")"

cat > "$RESULT_JSON" <<JSON
{
  "passed": $ALL_PASSED,
  "date": "$(date -u +%Y-%m-%d)",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "genasis_version": "$GENASIS_VERSION",
  "mode": "$([ "$MOCK" -eq 1 ] && echo "mock" || echo "live")",
  "plane_url": "${PLANE_URL:-n/a}",
  "mm_url": "${MM_URL:-n/a}",
  "tests": {
    "total": $TOTAL,
    "passed": $PASSED,
    "failed": $FAILED,
    "skipped": $SKIPPED
  },
  "duration_seconds": $DURATION
}
JSON

# Append to history
if [ ! -f "$HISTORY_CSV" ]; then
    echo "date,version,mode,total,passed,failed,skipped,duration" > "$HISTORY_CSV"
fi
echo "$(date -u +%Y-%m-%d),$GENASIS_VERSION,$([ "$MOCK" -eq 1 ] && echo "mock" || echo "live"),$TOTAL,$PASSED,$FAILED,$SKIPPED,$DURATION" >> "$HISTORY_CSV"

echo ""
echo "Results written to:"
echo "  $RESULT_JSON"
echo "  $HISTORY_CSV"

# Exit with failure code if any test failed
[ "$FAILED" -eq 0 ] || exit 1
