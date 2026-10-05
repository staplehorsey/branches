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
   realms, doors,                    architect (heuristic | command)            (worlds branch)
   things, gaze, map                 night shift (agent, long projects)            │
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
.branches/                        private, never committed: players, settings, GitHub token
```

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
* **Terrain and places.** Rolling value-noise terrain with level pads. Each
  address is a house, tower, cave or stone arch, by the biome's `places`
  mix. All share the same door, so portals work identically.

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
   the fork's `worlds` branch, tags included.
5. **Share?** A pull request adds your worlds to `universe.json`. Once
   merged, a door to them appears in **the Commons** (the house west of The
   Lush's spawn). Anyone can walk in; it reads straight from your repository
   (`client/src/githost.js`). Only you can change them.

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

* Worlds read from GitHub (the Commons) are read-only and have no live
  presence yet.
* Version bands draw older biomes outdoors and older rooms indoors. Biome
  terrain amplitude is taken from the present version so the land stays
  continuous.
* The app is ad-hoc signed, not notarized: right-click → Open the first
  time.
* Movement is reported by clients and not yet validated.
