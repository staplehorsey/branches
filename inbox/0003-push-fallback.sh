# timeout: 900
# Rebuild with the git push fallback, then watch for the worlds backup push.
BRANCHES_NO_OPEN=1 scripts/dev.sh
git -C ~/Library/Application\ Support/Branches log --oneline -3
echo "-- waiting for a backup push (the server pushes at most every 2 min, after a commit)"
for i in $(seq 1 24); do
  sleep 10
  scripts/dev.sh log 400 | grep -iE "push|git:" | tail -3 | grep . && break
done
echo "-- github.json (token hidden)"
sed -E 's/"token": *"[^"]*"/"token":"***"/' ~/Library/Application\ Support/Branches/.branches/github.json 2>/dev/null | head -c 600; echo
