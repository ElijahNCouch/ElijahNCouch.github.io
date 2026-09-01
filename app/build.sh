#!/bin/bash
# Build the WASM site and deploy it to the repo root.
# Run from the app/ directory:  ./build.sh
set -euo pipefail
cd "$(dirname "$0")"
ROOT="$(cd .. && pwd)"

dx build --release --platform web
PUB="target/dx/portfolio/release/web/public"

# refresh built output at repo root
rm -rf "$ROOT/assets" "$ROOT/wasm" "$ROOT/index.html"
cp -R "$PUB/." "$ROOT/"
find "$ROOT" -name '*.br' -delete
touch "$ROOT/.nojekyll"

# inline the CSS into <head> so styling loads immediately (no runtime dependency, no FOUC)
python3 - "$ROOT/index.html" "assets/style.css" <<'PY'
import sys
idx, cssf = sys.argv[1], sys.argv[2]
html = open(idx, encoding='utf-8').read()
css  = open(cssf, encoding='utf-8').read()
html = html.replace("</head>", "<style>\n"+css+"\n</style>\n</head>", 1)
open(idx, "w", encoding='utf-8').write(html)
print("CSS inlined into <head>")
PY
echo "Done. Commit the repo root and push."
