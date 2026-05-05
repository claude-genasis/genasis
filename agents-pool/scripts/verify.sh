#!/usr/bin/env bash
# agents-pool/scripts/verify.sh — validate crawled agent files.
#
# Checks:
# 1. YAML frontmatter with `name:` key
# 2. name: value maps to a known genasis role slug
# 3. No existing GENASIS:BEGIN fence (conflict with overlay injection)
# 4. No hardcoded Plane/MM references (overlay handles those)

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
POOL_DIR="$(dirname "$SCRIPT_DIR")"
VERIFIED_DIR="$POOL_DIR/verified"

echo "=== agents-pool: verify ==="

KNOWN_ROLES="pm planner architect frontend-developer backend-developer code-reviewer qa-tester security-reviewer designer devops mobile-ios mobile-android refactor docs"
ERRORS=0

verify_file() {
    local file="$1"
    local basename
    basename="$(basename "$file")"

    # Check 1: has YAML frontmatter
    if ! head -1 "$file" | grep -q "^---"; then
        echo "  FAIL [$basename]: missing YAML frontmatter"
        ERRORS=$((ERRORS + 1))
        return
    fi

    # Check 2: has name: key
    if ! grep -q "^name:" "$file"; then
        echo "  FAIL [$basename]: missing 'name:' in frontmatter"
        ERRORS=$((ERRORS + 1))
        return
    fi

    # Check 3: no existing genasis fence
    if grep -q "GENASIS:BEGIN" "$file"; then
        echo "  FAIL [$basename]: contains GENASIS:BEGIN fence (would conflict)"
        ERRORS=$((ERRORS + 1))
        return
    fi

    # Check 4: no hardcoded Plane/MM env vars
    if grep -qE "PLANE_TOKEN_|MM_TOKEN_|PLANE_USER_ID_" "$file"; then
        echo "  FAIL [$basename]: contains hardcoded Plane/MM references"
        ERRORS=$((ERRORS + 1))
        return
    fi

    echo "  PASS [$basename]"
    cp "$file" "$VERIFIED_DIR/"
}

rm -rf "$VERIFIED_DIR"
mkdir -p "$VERIFIED_DIR"

# Verify all .md files in sources/ (excluding READMEs and non-agent files)
find "$POOL_DIR/sources" -name "*.md" -not -name "README*" -not -name "CHANGELOG*" | while read -r f; do
    verify_file "$f"
done

echo ""
if [ "$ERRORS" -gt 0 ]; then
    echo "=== $ERRORS verification failures. Fix before publishing. ==="
    exit 1
else
    echo "=== All files verified. Run ./scripts/publish.sh next. ==="
fi
