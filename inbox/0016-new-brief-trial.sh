# timeout: 2700
# Restart, make a real space with the new brief (Sonnet) in The Lush 5,0, then the same brief with Opus in a scratch folder.
BRANCHES_NO_OPEN=1 scripts/dev.sh | tail -1
start=$(date +%s)
curl -s -XPOST -H 'content-type: application/json' localhost:7878/api/app/spaces/the-lush/5/0/visit -d '{"n":0}' >/dev/null
for i in $(seq 1 120); do sleep 5; s=$(curl -s localhost:7878/api/worlds/the-lush/rooms/5/0 | python3 -c 'import json,sys; d=json.load(sys.stdin); sp=[s for s in d.get("spaces",[]) if s["n"]==1]; print(sp[0]["status"] if sp else "none")'); [ "$s" != "building" ] && break; done
echo "-- sonnet (real build): $s after $(( $(date +%s) - start ))s"
D=~/Library/Application\ Support/Branches/worlds/the-lush/spaces/5_0/1
cat "$D/space.json"; echo; ls -la "$D"
mkdir -p inbox/trial-sonnet inbox/trial-opus; cp "$D"/* inbox/trial-sonnet/
T=$(mktemp -d /tmp/space-trial.XXXX); cd "$T"; start=$(date +%s)
env -u ANTHROPIC_API_KEY -u ANTHROPIC_AUTH_TOKEN -u ANTHROPIC_BASE_URL claude -p --model opus --permission-mode acceptEdits --allowedTools Read Write Edit Glob Grep < "$D/BRIEF.md" 2>&1 | tail -2
echo "-- opus took $(( $(date +%s) - start ))s"; cat space.json; echo; ls -la
cp "$T"/* ~/Projects/branches/inbox/trial-opus/
