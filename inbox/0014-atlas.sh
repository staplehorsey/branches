# timeout: 2400
# Restart on the atlas build, then time Sonnet on the same brief the first real space had (in a scratch folder).
BRANCHES_NO_OPEN=1 scripts/dev.sh | tail -2
B=~/Library/Application\ Support/Branches/worlds/the-lush/spaces/2_0/1/BRIEF.md
T=$(mktemp -d /tmp/space-trial.XXXX); cd "$T"
start=$(date +%s)
env -u ANTHROPIC_API_KEY -u ANTHROPIC_AUTH_TOKEN -u ANTHROPIC_BASE_URL claude -p --model sonnet --permission-mode acceptEdits --allowedTools Read Write Edit Glob Grep < "$B" | tail -3
echo "-- sonnet took $(( $(date +%s) - start ))s"; ls -la "$T"; cat "$T/space.json" 2>/dev/null | head -c 800; echo
echo "-- your default model in settings:"; python3 -c 'import json,os; print(json.load(open(os.path.expanduser("~/.claude/settings.json"))).get("model"))'
