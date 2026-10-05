#!/bin/bash
# Branches.app entry point. Starts the Branches app server in the background
# (it opens your browser), or just opens the browser if it is already running.
DIR="$(cd "$(dirname "$0")" && pwd)"
URL="http://localhost:7878/"
if /usr/bin/curl -s --max-time 1 "${URL}.well-known/branches.json" | /usr/bin/grep -q 'branches/0.1'; then
  /usr/bin/open "$URL"
  exit 0
fi
# Apps launched from Finder get a bare PATH. Borrow the login shell's, so an
# architect command such as `claude -p` can be found.
P="$(/bin/zsh -ilc 'echo "__PATH__$PATH"' </dev/null 2>/dev/null | /usr/bin/grep '^__PATH__' | /usr/bin/tail -1)"
[ -n "$P" ] && export PATH="${P#__PATH__}"
export PATH="$PATH:/opt/homebrew/bin:/usr/local/bin:$HOME/.local/bin"
mkdir -p "$HOME/Library/Logs"
BRANCHES_APP=1 nohup "$DIR/branches-server" --app >> "$HOME/Library/Logs/Branches.log" 2>&1 &
exit 0
