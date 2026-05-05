#!/usr/bin/env bash
# agents-pool/scripts/publish.sh — build catalog tarball + upload to genasis Releases.
#
# Usage: ./scripts/publish.sh <version>
#   e.g. ./scripts/publish.sh 1.0.0
#
# This script:
# 1. Assembles build/ directory (base agents + genasis overlays/commands/skills/hooks)
# 2. Creates agents-v{version}.tar.gz
# 3. Uploads to genasis GitHub Releases via `gh release create`
#
# Prerequisites:
# - `gh` CLI authenticated with genasis repo write access
# - verified/ populated by verify.sh

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
POOL_DIR="$(dirname "$SCRIPT_DIR")"
GENASIS_DIR="$POOL_DIR/.."
BUILD_DIR="$POOL_DIR/build"

if [ $# -lt 1 ]; then
    echo "Usage: $0 <version>"
    echo "  e.g. $0 1.0.0"
    exit 1
fi

VERSION="$1"
TAG="agents-v${VERSION}"
TARBALL="agents-v${VERSION}.tar.gz"

echo "=== agents-pool: publish v${VERSION} ==="

# Sanity checks
if [ ! -d "$POOL_DIR/verified" ] || [ -z "$(find "$POOL_DIR/verified" -name '*.md' 2>/dev/null)" ]; then
    echo "ERROR: verified/ is empty. Run ./scripts/verify.sh first."
    exit 1
fi

if ! command -v gh &>/dev/null; then
    echo "ERROR: gh CLI not found. Install: https://cli.github.com/"
    exit 1
fi

# 1. Clean + assemble build directory
echo "  Assembling build directory..."
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR/base"
mkdir -p "$BUILD_DIR/overlays"
mkdir -p "$BUILD_DIR/commands"
mkdir -p "$BUILD_DIR/skills"
mkdir -p "$BUILD_DIR/hooks"

# Copy selected base agents from verified/ (config.toml role filtering)
# For now: copy all verified .md files. TODO: role-based selection per config.toml.
cp "$POOL_DIR/verified/"*.md "$BUILD_DIR/base/" 2>/dev/null || true

# Copy genasis-owned overlays, commands, skills, hooks
if [ -d "$GENASIS_DIR/agents/overlays" ]; then
    cp -r "$GENASIS_DIR/agents/overlays/"* "$BUILD_DIR/overlays/" 2>/dev/null || true
fi
if [ -d "$GENASIS_DIR/agents/commands" ]; then
    cp "$GENASIS_DIR/agents/commands/"* "$BUILD_DIR/commands/" 2>/dev/null || true
fi
if [ -d "$GENASIS_DIR/agents/skills" ]; then
    cp -r "$GENASIS_DIR/agents/skills/"* "$BUILD_DIR/skills/" 2>/dev/null || true
fi
if [ -d "$GENASIS_DIR/agents/hooks" ]; then
    cp "$GENASIS_DIR/agents/hooks/"* "$BUILD_DIR/hooks/" 2>/dev/null || true
fi

# Copy and update manifest
if [ -f "$GENASIS_DIR/agents/manifest.json" ]; then
    cp "$GENASIS_DIR/agents/manifest.json" "$BUILD_DIR/manifest.json"
fi

# Update version in manifest
if command -v jq &>/dev/null && [ -f "$BUILD_DIR/manifest.json" ]; then
    jq --arg v "$VERSION" '.version = $v' "$BUILD_DIR/manifest.json" > "$BUILD_DIR/manifest.tmp"
    mv "$BUILD_DIR/manifest.tmp" "$BUILD_DIR/manifest.json"
fi

# Count
BASE_COUNT=$(find "$BUILD_DIR/base" -name "*.md" | wc -l)
OVERLAY_COUNT=$(find "$BUILD_DIR/overlays" -name "*.tera" | wc -l)
echo "  Base agents: $BASE_COUNT"
echo "  Overlays: $OVERLAY_COUNT"

# 2. Create individual assets list + tarball (both uploaded)
echo "  Creating individual agent assets..."
ASSETS_DIR="$POOL_DIR/release-assets"
rm -rf "$ASSETS_DIR"
mkdir -p "$ASSETS_DIR"

# Each base agent as an individual downloadable .md file
for f in "$BUILD_DIR/base/"*.md; do
    [ -f "$f" ] || continue
    cp "$f" "$ASSETS_DIR/"
done

# Also create the full tarball (for `--preset` bulk install)
echo "  Creating $TARBALL (full catalog)..."
(cd "$BUILD_DIR" && tar -czf "$ASSETS_DIR/$TARBALL" .)
sha256sum "$ASSETS_DIR/$TARBALL" > "$ASSETS_DIR/${TARBALL}.sha256"

INDIVIDUAL_COUNT=$(find "$ASSETS_DIR" -name "*.md" | wc -l)
echo "  Individual agents: $INDIVIDUAL_COUNT"
echo "  Full tarball: $TARBALL"

# 3. Upload to genasis Releases (individual + tarball)
echo "  Uploading to genasis Releases as $TAG..."
cd "$GENASIS_DIR"

# Build release notes
NOTES="Agents Catalog v${VERSION}

## Install individual agents
\`\`\`bash
genasis agents install frontend-developer
genasis agents install --preset web-app   # 9 core agents
\`\`\`

## Browse available agents
\`\`\`bash
genasis agents           # interactive TUI
/install-agent mobile    # Claude Code slash command
\`\`\`

## Stats
- Individual agents: ${INDIVIDUAL_COUNT}
- Overlay patches: ${OVERLAY_COUNT}
- Full tarball for preset install included"

# Create release with all assets
gh release create "$TAG" \
    --title "Agents Catalog v${VERSION}" \
    --notes "$NOTES" \
    "$ASSETS_DIR"/* \
    2>/dev/null || {
    # If release already exists, upload/clobber
    gh release upload "$TAG" "$ASSETS_DIR"/* --clobber
}

echo ""
echo "=== publish complete ==="
echo "  Release: $TAG"
echo "  Users: genasis agents update"
