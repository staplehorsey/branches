# Why is generation stuck? Spaces building/failed, claude processes, log.
R=~/Library/Application\ Support/Branches
python3 - "$R" <<'PY'
import json,sys,os,time,glob
now=time.time()*1000
for f in glob.glob(os.path.join(sys.argv[1],'worlds/*/rooms/*.json'))+glob.glob(os.path.join(sys.argv[1],'worlds/*/*.json')):
    pass
PY
curl -s localhost:7878/api/worlds | python3 -c 'import json,sys; print([w["id"] for w in json.load(sys.stdin)][:10])' 2>&1 | head -2
for w in moonlit-meadow fog-pines the-lush dusk-orchard salt-flat-noon hushed-meridian-3f02; do
  curl -s "localhost:7878/api/worlds/$w" | python3 -c '
import json,sys,time
d=json.load(sys.stdin)
' 2>/dev/null
done
grep -rl '"spaces"' "$R/worlds" 2>/dev/null | head -20 | while read f; do python3 - "$f" <<'PY'
import json,sys,time
d=json.load(open(sys.argv[1]))
rooms = d.get("rooms", {d.get("x","?"):d}) if isinstance(d,dict) else {}
def show(r):
    for s in r.get("spaces",[]):
        age=(time.time()*1000-s["started"])/1000
        print(sys.argv[1].split("worlds/")[-1][:60], f'{r.get("x")},{r.get("z")}', "space", s["n"], s["status"], f"started {age:.0f}s ago", f"took {s.get('secs')}s", "tries", s.get("tries"), "|", (s.get("error") or "")[:200], "| visited", r.get("space_visited"))
if "spaces" in d: show(d)
else:
    for r in (d.get("rooms") or {}).values(): show(r)
PY
done
echo "-- claude processes"; ps -axo pid,etime,command | grep -E "[c]laude .*-p" | cut -c1-160
echo "-- log"; scripts/dev.sh log 400 | grep -iE "space|warn|error|fail" | tail -15
echo "-- server"; scripts/dev.sh status | head -1
