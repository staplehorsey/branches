# timeout: 300
# Claude Code with only your login: no API key, auth token or base URL.
cd /tmp
echo "-- base URL host (where the override points)"; printf '%s\n' "$ANTHROPIC_BASE_URL" | sed -E 's#(https?://[^/]+).*#\1#'
for v in ANTHROPIC_API_KEY ANTHROPIC_AUTH_TOKEN ANTHROPIC_BASE_URL CLAUDE_OAUTH_TOKEN; do echo "$v set in profile files:"; grep -lE "$v" ~/.zshrc ~/.zprofile ~/.zshenv ~/.bash_profile ~/.profile ~/.config/fish/config.fish 2>/dev/null; done
echo "-- try"
time (echo 'Reply with exactly {"ok": true}' | env -u ANTHROPIC_API_KEY -u ANTHROPIC_AUTH_TOKEN -u ANTHROPIC_BASE_URL claude -p 2>&1 | head -5)
