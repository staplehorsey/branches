# What the compass has stored (your G notes), and the spaces built so far.
R=~/Library/Application\ Support/Branches
python3 - "$R" <<'PY'
import json,sys,os,time
r=sys.argv[1]
d=json.load(open(os.path.join(r,'.branches/dream.json')))
print("muses:", [m['name'] for m in d.get('muses',[])])
for m in d.get('compass',[])[-12:]:
    print(time.strftime('%H:%M', time.localtime(m['at']/1000)), m['world'], f"{m['x']},{m['z']}", 'space', m.get('space'), 'score', m['score'], '|', m['idea'][:60], '|', m.get('note'))
PY
for f in "$R"/worlds/*/spaces/*/*/space.json; do echo "${f#$R/worlds/}: $(python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); print(d.get("name"), "|", ", ".join(d.get("motifs",[])))' "$f")"; done
scripts/dev.sh log 300 | grep -iE "space|warn" | tail -8
