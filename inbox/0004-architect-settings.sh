# Which architect is chosen, and does claude -p answer from the app's folder?
R=~/Library/Application\ Support/Branches
cat "$R/.branches/settings.json"; echo
git -C "$R" log --format=%s -200 | grep -oE '\[[^,]+, [a-z]+\]$' | sort | uniq -c
cd "$R/.branches/agent" && time (echo 'Reply with exactly {"ok": true}' | claude -p 2>&1 | head -5)
