#!/usr/bin/env bash
# Push every label in labels.yml to the repo (create or update color/description), then
# report any label on the repo that the manifest does not describe.
# Idempotent — safe to re-run. Needs: gh CLI (authed), python3.
#
# Usage: ./sync-labels.sh [owner/repo] [--prune]
#   --prune  actually delete the labels reported as drift (default is a dry run, because
#            deleting a label also strips it from every issue and PR that carried it).
set -euo pipefail

REPO="Nipeno/nightzoom-fps-limiter"
PRUNE=0
for arg in "$@"; do
  case "$arg" in
    --prune) PRUNE=1 ;;
    -*)      echo "Unknown flag: $arg" >&2; exit 2 ;;
    *)       REPO="$arg" ;;
  esac
done

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
YML="$DIR/labels.yml"

# Tiny parser: emit "name<TAB>color<TAB>description" per label, no yaml dep.
parse_labels() {
  python3 - "$YML" <<'PY'
import sys, re
name = color = desc = None
def flush(rows):
    if rows.get("name"):
        print("\t".join([rows.get("name",""), rows.get("color",""), rows.get("description","")]))
rows = {}
for line in open(sys.argv[1]):
    line = line.rstrip("\n")
    if re.match(r"^\s*#", line) or not line.strip():
        continue
    m = re.match(r"^- name:\s*(.+)$", line)
    if m:
        if rows: flush(rows)
        rows = {"name": m.group(1).strip()}
        continue
    m = re.match(r"^\s+(color|description):\s*(.+)$", line)
    if m:
        rows[m.group(1)] = m.group(2).strip()
if rows: flush(rows)
PY
}

parse_labels | while IFS=$'\t' read -r name color desc; do
  echo "==> $name"
  gh label create "$name" --repo "$REPO" --color "$color" --description "$desc" --force
done

# --- Drift: labels that exist on the repo but not in the manifest ------------
# Without this the script can only ever add labels, so anything created by hand (or
# GitHub's stock set) sticks around forever and duplicates what the manifest defines.
echo
echo "Checking for labels not in labels.yml ..."
manifest_names="$(parse_labels | cut -f1)"
drift=0
while IFS= read -r live; do
  [ -n "$live" ] || continue
  if ! grep -Fxq "$live" <<<"$manifest_names"; then
    drift=1
    if [ "$PRUNE" -eq 1 ]; then
      echo "deleting: $live"
      gh label delete "$live" --repo "$REPO" --yes
    else
      echo "would delete: $live   (re-run with --prune to remove)"
    fi
  fi
done < <(gh label list --repo "$REPO" --limit 200 --json name --jq '.[].name')

[ "$drift" -eq 0 ] && echo "No drift — the repo matches labels.yml."

echo "Done."
