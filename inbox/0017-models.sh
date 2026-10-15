# timeout: 600
# Restart, and check both models answer on your subscription.
BRANCHES_NO_OPEN=1 scripts/dev.sh | tail -1
cd /tmp
for m in claude-opus-5-5 claude-sonnet-5-5; do
  s=$(date +%s); r=$(echo 'Reply with exactly {"ok": true}' | env -u ANTHROPIC_API_KEY -u ANTHROPIC_AUTH_TOKEN -u ANTHROPIC_BASE_URL claude -p --model $m 2>&1 | head -2)
  echo "$m: $r ($(( $(date +%s) - s ))s)"
done
curl -s localhost:7878/api/app/settings | grep -oE '"(design|build)_model":"[^"]*"'
