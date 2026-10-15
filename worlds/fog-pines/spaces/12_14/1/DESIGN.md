# THE MIDDLE WATCH — design

## The idea

**North Atlantic, the night of 31 August – 1 September 1985.** Aboard the research ship R/V *Knorr*, inside a steel control van bolted to the deck, the midnight-to-four watch sits in the dark watching a black-and-white video feed from *Argo*, a camera sled towed on a cable two and a half miles below. They have been looking for the *Titanic* for weeks. Almost everything that crosses the screen is mud, rock, a fish, somebody's beer can. Each one must be dealt with by the same small ritual — **freeze the frame, print a still, pin it on the board under the right heading, write it in the log** — and then the sled flies on.

The player is the watch-stander on the logbook. They sort the bottom of the ocean, one contact at a time, onto a corkboard with four columns: **ROCK · LIFE · JUNK · ?** The **?** column is empty, and that empty column is the desire of the whole piece. Then a strip of riveted plate. Then a boiler with three furnace doors. You hold the still up against a 1911 shipbuilding plate, they match, you cross out the **?** and write the ship's name, and you phone the galley to wake the chief scientist.

The van erupts. Then, about an hour later, someone looks at the clock: 02:14. She sank at 2:20. The noise stops. You walk out of the van onto the fantail, where the cable runs off the A-frame down into black water. You stand at the rail for the minute. Then you follow the cable down. That's the way onward.

**The beauty I'm building toward:** the thermal still. A video printer that whirrs and pushes out a curl of warm, grey-on-cream paper, an image of the deep sea about the size of a postcard, which you pin to a corkboard by hand. A hundred of them already line the board, and they're all mud. The whole epic search lives in a kitchen-table gesture: *is this a rock or not?* You keep asking that until one night the answer is no. Then the turn: getting what you wanted, cheering, and remembering what it is you found. The fourth column fills up, and then everyone goes quiet.

Tone: grounded, specific, a little funny (the board is labelled in marker by tired people), then plain and solemn at the end. Never mystical.

## Why first-person 3D

This needs bodies in a cramped room. You turn from monitor to printer to board to logbook. That's the ritual, and it's physical. There's also a world inside the world: the monitor shows a **second, live 3D scene** of the seafloor sliding under the sled's lights. That inner world has to read clearly as *the thing we're all looking at* (the dreamer asked for "a visual tie-in that jumps to the world of the game"). So the monitor is big and bright and the only strong light source in the van, its image comes off the printer as paper you hold, and at the end you step outside and *see the cable* that connects the van to that world. Then onward goes down the cable.

## Layout (metres; y up; player eye 1.6)

Use `/kit/space.js` (`createSpace({ background:'#05070a', eye:1.6, speed:2.2, reach:2.2 })`).

### The control van (interior): `s.room({ x:0, z:0, w:6, d:2.6, h:2.3, wall:'#3a3f44', floor:'#2a2c2e', ceiling:'#2f3236' })`
A shipping-container-sized steel box. The long axis runs along X (−3 … +3). z = −1.3 is the "console wall", z = +1.3 is the "board wall".

- **Player start:** (−0.6, 0, 0.4), facing the console wall (−Z), looking straight at the big monitor.
- **Console (z ≈ −1.0):** a desk box from x −2.4 … +1.6, height 0.78, depth 0.6, dark grey-blue `#2d3a48` with a lighter top `#4a5866`. On it and above it:
  - **ARGO MONITOR** (the hero): a box bezel 1.3 w × 0.95 h × 0.35 d, centred at (−0.6, 1.45, −1.1), charcoal `#1a1a1a`. The screen face is a plane 1.16 × 0.86 just in front of the bezel, textured with the **render target** of the inner seafloor scene (below). `MeshBasicMaterial` so it glows regardless of lighting. Add a `PointLight('#9fc8c0', 0.9, 4)` at (−0.6, 1.4, −0.6) so its glow spills on the desk and the player.
  - Two small **auxiliary monitors** at x = −1.75 and x = +0.55 (0.5 × 0.4), showing static green text via CanvasTexture: left: `SONAR  —  PORT / STBD` with a slowly scrolling noise column; right: `NAV  41°43'N  49°56'W` plus a blinking cursor. Decorative only.
  - **FREEZE button:** a red cylinder (r 0.05, h 0.03) on the desk at (−0.9, 0.80, −0.85), on a small grey plate labelled `FRAME HOLD`.
  - **Taped procedure card:** a small cream plane on the desk at (−0.3, 0.791, −0.8), tilted toward the player, CanvasTexture with handwritten-looking text (see Documents).
  - **Video printer:** a beige box 0.36 w × 0.14 h × 0.3 d at (0.55, 0.85, −0.85) (sits on the desk, below the right aux monitor), front slot facing +Z, a green LED. `Sony` is not written on it; a label reads `VIDEO PRINTER`.
  - **Coffee mug** at (−1.6, 0.83, −0.8): white cylinder with a chip. Usable (flavour).
  - **Wall phone:** at the right end of the console wall, (1.4, 1.35, −1.27), a beige handset box on a cradle with a coiled cord (a short tube is fine). Label `GALLEY / BRIDGE`.
- **Back of the van (x = +2.0 … +3):** the **door** in the end wall x = +3: a steel slab 0.9 w × 2.0 h at z 0, a handle, closed and inert until the end. A **wall clock** above it at (2.98, 2.0, 0) facing −X: round white face, black hands, it *runs* (see Clock). A coat hook with a yellow oilskin on it (decor).
- **Corkboard (board wall, z = +1.27):** a cork-coloured box `#b48a5a`, 2.6 w × 1.2 h × 0.04, centred at (−0.4, 1.5, 1.27), facing −Z (toward the player when they turn around). Four column headers along the top, written in black marker (CanvasTexture on the board top strip): **ROCK**, **LIFE**, **JUNK**, **?** Each column is 0.65 wide. Columns ROCK, LIFE and JUNK are already **crowded**: about 30 small grey pre-pinned stills each (0.12 × 0.09 planes, random slight rotation, cream `#e9e1cc` with a grey smear texture, red pin dots). The **?** column is **completely empty** apart from one pin with nothing on it. Make four invisible "target" boxes, one per column (0.6 × 1.0 × 0.1), registered as things for aiming.
- **Shipbuilder plate:** taped to the board wall at (1.4, 1.5, 1.27), 0.45 × 0.6, an old yellowed page `#d8c9a0` showing a line drawing of a single-ended boiler: a big circle with three smaller circles (furnace doors) across its lower half, rivet dots around the rim. Caption: `THE SHIPBUILDER, 1911 — SINGLE-ENDED BOILER`. Drawn in CanvasTexture.
- **Logbook:** an open green-cloth ledger on a narrow shelf under the board, (−1.9, 1.0, 1.05), 0.42 × 0.03 × 0.3, cream pages, pencil beside it.
- **Lighting:** very low. `AmbientLight('#20303a', 0.35)`, one dim red work light (`PointLight('#ff3a2a', 0.5, 5)` at (1.8, 2.1, 0), the night-watch red) and the monitor's blue-green glow. The board must be readable: add a small warm clip lamp over the board (`SpotLight('#ffd9a0', 1.2, 4, 0.7)` at (−0.4, 2.2, 0.6) aimed at the board centre).

### The fantail (exterior, only reachable at the end)
Past the door at x = +3 is open deck: a plane 10 × 12 at y=0, `#4b4f52`, from x = 3 … 13, z −6 … 6. Rails: thin boxes at height 1.0 around the deck's three outer edges, with the stern rail at x = 12.5. A large **A-frame** (two orange-yellow `#d9a12a` box legs 0.25 × 6 tall, angled, joined at the top) straddles the stern at x = 12. A sheave (torus) hangs from its top at y ≈ 5.5. The **cable**, a thin bright line (cylinder r 0.02, `#c8c8c0`), runs from a winch drum (cylinder r 0.6, length 1.4, at (6, 0.8, −3)) up to the sheave and then **straight down** past the stern into the water and out of sight.
- **Sea:** a large plane 400 × 400 at y = −2.5 beyond the stern, dark `#081018`, vertex-displaced gently over time (slow swells) with a faint specular from a low moon `DirectionalLight('#9aa8c0', 0.4)`. A few hundred star points on a sky sphere. A deck floodlight on the A-frame (`SpotLight('#fff2d0', 2, 20)`) pointing down at the water where the cable enters, making a pale oval with the cable vanishing into it.
- The whole exterior exists from the start (built behind the closed door); walls stay solid until the door opens. When the door opens, remove the door's wall segment from `s.walls`.

### The inner world (on the monitor)
A separate `THREE.Scene` + `PerspectiveCamera(50, 4/3)` rendered each frame into a `WebGLRenderTarget(512, 384)` that textures the monitor screen. It's black except what the sled's lights reach:
- Seafloor: a 60 × 400 plane with gentle noise displacement, colour `#7a8070`. Fog `FogExp2('#000000', 0.09)`.
- Sled camera: 8 m above the floor, looking down at about 55°, moving slowly along −Z (1.2 m/s) when "flying". A `SpotLight('#e8f0ff', 3, 30, 0.6)` attached to the camera pointing where it looks.
- Post-processing look (done cheaply): render the screen plane with a small custom ShaderMaterial that converts the render-target colour to luminance, tints it pale green-white `#d8f0e0`, adds horizontal scanlines (`0.85 + 0.15*sin(uv.y*384*3.14)`), a little animated noise, and a slight vignette. Over this, draw the **overlay text** with a second transparent CanvasTexture plane 1 mm in front: top-left `ARGO`, top-right the clock `00:41:07`, bottom-left `DEPTH 3781 m   ALT 8.2 m   HDG 184`, bottom-right `CAM 2`. When frozen, a blinking `FRAME HOLD` appears top-centre.
- Scattered small pebbles and a few faint drag marks pass continually, so the floor visibly moves.
- **Contacts** are built as small grouped meshes (see Contacts) placed ahead on the floor at intervals of about 14 m. When a contact reaches screen centre, the Pilot holds the sled ("Holding."), so the camera **stops**. The player never misses one.

If the render target fails for any reason, fall back to a CanvasTexture monitor that draws the floor as moving grey noise and the contact as a 2D silhouette. Never a blank screen.

## The people (voices only)

Three crew, heard as `s.say()` lines and labelled by role, never seen (they're "at the other consoles" behind your shoulder; the van is dark). Use roles, not real names:
- **PILOT**: flies the sled. Dry, careful, always quiet.
- **NAV**: navigator, the joker of the watch, tired.
- **THE CHIEF**: asleep in his bunk until you phone. Heard once, at the end.

Format lines as `PILOT: Holding.` One line on screen at a time, about 3.5 s each; queue them.

## The ritual (the core loop)

The procedure card says it: **1 FREEZE · 2 PRINT · 3 PIN · 4 LOG**. Each contact goes through exactly these steps, and each step only works in order. If the player tries a step out of order, the use() returns a short hint (below).

State per contact: `idle → holding → frozen → printed (in hand) → pinned → logged`, then the sled flies on to the next.

1. **Sled arrives.** The monitor shows the contact sliding in. When it's centred, the sled stops; `PILOT: Holding.` (for contact 1, `PILOT: Something. Holding.`)
2. **FREEZE** (E on red button). Plays a solid relay *clunk*. The screen image stops dead (stop rendering the inner scene; noise freezes too), `FRAME HOLD` blinks. Label becomes `the frame hold (held)`.
   - Out of order (nothing holding): `Nothing on the screen but mud. Wait for a contact.`
3. **PRINT** (E on the printer). It whirrs for 2.5 s (a rising/falling buzz, see Sound), and a **paper sheet** (0.16 × 0.12 plane, curled: a slightly bent PlaneGeometry with 8 segments and its vertices bent along a gentle arc) slides out of the slot over that time. The sheet's texture is a **copy of the frozen screen**: `renderer.readRenderTargetPixels` into a canvas, mapped to dark grey-brown `#3a3128` on cream `#efe6cf` with ordered dither, the overlay text included. When it's out, E on it again to **take it**. It goes to the inventory as e.g. `still: 00:41 contact` and a **held-paper** mesh appears bottom-right of view (parented to the camera, slightly curled, showing the same texture, about 18% of the screen). The label on the printer says `the printer (warm)`.
   - Out of order (not frozen): `The printer only prints a held frame. FREEZE first.`
4. **PIN** (look at a corkboard column and press E while holding a still). If it's the **right column**: the still flies (0.3 s tween) to a free spot in that column, rotates ±4°, a red pin dot appears, *tick* sound. The held paper vanishes from view and inventory. If **wrong**, it stays in your hand and a crew member says that contact's correction line (below). Hovering over a column with a still in hand shows the label `pin under ROCK` / `LIFE` / `JUNK` / `?`.
   - Without a still: the label is just `the board — ROCK` etc., and E reads a little caption: `ROCK: a hundred and twelve stills of rock. Someone has written "STILL ROCK" on one.` / `LIFE: fish, sea cucumbers, brittle stars. One fish is labelled GERALD and has been pinned three times.` / `JUNK: beer cans, a boot, a lot of coal clinker. Steamers have crossed here for a century.` / `?: Empty. One pin, waiting.`
5. **LOG** (E on the logbook after pinning). The logbook entry for that contact is written (show the whole log page in `s.read('Watch log — Knorr, Argo, 31 Aug / 1 Sep 1985', …)` with the new line added at the bottom; previous lines above). On closing, `PILOT: Releasing.` The frame-hold unblinks, the sled flies on.
   - Out of order: `Pin it first. The log follows the board.`

Make the procedure-card text available on E too (it's the first thing to try).

### The contacts (in order) with truth, correction line, log line

Clock times are what the overlay shows when that contact appears.

| # | Inner-scene mesh | Correct | If pinned wrong (spoken) | Log line |
|---|---|---|---|---|
| 1 | **Pillow-lava boulder**: a 1.2 m lumpy dodecahedron, noise-displaced, grey | ROCK | `NAV: That's a rock. I know, I know. They all look like something at this hour.` | `00:38  Boulder, pillow basalt. ROCK.` |
| 2 | **Rattail fish**: a long tapering cone body, big head, thin tail, hovering 0.3 m off the floor, its tail swaying slowly until FREEZE stops everything | LIFE | `PILOT: It has a face. Life.` | `00:44  Grenadier (rattail), ~40 cm. LIFE. Not Gerald.` |
| 3 | **Beer can**: a small bright cylinder lying on its side, a dull red band | JUNK | `NAV: Seventies can. The Titanic did not carry pull-tabs.` | `00:49  Beer can, pull-tab type. JUNK.` |
| 4 | **Dropstone**: a single angular granite rock sitting alone in the soft mud, with a little crater around it | ROCK | `PILOT: Dropstone. An iceberg carried that here and let go of it. Rock.` | `00:53  Dropstone, granite, in soft sediment. Rafted by ice. ROCK.` |
| 5 | **Lump of coal**: a small black faceted chunk, then a second and third trailing behind | JUNK | `NAV: Coal's not proof. Every steamer since 1850 has shovelled clinker over the side. Junk. …Keep an eye out, though.` | `00:58  Coal, several lumps, in a line. JUNK (?).` |
| 6 | **Riveted plate**: a twisted, bent rectangle of dark metal 1.5 m long, a double row of small round rivet heads along one edge, partly buried | ? | If pinned anywhere but ?: `PILOT: Rocks don't have rivets. Put it under the question mark.` | `01:02  Metal plate, riveted, man-made, large. ?` |
| 7 | **The boiler** (see below) | special | — | — |

After contact 5 is logged: `PILOT: Lot of coal. Debris trail, maybe. Or a coal barge's lunch.`
After contact 6 is pinned (the first still ever under ?): `NAV: …First one in that column in six weeks.` After it's logged: `PILOT: We don't wake him for a plate. Releasing. Coming up on something big.` For contact 7 the sled's lights first catch a **curved wall** looming up out of the black. The camera slows earlier (2 m out) and holds with the object filling the frame.

### Contact 7: the boiler
Inner-scene mesh: a large cylinder (r 2.4, length 3.5) lying on its side with its **round end facing the camera**, a dark rusty grey `#6a5a4a`, a ring of rivet bumps (small spheres, 48 of them) around the rim, and **three smaller round furnace doors** (discs r 0.45, slightly proud) set across the lower half of the face, evenly spaced. A few icicle-like rust drips (thin cones, "rusticles") hanging from the top edge. It sits half-buried at a tilt. Put a soft rim of the sled light on it.

On arrival: `PILOT: Holding. …Holding.` then `NAV: What is that.`

- **FREEZE** as usual.
- **PRINT** as usual; you take the still `still: 01:05 — the round thing`.
- If you try to **pin it** to ROCK/LIFE/JUNK: `NAV: Don't you dare.` To **?**: `PILOT: Not yet. Check it against the plate first. If we wake him, we're sure.`
- **Compare** (E on the Shipbuilder plate while holding the still): open `s.read('Holding the still against the plate', …)` with:

  > The still on the left, the 1911 page on the right.
  >
  > A round face. A ring of rivets around the rim.
  > Three furnace doors across the bottom: one, two, three.
  >
  > The plate: a single-ended Scotch boiler, three furnaces, 15 ft 9 in across. Twenty-four double-ended ones and five single-ended ones were fitted in the ship.
  >
  > They match.

  On close, inventory item renames to `still: 01:05 — boiler`, and `NAV: …Oh.` / `PILOT: Okay. Okay. Pin it.` 
- **PIN** under **?**: the still flies onto the empty ? column, the *only* still there besides the plate. Then: a **marker stroke animation** over the `?` header: redraw the header canvas over ~1.5 s, a black line striking through the `?`, and beneath it, hand-lettered, **TITANIC**. A felt-tip squeak sound. No speech for 1.5 s. Then the second print (see "The carried print" below). Once you've taken it: `NAV: Write it in the book. Write it in the book.`
- **LOG**: the page appears with the new line:
  `01:05  BOILER. Riveted, three furnace doors. Matches Shipbuilder plate. TITANIC.`
  On close: `PILOT: Somebody wake him.`
- **The phone**: (E on the wall phone, only now; earlier it says `GALLEY / BRIDGE. Nobody calls the galley at one in the morning for nothing.`) Opens a short read:

  > You lift the handset. The cook answers through a mouthful of something.
  >
  > "Galley."
  >
  > "Go and wake the chief. Tell him we've got something."
  >
  > "What kind of something?"
  >
  > You look at the board.
  >
  > "Tell him it's a boiler."

  On close: a pause of 2 s, then the van **erupts**. Raise a crowd-murmur/cheer sound bed, several overlapping lines: `NAV: A boiler! We've got a boiler!`, `PILOT: Hold her steady, hold her steady—`, and then (heavier footsteps, the door rattles but stays shut) `THE CHIEF: Where? Show me. Show me.` The red work light flickers up to full warmth `#ff7a5a`. The monitor unfreezes and, slowly, the sled creeps along the side of the boiler and past it, and **more shapes** come into the lights: a bent railing, a porthole ring, a long plate. (Simple meshes, ~8 of them, laid in a line after the boiler.) This runs for ~12 s.

## The turn: 02:14

After the eruption, a full-screen fade to black for 1.2 s with centred text (a CSS overlay, Georgia italic, cream): **An hour of shouting later.** Fade back in. The van is the same but the crowd sound has thinned to two or three voices laughing. The **wall clock** now reads **02:14** and its second hand is visibly moving.

`NAV: Somebody should get the champagne out of the—`
`PILOT: What time is it.`
(pause 2 s)
`NAV: Two fourteen.`
`PILOT: She went down at twenty past two.`

The crowd sound fades out completely over 3 s. Just the hum of the electronics and the ship. The monitor keeps showing the boiler, still, in the sled lights. Then:

`THE CHIEF: Everyone out on the fantail. Leave the cameras running.`

The door at x = +3 swings open (rotate 100° over 1.5 s, a heavy steel *clank* and a gust of sea sound). Cold blue night light falls in. Its label: `out to the fantail`.

From here the game clock runs at **real time × 6** (02:14 → 02:20 takes 60 s), shown on the wall clock and, outside, on nothing (you just know).

## The fantail

Walk out. The deck is wet, dim, lit by the floodlight on the A-frame. The sea is huge and black; the swells move. The cable comes off the drum, up over the sheave, and drops into the pale oval of water. Five or six dark standing figures (simple capsule silhouettes, `#111`) are lined up along the stern rail, one gap among them at x = 12.2, z = 0, which is your place, marked by nothing but the gap.

When you step within 1.2 m of the gap: the player's movement is gently disabled, the camera eases to face the cable (+X, slightly down) over 1.5 s. Sound: only sea, the creak of the sheave, the cable ticking. No text for 6 s. Then one centred caption, no speaker:

> **02:20.** Twelve thousand five hundred feet down this cable, the boiler sits with its three doors shut, where it came to rest seventy-three years ago tonight.

Wait 6 s. Second caption:

> Nobody says anything. That is the service.

Wait 5 s. Then movement returns, and the cable where it enters the water gains a very soft **light running down it** (a short bright segment that travels down the cable from the sheave into the sea every 2.5 s, like a bead of light), and the cable's label becomes `follow the cable down`. Its use(): `s.say('You keep your hand on the cable all the way down.')`, then after 1.5 s `s.onward()`.

**The carried print.** Right after the TITANIC marker stroke, `PILOT: Print another. One for the board, one for us.` The printer whirrs by itself (same 2.5 s animation, same boiler image) and its label becomes `take the second print`. E adds `the boiler still (thermal print, still warm)` to the inventory (the kit calls `branches.carry` for you). The LOG step for the boiler only unlocks after you've taken it (if not: `Take the second print first.`). That item stays in the inventory to the end and goes onward.

## Other small interactions (flavour, all optional)

- **Coffee mug:** first E: `Cold. It's been cold since eleven.` Second: `You drink it anyway. That's the watch.`
- **Aux sonar monitor:** `Side-scan. Mud to port, mud to starboard. Mud the whole way to Newfoundland.`
- **Aux nav monitor:** `41°43' N, 49°56' W. Somewhere under here, by everyone's arithmetic but the ocean's.`
- **Oilskin:** `Somebody's yellow jacket. It smells of the North Atlantic, which smells of diesel.`
- **Door (before the end):** `Locked against the weather. The watch stays in the van.`
- **Wall clock (before the end):** `00:41 and some seconds.` (reads live).
- **Pre-pinned stills** on the board (one random one, registered as a single thing "the stills"): `A hundred grey postcards of nothing. Each one was somebody's twenty minutes of hope.`

## Documents (exact text)

**Procedure card** (taped to the console; E reads it; also drawn on its texture):
```
MIDDLE WATCH — CONTACT PROCEDURE
1  FREEZE   (red button)
2  PRINT    (printer — take the still)
3  PIN      (board — right column!)
4  LOG      (book — time + what it is)
Then let her fly.

Do NOT wake the Chief for a rock.
```

**Log page** (top lines already written, in pencil; new entries appended):
```
WATCH LOG — R/V KNORR — ARGO TOW 23
31 AUG 1985

23:12  Mud.
23:40  Mud, ripple marks.
00:05  Fish (Gerald?).
00:20  Mud. Coffee.
```
(Then the player's entries as above.)

**The Shipbuilder plate** (E without a still): `The Shipbuilder, special number, 1911. A line drawing of a single-ended boiler: round face, rivets round the rim, three furnace doors in a row. Someone has pinned it here in July "for reference", and someone else has written underneath: "optimist".`

## Opening

Fade in from black over 1.5 s. The van is dark; the monitor glows. The clock overlay reads `00:36`. The seafloor is sliding by. Opening caption (centred, 5 s, CSS overlay):

> **North Atlantic, 1 September 1985.**
> Middle watch, 00:00 to 04:00, in the Argo van aboard R/V *Knorr*.
> Two and a half miles under you, a camera sled is flying over the mud.

Then `NAV: Your turn on the book. Card's on the desk.` After 2 s, the first contact slides into view. If the player hasn't pressed FREEZE 8 s after `Holding.`: `PILOT: Frame hold's the red button. Then the printer.`

General idle hint: if no step is taken for 20 s during the loop, the crew repeats the next step name: `NAV: Print it.` / `NAV: Board.` / `NAV: Book.`

## Timing

7 contacts × ~22 s ≈ 2.5 min + boiler compare/phone ~40 s + 02:14 scene ~25 s + fantail ~35 s. Onward becomes available at about 3.5–4 minutes. Clock during the loop: each contact jumps the clock to its listed time when it appears; between, it ticks 1 game-second per real second.

## Colour

- Van: gunmetal `#3a3f44`, night red work light, monitor green-white `#d8f0e0` as the dominant glow. Paper: cream `#efe6cf` with brown-grey ink `#3a3128`. Cork `#b48a5a`. Marker black `#111`. The one warm accent: the Shipbuilder page's yellow `#d8c9a0`.
- Fantail: blue-black sea `#081018`, sodium-white floodlight `#fff2d0`, the A-frame in safety orange-yellow `#d9a12a`, cable pale steel.

## Sound (WebAudio, synthesised; start on first click/pointer lock)

- **Van bed:** low electrical hum (60 Hz sine + 120 Hz at low gain) and the ship's engine (brown noise, lowpass 200 Hz, slow 0.2 Hz amplitude wobble). Quiet.
- **Freeze:** a relay clunk: short noise burst through a lowpass 800 Hz, 40 ms, plus a 90 Hz thump.
- **Printer:** 2.5 s of a buzzing square wave ~440 Hz, very low gain, amplitude-modulated at 30 Hz (the print head), with a soft ratcheting tick at 12 Hz.
- **Pin:** a crisp tick (a 3 kHz sine blip, 15 ms).
- **Pencil on log:** filtered noise scratches, three short strokes.
- **Marker squeak:** 2 kHz-ish sine with fast random pitch wobble, 1.2 s, in strokes.
- **Eruption:** layered noise bands with a jagged envelope (cheering), 12 s, then fading.
- **Fantail:** wide noise through a lowpass that sweeps slowly (the sea), a creak (slow sine glide 120→90 Hz) every ~7 s from the sheave, and a soft cable tick.

## What carries onward

`the boiler still (thermal print, still warm)`: a postcard-sized grey-on-cream image of a round riveted face with three furnace doors, timestamped 01:05.

## First thing to try

Look at the monitor, wait for `PILOT: Something. Holding.`, and press **E on the red FRAME HOLD button**. The procedure card next to it tells you the rest.

## Robustness notes for the builder

- All step gating via a single `state` object: `{ contact: 0..6, step: 'flying'|'holding'|'frozen'|'printing'|'printed'|'held'|'pinned', compared:false, phase:'loop'|'erupt'|'turn'|'fantail'|'onward' }`.
- The held-paper overlay and inventory label must both clear on pin.
- Raycast targets for board columns sit 2 cm in front of the board so the pre-pinned stills don't steal the aim.
- If the readPixels copy fails, draw the still from a 2D silhouette of the contact on a dithered grey ground.
- Never let the inner scene stall: if a contact mesh is missing, skip it.
