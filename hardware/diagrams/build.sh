#!/usr/bin/env bash
# Rebuild every SVG in hardware/diagrams/svg/ from its source (SQSO-HW-DIAG-001 §5).
#   block/*.dot        Graphviz dot   -> svg/B*.svg
#   wiring/*.yml       WireViz 0.4.1  -> svg/W*.svg
#   schematics/S*.py   schemdraw 0.23 -> svg/S*.svg
# Needs graphviz (dot) on PATH and the Python packages in requirements.txt.
set -euo pipefail
export PYTHONDONTWRITEBYTECODE=1

here="$(cd "$(dirname "$0")" && pwd)"
out="$here/svg"
mkdir -p "$out"

for f in "$here"/block/*.dot; do
    dot -Tsvg "$f" -o "$out/$(basename "${f%.dot}").svg"
done

for f in "$here"/wiring/*.yml; do
    wireviz -f s -o "$out" "$f" >/dev/null
done

for f in "$here"/schematics/S*.py; do
    python3 "$f" "$out"
done

echo "built $(ls "$out"/*.svg | wc -l) diagrams in $out"
