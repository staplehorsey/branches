# timeout: 600
# Does Claude Code answer with your subscription (no API key)? Then restart on the new code.
cd ~/Library/Application\ Support/Branches/.branches 2>/dev/null || cd /tmp
echo "-- claude -p without ANTHROPIC_API_KEY"
time (echo 'Reply with exactly {"ok": true}' | env -u ANTHROPIC_API_KEY claude -p 2>&1 | head -5)
cd ~/Projects/branches
echo "-- restart"
env -u ANTHROPIC_API_KEY BRANCHES_NO_OPEN=1 scripts/dev.sh
curl -s localhost:7878/api/app/workshop | head -c 200; echo
curl -s localhost:7878/api/app/settings | grep -o '"settings":{[^}]*' ; echo
