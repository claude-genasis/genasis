#!/usr/bin/env bash
# agents-pool/scripts/crawl.sh — fetch latest agent files from configured sources.
#
# Reads config.toml for source repos and paths, clones (shallow) into sources/,
# and copies target .md files for verification.

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
POOL_DIR="$(dirname "$SCRIPT_DIR")"

echo "=== agents-pool: crawl ==="
echo "Pool dir: $POOL_DIR"

# For each source in config.toml, shallow-clone and copy target paths.
# This is a skeleton — full TOML parsing would use a proper parser (e.g., dasel or yq).
# For now, manual per-source approach:

clone_source() {
    local name="$1" repo="$2" branch="$3"
    local dest="$POOL_DIR/sources/$name"
    echo "  Crawling $name ($repo @ $branch)..."
    rm -rf "$dest"
    git clone --depth=1 --branch="$branch" "$repo" "$dest" 2>/dev/null
    echo "  → $dest"
}

clone_source "ecc" "https://github.com/affaan-m/everything-claude-code.git" "main"
clone_source "wshobson" "https://github.com/wshobson/agents.git" "main"
clone_source "voltagent" "https://github.com/VoltAgent/awesome-claude-code-subagents.git" "main"
clone_source "dl-ezo" "https://github.com/dl-ezo/claude-code-sub-agents.git" "main"
clone_source "0xfurai" "https://github.com/0xfurai/claude-code-subagents.git" "main"

echo ""
echo "=== crawl complete. Run ./scripts/verify.sh next. ==="
