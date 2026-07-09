#!/usr/bin/env bash
set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$DIR"

compile_variant() {
  local name="$1"
  local left="${2:-1.25in}"
  local right="${3:-1.25in}"
  local top="${4:-1in}"
  local bottom="${5:-1in}"

  local tmpfile
  tmpfile=$(mktemp "$ROOT/.tmp-XXXXXX.typ")
  cat > "$tmpfile" <<TYPTEMPLATE
#set page(paper: "us-letter", margin: (left: $left, right: $right, top: $top, bottom: $bottom))
// blank page so body text starts on page 2 (page 1 is excluded as title page)
#pagebreak()
#set text(size: 12pt)
#include "synthetic-body.typ"
TYPTEMPLATE

  echo "Compiling $name.pdf (L=$left R=$right T=$top B=$bottom)..."
  typst compile --root "$ROOT" "$tmpfile" "$DIR/$name.pdf"
  rm -f "$tmpfile"
}

echo "=== Synthetic margin test suite ==="
echo

compile_variant "baseline"          "1.25in" "1.25in" "1in"   "1in"
compile_variant "left-narrow"      "0.75in" "1.25in" "1in"   "1in"
compile_variant "right-narrow"     "1.25in" "0.75in" "1in"   "1in"
compile_variant "left-wide"        "1.75in" "1.75in" "1in"   "1in"
compile_variant "right-wide"       "1.75in" "1.75in" "1in"   "1in"
compile_variant "top-narrow"       "1.25in" "1.25in" "0.5in" "1in"
compile_variant "bottom-narrow"    "1.25in" "1.25in" "1in"   "0.5in"
compile_variant "top-wide"         "1.25in" "1.25in" "2in"   "1in"
compile_variant "asymmetric"       "1.50in" "1.00in" "1in"   "1in"

echo
echo "=== Compiling messy.pdf ==="
typst compile --root "$ROOT" "$DIR/synthetic-messy.typ" "$DIR/messy.pdf"

echo
echo "Done. All PDFs in $DIR/"
ls -la "$DIR"/*.pdf
