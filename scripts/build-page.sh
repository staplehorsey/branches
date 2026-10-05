#!/bin/sh
# Package the client as a self-contained static page that runs every world
# in the browser (no server). Paths become relative and the document
# skeleton is left to the host page.
# Usage: scripts/build-page.sh <out-dir>
set -e
OUT=${1:?usage: build-page.sh <out-dir>}
ROOT=$(cd "$(dirname "$0")/.." && pwd)
rm -rf "$OUT" && mkdir -p "$OUT"
cp -r "$ROOT/client/src" "$ROOT/client/vendor" "$ROOT/client/style.css" "$OUT/"
python3 - "$ROOT/client/index.html" "$OUT/index.html" <<'PY'
import re, sys
s = open(sys.argv[1]).read()
for tag in ['<!doctype html>', '<html lang="en">', '</html>', '<head>', '</head>', '<body>', '</body>']:
    s = s.replace(tag, '')
s = re.sub(r'\s*<meta [^>]*>', '', s)
s = s.replace('href="/style.css"', 'href="style.css"').replace('"/vendor/three.module.js"', '"./vendor/three.module.js"').replace('src="/src/main.js"', 'src="src/main.js"')
title = re.search(r'\s*<title>.*?</title>', s).group(0)
s = title.strip() + '\n' + s.replace(title, '')
s = ''.join(c if ord(c) < 128 else '&#%d;' % ord(c) for c in s)
open(sys.argv[2], 'w').write(re.sub(r'\n\s*\n+', '\n', s).strip() + '\n')
PY
echo "built $OUT"
