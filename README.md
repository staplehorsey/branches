# Branches

An endless, tranquil, multiplayer web world made of worlds joined by doors.

It starts in **The Lush**: an overgrown suburb of identical houses that goes
on forever. Every house looks the same from the street. Walk through any
front door, with no loading and no cut, and you're inside one of twelve
themed interiors. Stay a while and the **architect** starts building more
house behind the misty archway at the back. What it builds depends on what
you linger in, what you ask for in the visitor log, and what you mark as
*more of this*. When a place you spent time in grows, you get a
notification.

Some doors lead to other worlds: Dusk Orchard, Fog Pines, Moonlit Meadow,
and the island Salt Flat Noon. There are no loading screens: through a door
you see the room on the other side, you walk through, and the door you came
through is behind you. A door can also lead to another host
entirely. Anyone can run a host, claim addresses, and let their own agents
grow them.

See **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)** for the primitives,
protocol, architect, federation model and roadmap.

## Run it

```sh
cd server
cargo run --release
# open http://localhost:8080
```

| env | default | meaning |
|---|---|---|
| `PORT` | `8080` | listen port |
| `DATA_DIR` | `data` | where `universe.json` snapshots go |
| `CLIENT_DIR` | `client` or `../client` | static client files |
| `ARCHITECT_PACE` | `40` | seconds of attention the first growth costs (try `8` to watch it build) |
| `ADMIN_KEY` | none | bearer key with owner rights everywhere on this host |

Open two browser windows to see each other.

**No server?** Open `client/index.html` from any static file host (or add
`?offline` to the URL) and the worlds run inside the page, single-player,
saved in your browser. `scripts/build-page.sh <dir>` packages that as a
self-contained page. `scripts/export-offline.sh` refreshes the page's copy
of the world list from a running host. Every address is a URL:
`/w/the-lush/3,-2`.

## Controls

**WASD** walk · **shift** faster · **space** hop · **mouse** look (click to
capture) · **V** first/third person · **E** visitor log (at the lectern
inside, or the mailbox outside) · **F** more of this · **enter** chat ·
**L** worlds · **N** notifications · **H** wake up at home. If pointer lock isn't available, drag to look. On touch screens, use the
stick to walk and drag to look.

## Let an agent build

Claim an address from its visitor log, then copy your token from the
owner tools. Your agent can use the same HTTP API the client does:

```sh
TOKEN=<player>:<secret>
H=http://localhost:8080/api/worlds/the-lush/rooms/3/-2

curl -s $H | jq '.leaning, .chambers[].name'                 # look
curl -s -XPOST $H/log -H "authorization: Bearer $TOKEN" \
  -H 'content-type: application/json' \
  -d '{"kind":"request","text":"a quiet pool with koi"}'      # ask the architect
curl -s -XPOST $H/features -H "authorization: Bearer $TOKEN" \
  -H 'content-type: application/json' -d '{"tag":"water"}'    # build directly
curl -s -XPOST $H/portals -H "authorization: Bearer $TOKEN" \
  -H 'content-type: application/json' \
  -d '{"target":"https://friend.example/w/their-world/0,0","label":"a friend","slot":0}'
```

## Tests

```sh
cd server && cargo test
```
