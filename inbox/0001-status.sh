# What's on this Mac, and where Branches stands.
echo "-- system"; sw_vers 2>/dev/null; uname -m
echo; echo "-- tools"
for t in git cargo rustc rustup claude node gh brew xcode-select; do
  printf '%-13s ' "$t"; command -v "$t" >/dev/null && { "$t" --version 2>&1 | head -1 || true; } || echo "not found"
done
ls ~/.cargo/bin 2>/dev/null | head -5
echo; echo "-- repo"; git status -sb | head -20; git log --oneline -3
echo; echo "-- app"
ls -d /Applications/Branches.app 2>/dev/null || echo "Branches.app not in /Applications"
pgrep -fl branches 2>/dev/null || echo "no branches process"
echo; echo "-- port 7878"
lsof -nP -iTCP:7878 -sTCP:LISTEN 2>/dev/null || echo "nothing listening"
curl -s -m 3 http://localhost:7878/.well-known/branches.json || echo "(no answer)"
echo; echo "-- worlds"
ls -la ~/Library/Application\ Support/Branches 2>/dev/null | head -15 || echo "no worlds folder"
