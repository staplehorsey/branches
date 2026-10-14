#!/bin/bash
# Command inbox: lets a Claude Code session in the cloud drive this checkout.
#
#   ./scripts/inbox.sh            # leave it running; Ctrl-C to stop
#
# Every few seconds it pulls the current branch. Each new inbox/NNNN-name.sh
# runs here, in the repo root, and its output goes to inbox/NNNN-name.out,
# which is committed and pushed back (also while it runs, every 30 s). A
# script that already has a .out file never runs again. Add a line
# "# timeout: 600" to change the 30 minute limit; start servers with nohup
# and & so they outlive their script.
#
# Anyone who can push to this branch can run commands as you on this
# machine, so only leave it running while you want that.
#
# Settings: INBOX_POLL (seconds between pulls, default 10),
# INBOX_TIMEOUT (default limit in seconds, default 1800).
set -u

# Run from a copy, so pulling a new version of this file can't change the
# script under bash's feet; restart on the new version when it changes.
if [ -z "${INBOX_COPY:-}" ]; then
  src="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
  tmp=$(mktemp -t branches-inbox.XXXXXX)
  cp "$src" "$tmp"
  INBOX_COPY=1 INBOX_SRC="$src" exec bash "$tmp" "$@"
fi

cd "$(dirname "$INBOX_SRC")/.." || exit 1
ROOT=$(pwd)
BRANCH=$(git rev-parse --abbrev-ref HEAD)
POLL=${INBOX_POLL:-10}
DEFAULT_TIMEOUT=${INBOX_TIMEOUT:-1800}
set -m # each command runs in its own process group, so a timeout kills all of it

log() { printf '%s %s\n' "$(date +%H:%M:%S)" "$*"; }

pull() {
  if ! git pull -q --rebase --autostash origin "$BRANCH" >/dev/null 2>&1; then
    git rebase --abort >/dev/null 2>&1
    log "pull failed (run 'git status' if this keeps happening)"
    return 1
  fi
}

# Commit inbox/ only (never your other changes) and push it.
push_out() {
  git add -A -- inbox >/dev/null 2>&1
  git commit -q -m "$1" -- inbox >/dev/null 2>&1 || return 0
  for i in 1 2 3; do
    git push -q origin "$BRANCH" >/dev/null 2>&1 && return 0
    pull
    sleep $((i * 2))
  done
  log "push failed"
}

run_one() {
  cmd=$1
  out="${cmd%.sh}.out"
  name=$(basename "$cmd")
  t=$(sed -n 's/^# *timeout: *\([0-9][0-9]*\).*/\1/p' "$cmd" | head -1)
  t=${t:-$DEFAULT_TIMEOUT}
  log "running $name (limit ${t}s)"
  {
    echo "== $name"
    echo "== started $(date '+%Y-%m-%d %H:%M:%S') on $(hostname -s), $(uname -sm)"
    echo "== in $ROOT at $(git rev-parse --short HEAD)"
    echo
  } >"$out"
  push_out "inbox: start $name"

  start=$(date +%s)
  last=$start
  bash "$cmd" </dev/null >>"$out" 2>&1 &
  pid=$!
  timed_out=
  while kill -0 "$pid" 2>/dev/null; do
    sleep 2
    now=$(date +%s)
    if [ $((now - start)) -ge "$t" ]; then
      timed_out=1
      kill -TERM -"$pid" 2>/dev/null
      sleep 3
      kill -KILL -"$pid" 2>/dev/null
      break
    fi
    if [ $((now - last)) -ge 30 ]; then
      push_out "inbox: progress $name"
      last=$now
    fi
  done
  wait "$pid" 2>/dev/null
  code=$?
  {
    echo
    [ -n "$timed_out" ] && echo "== timed out after ${t}s"
    echo "== exit $code after $(($(date +%s) - start))s"
  } >>"$out"
  log "finished $name: exit $code"
  push_out "inbox: done $name (exit $code)"
}

mkdir -p inbox
log "inbox on $BRANCH in $ROOT, checking every ${POLL}s; Ctrl-C to stop"
while :; do
  pull
  if ! cmp -s "$INBOX_SRC" "$0"; then
    log "inbox.sh changed, restarting"
    INBOX_COPY= exec bash "$INBOX_SRC"
  fi
  for cmd in inbox/*.sh; do
    [ -f "$cmd" ] || continue
    [ -e "${cmd%.sh}.out" ] && continue
    run_one "$cmd"
  done
  sleep "$POLL"
done
