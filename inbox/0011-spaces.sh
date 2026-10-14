# timeout: 600
# Restart on spaces.
BRANCHES_NO_OPEN=1 scripts/dev.sh | tail -3
curl -s localhost:7878/api/worlds/the-lush/rooms/0/0 | grep -o '"spaces_on":[a-z]*'
