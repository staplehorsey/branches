# timeout: 2400
# Time Sonnet on the same brief the first real space had (scratch folder; your worlds aren't touched).
B=~/Library/Application\ Support/Branches/worlds/the-lush/spaces/2_0/1/BRIEF.md
T=$(mktemp -d /tmp/space-trial.XXXX); cd "$T"
start=$(date +%s)
env -u ANTHROPIC_API_KEY -u ANTHROPIC_AUTH_TOKEN -u ANTHROPIC_BASE_URL claude -p --model sonnet --permission-mode acceptEdits --allowedTools Read Write Edit Glob Grep < "$B" 2>&1 | tail -3
echo "-- sonnet took $(( $(date +%s) - start ))s"; ls -la "$T"; cat "$T/space.json" 2>/dev/null | head -c 900; echo
echo "-- default model in your Claude settings:"; python3 -c 'import json,os; print(json.load(open(os.path.expanduser("~/.claude/settings.json"))).get("model"))'
mkdir -p ~/Projects/branches/inbox/trial && cp "$T"/index.html "$T"/space.json ~/Projects/branches/inbox/trial/ 2>/dev/null; echo copied
