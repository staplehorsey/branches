# Branches

An endless, tranquil world of identical liminal houses whose insides grow.
Walk into any house with no loading, no cut. Linger, and the **architect**
builds more: new chambers, characters with something to say, story pages
scattered across houses, little games. What it builds follows what you look
at, use, ask for in the visitor book, and mark as *more of this*. Doors lead
to other worlds, and you walk through them for real. Wander far enough and
new worlds grow from the shape of your path.

## Play

**On a Mac (recommended):** download
[Branches for Mac](https://github.com/staplehorsey/branches/releases/download/mac-latest/Branches-mac.zip),
unzip it, drag **Branches** to Applications, then right-click → **Open**
the first time (it isn't notarized yet). It lives in the menu bar (a door
icon, no Dock icon): status, open Branches, start, stop or restart the
server, show your worlds folder or the log, launch at login, quit. Branches
opens in your browser at `http://localhost:7878`. Your worlds are saved in
`~/Library/Application Support/Branches` as a git repository: every build is
a commit, and every few builds a version.

**In a browser:** https://staplehorsey.github.io/branches/ runs the worlds
in the page (single player, saved in the browser). When you want visitor
books, the architect and saving, it walks you through getting the app.

**In a headset:** open the web page or the app in the Quest browser and
press **enter in VR**.

## Controls

**WASD** walk · **shift** faster · **space** hop · **mouse** look (click to
capture, or drag) · **E** talk (to people and residents), read, play, visitor book, get on a
bike · **F** more of this · **Q** scooter · **R** residents · **M** map · **V** first/third person ·
**enter** chat · **L** worlds and settings · **N** notifications · **H**
wake up at home.

In VR: left stick walk (click to go faster), right stick turn, **A** use,
**B** more of this, **Y** leave VR. A card on your left wrist shows where
you are.

## In the app

Open **L** (worlds), then the settings at the bottom:

* **Architect:** the built-in one, or any program. `claude -p` makes Claude
  Code the architect: it gets a description of the house and what visitors
  want, and prints a JSON plan. Quick sketches always use the built-in
  architect; your program gets the rooms people linger in.
* **Night shift:** while nobody is playing, inside hours you choose, an
  agent builds something bigger, such as a zoo tycoon or a go-kart track.
  The default is `claude -p --permission-mode acceptEdits`, run in a folder
  of its own. Games asked for in visitor books come first. Finished games
  open as attraction booths in a well-loved house. **Build something now**
  starts one immediately.
* **Who builds:** the first time a house is about to grow, the app asks
  who should build. Choose Claude Code and it uses your own Claude
  subscription (it finds `claude` wherever it is installed and never uses
  an API key). **Test it** checks that it's signed in.
* **Residents (R):** people who live in the worlds, with memories, goals
  and coins. They wander the streets, visit houses, write in visitor books,
  talk to you (**E** near one) and commission builds. Thinking runs your
  agent and costs coins (1 coin = 1,000 tokens), paid from a treasury you
  fund. When people admire things, use them or linger in what a resident
  commissioned, the treasury pays them back. Fund the treasury, move
  someone in, or see who lives where from the panel.
* **GitHub: back up, share, publish.** It forks this repository and keeps
  your worlds on the fork's `worlds` branch. **Back up now** pushes at once
  and shows any error. On a new Mac, connecting brings your worlds back.
  **Share this house** (in a visitor book) or **share my worlds** opens a
  pull request that puts a door in the Commons, a street of doors west of
  The Lush's spawn. **Your own page** builds a copy of the web version on
  your fork's GitHub Pages that opens your worlds; anyone can also open
  them from the main page with `?at=gh://you/branches@worlds/w/the-lush/0,0`.
* **Bring in changes:** merges the main world's changes into yours (and
  keeps your fork's code current). Houses changed on both sides are merged
  by your architect agent; everything else keeps what both sides added.
  **Restore from my fork** does the same from your own fork.

## Build from source

```sh
cd server && cargo run --release -- --app   # the app, at http://localhost:7878
cd server && cargo run --release            # a plain host, at :8080
cd server && cargo test
```

`scripts/build-page.sh <dir> [--standalone]` packages the browser version.
`scripts/export-offline.sh` refreshes its copy of the starter worlds from a
running host.

See **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)** for how it all works
and where it's going next: meeting people nearby, neighbourhoods of about
Dunbar's number, and cities that grow.
