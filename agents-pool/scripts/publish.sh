#!/usr/bin/env bash
# agents-pool/scripts/publish.sh — copy verified agents to genasis/agents/base/
#
# After verify.sh passes, this script copies the curated .md files
# to the genasis public repo's agents/base/ directory.

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
POOL_DIR="$(dirname "$SCRIPT_DIR")"
GENASIS_AGENTS="$POOL_DIR/../agents/base"

echo "=== agents-pool: publish ==="
echo "Source: $POOL_DIR/verified/"
echo "Target: $GENASIS_AGENTS/"

if [ ! -d "$POOL_DIR/verified" ] || [ -z "$(ls -A "$POOL_DIR/verified/" 2>/dev/null)" ]; then
    echo "ERROR: verified/ is empty. Run ./scripts/verify.sh first."
    exit 1
fi

# Copy verified files to genasis agents/base/
cp "$POOL_DIR/verified/"*.md "$GENASIS_AGENTS/" 2>/dev/null || true

echo ""
echo "Published $(ls "$POOL_DIR/verified/"*.md 2>/dev/null | wc -l) agent files."
echo ""
echo "Next steps:"
echo "  cd $(dirname "$POOL_DIR")"
echo "  git add agents/"
echo "  git commit -m 'feat(agents): update catalog from pool'"
echo "  git tag agents-vX.Y.Z"
echo "  git push --tags"
echo ""
echo "=== publish complete ==="
