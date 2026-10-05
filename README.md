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
the first time (it isn't notarized yet). It opens in your browser at
`http://localhost:7878`. Your worlds are saved in
`~/Library/Application Support/Branches` as a git repository: every build is
a commit, and every few builds a version.

**In a browser:** https://staplehorsey.github.io/branches/ runs the worlds
in the page (single player, saved in the browser). When you want visitor
books, the architect and saving, it walks you through getting the app.

**In a headset:** open the web page or the app in the Quest browser and
press **enter in VR**.

## Controls

**WASD** walk · **shift** faster · **space** hop · **mouse** look (click to
capture, or drag) · **E** talk, read, play, ride a bike, visitor book ·
**F** more of this · **Q** scooter · **M** map · **V** first/third person ·
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
* **GitHub: back up and share.** It forks this repository and keeps your
  worlds on the fork's `worlds` branch. If you choose to share, it opens a
  pull request that puts a door to your worlds in the Commons, the house
  just west of The Lush's spawn.

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
