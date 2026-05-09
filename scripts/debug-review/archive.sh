#!/usr/bin/env bash
# scripts/debug-review/archive.sh — moves patches older than 6 months
# from debug-history/patches/ to debug-history/archive/YYYY-MM/. Run
# from CI on a monthly schedule (or manually) to keep the active set
# small enough for /debug-review iteration.
#
# Invariant: never deletes content. Move-only. The full audit trail
# lives under archive/.

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
PATCH_DIR="$ROOT/debug-history/patches"
ARCHIVE_ROOT="$ROOT/debug-history/archive"
CUTOFF_DAYS=180

if ! compgen -G "$PATCH_DIR/*.patch.json" >/dev/null; then
  echo "no patches; nothing to archive."
  exit 0
fi

now_epoch=$(date -u +%s)
moved=0
for patch in "$PATCH_DIR"/*.patch.json; do
  collected=$(jq -r '.collected_at // empty' "$patch" 2>/dev/null || true)
  if [ -z "$collected" ]; then
    continue
  fi
  patch_epoch=$(date -u -d "$collected" +%s 2>/dev/null || true)
  if [ -z "$patch_epoch" ]; then
    continue
  fi
  age_days=$(( (now_epoch - patch_epoch) / 86400 ))
  if [ "$age_days" -ge "$CUTOFF_DAYS" ]; then
    yyyy_mm=$(date -u -d "$collected" +%Y-%m 2>/dev/null || echo "unknown")
    target="$ARCHIVE_ROOT/$yyyy_mm"
    mkdir -p "$target"
    mv "$patch" "$target/"
    moved=$(( moved + 1 ))
  fi
done

echo "archived $moved patch(es) older than $CUTOFF_DAYS days"
