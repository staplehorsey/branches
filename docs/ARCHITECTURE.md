# Branches architecture

Branches is an endless, tranquil world made of many worlds joined by doors.
You play it on your own computer (Branches for Mac), in a browser, or in a
headset. Your worlds live in a git repository you own, so you can back them
up, share them and fork other people's. An architect (built in, or an agent
such as Claude Code) grows them from how people actually play.

```
 browser / headset                 Branches for Mac (localhost:7878)        GitHub
 ─────────────────                 ─────────────────────────────────        ──────
 three.js client  ── HTTP + ws ──▶ Rust server ── git ──▶ worlds repo ──push──▶ your fork
   realms, doors,                    architect (heuristic | command)            (worlds branch,
   things, gaze, map                 night shift, residents, sync                gh-pages)
        │                            story kit, genesis, evolution                 │ PR
        └─ no app? an in-page host                                                 ▼
           runs the same worlds                                       staplehorsey/branches
           (single player, browser storage)                           universe.json → the Commons
```

---

## 1. Primitives

| Primitive | What it is |
|---|---|
| **Address** | A place: `/w/<world>/<x>,<z>`, on a host (`http://localhost:7878`), in the page (`local://branches`), or in a git repository (`gh://<owner>/<repo>@<branch>`). |
| **World** | A manifest: name, seed, biome (`generator.params`), door policy, spawn, `hub` flag, and, for grown worlds, the parent and the path signature that made them. |
| **Room** | The house at an address: chambers, features, **things**, doors, visitor book, gaze and use counts, learned weights. |
| **Thing** | Something with a story: a character, a note or story page, a game (lanterns, bells, the cat), or an attraction. Free-form JSON, so architects can invent new kinds; unknown kinds still render. |
| **Door** | A graph edge. Doors come in physical pairs, and walking through one carries your body to the other side. |
| **Player** | The one forced primitive: id, name, colour, body, transform, momentum, vehicle. The same in every world, on screen or in a headset. |
| **Architect** | Turns attention into building. Built in, or any program that reads a prompt and prints a JSON plan. |
| **Version** | A git tag per world (`worlds/<id>/v0.N.0`), cut every few builds. |

Untouched houses are never stored; they are generated from `(world seed, x, z)`
by the same algorithm on every client and host (`server/src/procgen.rs`,
ported exactly to `client/src/procgen.js`, checked over 3,145 rooms). That
is what lets a door stored in one place pair with a house generated
somewhere else.

## 2. One physical space

There are no loading screens and no menu teleports.

* **Pockets.** A house's interior lives directly below it. The front door
  and the pocket door are a linked pair.
* **Realms.** Several worlds are loaded into the same scene at once, stacked
  1,500 m apart. Only one space is drawn per render pass.
* **Doors are frames.** Linking door A to door B gives
  `A.M = B.frame · rotY(π) · A.frame⁻¹`. The renderer draws B's side through
  A's opening from a virtual camera, once per eye in a headset. Crossing the
  opening applies `M` to your body.
* **Pairs.** Generator doors are symmetric: house (x,z) in world A links to
  house (x,z) in world B, and spawn houses link to each other. Doors into a
  closed world are sealed on the far side (one way in). Owner-made doors get
  a door back. A remote world that doesn't list a door back gets one added
  behind you.
* **H** (wake up at home) is the single deliberate exception.

## 3. Worlds in git

Branches for Mac keeps everything in a git repository at
`~/Library/Application Support/Branches`:

```text
worlds/index.json                 world list (for readers on GitHub)
worlds/<id>/world.json            manifest, biome included
worlds/<id>/index.json            stored houses, one line each
worlds/<id>/rooms/<x>,<z>.json    chambers, features, things, doors,
                                  visitor book, gaze and use counts
worlds/<id>/heat.json             seconds inside / outside and visits per cell
worlds/<id>/versions.json         tags, newest first
worlds/<id>/paths.json            recent path signatures
worlds/<id>/log.json              the world's guestbook
worlds/<id>/attractions/<slug>/   night-shift games: BRIEF.md, index.html, meta.json
residents/<id>.json               a resident: persona, goal, memories, coins
residents/economy.json            the treasury and the ledger
.branches/                        private, never committed: players, settings, GitHub token
.branches/agent/                  the folder agents run in
```

Besides `worlds`, the fork carries a `gh-pages` branch (section 5): the
static site plus `site.json`. It is built by the app, not edited by hand.

Every build is a commit whose message says what was built and by whom.
Visits, visitor books and heat are committed every couple of minutes. Every
three builds a world is tagged, Go-submodule style:
`worlds/the-lush/v0.3.0`.

### Version bands

At distance `d` from spawn a house shows the version a fraction
`d/(d + R)` of the way back through history (R = 6 houses). Each new tag
makes every place a little newer, while the far edge keeps reaching older
versions. Far bands are drawn with the biome as it was. When the architect
builds on a house seen at an older version, that version is **re-plunged**:
it becomes the house's present, keeping the visitor book and everything
learned.

### Evolving biomes and new worlds

* Before each tag, a world's biome drifts a little (`genesis::evolve`),
  steered by how people move through it. Far bands show older biomes.
* **Path-born worlds.** Clients send a path signature (turning per metre,
  straightness, speed, revisiting, heading). After 36 fresh cells of
  exploration, a new world is born from the blended signatures and the
  parent biome: winding and lingering paths make close, foggy, overgrown
  places, while straight, hurried ones make open, sparse ones. A door pair
  joins the house you were walking past to the new world's first house.
* **Terrain and places.** Each address is a house, tower, cave or stone
  arch, by the biome's `places` mix. All share the same door, so portals
  work identically.

### The liminal start

Worlds begin liminal (`procgen::liminal`, `start: "liminal"` in the
manifest): flat land, the same house on the same lawn everywhere. Terrain
never gains height; worlds made before this were migrated once. Only
**grown** houses (ones the architect has built on) differ:

* a storey for every two chambers, up to 9 (`shapeOf` and
  `buildHouse(storeys)` in `client/src/outdoor.js`);
* their own trees and ponds;
* place shapes (tower, cave, arch) on the lawn.

The in-page starter worlds are liminal too.

### Bikes

A bike from a rack is yours. It stays where you step off, and by the front
walk when you go inside (remembered in the browser's `localStorage`). **E**
near it gets back on. The scooter folds indoors and unfolds outside.

## 4. The architect

```
dwell ─┐
gaze ──┤                 ┌─▶ build timer (sketch 5–13 s, normal 12–30 s, rich 20–45 s)
use ───┼─▶ room attention┤
admire ┤                 └─▶ plan (off the main loop) ─▶ apply ─▶ commit ─▶ notify
asks ──┘
```

* **Signals.** These are dwell per chamber, gaze (seconds the view rests on
  each feature or thing, by id), use (talking, reading, playing, finishing a
  game), "more of this", and visitor-book requests (mined for tags and
  passed verbatim to creative architects).
* **Budget from pace.** A world's average seconds per house visit sets the
  budget. Under 25 s gets **sketch** builds: cheap, sooner, built in. Over
  90 s gets **rich** builds: the configured agent, more features and things,
  and detail on what is there. Near spawn, well-loved houses alternate a
  new chamber with a detail pass.
* **Providers.** `heuristic` is built in. `command` runs any program with
  the prompt on stdin and expects a JSON plan on stdout: `claude -p` makes
  Claude Code the architect, and a local-model wrapper or script works the
  same way. The plan shape is in `architect::prompt`; replies are parsed
  leniently.
* **Running agents** (`server/src/agent.rs`). Every agent run (architect,
  night shift, residents, sync merges) goes through here. It finds `claude`
  wherever it is installed (an app launched from Finder doesn't see your
  shell's PATH) and removes `ANTHROPIC_API_KEY` from its environment, so
  your Claude subscription is used, not an API key. Agents run in
  `.branches/agent`. The first time a house is about to grow, the app asks
  who should build (`architect::hold_for_setup`, the `setupAi` card) and
  holds builds for up to 3 minutes. `POST /api/app/agent/check` backs the
  "test it" button.
* **Story kit** (`server/src/stories.rs`). A cast of characters whose last
  lines point to other characters and worlds, story threads whose pages
  scatter across houses in order, and small games. Creative architects can
  continue these or invent their own.
* **Night shift** (`server/src/nightshift.rs`, off by default). Inside hours
  you choose, after a quiet period, the app picks a project: a game asked
  for in a visitor book, or the next idea (zoo tycoon, go-kart track, train
  set…). It writes a brief and runs your agent in that project's folder for
  up to N minutes. The resulting self-contained `index.html` opens in-world
  as an attraction booth in the most lingered-in house.

## 4b. Residents and the economy

`server/src/residents.rs`. Residents are persistent agents who live in the
worlds: a name, a persona, a goal, memories (capped) and a wallet of coins.
They are shown to clients as other people through the peers list.

* **Life.** They walk the streets (simple street-walking between houses,
  no collision), visit houses (their lingering feeds the architect like
  anyone's), write visitor-book notes, talk in chat, and commission builds.
  Talk to one with **E**; **R** opens the panel (fund, move someone in,
  list). The HTTP API is under `/api/residents`.
* **Thinking costs coins.** One coin stands for 1,000 tokens. A think runs
  your agent and is charged what it used, from the resident's coins. With
  no agent configured they still wander and talk by simple rules, free.
* **Treasury.** You fund it; everything paid out comes from it, so nothing
  is spent that you didn't put in. A commission costs at least 3 coins.
* **Rewards** flow to whoever commissioned a chamber: 2 coins when someone
  admires it (**F**), 0.5 when they use something in it, 0.3 a minute for
  lingering.
* **Between residents:** offers, tips and favours, all in the ledger.
* **Stored in git:** `residents/<id>.json` and `residents/economy.json`,
  committed with the rest, so they back up and sync like houses.

## 5. Onboarding

Each step appears only when it's needed:

1. **Open the web page and play.** No account; worlds run in the page.
2. **Write in a visitor book or claim a house.** One card explains that
   this needs Branches for Mac, with a download link and a *connect* button.
   After about eight minutes of play, a gentle corner card offers the same.
3. **Connect.** The page checks for the app on `localhost:7878`, imports
   what grew in the browser, remembers the app, and continues there. Later
   visits to the web page go to the app automatically.
4. **In the app, Connect GitHub.** It reuses the GitHub CLI's login if you
   have one, else device sign-in (when the build has an OAuth app id), else
   a pasted token. It forks `staplehorsey/branches` and keeps your worlds on
   the fork's `worlds` branch, tags included. `github::fork` waits until
   GitHub has actually made the fork, then pushes patiently. On connecting,
   worlds already on your fork are brought back and merged in by sync (a new
   Mac, or a reinstall). For the owner of the main repository the "fork" is
   the main repository itself. Backups show their errors and fall back to
   the git command line (`push_with_git`); **back up now** pushes at once.
5. **Share?** A pull request adds to `universe.json` (`api::github_share`,
   `github::share`): any house (claimed or empty, from its visitor book) or
   all your worlds. Once merged, a door appears in **the Commons**, a
   street of doors going west from The Lush's `-1,0`, six per house.
   Anyone can walk in; it reads straight from your repository
   (`client/src/githost.js`). Only you can change them.
6. **Your own page.** `server/src/pages.rs` builds the static site from the
   bundled client, adds `site.json` (`{"home": "gh://owner/repo@worlds", …}`),
   commits it to `gh-pages`, pushes to your fork and turns on GitHub Pages
   through the API. No Actions needed. The client reads `site.json` at boot,
   so your page opens your worlds. The main page also opens anyone's with
   `?at=gh://owner/branches@worlds/w/the-lush/0,0`. For the owner of the
   main repository the main page is left alone.

### Sync from upstream

`server/src/sync.rs`, behind **bring in changes** and **restore from my
fork** (`POST /api/app/sync`).

* Fetches the main world's `worlds` branch (or your fork's) and merges it
  into yours. `BRANCHES_UPSTREAM_URL` overrides the source.
* Houses changed on both sides go to your agent (the architect command,
  e.g. `claude -p`) with base, ours and theirs. Everything else uses a
  built-in JSON merge that keeps both sides' additions.
* The agent runs without holding the game lock: sync merges a snapshot,
  then does a quick second merge for anything that changed meanwhile.
* The fork's code is kept current through GitHub's merge-upstream API.

## 6. Code map

```
server/src/
  main.rs        app mode, routes, persistence + commit + tag loop, night-shift tick
  model.rs       primitives
  procgen.rs     hashing, themes, starter worlds, default rooms
  state.rs       shared state, settings, version bands, re-plunge, room views
  store.rs       the git repository: load, save, commit, tag, read at tag
  architect.rs   budgets, signals, heuristic and command architects, apply
  stories.rs     the story kit
  genesis.rs     path-born worlds, biome evolution, colour
  nightshift.rs  long projects by an agent while nobody plays
  github.rs      connect, fork, push, share
  agent.rs       finds and runs agents (subscription, no API key)
  sync.rs        fetch and merge from upstream or your fork
  residents.rs   residents, coins, treasury, rewards
  pages.rs       your own page: site build, gh-pages, Pages API
  api.rs / ws.rs HTTP and live layer
client/src/
  main.js        realms, spaces, doors, interaction, onboarding, loop
  portals.js     frame-linked doors, per-eye render-to-texture
  outdoor.js     terrain, places, version-band biomes, bikes, sky
  interior.js    chambers, things, gaze boxes
  things.js      characters, notes, games, attractions
  play.js        gaze, path signatures, exploration, tones
  xr.js          WebXR (Quest 3): rig, sticks, wrist card
  procgen.js     exact port of the generator
  localhost.js   the in-page host; githost.js: worlds read from GitHub
macos/           Branches.app launcher, Info.plist, icon
.github/workflows/mac.yml    universal app, rolling mac-latest release
.github/workflows/pages.yml  the web version on GitHub Pages
```

## 7. Next: meeting people, neighbourhoods, cities

This is the direction for the social layer: people meet organically, in
groups small enough to know each other, and density grows into cities over
time. None of it is built yet; it needs a small shared matchmaking service,
the first piece of hosted infrastructure.

1. **Opt-in location, coarse only.** Location is shared as a coarse
   geohash cell (around 20–40 km), never coordinates, and only when you
   choose. It decides where in the shared world you arrive: people who are
   near each other in life spawn near each other.
2. **Neighbourhoods of about Dunbar's number.** The shared world is
   partitioned into neighbourhoods, each sized for about 150 active people.
   A neighbourhood splits when it outgrows that and merges when it thins.
   Assignment prefers your geohash, then people you have played with.
   Everyone starts at uniform density.
3. **Invitations, safely.** You can leave a request on someone's door or
   in their visitor book to explore together. Nothing connects until they
   accept. On acceptance, the two apps connect peer to peer (WebRTC; the
   matchmaker only introduces them) and share presence in the same worlds.
   Blocking and leaving are one click; requests are rate-limited.
4. **Migration and building up.** You can move your home address toward
   where your friends are. Claimed houses can grow upward: floors, then
   your own designed home, with the same door contract at street level.
   Busy neighbourhoods accumulate height and density. Quiet ones stay
   suburban. Cities form where people choose to be, not where they are put.
5. **Scale.** The matchmaker holds only neighbourhood membership and
   pending invitations. World state stays in each player's git repository,
   and live presence goes peer to peer. That keeps the hosted part small
   as the platform grows.

## 8. Known limits

* Worlds read from GitHub (the Commons, anyone's own page) are read-only
  and have no live presence yet.
* Residents walk the streets in simple lines with no collisions, and live
  only in the app (not the in-page host).
* Thinking residents need an agent; without one they use simple rules.
* Version bands draw older biomes outdoors and older rooms indoors. Biome
  terrain amplitude is taken from the present version so the land stays
  continuous.
* The app is ad-hoc signed, not notarized: right-click → Open the first
  time.
* Movement is reported by clients and not yet validated.
