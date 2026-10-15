# Inside Johnny: The Day Shift

## The idea

It is 7 a.m. on Saturday, 1955. Johnny Novak's fever broke at three. On the bed tray in front of him is a tin-and-cardboard expansion set, **INSIDE JOHNNY: THE DAY SHIFT (Expansion Set No. 2, Milton & Harrow Co.)**, open and set up.

During the night, the night shift took apart the **Rumble**, the machine in the stomach that makes hunger, and used it for spare parts. Each foreman took one piece home to their own department "for safe keeping." The fever is gone now, and they have nothing left to guard. Nobody knows what to do. They sit on their parts.

Then the morning post comes down the chute from the ear. Mrs. Novak is reading Johnny the get-well cards his third-grade class wrote, one at a time. There are five, written in crayon. Each one was really written to one part of him, even though the children didn't know it. Beverly's card about the butterscotch pudding she saved and then ate is for the stomach. Eddie's kickball card is for the lungs. Linda's card about the class hamster running on its wheel all night is for the heart. Arthur's card apologising for sneezing on Johnny on the slide is for the filter works, which has spent three days cleaning up after Arthur. Ronnie's terrible joke is for the switchboard in the head. Each card gets one foreman to stand up and hand over a part.

You put the Rumble back together in the order on the printed assembly sheet. The sheet says the crank is "not supplied; any straight thing of the right length will do." The thermometer from last night, shaken down to 37.0, is exactly that thing. You turn it. The whole tin board shakes with a stomach rumble, then the **shift whistle** blows. That is the old factory sound of a shift changing. Beyond the board, a boy's voice says: *"Mom? Can I have toast?"*

**The beauty:** for parents, a child's first hunger after a fever is the moment of relief. *He's hungry, so he's better.* Here, that moment is a factory whistle, and it is powered by five clumsy, kind letters from eight-year-olds. The safety sign reading DAYS WITHOUT FEVER: 0 clicks over to 1.

## Medium, and why (and the player's note)

The medium is a 3D tin board game that you stand inside. The player's note on space 1 asked for "a visual tie-in that jumps to the world of the game." So this space **starts outside the game**, at Johnny's eye level above the bed tray, looking down at the open board. Pressing E makes the camera **dive down onto the board**. From then on you can always tell you're inside a toy: the board is printed cardboard, the buildings are lithographed tin, the foremen are cardboard stand-ups on red plastic bases, and past the board's edge you can see the real, giant bedroom (quilt, ginger-ale glass, box lid, window).

## Scale and layout

Units are metres at the player's scale on the board. Eye height is 1.6.

- **The board:** a flat slab of 24 (x, from −12 to 12) × 16 (z, from −8 to 8) × 0.2, with its top at y = 0.
  - Base colour is cream `#efe2b8`.
  - A printed **cutaway of a boy** sitting up in bed, drawn in a canvas texture: a big pale-peach (`#f1c9a5`) silhouette outlined in thick red `#b8382a`. The head is at the north end (z ≈ −6) and the torso fills the middle.
  - A printed **game track**: about 40 squares, each 0.8 m, alternating cream and mustard `#e8a317`, numbered in black. The track loops through all the stations.
  - The printed title "THE DAY SHIFT" runs along the south edge.
  - Soot smudges and a coffee-ring stain, for grit.
- **The tray rim:** a 0.4 m high tin lip all the way round the board, painted green `#3f6b5c` and riveted. Clamp the player's walking to x ∈ [−11.4, 11.4] and z ∈ [−7.4, 7.4].
- **Beyond the rim, at giant scale.** The board sits on a bed tray, which sits on a quilt.
  - **Quilt:** a big plane 1 m below the board, about 200 × 200, in a faded blue-and-white check (`#9fb4c8` / `#f3efe4`).
  - **Box lid:** propped upright behind the north edge at z = −14. It is 34 wide × 22 tall × 0.5 deep, a box so it's opaque from both sides. Its cover art is a canvas texture with these lines:
    - "INSIDE JOHNNY" in Impact, mustard on soot-black, 3 m letters
    - "THE DAY SHIFT" in red
    - "Expansion Set No. 2 · For 2 to 6 Players · Ages 6 and Up"
    - "MILTON & HARROW CO. · 1955"
    - A painted smokestack skyline built into a boy's ribcage.
  - **Ginger-ale glass:** at the SE corner, around (20, −1, 14). An open cylinder 4 m in radius and 14 m tall, translucent amber `#d9b25a` at opacity 0.35, with a few slow-rising bubble spheres.
  - **Window:** to the east at x = 60. A big bright plane 40 × 30 in `#ffe4b0`, with a cross-shaped frame of dark boxes.
  - **Bedroom walls:** a huge BackSide box, 160 × 80 × 160, pale 1950s green `#cfdcc4`, so the world is enclosed.
- **Stations.** Each is a tin building about 3.5 w × 3 h × 3 d. Make each one a box with an open front facing the centre of the board and a pitched or sawtooth roof. Give it a lithographed texture: rivets, a printed brick or corrugated pattern, a department name painted on a header board in mustard on black, and a small smokestack.

| Station | Organ | Position (x, z) | Who |
|---|---|---|---|
| MAIL CHUTE (at the ear) | ear | (−6, −6) | Gatekeeper Hobbs |
| SWITCHBOARD | head | (0, −6.2) | Mabel, operator |
| BOILER HOUSE | lungs | (−6.5, −0.5) | Foreman Pruitt |
| PUMP ROOM | heart | (3, −1.5) | Ines, pump tender |
| FILTER WORKS | liver | (7, 3) | Dov, sump man |
| VAT HALL + THE RUMBLE | stomach | (−1, 4) | Wren, vat keeper |

- **The Rumble** sits in the open in front of Vat Hall at (−1, 5.2).
- **The safety sign** stands at (−4, 6) and reads "DAYS WITHOUT FEVER: 0". Make it a tin placard on two posts, 1.6 × 1.0, with a canvas texture so the number can change.
- **The DAY square** is the onward square: a track square at (3, 6.3), slightly larger at 1.2 m, printed "ADVANCE TO DAY". It stays dull until the end.
- **The giant die** sits at (9, −5): a 1.2 m ivory cube with red pips, slightly rotated.

**The foremen** are cardboard stand-ups, each 1.0 wide × 2.0 tall × 0.05 deep, mounted on a red plastic base that is a half-cylinder 0.4 tall.
- Each stands just in front of their building and faces the board centre.
- Paint each one on canvas: a flat, simple 1950s-illustration figure in overalls or a shop coat, plus a name banner at the feet.
  - Hobbs: brown coat, peaked cap.
  - Mabel: headset, cardigan, pearls.
  - Pruitt: soot-blacked face, rolled sleeves.
  - Ines: blue coverall, wrench.
  - Dov: rubber apron, boots.
  - Wren: white cap, ladle.
- Each holds their part in a painted hand. The part also stands as a 3D object leaning against the stand-up (see the Rumble section). When they hand it over, the 3D part disappears from beside them.

## On arrival

1. **The establishing shot (the tie-in).**
   - The camera starts at (0, 18, 22) looking at (0, 0, 0), so the whole board, the box lid and the ginger-ale glass are in view. Pointer lock is not engaged yet.
   - A centred overlay card on a cream panel with a thin red border reads:

     > **INSIDE JOHNNY: THE DAY SHIFT**
     > Saturday, 7 a.m. The fever broke at three. Johnny is sitting up in bed with the expansion set open on his tray.
     > *Click, then press E to lean in.*

   - On the first click, start audio and lock the pointer. On E, tween the camera along a curve over 2.8 s (ease in-out) down to the player position (−6, 1.6, −2.5), facing north toward Hobbs at the Mail Chute.
   - As it lands, play a small tin *tock* sound, and walking unlocks.
   - If the player never presses E, a click-and-hold or a second click also triggers the dive after 1 s. Never get stuck here.
2. After landing, call `branches.say("Gatekeeper Hobbs is at the Mail Chute. Walk up and press E.")`.
3. **HUD:**
   - Top-left, small, always visible: `THE DAY SHIFT · Rumble parts fitted: 0/5 · Cards in hand: 0`.
   - Bottom-centre: the kit's "E: talk to Hobbs"-style prompt when looking at something usable.
   - Inventory slots show along the bottom. Number keys 1–9 select an item, and the selected item is highlighted. **E on a person or the machine uses or gives the selected item.** If nothing is selected, E talks or reads.

## Inventory and selection

Items are cards and parts.
- The five cards: `Beverly's card`, `Eddie's card`, `Linda's card`, `Arthur's card`, `Ronnie's card`.
- **R reads the selected card** in the reading panel. Give a one-time hint after the bundle is taken: "R reads a card. Number keys pick which one you're holding."
- Parts: `Strainer`, `Churn`, `Flywheel`, `Bellows`, `Whistle`, `Thermometer`.
- At the end: `Mrs. Novak's note`, then `Toast soldier`.

**The thermometer:**
- If `branches.carried` contains anything with "thermometer" in it, start the player holding `Thermometer (37.0)`.
- Otherwise, place a thermometer on the board in Vat Hall's doorway at (−1, 0.05, 2.6). It is a thin glass cylinder 0.06 r × 0.9 long, lying flat, with a red tip. Label it "A thermometer, shaken down. 37.0. Somebody left it here after last night." It goes into the inventory on E.
- Either way, the thermometer is guaranteed.

## People and documents: every line

Show speech in the kit's reading panel or say-line: the speaker's name in caps, then the line. Keep each beat to one panel. A speech blip plays (see Sound).

### Gatekeeper Hobbs (Mail Chute)

The Mail Chute is a tin hopper shaped like an ear trumpet, coming down from the board edge. A brass slot is set in it.

On his desk, a 0.8 × 0.8 × 1.0 box, is the **clipboard** with the Night Shift Handover Log. It is readable with E as a separate object.

**First E:**
> **HOBBS:** Morning. Gatekeeper Hobbs, Sweat Vent. Well, used to be. Vent's shut. Nothing to let out. So they've put me on post.
> First mail's just come down the ear. Mrs. Novak's reading him his cards, one at a time, out loud, and they drop down the chute as she goes. Five of them. Here.

→ The player gets all 5 cards. The HUD shows Cards in hand: 5.

**Second E:**
> **HOBBS:** Read the log if you want the whole sorry night. Short of it: the Rumble's in pieces down in Vat Hall. Night shift stripped it for spares at two in the morning, and every foreman took a bit home "for safe keeping." Now the fever's gone and they're all just sitting on their bits.
> Day shift doesn't start till the Rumble runs. Maybe the post'll get them up. Kids write to the whole of a person, you know, not just the front of him. Give each card to whoever it was really for.

**Later E, any time before the end:**
> **HOBBS:** Nothing else in the chute yet. There'll be one more. There always is.

**Night Shift Handover Log** (on the clipboard). Show it in a typewriter font on a yellowed form with a printed header and a ruled table:

```
MILTON & HARROW WORKS — NIGHT SHIFT HANDOVER LOG
Plant: J. NOVAK, age 8.          Shift: Fri 22:00 – Sat 07:00

Fri 23:10  Boiler House reports overheat. Pruitt opens windows.
Sat 00:30  Extra blanket received. Hobbs objects.
Sat 01:40  Fever located Vat Hall. Moved to Filter Works.
           Moved to Pump Room.
Sat 02:15  RUMBLE dismantled for spares on orders of the night
           foremen. Parts dispersed to departments for safe
           keeping. DO NOT REASSEMBLE UNTIL ALL CLEAR.
Sat 03:05  Fever left via Sweat Vent. Hobbs signed it out.
Sat 03:06  Thermometer 37.0. Shaken down twice.
Sat 03:07  ALL CLEAR.
Sat 03:08 – 07:00  Nothing happened. Nobody knew what to do.
           Mabel made tea.

To the day shift: it's yours. Put it back together.
      — Pruitt · Ines · Dov · Wren · Mabel · Hobbs
```

### The five get-well cards

Render each in the reading panel as a sheet of construction paper in the colour given.
- Use a crayon-style hand font: `"Comic Sans MS", "Chalkboard SE", cursive`, at about 22px. Make the letters slightly wobbly by giving each line a tiny random rotation of ±1°.
- Add a simple crayon drawing in the corner, drawn with canvas strokes: thick, wobbly, 2–3 colours.
- Keep the children's spelling and capitals exactly as written below.

**Beverly's card** (pink `#f2b8c6`; drawing: a bowl with three steam squiggles)
> Dear Johnny,
> Get well soon. Today at lunch it was butterscotch pudding. I saved you mine but Mrs. Dunn said it would go bad so I ate it. Sorry. It was very good.
> Your friend,
> Beverly

**Eddie's card** (green `#a9cf8e`; drawing: a ball arcing over a fence)
> Johnny —
> We played kickball. I kicked it over the fence on Elm st and we had to stop. We need you in left field. Hurry up.
> Eddie
> P.S. Ronnie is in left field now and he is BAD.

**Linda's card** (yellow `#f3dd7a`; drawing: a round hamster in a wheel, with motion lines)
> Dear Johnny,
> I am taking care of Pickles this weekend. He ran on his wheel all night. I think he misses you. I stayed up and listened to him go around.
> Get well soon.
> Love, Linda (and Pickles)

**Arthur's card** (blue `#9fc0e6`; drawing: a slide and a small figure with a big "ACHOO")
> Dear Johnny,
> Miss Halloran says we all have to write to you. I think I gave you my cold on Tuesday when I sneezed on the slide. I am sorry. I am better now. You will be too.
> Arthur

**Ronnie's card** (orange `#f3a65a`; drawing: a cookie with a sad face and a thermometer in its mouth)
> JOHNNY!
> Why did the cookie go to the doctor?
> BECAUSE IT FELT CRUMMY.
> ha ha ha
> get well
> Ronnie

### The foremen

Each foreman has four kinds of line:
- **Idle:** E with nothing selected, or with a non-card selected.
- **Wrong card:** a gentle no, with a hint. The card stays in the inventory.
- **Right card:** the card is consumed, the foreman's 3D part moves to the player's inventory, the stand-up does a little hop (y +0.3 and back over 0.4 s) and **turns to face the Rumble**, and the HUD updates.
- **After:** a short thanks.

**WREN, vat keeper (Vat Hall, the stomach), holds the CHURN.** Right card: Beverly's.
- Idle:
  > **WREN:** Vat Hall's been on dry-toast rations three days. I've had the Churn under a tarp since two this morning. I'm not uncovering it for nothing. Got anything?
- Wrong:
  > **WREN:** That's a nice card. It's not for the stomach, though. The stomach only reads about food.
- Right:
  > **WREN:** "I saved you mine." …And then she ate it. Of course she ate it. *Butterscotch.*
  > *(Wren pulls the tarp off.)* Take the Churn. Number two on the sheet. And tell Beverly the stomach says thank you anyway.
- After:
  > **WREN:** Strainer in first, then my Churn. Don't you go putting it on a dirty strainer.

**PRUITT, foreman (Boiler House, the lungs), holds the BELLOWS.** Right card: Eddie's.
- Idle:
  > **PRUITT:** Windows shut, boilers on low. Bellows are in the shed. Lungs don't run for nothing, son. They run *for* something.
- Wrong:
  > **PRUITT:** Not a lungs letter, that. Lungs want somewhere to *go*.
- Right:
  > **PRUITT:** Left field. Over the fence on Elm. *(he coughs, then laughs)* And Ronnie's bad. Well, then. We'll need the wind back.
  > Bellows. Fig. 4. Hook 'em to the flywheel.
- After:
  > **PRUITT:** Three days in bed. He'll be puffing for a week. Good.

**INES, pump tender (Pump Room, the heart), holds the FLYWHEEL.** Right card: Linda's.
- Idle:
  > **INES:** I kept the beat all night. Never once stopped. I've got the Flywheel in my lap and I'm not handing it to anyone who doesn't know what it's for.
- Wrong:
  > **INES:** That's sweet. It isn't a heart letter. A heart letter's about somebody keeping something going all night.
- Right:
  > **INES:** "I stayed up and listened to him go around." …
  > *(a long pause)* That's all I do, you know. Go around. Somebody listening. Take the Flywheel. Number three, on the side axle.
- After:
  > **INES:** Seventy-two and steady. Tell Pickles.

**DOV, sump man (Filter Works, the liver), holds the STRAINER.** Right card: Arthur's.
- Idle:
  > **DOV:** Filter's clean. Scrubbed it myself. Every last bit of what came in Tuesday, I strained it out by hand. I would like to know who *sent* it, I would.
- Wrong:
  > **DOV:** No. Not to me. I'm the one who deals with what somebody *gave* him.
- Right:
  > **DOV:** Arthur. On the slide. Tuesday. *(a very long pause)* He says sorry. He says he's better now.
  > *(Dov sniffs)* Well. That's all filtered, then. Strainer goes in first. Nothing sits right on a dirty strainer.
- After:
  > **DOV:** I'm not crying. It's the sump.

**MABEL, operator (Switchboard, the head), holds the WHISTLE.** Right card: Ronnie's.
- Idle:
  > **MABEL:** Quiet as a church up here. Nobody's rung down to the mouth in three days. I've got the Whistle, that's the line that says HUNGRY, and I'll give it up when somebody makes me laugh.
- Wrong:
  > **MABEL:** Aw. That's lovely, dear. Doesn't make me laugh, though.
- Right:
  > **MABEL:** Why did the cookie go to the… *because it felt crummy.* Oh, that's *terrible*. *(she laughs; every lamp on the switchboard flickers)*
  > Take the Whistle. On top, last of all. I'll be listening for it.
- After:
  > **MABEL:** Crummy. Oh dear.

The **Switchboard** building has a 4 × 3 grid of small lamp spheres (r 0.08) on its front face. They are unlit (`#553`) until Mabel laughs: then they flicker for 1.5 s and settle to a dim amber.

## The Rumble (the machine)

It stands at (−1, 0, 5.2). At first it is a **riveted tin base** with ghost outlines where the parts go.
- **Base:** a box 2.6 × 0.4 × 2.6, dark tin `#2f3a35`, rust blooms `#8a4b2a`, rivets at the edges.
- Stamped into the front of the base: "THE RUMBLE · STOMACH WORKS".

Each socket shows a **ghost**: the part's shape in pale wireframe or transparent cream (opacity 0.2), with its number stamped beside it in red (a 0.3 m canvas-text sprite: "1", "2"…). This lets the order be read off the machine as well as the sheet.

| # | Part | Shape | Where | Comes from |
|---|---|---|---|---|
| 1 | STRAINER | Perforated disc: cylinder r 1.0, h 0.15, grey tin with a dark dot-grid texture | Top of base, y 0.45 | Dov |
| 2 | CHURN | Barrel: cylinder r 0.7, h 1.1, red tin `#b8382a` with three dark hoops (thin tori) | On the strainer, y ≈ 1.1 | Wren |
| 3 | FLYWHEEL | Torus r 0.75, tube 0.08, plus 6 box spokes and a hub, mustard `#e8a317` | Side axle at (+1.4, 1.2, 0), facing ±x | Ines |
| 4 | BELLOWS | 4 stacked boxes alternating 0.9 and 0.75 wide, brown leather `#6b4428`, with wooden end boards | Opposite side at (−1.4, 0.9, 0) | Pruitt |
| 5 | WHISTLE | Brass cylinder r 0.14, h 0.7, `#c9a24a`, with a small cone cap | On top of the churn, y ≈ 2.0 | Mabel |
| – | CRANK | The thermometer (glass with a red tip), fitted in the flywheel hub sticking out along +x | Flywheel hub | carried, or found |

**Assembly sheet.** A giant folded paper, 1.4 × 1.0, leans on a little easel at (1.2, 0, 4.4) beside the machine. E reads it. Show it in the panel in a technical-manual style: black ink on off-white, numbered figs in boxes, using a typewriter or condensed sans font:

```
INSIDE JOHNNY — THE DAY SHIFT
ASSEMBLY OF THE RUMBLE (Stomach Works)

PARTS:  1 Strainer · 2 Churn · 3 Flywheel · 4 Bellows · 5 Whistle
        Crank — NOT SUPPLIED. Any straight thing of the right
        length will do.

Fig. 1  Seat the STRAINER in the base.
Fig. 2  Set the CHURN on the Strainer.
Fig. 3  Mount the FLYWHEEL on the side axle.
Fig. 4  Hook the BELLOWS to the Flywheel.
Fig. 5  Fit the WHISTLE on top.
Fig. 6  Fit a crank. Turn.

If correctly assembled, the RUMBLE will rumble, the WHISTLE
will signal the Switchboard, and the patient will ask for toast.
ALL PLAYERS ADVANCE TO DAY.
```

**Fitting.** With a part selected, press E on the Rumble.
- **The correct next part:** it slots in. Tween it from 0.6 above its socket down into place over 0.35 s, then play a tin *clank*. Its ghost and number disappear. The HUD updates: `Rumble parts fitted: n/5`. Say:
  - Strainer: "The Strainer drops into the base with a clean clank. Fig. 1."
  - Churn: "The Churn seats on the Strainer, snug. Fig. 2."
  - Flywheel: "The Flywheel slides onto the axle and turns half a turn by itself. Fig. 3."
  - Bellows: "You hook the Bellows to the Flywheel. They breathe in a little. Fig. 4."
  - Whistle: "The Whistle screws in on top. Five of five. It needs a crank."
- **A wrong-order part:** it tweens toward its socket, wobbles, and drops back. Play a hollow *bong*. The part stays in the inventory. Say the line for the part:
  - Churn too early: "The Churn sits crooked and rolls off with a hollow bong. Something goes under it first. (Fig. 1.)"
  - Flywheel too early: "The Flywheel has no axle until the Churn's in. It wobbles off. (Fig. 2 first.)"
  - Bellows too early: "The Bellows have nothing to pump. They wheeze once and slump. (Fig. 3.)"
  - Whistle too early: "You fit the Whistle. Nothing to blow it. It looks silly up there on its own, so you take it back. (Fig. 4.)"
- **E with nothing selected, or with a card:** "The Rumble, in bits. The sheet on the easel says what goes in first." If parts are fitted, add " n of 5 fitted."
- **Thermometer before 5/5:** "That would make a fine crank. Later. It's not all there yet."

**Crank.** At 5/5 with the thermometer selected, E fits it. It tweens into the hub.
> The thermometer slides into the crank socket as if it was made for it. 37.0, upside down. Turn it. Press E.

**Turning.** It takes **6 presses of E** (any selection) while looking at the machine. With each press:
- The flywheel spins one turn over 0.5 s.
- The churn rotates 60°.
- The bellows compress and expand (scale y 0.7 → 1).
- A ratchet *click-click-click* plays.
- A low rumble layer fades in, louder each turn.
- Say lines for turns 1–5: "Clunk." · "Clunk-a." · "Something deep down goes *mrrr*." · "The board hums under your feet." · "The Churn's turning on its own now."

**On turn 6, THE RUMBLE** (about 9 s, scripted, input ignored except looking around):
1. **0.0 s.** A huge stomach rumble (see Sound). The camera shakes (amplitude 0.06, 1.8 s). Every foreman stand-up wobbles. The giant die hops 0.3 and rolls a quarter turn. Bubbles in the ginger-ale glass speed up.
2. **1.8 s.** The **shift whistle** blows from the Whistle for 2.5 s: two sines, 392 and 494 Hz, slow attack, slight vibrato. A puff of white steam (a few growing, fading sprites) rises from the whistle. Along a printed red wire line on the board from Vat Hall up to the Switchboard, a row of small lamp sprites lights up one by one toward the head (about 12 dots over 1 s). All the Switchboard lamps go full amber.
3. **4.5 s.** Lines appear one after another, big, centred, in the overlay font style, as if from outside the board:
   > *(from somewhere enormous, beyond the box lid)*
   > **"Mom? …Mom."**
   > **"Can I have toast?"**
4. **6.5 s.** The safety sign flips to **"DAYS WITHOUT FEVER: 1"** with a card-flip animation: rotate on x through 180° while the texture swaps. Play a tin click. The window light brightens and warms (see Light).
5. **7.5 s.** Every foreman says one line at once, shown stacked in the say area or one panel:
   > **WREN:** *Toast.* **PRUITT:** Ha! **INES:** Seventy-two and steady. **DOV:** That's all filtered. **MABEL:** Line's open. **HOBBS:** There it is.
6. **9 s.** At the Mail Chute, a cream envelope flutters down out of the slot, spinning slowly, and lands on Hobbs's desk.
   - `branches.say("Something's come down the chute.")`
   - Hobbs's next line:
     > **HOBBS:** Told you. One more. There always is. It's from the front office. Read it.

## The last document

E on the envelope (or on Hobbs) puts `Mrs. Novak's note` in the inventory and opens it at once in the panel. Show it in a neat ballpoint script on blue-lined notepaper with a small "From the desk of" header and a faint ring stain:

> Dear Miss Halloran,
>
> Please excuse John from school on Friday and Monday. He had a fever of 103 on Friday night. It broke at three o'clock this morning.
>
> He has just asked for toast.
>
> He will be back on Tuesday. Thank you for the cards. I have read them to him twice, and he laughed at Ronnie's.
>
> Sincerely,
> Mrs. Helen Novak

Closing the panel triggers the **toast**:
- A giant white plate, 12 r, slides in from the west beyond the tray rim at board height −0.5, centred at (−22, …, 0), over 2 s.
- On it is a slice of toast: a box 10 × 1.2 × 10 in golden `#d9a05b`, crust edges `#8a5a2b`, with a butter pat (2 × 0.4 × 2, `#f6e48a`). The toast is cut into five long soldiers: five boxes 1.9 wide with small gaps.
- One toast soldier, small enough to carry, appears leaning against the rim inside the board at (−11, 0, 0): a box 0.25 × 0.25 × 1.2, golden. Label: "A toast soldier. Still warm."
- `branches.say("Toast. Cut into soldiers. Take one for the road.")`

Then **the DAY square lights up.** It glows mustard and pulses (emissive 0.2 → 0.8, 1.2 s cycle). A small tin flag pops up on it reading "ADVANCE TO DAY". Hobbs's line:
> **HOBBS:** Day shift's on. Off you go, all players advance. Mind the track.

## The way onward

- Taking the toast soldier is optional.
- Walking onto the lit DAY square (within 0.8 m), or pressing E on it, calls:
  - `branches.carry(["Mrs. Novak's note to school", "a toast soldier, still warm"])`, leaving out the toast if it wasn't taken;
  - then `branches.onward()`, after a 0.6 s fade to warm cream.
- The arc earns it: 5 cards → 5 parts → assembly → crank → rumble → note. That takes about 3–4 minutes.

**Failsafe.** The player complained the last space didn't seem to let them on.
- At **5:30** after landing, if the Rumble hasn't run:
  - Light the DAY square anyway.
  - Hobbs calls out: `branches.say("HOBBS: Look, the day shift'll start with or without the paperwork. The DAY square's lit whenever you're ready.")`
  - In that case, carry `["the thermometer, 37.0", "an unread get-well card"]`.
- The HUD part count always shows progress, so it's never unclear what's left.

## Small extra interactions

- **Giant die (E):** it rolls. Tumble it over 0.8 s and land on a random face. Say: "You roll a {n}. The Day Shift rules say: *Any roll will do. He's getting better.*" The rolls are endless and harmless.
- **The ginger-ale glass** can't be reached. Pointing at it shows the label "Ginger ale. Gone flat. Perfect."
- **The box lid** is labelled from the board: "The lid. Ages 6 and up."
- **The safety sign (E):** before the Rumble runs: "DAYS WITHOUT FEVER: 0. Somebody has hung a pencil on a string beside it, ready." After: "DAYS WITHOUT FEVER: 1."

## Colour, light, sound

**Palette:**

| Use | Colour |
|---|---|
| Board cream | `#efe2b8` |
| Printed red | `#b8382a` |
| Mustard | `#e8a317` |
| Tin green | `#3f6b5c` |
| Soot | `#1d130c` |
| Rust | `#8a4b2a` |
| Brass | `#c9a24a` |
| Boy-silhouette peach | `#f1c9a5` |

Background and fog are a pale bedroom green `#cfdcc4`, with fog far, from 60 to 180, so the giant room reads hazy but present.

**Light:**
- A HemisphereLight: sky `#fff3d6`, ground `#6b5a44`, intensity 0.7.
- A low **morning sun** DirectionalLight `#ffd59a`, intensity 1.3, from the east window: position (40, 18, 6), aimed at the board centre. Shadows are on, with the shadow camera covering the board (±14), so the tin buildings throw long, warm, raking shadows across the printed track. This is the look.
- A dim AmbientLight at 0.15.
- **After the rumble:** over 4 s, ease sun intensity to 1.8 and raise its y to 30 so the shadows shorten (morning has properly arrived), and lift the hemisphere to 0.9.

**Sound** (Web Audio, all synthesized, started on the first click):
- **Ambient:** a slow bedroom **clock tick**, a short filtered click every 1.0 s, quiet, slightly reverberant via a feedback delay. Under it, a very faint tin hum: a 55 Hz sine at gain 0.02.
- **Speech blip:** when a foreman's panel opens, 4–6 short square-wave chirps at a pitch per person, gain 0.05:
  - Hobbs 140 Hz
  - Pruitt 120 Hz
  - Wren 220 Hz
  - Ines 260 Hz
  - Dov 110 Hz
  - Mabel 330 Hz
- **Card take or give:** a paper rustle, white noise bandpassed at 3 kHz, 0.15 s.
- **Clank** (part fitted): a noise burst through a bandpass at 1.2 kHz with Q 8, 0.25 s decay, plus a 600 Hz sine ping.
- **Bong** (wrong order): a 180 Hz sine with a 0.8 s decay and slight pitch drop.
- **Ratchet:** three 0.02 s clicks spaced 60 ms apart per turn.
- **THE RUMBLE:** brown noise through a lowpass at 90 Hz, with gain swelling over 1.8 s while an LFO wobbles it at 3–5 Hz, plus a 45 Hz sine gliding to 38 Hz. It should sound funny and enormous, like a stomach the size of a factory.
- **Shift whistle:** as described above. Two detuned sines plus a little noise breath, 2.5 s, with a slow release.
- **Sign flip:** a tin click.

## Text panels

Use the kit's reading panel, styled per document:
- Cards: construction paper with crayon font.
- Log and assembly sheet: typewriter on yellowed paper.
- Mrs. Novak's note: lined notepaper.
- Speech: a cream panel with the speaker's name in a red caps label.

Panels are large, about 560 px wide, with body text at 20–22 px. They close with E or by clicking.

## First thing to try

Press E to lean in, then walk to Hobbs at the Mail Chute and press E. He hands you five get-well cards. R reads the card you're holding.

## Arc summary

1. **Question:** the fever's gone, the Rumble's in pieces, and every foreman is sitting on a part. How does the day start?
2. **Middle:** read the class's cards, give each to the foreman it was really written to, get the parts, and build the machine in order.
3. **End:** the thermometer becomes the crank, the stomach rumbles, the shift whistle blows, the boy asks for toast, the sign goes to 1, and Mrs. Novak's note says he'll be back Tuesday.
4. **Onward:** advance to DAY.
