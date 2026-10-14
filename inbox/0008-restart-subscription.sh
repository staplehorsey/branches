# timeout: 900
# Restart with the subscription fix, then watch for Claude's first builds.
BRANCHES_NO_OPEN=1 scripts/dev.sh | tail -3
echo "-- waiting up to 10 minutes for architect activity (go into a house and linger)"
for i in $(seq 1 60); do
  sleep 10
  scripts/dev.sh log 400 | grep -iE "architect|claude" | tail -5 | grep . && break
done
git -C ~/Library/Application\ Support/Branches log --format='%ci %s' -5
