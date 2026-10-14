# timeout: 600
# Restart on the dream build.
BRANCHES_NO_OPEN=1 scripts/dev.sh | tail -4
curl -s localhost:7878/api/app/dream; echo
