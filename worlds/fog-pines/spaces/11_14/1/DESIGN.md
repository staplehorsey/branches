# The Witness's Night — DESIGN

## The idea

Rome, about 30 BC, just inside the Esquiline gate, where two roads fork. Marcus Vergilius Eurysaces is a freedman, a baker, and a public contractor (*pistor redemptor*). He once turned a mill himself as a slave. Now he bakes bread for a whole quarter of the city under contract to the aediles. The contract is up for renewal. Before the aediles will renew it, the rules require **one batch made in a single night, from grain to loaf, in front of a witness from the aediles' office. The witness's hand is on every step, and the witness is not to sleep.**

**You are the witness.** You keep the four Roman night watches (*vigiliae*) beside a proud, exact man who has done this every night for twenty-two years. His wife Atistia used to sit up with him. She died this spring, and her ashes are in a stone breadbasket on the shelf. Over one night, grain turns into flour, flour into dough, and dough into loaves stamped with his name. Your official record of all this turns into something stranger: at dawn he shows you the plan for his tomb. It is a frieze of exactly the night you just witnessed, with the officials weighing the bread, and one of them is unshaven.

The tomb is real. It still stands at Porta Maggiore. Its sides are pierced with round holes like the mouths of kneading machines. The frieze shows grain, milling, sifting, kneading, shaping, the oven, and the loaves weighed in front of men in togas. The inscription reads *EST HOC MONIMENTVM MARCEI VERGILEI EVRYSACIS PISTORIS REDEMPTORIS APPARET*. The last word means "he serves the magistrates", and it can also be read as "it is evident".

**The beauty I'm building toward:** a man so proud of an ordinary procedure that he carved the weighing scene into his own grave. On the way there is one small piece of cause and effect. Before you cover the dough, he has you hold back a fist of it as leaven for tomorrow. Because of that, the twelfth loaf comes up light on the steelyard. "It does not go out under my name." You get to eat the evidence. Tomorrow's bread cost tonight's last loaf an ounce and a half.

## The coordinates, honestly
- **ancient:** Rome under the late Republic. Freedmen, aediles, the four watches of the night, a donkey mill, a horse-driven kneading machine, a steelyard, stamped *panis quadratus*.
- **pride:** Eurysaces's pride is in exactness. He went from slave at the mill, to donkey owner, to free man. He will not let a short loaf leave under his name, and he builds a tomb out of his trade.
- **a diorama you orbit:** the whole space is a tabletop model of the city quarter at night on a walnut plate with a brass rim. The bakery has its roof lifted off like a dollhouse. The moon is a brass disc on a wire arc.
- **clinical:** you are an official witness. Every object carries a museum specimen tag ("Fig. 6 — …"). Your report is a numbered record. Every test is a procedure with a criterion: the flour browns at a count of ten, the dent returns slowly, the loaf sounds hollow, the loaf weighs two pounds.
- **a city:** you see the quarter all around the bakery: other bakers' ovens glowing, the aqueduct, the gate, and at dawn eleven doorways lighting up as the bread arrives.
- **food and cooking:** the full chain from grain to loaf.
- **transformation:** grain becomes bread, night becomes dawn, the witness's report becomes a tomb frieze, and a slave at the mill becomes the man on the monument.
- **keep a vigil:** the four *vigiliae*. If you go idle, you doze off and he says "Witness."

## Medium and controls

This is an **orbited diorama**, not first-person. Do not use pointer lock. Do not use `/kit/space.js` for movement; write a small spherical-orbit camera instead.

- **Left-drag:** orbit around the target (azimuth and polar angle).
- **Wheel:** zoom.
- **Right-drag:** pan the target, clamped to within 6 units of the bakery centre.
- **Click** (a press–release that moves less than 5 px) on a hovered object uses it. **E** also uses the hovered object.
- **F:** reset the view to the bakery.
- **R:** open or close the witness's report.
- Never bind Esc, `=`, `-`, `G`, `?` or `` ` ``.

Camera details:
- Target defaults to (0, 1, 0).
- Radius runs from 7 to 34; default 17.
- Polar angle is clamped between 18° and 80° from vertical; default 52°.
- Starting azimuth looks from the +z side, from the gate toward the bakery, slightly to the right.
- On arrival, start at radius 32 and polar 35°, then ease to the default over 3.5 s (ease-in-out).
- Drag damping is about 0.12 lerp per frame.

**Hover:** raycast the interactive objects. The hovered object's tag appears as a paper museum label in an HTML overlay. It sits off to the side of the object with a thin dark connecting line drawn on a 2D canvas overlay. The label shows a "Fig. n — Name" line and one or two lines of clinical description. The cursor becomes a pointer.

**The current step:** the object(s) for the current step carry a small brass map-pin (a sphere r 0.12 on a 0.5 stalk) that bobs and pulses. The pins are the visual "try this first". If the player does nothing useful on a step for 30 s, Eurysaces repeats that step's hint line.

## Layout (world units ≈ metres in the model)

### Plinth and plate
- The plinth is a dark cylinder, r 15.6, height 4, top at y = −0.6, colour #1a1310.
- The plate is a walnut cylinder, r 15, height 0.6, top at y = 0, colour #4a3222. It carries a brass torus rim, r 15.05, tube 0.12, colour #b08a4a, metalness 0.8.
- **Engraving:** the plate's +z edge carries text as a canvas texture on a curved strip or flat plaque at the front, reading `ROMA · INTRA PORTAM ESQVILINAM · NOCTE` and a small scale bar marked `1 : 50`. At the end it changes to `· PRIMA LVCE` ("at first light").

### Ground
- A pale travertine/dust disc, r 14.6, at y = 0.01, colour #8a7d68.
- Streets are slightly darker strips, #6e6352.

### The bakery (centre)
The bakery is the cutaway building.
- **Floor:** 9 × 7, x −4.5..4.5, z −3.5..3.5, colour #a99a7e beaten earth.
- **Walls:** boxes 0.25 thick, ochre plaster #c79a5e, with a darker plinth band. They must be solid boxes so they are visible from both sides.
  - Back wall (z = −3.5): height 2.4.
  - Side walls: height 2.4. The left wall (x = −4.5) has a door gap from z = 0.6 to 2.2.
  - Front wall (z = +3.5): cut down to 0.5 high (the cutaway), with a gap from x = 1 to 2.6 as the street door.
- **Roof:** a red tile slab, 9.6 × 0.15 × 7.6, colour #9c4a2e with tile stripes. It is lifted off and leans against the outside of the back wall at about 70°, its bottom edge on the ground at z ≈ −4.2, like a dollhouse lid propped aside.

### Inside the bakery

| Fig | Object | Position | Build |
|---|---|---|---|
| 1 | Grain cart | (−3.4, 0, 2.2) just inside the left door | Box bed 1.6×0.35×0.9 on two disc wheels r0.35, raised by 0.35. Three sacks: squashed spheres r0.32, burlap #b39a6a. |
| 2 | Mill + donkey | centre (−2.6, 0, −1.8) | *Meta*: cone r0.55, h0.9, grey lava #4b4744. *Catillus*: hourglass of two cones r0.5, total h1.1, sitting on the meta. A wooden bar (box 2.6×0.1×0.1) passes through its waist. Donkey: grey-brown #7a6a5c, body box 0.9×0.45×0.35, head, ears, four legs. It is harnessed at the bar end and walks a circle r1.25 around the mill. Flour mound: a cone at the meta's base that grows from r0 to r0.55. |
| 3 | Sieve + three bins | table at (−0.6, 0, −2.7), 1.6×0.85×0.8 | Sieve: a torus ring r0.35 with a mesh disc, lying on the table. Three small boxes 0.4×0.3×0.35 in front of the table on the floor, each with a tiny painted tag: SILIGO, SECVNDARIVS, FVRFVR. The fill level inside each rises (inner box, white / cream / brown). |
| 4 | Contract | back wall, (0.8, 1.6, −3.36) | Bronze plaque 1.3×0.75, #8a6a3a. Dark incised lines suggest text. |
| 6 | Kneading machine + horse | centre (1.6, 0, −1.6) | Stone basin: open cylinder r0.65, h0.9, grey #7d7a74, with a dark inner floor disc. Vertical wooden shaft with 3 paddles inside. A horizontal pole runs out to the horse. Horse: bay #6b4630, body 1.3×0.55×0.42, legs. It walks circle r1.35. Dough: inside the basin, a cream #efe3c6 squashed sphere whose y-scale grows during rising. Cloth: a pale linen disc #ddd3bd that drapes over the basin when covered and domes up as the dough rises. |
| 7 | Leaven crock | shelf on back wall, (2.7, 1.15, −3.2) | Shelf: box 2.4×0.08×0.4 at y 1.1, x 2.2..4.4. Crock: cylinder r0.16, h0.28, terracotta #b5653a, with a linen tie. |
| 8 | Salt jar | (3.15, 1.15, −3.2) | Cylinder r0.12, h0.22, glazed grey-green. |
| 9 | Stone breadbasket | (3.95, 1.15, −3.2) | Cylinder r0.22, h0.26, travertine #d9cfb8, with a woven pattern (canvas texture of crosshatch). A small dried laurel sprig lies beside it. |
| 10 | Oven | (3.1, 0, 1.0) | Base box 2.6×0.9×2.6, brick #8d4b32. Dome: hemisphere r1.2 on top. The dome's material colour animates from soot #1e1612 to pale white-hot #e8dcc8 as it heats. The mouth is an arched opening facing −x (toward the room centre), 0.7 wide × 0.55 high, at the base top. Inside the mouth is a disc/plane with emissive fire colour (#ff6a1f, intensity follows heat), plus a PointLight (#ff7a30, distance 6) whose intensity follows heat. |
| 11 | Woodpile, rake, wet broom, peel | woodpile (3.9, 0, −1.0); tools leaning on oven side at (1.75, 0, 2.3) | Woodpile: 6 bundles (cylinders r0.18, length 0.8, bark #5a4430, tied). Rake: thin pole + crossbar. Broom: pole + dark wet bristle bundle. Peel: pole + flat paddle 0.5×0.35. |
| 12 | Water clock | (−4.0, 0, −0.4) | Tall terracotta jar (cylinder r0.3, h1.1). A float rod with a bronze pointer rises out of it against a small upright board marked with four bands: I, II, III, IV. A drip falls from a spout every 1.5 s (small sphere falling). The pointer climbs through the bands as watches advance. |
| 13 | Lamp | on the shaping table, (0.9, 0.93, 1.4) | A clay oil lamp (flattened sphere + nozzle). Flame: a small emissive cone. PointLight #ffb35a, distance 7, intensity 1.2, with gentle flicker. |
| 14 | Shaping table | (−0.2, 0, 1.6) | Long table 3.2×0.9×0.9, scrubbed wood #b89a72. The 12 loaves appear here in two rows of six. |
| 15 | Steelyard | post at (−2.2, 0, 2.7), height 2.0 | Bronze beam 1.3 long hung from the post's crossarm. A hook pan on the short arm and a sliding counterweight (small bronze bust shape = sphere). The beam tilts and settles per loaf. |
| 16 | Baskets | by the street door, (2.0, 0, 3.0) | Stack of 4 shallow wicker baskets (squat cylinders r0.4, h0.18, #a8834f). |
| 20 | The witness (you) | follows stations | Small figure 1.7 tall: off-white toga drape (#e8e2d4) over a cone/box body, sphere head with dark hair, holding a small wax tablet (dark box). The head droops when you doze. It stands about 0.9 from the active station, facing it. |
| 21 | Eurysaces | follows stations | Figure 1.72 tall: undyed tunic #cdbf9f, leather apron #6b4a30, forearms bare and floury (lighter), grey short hair, clean-shaven, a bit thick in the middle. He walks to each active station (lerp 1.5 u/s, simple bob). |

### The city outside the bakery (on the plate)
- **Streets:** a 2.5-wide ring street around the bakery, plus streets running out to the rim in four directions (along ±x and ±z).
- **Public basin** (Fig. 5): in the street at (6.0, 0, 2.6). It is a stone trough 1.3×0.6×0.7 #bdb4a2 with a pillar spout 1.1 tall and a water-surface plane #3d6f8a with a small animated drip. A bucket (cylinder r0.15, h0.25, wood) sits beside it.
- **Insulae:** about 22 blocks of boxes in the four quadrants beyond the ring street.
  - Footprints are 2–4 × 2–4 and heights 2.5–5.5 (2–4 storeys).
  - Plaster colours vary among #c9925a, #b9775a, #d8b98a and #a8664c.
  - Each has a red-tile top slab, a slightly darker base band, and rows of tiny dark window rectangles (thin boxes or canvas texture).
  - Keep everything inside r 13.5, and keep the gate corridor clear (x −2.5..2.5, z 6..15).
- **The eleven doorways:** pick 11 insula faces that look onto streets and give each a door, a box 0.5×0.9×0.05 #241a14. At dawn their material switches to emissive warm #ffb35a (intensity 0 → 1.2) as a basket arrives. Store their world positions for the basket paths.
- **Aqueduct** (Fig. 18): a row of 12 arches along x = −10.5, from z = −12 to z = +8. Each pier is a box 0.6×3.6×0.8, piers are spaced 1.8 apart, and a channel box 0.9×0.6×0.9 runs on top for the full length. Stone colour #b7aa8f.
- **City wall and gate** (Fig. 19): a wall box running from x = −9.5 to x = 9.5 at z = 9.8, 0.9 thick, 3 high, colour #a89a80. It has a gate of two arches (2 × 1.6 wide openings, 2.6 high) centred at x = 0. The gate piers are boxes and the arch tops are half-cylinders.
- **Beyond the gate:** two roads fork from the gate at (0, 0, 10.5), one to (−6, 0, 14.3) and one to (6, 0, 14.3). The fork point, (0, 0, 12.6), is where the ghost tomb will appear.
- **Other bakers' ovens** (Fig. 17): three small dome ovens, each a hemisphere r0.6 on a box, with an emissive mouth and a weak PointLight (distance 4).
  - "Lucius" at (9.5, 0, −8.5).
  - "Dionysios" at (−8.6, 0, 3.5), by the aqueduct.
  - "Unknown" at (6.5, 0, 7.5).
  - Their glow follows the schedule in the arc: Lucius lights in watch II, Dionysios in watch III, Unknown in watch IV.
- **Moon armature:** a brass half-torus arc (r 19, tube 0.06, #b08a4a) standing vertically in the x–y plane, rooted in two brass sockets on the plinth at x = ±19 (extend the plinth with two small brass posts there).
  - Moon: a sphere r0.9, emissive pale #dfe6f2, threaded on the arc. Its arc angle runs from 165° (east, low) at watch I start, to 120° at watch II, 75° at watch III, and 35° at watch IV. At dawn it sets to 5°.
  - Sun: a sphere r1.1, emissive #ffb070, on a second identical arc rotated 90° (the z–y plane). It is hidden until dawn, then rises from 175° to 145°.
  - Lerp positions over 6 s whenever a watch changes.

## Light and colour

- **Night** (watches I–IV):
  - Background #0b0e1c.
  - HemisphereLight sky #2a3560, ground #120c08, intensity 0.35.
  - DirectionalLight from the moon's position, #9fb3dd, intensity 0.45, casting shadows (map 2048, camera covering r 16).
  - Warm interior light comes from the lamp and the oven.
  - The plate is a lit island in a dark museum room.
- **Watch III** is the darkest: hemisphere 0.25, moon 0.35. The lamp matters, because it gutters (see below).
- **Dawn:** over 12 s, lerp scene.background #0b0e1c → #d99a86 (at 6 s) → #f0c8a8. Lerp the hemisphere sky to #f2c6a0 at intensity 0.8, and turn on a DirectionalLight from the sun, #ffc79a, intensity 1.4. Fade the moon light to 0, the other bakers' ovens to 0.2, and the oven glow to a low ember.
- **Palette:**

| Name | Colour |
|---|---|
| lamplight | #ffb35a |
| oven | #ff6a1f |
| terracotta | #b5653a |
| travertine | #d9cfb8 |
| flour | #f3ecdc |
| bronze | #a6793a |
| night | #0b0e1c |
| dawn rose | #f2a48a |

- **UI type:**
  - Specimen tags and report: a serif face (Georgia / "Iowan Old Style" / serif) on cream paper #f1e8d2, ink #2a2118, with a 1 px brown border and a slight drop shadow. Fig numbers in small caps.
  - Latin words in letter-spaced capitals.

## HUD
- **Top-left:** watch indicator, e.g. `VIGILIA III · third watch`, with a tiny CSS water-clock bar filling.
- **Top-right:** a small "Report (R)" tab that opens the report tablet.
- **Bottom-centre:** the subtitle bar for Eurysaces's lines. A dark translucent strip #120d0acc with cream text and the speaker label `EURYSACES` in small caps. Each line holds for max(3.5 s, 60 ms per char). Lines queue and never overlap. Also send the important ones (marked ★) through `branches.say()`.
- **Reading panel:** a centred paper sheet, max 620 px wide. It is used for the contract, the report and the tomb plan. Close it with a click outside it, with E, or with the same key. While it is open, orbit and idle are paused.
- **Choice buttons:** when a choice is needed (only at the weighing), two buttons appear in the subtitle bar.

## Idle / dozing (the vigil)
- Track the time since the last meaningful input: a click on any object, or a drag longer than 1 s. Panel-reading time doesn't count.
- If 25 s pass in watches I–IV with no panel open:
  - Start "dozing". Over 3 s a CSS vignette darkens the edges to 70 %, the witness figure's head droops forward 35°, and the lamp dims.
  - At 4 s into the doze, Eurysaces says one of these, in rotation:
    - "Witness."
    - "Witness. Eyes."
    - "You can sleep when it's in the oven. — No, you can't. Then either."
  - Any click or drag wakes you instantly.
  - `dozes += 1` (count each doze once). The report records it.

## The arc, step by step (with every line)

Lines marked ★ also go through `branches.say`. "Tag" is the hover label text.

### Arrival (≈ 20 s)
- Camera eases in. The water clock is at the start of band I.
- The moon is low in the east. The quarter is dark apart from window glints and the bakery lamp.
- Eurysaces stands by the cart; the witness figure stands just inside the door. The pin is on the grain cart.

Lines:
1. ★ "Rome, inside the Esquiline gate. The night before the bread contract is renewed. You are the aediles' witness. Drag to look around the city; click to act."
2. EURYSACES: "You're the witness. Good. Stand there. Witnesses who sit, sleep."
3. EURYSACES: "The contract's on the wall. It says your hand on every step. So: your hand. Grain's on the cart. Tip it in."

**Contract** (Fig. 4, click any time; the reading panel):
> **LOCATIO — CONTRACT**
> Let by the aediles of the Roman people to M. Vergilius Eurysaces, baker:
> I. Bread of second flour (*panis secundarius*) for the quarter within the Esquiline gate, daily, before first light.
> II. Each loaf of two pounds. Bound with cord, cut in eight, stamped with the contractor's name.
> III. Before renewal, one batch to be made in a single night, grain to loaf, before a witness of the aediles' office.
> IV. The witness's hand on every step.
> V. The witness not to sleep.

Tag: *Fig. 4 — Contract. Bronze. Terms of supply; renewal pending.*

### VIGILIA I — Milling and sifting (≈ 1.5 min)

1. **Grain cart** (Fig. 1). Tag: *Fig. 1 — Grain. Three sacks, Campanian wheat. Received at dusk.*
   - Each click lifts one sack in a 0.6 s arc into the catillus's top (the hopper). The sack disappears and a grain-gold disc fills the hopper a little more.
   - After 3 sacks: EURYSACES: "Three sacks. Dry. Smell that? No? You will."
   - The pin moves to the donkey.
2. **Mill and donkey** (Fig. 2). Tag: *Fig. 2 — Mill, lava stone, the upper stone turned on the lower. Power: one donkey.*
   - Click: the donkey walks its circle (one lap per 4 s) and the catillus rotates with the bar. A low grinding sound loops. The flour cone grows over 20 s.
   - Lines during the 20 s, spaced 6 s apart:
     - "Listen to it. Dry wheat grinds like that. Wet wheat whispers."
     - ★ "I turned that stone myself, before there was a donkey. Nine years. Then I bought the donkey. Then I bought myself. In that order. The donkey was cheaper."
     - "Your report should say: one donkey. Don't let them tell you it was two. It's never two."
   - At 20 s the donkey stops and the flour mound is full. The pin moves to the mound.
3. **Flour mound → sieve.** Click the mound: it shrinks while a white puff travels to the sieve.
   - The sieve tag becomes: *Fig. 3 — Sieve. Separates flour by fineness into three grades.*
   - The pin moves to the sieve.
   - EURYSACES: "Shake it. It'll tell you what you've got."
4. **Sieve, 5 clicks** ("shakes"). Each click gives a 0.4 s side-to-side jiggle, a soft sifting hiss, and each of the three bins fills by 1/5.
   - Bin tags:
     - *SILIGO — fine white flour. For the rich.*
     - *SECVNDARIVS — second flour. Coarser, sound.*
     - *FVRFVR — bran.*
   - After 5: EURYSACES: "Three grades. The contract says which. You choose; you're the hand."
   - Pins appear on all three bins.
   - **Choosing a bin** (click it):
     - SILIGO: "Siligo. Beautiful, and wrong. The contract says second bread. Give the city white bread at second-bread price and next year the aediles will ask why I don't always. Read the wall." Set `tookFine = true`. The step stays open.
     - FVRFVR: "Bran is for the donkey. He's earned it more than the aediles have." The step stays open.
     - SECVNDARIVS: "Second flour. Honest bread." The bin's flour arcs into the kneading basin, which shows a flour level.
   - **→ The watch advances.** The water-clock pointer rises to band II and the moon moves.
   - The HUD reads `VIGILIA II · second watch`. Lucius's oven lights.
   - ★ EURYSACES: "Second watch. Somewhere up the hill Lucius has just lit his oven. Too early. Every night."

### VIGILIA II — Kneading (≈ 1.5 min)
Pins go on the public basin, the leaven crock and the salt jar. These three can be done in any order. Floating over the basin is a small checklist tag that ticks as you go: `water 0/2 · leaven · salt`.

5. **Public basin** (Fig. 5). Tag: *Fig. 5 — Public basin. Aqueduct water, Aqua Marcia. Free to all.*
   - Click: the bucket fills (water rises, a splash sound) and the witness figure walks to the basin and back. The bucket is now "held"; a bucket icon shows in the HUD.
   - Then click the kneading basin to pour: a short water arc, a pour sound, `water 1/2`.
   - Repeat for 2/2.
   - On the first fill: EURYSACES: "The aqueduct brings that eleven miles so I can put it in bread. Two buckets."
   - If you click the basin again with the bucket already full: "It's full. Pour it."
6. **Leaven crock** (Fig. 7). Tag: *Fig. 7 — Leaven. Kept back from yesterday's dough, and yesterday's from the day before's. Age: undetermined.*
   - Click: a cream lump drops into the kneading basin.
   - EURYSACES: "The mother. Yesterday's dough, and the day before's, back and back. It was in that crock before I was free. Atistia fed it the nights I couldn't."
7. **Salt jar** (Fig. 8). Tag: *Fig. 8 — Salt. From the Ostia pans.*
   - Click: salt grains sprinkle in.
   - EURYSACES: "A fist. Not a handful, a fist. The aediles can't taste the difference. I can."
8. When all three are done, the pin moves to the horse. Tag: *Fig. 6 — Kneading machine. Stone basin, paddles on a shaft. Power: one horse.*
   - EURYSACES: "Now the machine. Half of Rome kneads with its feet. I have a horse and a stone. Write that down."
   - Click: the horse walks the circle (1 lap per 3 s) and the paddles turn. The ingredients become a cream dough (the flour level is replaced by the dough sphere, which slowly smooths, scale 0.4 → 0.6). The sound is a rhythmic slap. This takes 15 s.
   - At the end: ★ EURYSACES: "Before you cover it, take a fist back. Into the crock. Tomorrow's bread is in tonight's."
   - The pin moves to the crock.
9. **Crock again:** a lump arcs from the basin back into the crock, and the dough sphere shrinks very slightly. `leavenKept = true`.
   - EURYSACES: "Good. Now cover it."
   - The pin moves to the cloth, which is folded on the basin rim. Tag: *Linen cloth.*
10. **Cloth:** it drapes over the basin.
    - EURYSACES: "Now we wait. This is the part of the night nobody pays for."
    - **→ VIGILIA III.** The water clock reaches band III and the moon is at 75°. The hemisphere dims. Dionysios's oven lights.

### VIGILIA III — The vigil (≈ 2.5 min)
Two things run at once: **the rise** (a 90 s timer from covering, during which the cloth dome rises from 1.0 to 1.6 y-scale) and **the oven**. The lamp also gutters.

**Oven firing** (Fig. 10 tag: *Fig. 10 — Oven. Brick vault. Ready when the soot burns off the inside.*). The pins go on the woodpile and the oven.
- **Woodpile** (Fig. 11a, *Olive prunings, bundled.*): a click carries a bundle into the oven mouth. Heat jumps up, then decays.
  - Bundle 1: "Olive prunings. Burns hot, burns clean."
  - Each bundle adds +0.25 heat over 4 s and then burns down over 12 s. The fire glow follows the current burn; the **vault colour** follows the accumulated `fired` value (0 → 1, +0.25 per bundle, applied as the bundle burns).
  - If you click while the previous bundle is still blazing (under 6 s since the last one): "Not yet. Feed it when it asks. When the flame lies down." Ignore the click.
  - After the 4th bundle has burned for 8 s, the vault is fully pale. EURYSACES: "There. The vault's gone white. Rake it out." The pin moves to the rake.
- **Rake** (Fig. 11b, *Rake. Iron.*): the coals slide to the sides of the mouth (an ember sprite moves), with a scrape sound.
  - EURYSACES: "Coals to the side. Now the wet broom. It cleans the floor and calms it."
- **Broom** (Fig. 11c, *Broom, wet.*): a hiss and a puff of steam particles at the mouth. Start `swabTime`.
  - EURYSACES: "Test it. A pinch of flour on the floor. Count."
  - The pin moves to the oven (now tagged *Flour test.*).
- **Flour test** (click the oven):
  - Under 10 s since swabbing: a tiny white fleck darkens to black in 1 s. "Black at a count of two. It'd burn the bottoms and leave the middles raw. Wait."
  - 10 s or more: the fleck browns over 2.5 s, with an on-screen count shown as small Roman numerals I…X ticking above the oven. "Brown at a count of ten. That's the heat. Write it." `ovenReady = true`.

**Dough check** (the cloth/basin, pin visible throughout watch III):
- Rise under 90 s: "It springs straight back. Still young. Leave it."
- Rise of 90 s or more: the cloth lifts, a dent appears and slowly half-fills. "Slow, and not all the way back. Ready." `doughReady = true`.

**Lamp** (Fig. 13, *Lamp. Olive oil, linen wick. Gutters at about the middle of the night.*):
- Starting 30 s into watch III, the lamp's intensity sinks over 25 s to 0.25, flickering harder, and a pin appears.
- Click: the wick is trimmed, oil is added, the lamp is bright again. "Trim it. In this trade, everything tells you when it needs something, if you look."
- No penalty if ignored, but the room gets dim.

**Ambient lines.** One every 11 s while watch III is running (skip when another line is queued), in order:
1. ★ "Third watch. The whole city is asleep except bakers, thieves, and you."
2. "See that glow by the aqueduct? Dionysios. He's good. Don't put that in the report."
3. "Lucius on the hill fires too early. By noon his bread could pave the Via Labicana."
4. "A man from Ostia has bid lower for the contract. Let him. Let him do this."
5. "Atistia sat where you're standing. Twenty-two years. She said the dough talks if you're quiet. It doesn't. You hear it anyway."
6. "I'm going to build a tomb, witness. I've drawn it. I'll show you at dawn, if the bread is good."

**Breadbasket** (Fig. 9, clickable any time). Tag: *Fig. 9 — Breadbasket, stone. Contents: not bread.*
- Click: "That's Atistia. Leave her there; she liked to be where the work was."

When `doughReady && ovenReady`: **→ VIGILIA IV.**
- The water clock reaches band IV and the moon is at 35°. The unknown oven lights.
- ★ EURYSACES: "Fourth watch. The last one. Now it becomes bread."

### VIGILIA IV — Shaping, stamping, baking, weighing (≈ 2.5 min)

11. The pin goes on the basin. Click: the dough lifts and splits into **12 rounds** on the shaping table (two rows of six, each a squashed sphere r0.17). **Loaf 12 is visibly smaller (r0.15).**
    - EURYSACES: "Twelve. By eye. The steelyard will tell us if your eye is a liar."
12. Pin on the cord coil on the table (*Cord, linen.*). Click: a thin darker torus appears around every loaf's waist, in a quick cascade of 0.08 s each.
    - "A cord round each, so it bakes with a belt."
13. Pin on the knife (*Knife.*). Click: 8 radial score lines appear on every loaf (thin dark boxes on the top), in a cascade.
    - "Eight ways, so it breaks fair."
14. **Stamp** (Fig. 14b, *Stamp, bronze. Reads EVRYSACIS.*). Click the stamp to pick it up; the cursor shows a small stamp icon. Then **click each loaf** (12 clicks), each with a deep thunk sound. A small circular decal reading `EVRYSACIS` appears in the centre of each loaf (canvas texture on a tiny disc). The HUD counts `stamped 7/12`.
    - At first pickup: ★ "Now my name. Every loaf. If it's bad bread, they'll know whose. If it's good, they'll know whose. That's the whole point."
15. Pin on the peel. Click: the loaves slide one by one into the oven mouth (a fast 2 s animation) and a door slab (a dark box) closes the mouth.
    - "In. Now it's the oven's business."
    - A bake timer of **40 s** starts. A HUD bar fills.
    - Lines during baking (one per ~10 s):
      - "Vergilius is my old master's name; every freedman wears his master's name in front. Eurysaces is mine. Greek. 'Broad shield.' A dealer's joke. I kept it."
      - "Two thousand loaves a day with that joke stamped on them. Who's laughing."
      - "Smell that. That's the crust closing. Write: 'the smell, correct.'"
    - The bread smell is suggested visually by a few warm particles drifting from the oven.
16. At 40 s the pin goes on the oven. Click: the door comes off and the loaves slide back onto the table, now coloured golden brown #b8783a with a darker #7a4a22 top.
    - The pin goes on any loaf. Click one: two wooden clicks. "Hollow. Like a good argument."
17. **Weighing** (Fig. 15, *Steelyard, bronze. Standard: two pounds the loaf.*). Pin on the steelyard.
    - Each click on the steelyard hangs the next loaf on the hook (loaf arcs over) and the beam swings and settles over 0.8 s with a soft ting.
    - A tiny paper result card appears beside it, and the loaf then moves to the baskets' side of the table.
    - Loaves 1–11: card "II librae — true." (Eurysaces says nothing, or "True." every third.)
    - Loaf 12: the beam settles visibly high on the weight side. Card: **"I libra, X½ unciae — short by I½ unciae."**
    - ★ EURYSACES: "Short. That's the fist of leaven. It had to come from somewhere."
    - Choice buttons:
      - **[Send it anyway]**: "Under my name? No." It is then treated as withheld.
      - **[Withhold it]**: "Good."
    - Then: "Eat it, witness. You've been awake all night. The aediles can't object to evidence they never see."
    - Carry the loaf (see Carrying). The small loaf jumps toward the camera and vanishes into a HUD item icon.

### Dawn (≈ 1.5 min)
18. Pin on the baskets. Click: the 11 loaves drop into the baskets.
    - Then **eleven small warm lights** (a loaf-coloured sphere r0.12 with a tiny point glow) leave the bakery's street door. Each travels along the streets (a simple path: out the door → ring street → radial street → door) to one of the 11 doorways, staggered 0.6 s apart, at about 4 u/s.
    - As each arrives, its doorway warms to lamplight and a soft wooden shutter sound plays.
    - At the same time the **dawn transition** runs (12 s):
      - The moon slides down to 5°.
      - The sun rises on its arc.
      - The background and lights warm.
      - The other bakers' ovens dim.
      - The plate engraving changes to `PRIMA LVCE`.
      - A distant cockerel is approximated by synthesis (see Sound), or skipped if that's awkward.
    - ★ EURYSACES (after the last door): "Listen. That's Rome eating my bread. I've never got tired of that sound."
19. EURYSACES: "Your report, witness. Seal it."
    - The report tab pulses. Opening it (R, or click the witness figure) shows the **report**. At its bottom is a **[Seal]** button: a red wax disc stamps onto the tablet with a thunk.
    - After sealing, the report reads as sealed. Carry it.

**The witness's report** (generated; bracketed parts depend on state):

> **TABULA TESTIS — REPORT OF THE WITNESS**
> On the batch made by M. Vergilius Eurysaces, baker, contractor, in one night, inside the Esquiline gate.
>
> I. Grain received: three sacks, Campanian wheat. Dry. Weevil: none observed.
> II. Milling: lava stones, one donkey, through the first watch.
> III. Sifting: three grades obtained. Second flour taken, per contract. [if tookFine: *The witness at first took the fine flour; corrected by the contractor.*]
> IV. Water: public basin, two buckets. Clear.
> V. Leaven: one crock, age undetermined. The contractor states it is older than his freedom.
> VI. Salt: one fist.
> VII. Kneading: stone machine, one horse, to smoothness.
> VIII. Withheld for the following day: one fist of dough, as leaven.
> IX. Rising: through the third watch, under cloth. Dent test: slow return, incomplete. Ready.
> X. Oven: four bundles, until the soot burned off the vault. Flour test: brown at a count of ten.
> XI. Loaves: twelve. Bound, cut in eight, stamped EVRYSACIS. Baked through the fourth watch. Hollow when knocked.
> XII. Weighing: eleven loaves true at two pounds. One loaf short by one and a half ounces (see VIII). Withheld by the contractor. Eaten by the witness.
> Lamp: [trimmed once by the witness / not trimmed; the witness worked in the dark].
> Sleep of the witness: [none observed / *the witness slept once, briefly; woken by the contractor* / *the witness slept n times; woken each time by the contractor*].
>
> Finding: **The contractor's bread is as stated.**
> [Seal]

Before dawn, opening the report with R shows only the lines filled so far. Unfilled lines show as faint `—`.

20. After sealing, Eurysaces walks to the back wall and takes down a wax tablet. Pin on him (Fig. 21, *M. Vergilius Eurysaces. Baker, contractor, freedman.*). Click: the reading panel shows **the tomb plan**:

> **WAX TABLET, DRAWN BY THE CONTRACTOR**
> A tomb, outside this gate, where the two roads fork. Travertine.
> Its sides pierced with round mouths, like the mouths of the kneading machines and the measures for grain, so that anyone walking past knows what was done here.
> Round the top, a frieze, in order: grain received. Milling: two donkeys (he has crossed out *one*). Sifting. Kneading, by horse. Shaping at the long table. The oven, and the peel. The loaves weighed on the steelyard before the officials of the aediles, in togas. One official is unshaven.
> Beside the door: a man and his wife, side by side. Her ashes in a stone breadbasket.
> Round all sides, the letters:
> **EST HOC MONIMENTVM MARCEI VERGILEI EVRYSACIS PISTORIS REDEMPTORIS APPARET**

Close the panel. Then:
- ★ EURYSACES: "'This is the monument of Marcus Vergilius Eurysaces, baker, contractor. *Apparet.*' It means he serves the magistrates. It also means: it is evident. I like both."
- EURYSACES: "Let them stand in front of it and see."
- EURYSACES: "Two donkeys. It's a tomb; you're allowed one lie. — Go on, witness. Out the gate, where the roads fork. Mind the loaf."

21. **The ghost tomb appears** at the fork, (0, 0, 12.6). It is a translucent travertine box (opacity rising to 0.55; colour #e9dfc8; emissive at 0.15 so it reads at dawn), trapezoid-ish, 2.2 wide × 2.4 tall × 1.6 deep. It has three rows of round dark holes (circles r0.18, in a grid of 3 × 4 per long face) and a thin frieze band near the top. Its edges glow with a thin line (EdgesGeometry).
    - The camera gently swings its target toward (0, 1, 8) over 3 s, so the gate and tomb come into view.
    - Pins go on the gate and the tomb.
    - Tomb tag: *Fig. 22 — Tomb of M. Vergilius Eurysaces, c. 30 BC. Still standing at Porta Maggiore, Rome. The frieze survives on three sides.*
    - Gate tag: *Fig. 19 — Esquiline gate. Go out, to the fork of the roads.*
    - **Clicking either one calls `branches.onward()`.**
    - Before this point, clicking the gate says: "The witness doesn't leave before the bread."

## Things the player can poke at any time (flavour, no state)
- **Aqueduct** (Fig. 18): *Aqua Marcia, high-level channel. Arrives cold.* Click: "Eleven miles of arches so I can make dough. Rome is a machine for bread, witness."
- **Other ovens** (Fig. 17). Clicking one gives its line:
  - Lucius: *Fig. 17a — Oven of Lucius. Fires early.* "Too early."
  - Dionysios: *Fig. 17b — Oven of Dionysios. Fires correctly.* "Don't write that down."
  - Unknown: *Fig. 17c — Oven, owner unknown. Fires late.* "No idea. Late, whoever it is. Brave."
- **Donkey/horse after use:** "He's resting. So should you not be."
- **Eurysaces** (before dawn): one of his last lines, repeated. Or the step hint.
- **Witness figure:** opens the report.
- **Water clock** (Fig. 12): *Water clock. Float and pointer. Marks the four watches.* Click: "It's slow. It's always slow. I go by the dough."

## Carrying (`branches.carry`)
Always set the full list:
- After withholding the loaf: `["a light loaf stamped EVRYSACIS, short by an ounce and a half", "a fist of leaven in a cloth"]`.
- After sealing, add `"the witness's report, sealed"`.

The leaven is the thread. Set the first carry when step 9 completes, as just the leaven.

## Sound (WebAudio, synthesized; start on first click)
- **Night bed:** very quiet filtered noise (wind, lowpass 400 Hz, gain 0.03), plus sparse crickets (sine 4.2 kHz chirps in groups of 3, every 2–5 s, gain 0.01). Fade both at dawn.
- **Water clock drip:** sine at 1200 Hz falling to 800 Hz over 60 ms, gain 0.04, every 1.5 s.
- **Mill grinding:** brown noise, lowpass 220 Hz, amplitude-modulated at the lap rate. Plays only while milling.
- **Sieve:** bandpassed noise at 3 kHz, 0.4 s.
- **Kneading:** a soft low thump (sine 110 Hz, 80 ms decay) once per 0.75 s while the horse turns.
- **Fire:** crackle (random short noise clicks through a bandpass at 2 kHz) plus a low roar (noise lowpass 300 Hz). Gain follows heat.
- **Broom hiss:** white noise highpass 2 kHz, 1 s decay.
- **Stamp thunk:** sine 90 Hz → 60 Hz, 150 ms, plus a little noise click.
- **Knock:** two bandpassed noise clicks (900 Hz) 120 ms apart.
- **Steelyard ting:** sine 2.6 kHz + 3.9 kHz, 400 ms decay, gain 0.05.
- **Door shutters at dawn:** short wood clacks at random pitch.
- **Dawn swell:** a soft major chord (A3 C#4 E4, sine/triangle) fading in over 6 s and out over 8 s, gain 0.04.
- **Cockerel (optional):** a sawtooth sweeping 500 → 900 → 700 Hz through a bandpass, 0.9 s, very quiet and distant. Skip it if it sounds silly.

## Robustness
- Every step is gated by state.
- Clicking an object out of turn gives its tag-flavour line, or "Not yet." It never breaks state.
- Pins always show the current step.
- All panels can be closed.
- If WebGL fails, show a cream panel with the contract text and a note. Never a blank screen.

## Pacing
Estimated play time is 8–9 minutes. The fixed waits alone add up to about 3 min: milling 20 s, kneading 15 s, rise 90 s, bake 40 s, plus the oven pacing and the dawn sequence. The way onward only exists after the tomb plan is read.

## First thing to try
On arrival, a brass pin bobs over the grain cart and the third line says "Grain's on the cart. Tip it in." Hovering the cart shows its tag *Fig. 1 — Grain.* Click it.
