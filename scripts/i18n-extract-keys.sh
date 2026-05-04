#!/usr/bin/env bash
# Compare YAML key sets between en.yml and ko.yml.
#
#   --warn     (default) emit ::warning:: for missing keys, exit 0
#   --strict   emit ::error:: + exit 1 if either side has missing/surplus
#
# A key missing in ko.yml falls back to English at runtime — annoying but
# usable. A key present only in ko.yml is dead and always rejected.

set -euo pipefail

mode="${1:---warn}"
repo_root="$(cd "$(dirname "$0")/.." && pwd)"
en="$repo_root/crates/genasis-i18n/locales/en.yml"
ko="$repo_root/crates/genasis-i18n/locales/ko.yml"

# Use Python's PyYAML if present; otherwise grep-based heuristic that
# extracts dotted keys.
extract_keys() {
    local file="$1"
    if command -v python3 >/dev/null 2>&1; then
        python3 - "$file" <<'PY'
import sys, yaml
def walk(node, prefix=""):
    if isinstance(node, dict):
        for k, v in node.items():
            sub = f"{prefix}.{k}" if prefix else str(k)
            if isinstance(v, dict):
                yield from walk(v, sub)
            else:
                yield sub
try:
    with open(sys.argv[1]) as f:
        data = yaml.safe_load(f)
    for k in sorted(walk(data or {})):
        if not k.startswith("_meta"):
            print(k)
except yaml.YAMLError as e:
    print(f"::error::yaml parse failed in {sys.argv[1]}: {e}", file=sys.stderr)
    sys.exit(1)
except ImportError:
    sys.exit(2)
PY
    else
        # Fallback: line-based heuristic. Misses nested-only keys but
        # still flags obvious surplus/missing. CI runners reliably have
        # python3 available, so this branch is operator-machine fallback.
        awk -F: '/^[a-zA-Z_]/ {print $1}' "$file" | sort -u
    fi
}

en_keys="$(extract_keys "$en")"
ko_keys="$(extract_keys "$ko")"

missing_in_ko="$(comm -23 <(printf "%s\n" "$en_keys") <(printf "%s\n" "$ko_keys"))"
surplus_in_ko="$(comm -13 <(printf "%s\n" "$en_keys") <(printf "%s\n" "$ko_keys"))"

missing_count=$(printf "%s\n" "$missing_in_ko" | grep -c . || true)
surplus_count=$(printf "%s\n" "$surplus_in_ko" | grep -c . || true)

if [ "$missing_count" -gt 0 ]; then
    if [ "$mode" = "--strict" ]; then
        printf "::error::%d key(s) missing in ko.yml:\n%s\n" "$missing_count" "$missing_in_ko" >&2
        exit 1
    else
        printf "::warning::%d key(s) missing in ko.yml:\n%s\n" "$missing_count" "$missing_in_ko" >&2
    fi
fi

if [ "$surplus_count" -gt 0 ]; then
    # Surplus keys are always an error — they are dead in production
    # (English fallback never reaches them) and grow over time.
    printf "::error::%d surplus key(s) in ko.yml (no en.yml counterpart):\n%s\n" \
        "$surplus_count" "$surplus_in_ko" >&2
    exit 1
fi

if [ "$missing_count" -eq 0 ] && [ "$surplus_count" -eq 0 ]; then
    echo "i18n key parity OK ($(printf "%s\n" "$en_keys" | wc -l) keys)."
fi
