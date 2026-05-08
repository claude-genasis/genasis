#!/usr/bin/env bash
# scripts/debug-review/run.sh — minimal clustering pass over
# debug-history/patches/. Produces:
#   - debug-history/analysis/clusters.md       (file → occurrence count)
#   - debug-history/analysis/proposed-fixes.md (placeholder; the heavy
#                                              lifting lives in the
#                                              /debug-review skill)
#   - debug-history/analysis/processed.txt     (registry of patch IDs
#                                              already absorbed into a
#                                              cluster).
#
# Invariant: this script never mutates patches/*.patch.json — it only
# writes under debug-history/analysis/. ADR-012 §8.

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
PATCH_DIR="$ROOT/debug-history/patches"
ANALYSIS_DIR="$ROOT/debug-history/analysis"
mkdir -p "$ANALYSIS_DIR"

: > "$ANALYSIS_DIR/clusters.md"
: > "$ANALYSIS_DIR/proposed-fixes.md"

cat > "$ANALYSIS_DIR/clusters.md" <<HDR
# Clusters

Generated $(date -u +%FT%TZ).

| File | Occurrences | Distinct project hashes | Sample patch |
|---|---|---|---|
HDR

if compgen -G "$PATCH_DIR/*.patch.json" >/dev/null; then
  # Use jq if present; otherwise fall back to grep.
  if command -v jq >/dev/null 2>&1; then
    declare -A counts
    declare -A hashes
    declare -A samples
    for patch in "$PATCH_DIR"/*.patch.json; do
      ph=$(jq -r '.project_hash // "unknown"' "$patch")
      while read -r f; do
        [ -z "$f" ] && continue
        counts[$f]=$(( ${counts[$f]:-0} + 1 ))
        hashes[$f]="${hashes[$f]:-} $ph"
        samples[$f]=${samples[$f]:-$(basename "$patch")}
      done < <(jq -r '.entries[]?.file // empty' "$patch")
    done
    for f in "${!counts[@]}"; do
      uniq=$(echo "${hashes[$f]}" | tr ' ' '\n' | sort -u | grep -c .)
      printf '| %s | %d | %d | %s |\n' "$f" "${counts[$f]}" "$uniq" "${samples[$f]}" \
        >> "$ANALYSIS_DIR/clusters.md"
    done
  else
    echo "(jq missing; only patch count surfaced)" >> "$ANALYSIS_DIR/clusters.md"
    n=$(ls "$PATCH_DIR"/*.patch.json 2>/dev/null | wc -l)
    printf '| (all patches) | %d | ? | (jq required for breakdown) |\n' "$n" \
      >> "$ANALYSIS_DIR/clusters.md"
  fi
else
  printf '| (no patches yet) | 0 | 0 | — |\n' >> "$ANALYSIS_DIR/clusters.md"
fi

cat > "$ANALYSIS_DIR/proposed-fixes.md" <<HDR
# Proposed fixes

Generated $(date -u +%FT%TZ).

> Heuristic fixes (concrete template edits) are produced by the
> \`/debug-review\` Claude Code skill, not by this script. This file
> records the placeholder until the skill runs.

See \`clusters.md\` for the candidate set. A cluster is actionable when
it has **≥ 2 distinct project_hash values** in the same file row.
HDR

echo "wrote $ANALYSIS_DIR/clusters.md + proposed-fixes.md"
