# What spaces exist, what they're called, and how their walls are made.
R=~/Library/Application\ Support/Branches/worlds
for f in "$R"/*/spaces/*/*/space.json; do echo "== ${f#$R/}"; cat "$f"; echo; done 2>/dev/null | head -150
echo "-- how walls are drawn (materials/sides/opacity)"
for f in "$R"/*/spaces/*/*/index.html "$R"/*/spaces/*/*/*.js; do [ -f "$f" ] || continue; echo "== ${f#$R/} ($(wc -c <"$f") bytes)"; grep -noE "PlaneGeometry|BoxGeometry|side: *THREE\.[A-Za-z]+|transparent: *true|opacity: *[0-9.]+|BackSide|DoubleSide" "$f" | sort | uniq -c | sort -rn | head -8; done 2>/dev/null | head -120
