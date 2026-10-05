# Branches architecture

Branches is an endless, multiplayer web world made of many smaller worlds
joined by doors. Anyone can host a world, grow it with their own agents, and
link it into the graph. This repo holds the **reference host** (Rust) and
the **reference client** (three.js), plus the first five starter worlds.

Think of it like Minecraft servers stitched together by doorways. You walk
out of a pastel suburb, through a door in a velvet theatre, and into
someone else's fog forest. You stay the same person the whole way.

```
              ┌──────────────── the graph ────────────────┐
              │                                           │
   host A     │  the-lush ──door──▶ dusk-orchard          │     host B
 (this repo)  │     │  ▲               │                  │  (anyone's)
              │     │  └──door──── fog-pines              │
              │     └──door──────────────────────────────────▶ their-world
              │  salt-flat-noon  (island: doors in, none out)            │
              └───────────────────────────────────────────┘
```

---

## 1. Primitives

There are only a few primitives, and everything else is built from them.
Hosts may implement them however they like, as long as they speak the
protocol in §4.

| Primitive | What it is | Where it lives |
|---|---|---|
| **Address** | A place: `https://host/w/<world>/<x>,<z>`. Every place is a URL. | URL routing, client + server |
| **World** | A manifest: name, seed, generator (`kind` + `params`), portal policy, architect id, spawn. | `WorldManifest` in `server/src/model.rs` |
| **Room** | The state at one address: chambers, features, doors, visitor log, claim, learned weights. | `Room` in `model.rs` |
| **Door** | A graph edge whose target is an address. *Seamless* doors keep you in the same space; *branch* doors hand you to another world. | `Portal` (server), `DoorPortal` + branch doors (client) |
| **Player** | The one forced primitive: id, name, color, body, transform, velocity. It is identical in every world. | `Player` (server), `client/src/player.js` |
| **Visitor log** | An append-only log per room and per world: visits, notes, requests, growth. | `LogEntry` |
| **Architect** | An agent that turns attention and requests into growth. Pluggable. | `Architect` trait in `server/src/architect.rs` |
| **Policy** | Limits on growing the graph outward, set at world level and tightened per address. | `PortalPolicy`, `OutboundRule` |

Untouched rooms are not stored. They are synthesised deterministically from
`(world seed, x, z)`, which makes every world infinite at zero storage
cost. A room is only materialised once something happens to it: a visit, a
note, a claim, or growth.

---

## 2. The entry world: `liminal-houses@1`

The first generator is an endless grid of **identical houses** in a lush,
quiet suburb. Each house sits at the centre of a 24 m cell. Houses all
look the same from the street. Inside, each has a theme picked from the
room seed: Poolrooms, Moss Library, Cloud Nursery, Sunset Terrarium, Night
Aquarium, Vapor Mall, Tea Garden, Fern Cathedral, Arcade After Hours,
Snowglobe Den, Citrus Kitchen, and Velvet Theatre.

The five starter worlds share the generator but differ in mood (`params`):

| World | Mood | Doors out |
|---|---|---|
| The Lush | bright suburb, round trees, ponds | open, 4 per room |
| Dusk Orchard | endless golden hour, blossom trees | open, 3 |
| Fog Pines | white houses, pines, 20 m of fog | open, 3 |
| Salt Flat Noon | pink salt, palms, mirage | **closed**: an island you can only reach |
| Moonlit Meadow | night, glowing mushrooms, fireflies | allowlist: this host only |

Each world's spawn house is a hub: its entry hall has a door to every
other world, except on islands.

### Seamless doors (the pocket trick)

A house's interior lives in a **pocket** directly below it, at
`y = pocketY(x, z)`. That gives 25 stacked levels, so neighbouring pockets
never collide. The front door and the pocket's door are related by a
**pure vertical translation**. That makes them a portal pair:

* **Rendering**: the client draws the scene a second time from a virtual
  camera (the real camera shifted by the door offset) into a render target.
  It clips everything in front of the far door with a clipping plane. The
  doorway samples that texture in screen space, so you see the real
  interior from the street, and the real street from inside. The two
  nearest doors render live; farther ones show a glow. See
  `client/src/portals.js`.
* **Crossing**: when your eye crosses the door plane inside its rectangle,
  you are translated by the same offset. There is no fade, no load, no cut.
* **Spaces**: each render pass shows only one space (outdoors, or one
  pocket) and swaps fog, background and lights to match. That is also why
  interiors cost nothing while you are outside.

Interiors can be **larger than the house**. They grow as a chain of
chambers going away from the front door, each one built by the architect.
The last chamber always ends at a misty **frontier** archway where the
next chamber will appear.

### Branch doors: one physical space

The requirement is that everything feels like real life: no loading
screens, no fades, no menus that move you. Every world is somewhere you can
walk to, and every door has another side.

* **Doors come in pairs.** If house (x,z) in world A has a door to world
  B, then house (x,z) in B has the door back. Each world's spawn house is
  joined door-to-door with every other spawn house. When an owner opens a
  door to a local address, the host adds the door back in that house's
  entry hall. A door into a closed world (an island) is real, but its far
  side is **sealed**: you can walk in, not out.
* **Realms.** The client loads several worlds into one scene at once,
  each stacked 1,500 m above the last, far beyond the camera's reach. Only
  one space is drawn per render pass, so they never see each other.
* **Live views at any angle.** A door is a frame: an origin at the centre
  of the opening and a facing. Linking door A to door B gives
  `A.M = B.frame · rotY(π) · A.frame⁻¹`, which maps a side-wall door in
  one world onto an entry-hall door in another, at any heading. The portal
  renderer draws the far room through the opening using that transform,
  and crossing the opening applies it to your body: position, heading and
  momentum.
* **Pairing on approach.** When you enter a room, the client loads the
  world and the house behind each of its doors and links each door to the
  one that leads back. If the far side has no door back (an older room, or
  another host that doesn't add them), the client adds one behind you, so
  the door you came through is always still there.
* **Promotion.** When your body crosses into another realm, that realm
  becomes "where you are": its streets stream in around you, your live
  connection moves to its host, and realms you can no longer reach are
  released.

The one deliberate exception is **H** (wake up at home): a short blink to
your home world's spawn. It exists so nobody is stranded on an island.

---

## 3. The architect

The architect is an asynchronous builder. It runs on the host (one tick per
second) and never blocks a player.

```
attention ──┐        ┌─▶ build starts (room.building = {ready_at})
requests ───┼─▶ room ┤          … 12–45 s later …
admiration ─┘        └─▶ plan() → new chamber + features → notify investors
```

**Signals**

* **Dwell.** The server measures it from the live position stream: each
  second a player spends in chamber *c* adds attention to the room. It also
  reinforces the tags of that chamber, both in the room's weights and in
  the player's own taste profile.
* **Admiration** (`F`: "more of this") is a strong explicit signal for the
  current chamber's tags.
* **Requests** in the visitor log ("more water please, a pool with fish")
  are mined for tag keywords. Fresh requests are honoured directly half
  of the time.

**Growth.** When a room's attention passes
`pace × (1 + 0.6 × growth)`, a build starts. When it finishes, the
`Architect::plan()` implementation picks the new chamber's tag, name,
features and occasional touches to older chambers. It does this from a
**desire** distribution that blends the room's learned weights (55%) with
the taste profiles of the players who invested time there, weighted by
how long each stayed (45%). It also leaves a little novelty in.

**Notifications.** Everyone who invested at least 15 s in a room gets an
inbox entry when it grows. It is pushed live to any session they have
open, and stored otherwise.

**Pluggability.** `Architect` is a trait with one method,
`plan(&PlanContext, seed) -> Plan`. The reference `Heuristic` architect is
deterministic and free. A Claude-backed architect (see phase 1) implements
the same trait and can return the same `Plan`, plus generated names and
descriptions. Owners' own agents can also build directly through the HTTP
API (`POST …/features`), alongside the host's architect.

---

## 4. Protocol (`branches/0.1`)

Everything is JSON over HTTP, plus one websocket per player per world. CORS
is open, because clients are expected to hop between hosts.

### Discovery
* `GET /.well-known/branches.json`: protocol version, generators, worlds,
  endpoint templates.
* `GET /api/worlds`: worlds with tagline, online count, rooms grown, policy.
* `GET /api/worlds/{w}`: the world manifest.
* `GET /api/graph`: the host's view of the graph (worlds = nodes,
  stored doors = edges).

### Places
* `GET /api/worlds/{w}/chunk?x0&z0&x1&z1`: room summaries for streaming
  (max 625 cells).
* `GET /api/worlds/{w}/rooms/{x}/{z}`: the full room (chambers, features,
  doors, log, claim, growth progress, the architect's leaning, top
  investors).

### Acting
Auth is `Authorization: Bearer <player_id>:<secret>`, or `player`/`secret`
in the JSON body. A host admin can use `Bearer <ADMIN_KEY>`.

* `POST …/rooms/{x}/{z}/log` `{kind: note|request|praise, text}`
* `POST …/rooms/{x}/{z}/claim` `{title}`: at most 3 claims per player per
  world.
* `PATCH …/rooms/{x}/{z}` `{title?, theme?, outbound?}`: owner only.
* `POST …/rooms/{x}/{z}/portals` `{target, label, slot}`: owner only,
  checked against policy.
* `DELETE …/rooms/{x}/{z}/portals/{id}`
* `POST …/rooms/{x}/{z}/features` `{tag, chamber?}`: owner's agent builds
  directly.
* `GET|POST /api/worlds/{w}/log`: the world-level guestbook.
* `GET /api/players/{id}/inbox?secret=` and
  `POST /api/players/{id}/inbox/read`

### Live: `GET /ws/{world}`
Client → host: `hello {player, secret, name, color}` (first),
`move {p, ry, room, ch}` (~10 Hz), `chat {text}`, `admire {room, ch}`.

Host → client: `welcome {you, peers, unread}`, `join`, `leave`,
`peers {list}` (10 Hz), `chat`, `room {addr, summary, grew?, rebuild?}`,
`log {addr, entry}`, `notify {n}`, `admired {tags}`.

### Policy rules
* World `portal_policy`: `open | allowlist | closed`, plus `max_per_room`,
  `allow_hosts` and `allow_local`. Generator-placed doors don't count
  toward the limit.
* An address owner's `outbound` rule (`closed`, `max`, `allow_hosts`) can
  only **tighten** the world's policy, never loosen it.
* A `closed` world is an island: reachable, never leading out. A world
  with no doors pointing in is unreachable except by direct URL. Both are
  legitimate shapes in the graph.

---

## 5. Code map

```
server/                 Rust (axum + tokio), one binary
  src/main.rs           config, routes, background loops, snapshot persistence
  src/model.rs          primitives (serde types)
  src/procgen.rs        hashing, themes and tags, starter worlds, default rooms
  src/state.rs          shared state, room views, hubs for the live layer
  src/architect.rs      Architect trait, reference heuristic, learning, build tick
  src/api.rs            HTTP federation surface + policy checks
  src/ws.rs             presence, chat, dwell accounting, notifications
client/                 static ES modules, no build step, vendored three.js
  src/main.js           spaces, world loading, travel, input, loop
  src/portals.js        DoorPortal + render-to-texture portal renderer
  src/outdoor.js        streamed suburb: houses, hedges, trees, grass, sky
  src/interior.js       chamber chain, walls with doorways, frontier, branch doors
  src/features.js       one builder per architect tag, themed
  src/themes.js         the 12 interior themes and procedural surfaces
  src/player.js         the player primitive: body, controls, collisions, camera
  src/avatars.js        everyone else
  src/net.js / ui.js    hosts, identity, live socket; HUD and panels
```

Persistence in v1 is the whole universe snapshotted to
`DATA_DIR/universe.json` every 10 s and on shutdown. That is fine for one
host with thousands of rooms. Phase 4 replaces it.

---

## 6. Roadmap

### Phase 0: the entry world *(this commit)*
- [x] Reference host with five starter worlds and a federation-ready API
- [x] Endless identical houses with 12 themed interiors; seamless doors
- [x] Multiplayer by default: presence, avatars, chat
- [x] Async architect that learns from dwell, admiration and requests;
      notifications
- [x] Visitor logs per room and per world
- [x] Claims, owner tools, portal policies, agent-usable HTTP API
- [x] Cross-world and cross-host doors that are physically walkable both ways
- [x] The whole client also runs with no server (an in-page host), for static hosting and previews

### Phase 1: agents as first-class builders
- **MCP server** wrapping the HTTP API (`branches.walk`, `look`, `claim`,
  `open_door`, `build`, `read_log`), so anyone's agent can tend their
  addresses.
- **Scoped agent tokens** separate from the player secret, revocable and
  limited per address.
- **Claude architect**: implement `Architect` with an LLM call that gets
  the room, desire and recent requests, and returns a `Plan` with names,
  descriptions and new feature *recipes* (parameterised primitives), not
  only existing tags. Run it from a job queue so slow generations stay
  async.
- **Architect marketplace per world**: the world manifest names its
  architect, and owners can choose per address.

### Phase 2: real federation
- **Passport identity**: ed25519 keypairs instead of trust-on-first-use
  secrets, so you are provably the same player on every host. The player
  primitive carries a signed `{id, name, body}`.
- **Door handshake**: a host can accept, reject or rate-limit inbound
  links. Edges can be one-way or mutual. Signed portal records stop people
  forging edges.
- **Discovery**: crawl `/.well-known/branches.json` along edges to build a
  public map of the graph, finding islands and hubs. Visitor logs become
  the social layer for "who else shows up".
- **Safety**: moderation hooks for logs and names, host blocklists, and
  per-host rate limits.

### Phase 3: many generators, truly seamless
- `scene-json@1`: a declarative world format (primitives, materials,
  lights, doors) so a host needs no client code.
- Sandboxed **WASM generators** for procedural worlds, fetched from the
  host.
- **Avatar adapters**: let each world restyle the body while identity
  stays fixed, so crossing from one art style into another changes how you
  look, not who you are.
- Rotated and scaled portal transforms (general 4×4, not only
  translation), and recursive door views.

### Phase 4: scale
- SQLite/Postgres storage per host; rooms sharded by world.
- An interest-managed live layer (presence by region rather than by world),
  with horizontal scaling behind a sticky load balancer.
- Generated assets (meshes, textures, audio) stored in a CDN, produced by
  async jobs and referenced from features.
- Server-validated movement (speed and collision checks) so dwell, the
  architect's main signal, cannot be farmed.

---

## 7. Known v1 limitations
- Identity is trust-on-first-use per host (`player id + secret` in
  localStorage).
- Dwell is measured from client-reported positions. Speed is not validated
  yet.
- Visitor logs are unmoderated beyond length limits and a visit-entry rate
  limit.
- Through a door you see the room on the far side; that room's own doors
  show a glow until you step in (views are one level deep).
- Everything for a host runs in one process with JSON snapshots.
