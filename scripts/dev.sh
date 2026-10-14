#!/bin/bash
# Build the server from this checkout and (re)start it as the app, in the
# background, at http://localhost:7878. Worlds stay in the usual place.
#
#   scripts/dev.sh           # build, restart, wait until it answers
#   scripts/dev.sh stop
#   scripts/dev.sh status
#   scripts/dev.sh log       # last 80 lines of the server log
#
# Set BRANCHES_NO_OPEN=1 to skip opening the browser.
set -u
cd "$(dirname "$0")/../server" || exit 1
PIDFILE=target/dev.pid
LOG=target/dev.log
URL=http://localhost:7878

plain() { sed -e $'s/\x1b\\[[0-9;]*m//g'; }

running() { [ -f "$PIDFILE" ] && kill -0 "$(cat "$PIDFILE")" 2>/dev/null; }

stop() {
  if running; then
    kill "$(cat "$PIDFILE")"
    for _ in 1 2 3 4 5 6 7 8 9 10; do running || break; sleep 0.5; done
    running && kill -9 "$(cat "$PIDFILE")"
    echo "stopped"
  fi
  rm -f "$PIDFILE"
  # One started some other way (by hand, or the app) still holds the port.
  for p in $(lsof -tnP -iTCP:7878 -sTCP:LISTEN 2>/dev/null); do
    ps -o command= -p "$p" | grep -q branches-server && kill "$p" && sleep 1 && echo "stopped $p"
  done
  other=$(lsof -tnP -iTCP:7878 -sTCP:LISTEN 2>/dev/null)
  [ -n "$other" ] && echo "port 7878 is still taken by: $(ps -o pid=,command= -p $other)"
}

status() {
  if running; then echo "running, pid $(cat "$PIDFILE")"; else echo "not running"; fi
  curl -s -m 3 "$URL/.well-known/branches.json" && echo || echo "no answer at $URL"
}

case "${1:-start}" in
  stop) stop; exit ;;
  status) status; exit ;;
  log) tail -n "${2:-80}" "$LOG" | plain; exit ;;
  start) ;;
  *) echo "usage: $0 [start|stop|status|log]"; exit 2 ;;
esac

echo "-- building"
mkdir -p target
cargo build --release >target/build.log 2>&1
rc=$?
grep -vE '^ +(Compiling|Downloaded|Downloading|Locking|Adding|Updating) ' target/build.log | tail -n 60
[ "$rc" -eq 0 ] || { echo "build failed"; exit 1; }

echo "-- restarting"
stop
nohup ./target/release/branches-server --app >"$LOG" 2>&1 &
echo $! >"$PIDFILE"
for _ in $(seq 1 30); do
  curl -s -m 2 -o /dev/null "$URL/.well-known/branches.json" && break
  running || break
  sleep 1
done
status
echo "-- log"
tail -n 20 "$LOG" | plain
