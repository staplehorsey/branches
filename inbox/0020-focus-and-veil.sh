# timeout: 600
# Stop the old server's space builds (it can't), restart on the new version, show what's building.
pkill -f "claude --model .* -p --permission-mode acceptEdits --allowedTools Read Write Edit Glob Grep" && echo "stopped old space builds"
BRANCHES_NO_OPEN=1 scripts/dev.sh | tail -1
sleep 20
grep -rl '"spaces"' ~/Library/Application\ Support/Branches/worlds/*/rooms 2>/dev/null | while read f; do python3 - "$f" <<'PY'
import json,sys,time
r=json.load(open(sys.argv[1]))
for s in r.get("spaces",[]):
    if s["status"]!="ready": print(sys.argv[1].split("worlds/")[-1], s["n"], s["status"], s.get("stage"), (s.get("error") or "")[:120])
PY
done
ps -axo etime,command | grep -E "[c]laude --model" | cut -c1-90
