# timeout: 300
# Where does Claude Code's expired API key come from? Names only, never values.
echo "-- auth-related environment variables (names only)"
env | cut -d= -f1 | grep -iE 'anthropic|claude|api_key' || echo "(none)"
echo "-- settings files mentioning keys or helpers (keys only)"
for f in ~/.claude/settings.json ~/.claude/settings.local.json ~/Projects/branches/.claude/settings.json ~/Projects/branches/.claude/settings.local.json /Library/Application\ Support/ClaudeCode/managed-settings.json; do
  [ -f "$f" ] && { echo "$f:"; python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); print("  top-level keys:", sorted(d)); print("  env keys:", sorted((d.get("env") or {}).keys())); print("  apiKeyHelper set:", "apiKeyHelper" in d)' "$f"; }
done
echo "-- ~/.claude.json auth fields (names only)"
python3 -c 'import json,os; d=json.load(open(os.path.expanduser("~/.claude.json"))); print([k for k in d if any(s in k.lower() for s in ("key","auth","oauth","token","account"))]); print("customApiKeyResponses:", {k: len(v) for k,v in (d.get("customApiKeyResponses") or {}).items()})' 2>&1
echo "-- shell profile lines that set ANTHROPIC vars (values hidden)"
grep -nE 'ANTHROPIC|CLAUDE_CODE' ~/.zshrc ~/.zprofile ~/.zshenv ~/.bash_profile ~/.bashrc ~/.profile 2>/dev/null | sed -E 's/=.*/=***/'
echo "-- claude auth status"
env -u ANTHROPIC_API_KEY claude auth status 2>&1 | sed -E 's/(sk-ant-[A-Za-z0-9_-]{4})[A-Za-z0-9_-]+/\1***/g' | head -20
