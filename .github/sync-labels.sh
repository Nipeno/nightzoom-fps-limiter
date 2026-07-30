#!/usr/bin/env bash
# Push every label in labels.yml to the repo (create or update color/description).
# Idempotent — safe to re-run. Needs: gh CLI (authed), python3.
set -euo pipefail

REPO="${1:-Nipeno/nightzoom-fps-limiter}"
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
YML="$DIR/labels.yml"

# Tiny parser: emit "name<TAB>color<TAB>description" per label, no yaml dep.
python3 - "$YML" <<'PY' | while IFS=$'\t' read -r name color desc; do
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
  echo "==> $name"
  gh label create "$name" --repo "$REPO" --color "$color" --description "$desc" --force
done

echo "Done."
