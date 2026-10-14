# timeout: 1500
# Make one real space with Claude (The Lush, house 2,0) and report on it.
curl -s -XPOST -H 'content-type: application/json' localhost:7878/api/app/spaces/the-lush/2/0/visit -d '{"n":0}' | head -c 300; echo
for i in $(seq 1 140); do
  sleep 10
  s=$(curl -s localhost:7878/api/worlds/the-lush/rooms/2/0 | python3 -c 'import json,sys; d=json.load(sys.stdin); sp=[s for s in d.get("spaces",[]) if s["n"]==1]; print(sp[0]["status"] if sp else "none")')
  [ "$s" != "building" ] && break
done
curl -s localhost:7878/api/worlds/the-lush/rooms/2/0 | python3 -c 'import json,sys; d=json.load(sys.stdin); print(json.dumps({"idea": d.get("idea"), "spaces": d.get("spaces")}, indent=1)[:2500])'
D=~/Library/Application\ Support/Branches/worlds/the-lush/spaces/2_0/1
ls -la "$D"; wc -c "$D"/* 2>/dev/null | tail -1
echo "-- index.html (first 60 lines)"; head -60 "$D/index.html"
scripts/dev.sh log 30 | grep -i space
