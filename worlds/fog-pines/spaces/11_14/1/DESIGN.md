# The Viaduct Cake — design

## The idea

Saturday, 3 o'clock, in Nan's kitchen. Dot is nine. It's her little brother Pip's sixth birthday. Tea is at five, and Nan always makes the birthday child a **map cake**: a landscape built in a glass dish from sponge, jam, coconut and sweets. Pip has pinned his order to the fridge: the big railway viaduct with the train going over, a purple river, hills, five sheep and a sun.

Dot is the helper. She also keeps **a ledger**: an exercise book headed WHAT PIP GOT | WHAT I GOT, where she tallies every unfairness. Pip is winning 10 to 2.

While she builds his valley she gets small chances to even the score: lick the whole spoon, eat the offcuts, eat a sheep, eat the cherry that was meant to be the sun. Each one gets written in her column, and each one leaves a mark on his landscape (four sheep, an overcast sky). Nan never tells her off. She just says what happened, kindly ("One's wandered").

The bottom of Pip's order is folded under the strawberry magnet. "That bit's private. He made me promise." That goes in the ledger too: *secrets with Nan.* At the end Nan unfolds it:

> 7. DOT. UNDER THE BRIDGE. HOLDING MY HAND LIKE WHEN THE TRAIN WENT OVER AND I WASNT SCARED BECAUSE SHE SAID ITS ONLY LOUD. DONT TELL DOT

Dot models the two of them in marzipan and stands them hand in hand on the riverbank under the arches. Then Nan says: *get down to his height and look.* Dot crouches until her eye is level with the counter, and **the cake becomes a real landscape**: sponge hills against the bright window, the jam river shining, Rich Tea arches dark against the sky. Nan pushes the liquorice train across the viaduct with one finger over two small marzipan children. *It's only loud.* Then Dot reckons her ledger.

**The beauty I found:** a child crouching to put her eye at counter level, so the cake on the counter turns into a horizon. The player cuts that horizon themselves, with a knife, in the first minute, and the jam finds its own way down to the lowest point of whatever valley they carved. The landscape in the climax is the one their hands made. The jealousy changes the same way the batter does: by heat and time, into something you can stand on.

**Coordinates check:** child's afternoon (3 to 5 pm, the clock moves), jealousy (the ledger, the thefts, "Is Dot helping?"), side-on 2D (a glass dish seen side-on shows the strata, which is why it's Pyrex), warm and domestic (Nan, Formica, oven gloves), a landscape (the cake is one, and the crouch makes it literal), food and cooking (eggs → batter → sponge, berries → jam), transformation (every step changes a material, and the grievance turns into being asked for), reckon a ledger (tallies throughout, ruled off and reckoned at the end).

## Medium

A **side-on 2D canvas** (no three.js needed). Draw everything with Canvas 2D in a logical space of **1600 × 900**, scaled uniformly to fit the window and letterboxed with the wallpaper colour. Mouse-driven, with cursor coordinates converted to logical space. It's a side-on cutaway of one kitchen wall, like a picture book page or a doll's house. Side-on suits this because the glass dish shows the cake's strata, and because the crouch can then "drop the camera" to counter level.

No pointer lock. Use a normal cursor, hidden over the canvas and replaced by a drawn one: a small hand, or the held item drawn at the cursor.

## Arrival: the visual tie-in (important)

The dreamer asked that a space which looks nothing like the street should **visibly connect to it**. So the first 6 seconds are an establishing shot, not a menu:

1. (0–2.5 s) A side-on view of the house from the street: the same plain house as every house on the street, grey-blue fog pines behind, late-afternoon light. One window glows warm yellow. Caption, bottom centre, in the serif speech font: *"Inside: Nan's kitchen. Saturday. Pip's birthday."*
2. (2.5–5.5 s) The camera eases into the glowing window. Scale the exterior drawing around the window centre from 1× to about 6× with an ease-in-out curve. As the window fills the screen, crossfade (0.6 s) to the kitchen. Through the kitchen window you can then see the same fog pines, so you know where you are.
3. The kitchen fades in. Nan's first line appears. The wall clock reads 3:00.

Clicking during the intro skips it.

## Layout of the kitchen (logical px, y down; counter top at y = 600)

| Thing | Box | Notes |
|---|---|---|
| Wall | 0–1600 × 0–600 | buttery wallpaper `#F2D98A`, small orange daisies `#D9822B` every 80px on a lattice, low contrast |
| Counter top | 0–1330 × 600–620 | mint Formica `#A8D5C2` with faint boomerang squiggles; a chrome edge strip `#C9CED1` at 618–622 |
| Cupboard fronts | 0–1330 × 622–900 | cream `#E8E0CC`, door seams every 220px, small brass knobs |
| Fridge | x 20–200, y 170–900 | rounded cream `#EFE8D8`, chrome handle; drawn in front of the counter (it's a freestanding tall fridge) |
| Pip's order slip | x 50–170, y 250–420 on the fridge | lined paper with crayon drawing; bottom 40px folded up and held by a **strawberry magnet** (red, green leaf, 26px) |
| Dot | x 215–335, head top y 350 | behind the counter, standing on a stool; visible from the waist up. Round face, brown bob, red cardigan `#C0392B`, white collar. Large hit box (x 215–335, y 350–600) = "Dot" (eat target) |
| Left counter zone | x 220–420 | egg basket (x 225–300, y 560–600), mixing bowl (cream with blue band, x 310–415, y 528–600) |
| **Pyrex dish** | x 430–1110 (680 wide), y 440–600 (160 tall) | glass. Draw the contents first, then the glass on top: pale blue-green `rgba(190,225,220,0.35)` fill, 3px edge `rgba(150,200,195,0.9)`, a vertical white highlight streak at x+30 and x+640, rounded bottom corners r = 14. Thick lip at the top |
| Window | x 520–1020, y 70–330 | white frame 14px, cross-bar. Outside: sky gradient `#F7E7C6`→`#E9C9A0`, a lawn strip, a tyre swing on a branch where **Pip** (small: yellow anorak, swings gently) plays, fog pines `#6E7F86` at 60% opacity behind. Net curtain halves (semi-transparent white) at each side |
| Wall clock | centre (1160, 130), r 42 | cream face, black hands; shows story time (see clock) |
| Nan | x 1150–1290, head top y 290 | behind counter: grey set curls, glasses, floral pinny `#7A9CC6` with white dots, cardigan sleeves pushed up. Mouth moves (small oval toggles) while her bubble is typing |
| Right counter zone | x 1115–1325, y 540–600 | where Nan "puts out" each step's items (only the current step's items are shown) |
| Cooker | x 1330–1580, y 380–900 | 1970s freestanding: eye-level grill box y 380–440, control panel y 450–500 with a **kitchen timer dial** (centre 1455, 475, r 18), hob top at y 600 with a pan on the near ring, oven door y 640–860 with a window (x 1370–1540, y 680–790) that shows the dish when it's inside, lit orange |
| Ledger (closed/small) | x 230–520, y 650–870 | open exercise book lying on the cupboard area as an overlay, always visible after it's introduced. Click to enlarge to centre (900×640); click outside or the book again to shrink |

The afternoon sunlight from the window is a soft parallelogram of `rgba(255,236,190,0.25)` falling on the counter and dish. It starts slanting right and moves left a little as the clock advances.

**Wall clock (story time):** 3:00 start · 3:10 eggs done · 3:15 beaten · 3:20 into oven · bake spins to 3:40 · 3:55 carved · 4:05 river · 4:10 fields · 4:20 bridge · 4:30 sheep · 4:40 marzipan · 4:50 reckoned. Animate the hands to each new time over 1 s (12 s during the bake).

## Text styles

- **Nan** speaks in a cream speech bubble `#FFF8E7` with a 2px `#8A6A4A` border and a tail toward her head, placed above-left of her (keep it inside the screen; max width 420, wrap). Font: Georgia, 22px, colour `#3B2A1E`. Characters type in at 28 ms each. A line stays until the next Nan line, or 9 s.
- **Dot's thoughts** are pencil handwriting with no bubble, above her head: `"Chalkboard SE", "Comic Sans MS", "Marker Felt", cursive`, 21px, `#5A5A5A`, italic, with a faint paper-white glow behind for legibility. 7 s.
- **Pip** (through the window) is a small white bubble with capitals, Arial Rounded / sans 18px bold, `#2F4F7F`.
- **Hint line**: bottom centre, y 885, 18px sans, `#6B4F3A` on a translucent cream pill. It always says what to try next. If the player has done nothing useful for 20 s, Nan repeats the step's instruction and the target object pulses with a warm outline (`#FFB347`, 3px, alpha 0.4–1 sine at 1.5 Hz).
- **Hover**: anything interactive under the cursor gets the warm outline, plus a small label by the cursor (13px sans on a cream tag), e.g. "Pip's order", "an egg (Henrietta's)".

## Interaction model

- **Click an item to pick it up** (it follows the cursor, drawn at 1.1× with a soft shadow). **Click a target** to use it there. **Click empty space or right-click** to put it back (it animates home).
- **Holding something edible and clicking Dot eats it.** Edible: the spare wooden spoon, the offcuts, a marzipan sheep, the glacé cherry. Eating: Dot's arm comes up, a 0.6 s munch (her cheeks puff twice), a soft "nom" sound, then the ledger entry is written. The hover label on Dot while holding an edible thing reads **"eat it"**, so the option is discoverable but never pushed.
- Drag actions (stirring, carving, sieving) need the mouse held down while you move.
- Keys: **S / ↓** crouch, **W / ↑** stand, **E** knock on the window at the end. Everything also has a clickable on-screen equivalent. Don't bind Esc, =, -, G, ?, or `.

## The ledger (Dot's book)

An exercise book: paper `#FBF8EE`, feint blue lines `#A9C4E0` every 28px, red margin `#E07A7A`, a vertical pencil line splitting the page into two columns. Heading in Dot's pencil hand:

**WHAT PIP GOT** | **WHAT I GOT**

Cover (shown only during the first open animation, 0.8 s): *"DOT'S BOOK. PRIVATE. THIS MEANS PIP."*

Starting entries:

PIP column (10):
1. the top bunk
2. the blue cup (MINE FIRST)
3. carried in from the car asleep (I WAS ASLEEP TOO)
4. sat on Nan's lap all of church
5. the last Wagon Wheel
6. let off drying up ("he's only little")
7. a PARTY (I got a "tea")
8. Grandad's penknife to HOLD
9. top of the milk x3
10. a birthday cake THE SIZE OF A VALLEY

I column (2):
1. 1 Wagon Wheel (Tues)
2. my hens (but they are WORK)

Totals sit at the bottom of each column as a number plus tally marks: "10 卌卌" and "2 ||". Draw tallies as pencil strokes, with the fifth stroke diagonal across four.

New entries write themselves in letter by letter (35 ms per character) with a pencil-scratch sound, and the total ticks up with one new tally stroke. When an entry is added while the book is small, it briefly enlarges (scales 1.0 → 1.35 around its centre for 1.6 s) so the player sees it, then shrinks back.

Entries added during play (each at most once unless noted):
- PIP: `secrets with Nan` (folding the slip)
- I: `double yolk (my hen)` (third egg)
- either `the spoon (WHOLE)` in PIP **or** `the spoon (whole)` in I, depending on who gets it
- I: `offcuts (cook's perks)` (if eaten)
- I: `1 sheep`, which updates to `2 sheep` and so on rather than adding new lines (if eaten). Count it as +1 total per sheep.
- I: `the sun` (if the cherry is eaten)
- final reckoning line (see the end)

So Pip ends up with 11 or 12. Dot ends up with somewhere between 3 and 9. It never balances, and that's the point.

## The arc, step by step

The state machine is `step` from 0 to 13. Each step lists what's on the counter, the hint, and what moves things on. Nan "puts out" a step's items: they slide in from just behind her onto the right counter zone over 0.4 s with a soft clunk.

### Step 0: Arrival
Nan: **"There you are. Your brother's tea is at five and there isn't a crumb made. He's put his order on the fridge."**
Hint: *Click Pip's order on the fridge.* (The slip pulses from the start.)

### Step 1: Pip's order
Clicking the slip opens a **reading panel**: a centred paper 620×720, lined, with a crayon drawing across the top half. The drawing: a wobbly bridge with three arches in brown crayon, a squiggly purple river, green hump hills, five cloud-sheep with stick legs, a yellow sun with a face and lines, and a black train with a smoke curl. Below it, in wobbly blue capitals (slightly rotated letters, ±3°):

> MY CAKE ORDER
> BY PIP (6 ON SATERDAY)
> 1. THE BIG BRIDGE WITH ARCHES (THE ONE WITH THE TRAIN)
> 2. A RIVER UNDER IT. PURPEL.
> 3. HILLS
> 4. SHEEP. 5 OF THEM.
> 5. THE SUN
> 6. A TRAIN GOING OVER

Under item 6, the bottom of the paper is drawn **folded up**: a darker flap with a shadow, held by the strawberry magnet, labelled on hover *"folded under — stuck with the magnet."* Clicking the flap (or the magnet) gives:
- The magnet won't budge (a little shake).
- Nan: **"Ah-ah. That bit's private. He made me promise."**
- Dot (pencil): **"Secrets with Nan. That's going in the book."**
- The **ledger is introduced**: it flies in from the bottom left, the cover shows for a moment, it opens, and `secrets with Nan` is written in PIP's column (10 → 11).

Closing the panel (click outside it, or a small "×") is allowed at any time. Step 1 completes once the flap has been tried **or** the panel has been closed after being open at least 3 s. If the panel was closed without trying the flap, introduce the ledger anyway with Dot thinking: **"Bottom's folded under. Secrets. That's going in the book."** and the same entry.
Then Nan: **"Eggs first. Your hens'."** The egg basket pulses.
Hint: *Pick up an egg, then click the bowl.*

### Step 2: Eggs
The basket holds 3 brown eggs, each with a hover label: "an egg (Henrietta's)", "an egg (Mavis's)", "an egg (Henrietta's again)". Click an egg, then the bowl: the egg taps the rim (crack sound), splits into two shell halves that tip, a yolk drops in, and the halves vanish.
- After egg 1: nothing said.
- After egg 2: Dot: **"Mavis lays small but she means it."**
- Egg 3 is a **double yolk** (two yolks drop). Nan: **"A double-yolker! Henrietta's showing off for him."** Dot: **"Henrietta is MY hen. That's mine."** Ledger: I `double yolk (my hen)`.
Then Nan: **"Now beat it till it goes pale. Round and round."**
Hint: *Hold the mouse down on the bowl and stir in circles.*

### Step 3: Beat
The bowl shows dull yellow batter (`#E2B84A`) with yolk blobs and a wooden spoon standing in it. Hold the mouse within 120px of the bowl centre (362, 560) and move it in circles. Add up the absolute change in angle around the centre; you need 6 full turns (12π). The spoon follows the cursor angle, the surface swirls (draw a spiral stroke that rotates), the yolk blobs fade, and the colour lerps to pale cream `#F3DFA2`. Play a soft rhythmic slosh about every 90° of travel.
Done → Nan: **"Lovely. Into the dish with it."** The spoon now lies on the counter beside the bowl, coated, with the hover label "the wooden spoon (going spare)".
Hint: *Click the bowl, then the glass dish.* (After 4 s, add: *The spoon's going spare, if you want it.*)

Pouring: click the bowl, then the dish. The bowl tilts over the dish and a ribbon of batter pours for 1.2 s. Batter fills the dish from y 590 up to y 530 (60px thick, colour `#F3DFA2`, flat top with a slight wobble).
Nan: **"Into the oven. I'll carry, you set the timer: twenty minutes."** The dish pulses.
Click the dish: Nan, in quilted oven gloves (`#C0392B` check), carries it to the oven. Animate the dish moving in an arc to the oven window over 1 s, then the oven door shows it inside, glowing. Then the timer dial pulses.
Hint: *Click the timer on the cooker to wind it.*
Click the dial: it winds round to 20 with ratcheting clicks over 0.8 s, then ticks.

**The spoon's fate** is resolved at the moment the timer is wound:
- If Dot ate the spoon earlier (holding the spoon and clicking Dot): ledger I `the spoon (whole)`. At the time of eating, Nan: **"Usually you'd go halves on that with your brother."** Dot: **"Usually."**
- If the spoon is still on the counter: Pip's face pops up at the window (pressed to the glass, nose squashed, 0.4 s). Nan opens the window a crack and passes the spoon out: **"Here, birthday boy. Now shoo."** Pip: **"THE WHOLE SPOON!"** Dot: **"He got the spoon. The WHOLE spoon."** Ledger: PIP `the spoon (WHOLE)`.

### Step 4: Bake and jam (runs in parallel)
The bake takes **12 real seconds**. The clock hands sweep from 3:20 to 3:40. In the oven window, the batter rises from 60 to 120px thick and its colour goes from pale cream to golden, with a darker top crust appearing. A faint shimmer of heat lines rises from the cooker top. The timer ticks.

At the start of the bake, Nan puts the **pan of blackberries** on the hob (it was there already; now it gets a sugar bag tipped in) and says: **"While that's in: blackberries. You picked them, you mash them."**
Hint: *Click the pan to mash the blackberries.*
Each click on the pan (8 needed) makes a masher plunge, a squelch sound, and colour lerps from berry-dots on pale syrup to deep jam `#5B1A3A`. After 4 clicks, bubbles start (small circles rising and popping, with a low "blup" sound every 0.7 s).

At bake time 6 s, Pip pops up at the window again: **"IS IT THE BRIDGE? IS IT? IS DOT HELPING?"** Nan: **"Out, you! Not till five."** Pip drops out of sight. Dot: **"'Is Dot helping.' Like I'm the mixer."**

When the 12 s are up **and** the jam is done (if the jam isn't done, the timer has already rung once and Nan says **"Another minute won't hurt it."**, then you wait for the jam): ding! (a bright bell, two partials). Nan opens the oven and carries the dish back to the counter (1 s). Nan: **"Look at that. Risen like a Sunday."**

The sponge is now a golden slab: top at y 470, bottom at y 590 (120 thick). Draw the cross-section crumb (dots of a slightly darker gold) and a 6px darker top crust `#C98A3A` that follows the top surface (see heightmap below).

Then Nan sets a round-ended knife on the counter: **"Now. A valley for his river, with hills either side. Cut down from the top. Mind, you can't put back what you take off."**
Hint: *Pick up the knife, then hold the mouse and drag it across the top of the sponge.*

### Step 5: Carve the horizon
**Heightmap:** an array `top[i]` for i = 0..679 (one per pixel column of the dish interior), each the y of the sponge surface. They start at 470. The dish floor is at 590.
While holding the knife with the mouse down over the dish: for every column within ±7px of the cursor x, set `top[i] = max(top[i], clamp(cursorY, 470, 578))`. Carving only lowers the surface, never raises it. Smooth the edited band lightly (average each edited column with its neighbours once) so the edges aren't jagged. The knife is drawn at the cursor tilted 30°. Make a soft "shhk" cut sound every 12px of travel while it's cutting.
Each column cut adds to `offcutVolume` (the sum of removed pixels). Once `offcutVolume > 600`, a **pile of offcuts** (crumbly golden lumps, growing with volume up to a 70×30 heap) appears on the left counter at x 230–300, y 570–600 where the egg basket was (the basket is gone by now). Its hover label is "the offcuts". Dot thinks once: **"Offcuts don't count. Everyone knows that."**
If the player eats the offcuts (pick up the pile, click Dot): ledger I `offcuts (cook's perks)`. Nan: **"Cook's perks. That one's fair."** Dot: **"Writing it down anyway."**

**Valley detection:** a valley exists when there's a column `v` with `top[v] ≥ 540`, some column left of `v` with `top ≤ 505`, and some column right of `v` with `top ≤ 505`. When this first becomes true, Nan says **"That's a valley. Say when."** and a cream button appears beside the dish at (1010, 410): **"That'll do."**
**Fallback:** if after 25 s of carving there's no valley yet, show the button anyway with Nan: **"Whatever you've got, he'll call it a valley."** Clicking "That'll do" puts the knife away and moves on.
If the offcuts are never eaten, at step 13 Nan sweeps them into a tin: **"Offcuts in the tin for a trifle."** (no tally).

### Step 6: The river
Nan: **"Pour your river. It'll find the low bit on its own."**
Hint: *Click the jam pan, then the dish.*
Click the pan, then the dish. The pan tips over the dish centre. Jam falls in a stream to the **lowest column** (largest `top[i]`; if there's a tie, take the one nearest the centre). Then it **fills like water**. Let `V = 2400` (jam area in px²). Starting at level `L = top[min]`, lower L (move the surface up) 1px at a time. Find the contiguous run of columns around `min` with `top[i] > L`. If the run touches the dish wall at i = 0 or 679, stop: the jam is against the glass, which is fine. Otherwise, the filled area is `Σ (top[i] − L)` over that run; stop when it's ≥ V. Animate the level rising over 2 s.
Store `riverL` (the level) and `riverRun = [a, b]`. Draw the jam from `L` down to `top[i]` in `#5B1A3A`, with a glossy highlight line 2px `#A33A6A` along L and a few slow-drifting light glints. Through the glass you can see the jam soaking a little into the sponge: a 4px purple-tinted band just under the jam surface.
Nan: **"Purple, as ordered."**

Nan puts out a bowl of **green coconut** with a sieve: **"Fields. Shake that over the hills."**
Hint: *Pick up the sieve, then hold the mouse and drag it over the hills.*

### Step 7: Fields
Holding the sieve with the mouse down over the dish: for columns within ±20px of the cursor x that aren't inside `riverRun`, add to `green[i]` (0..6px) at about 6px per second of hover. Draw green `#7DBB4A` speckle on top of the sponge surface, `green[i]` thick, with a few lighter flecks `#A6D96A`. The sieve jiggles and sprinkles falling dots, with a soft shushing noise while it moves. A few flecks landing on the jam float there; that's fine.
Done when 60% of the non-river columns have `green ≥ 3` (or after 15 s of sieving). Nan: **"Spring's come."**

### Step 8: The viaduct and the train
Nan puts out a packet of **Rich Tea** (red and white wrapper) and a box of **chocolate fingers**: **"Now the bridge. Pass me the Rich Teas."**
Hint: *Click the biscuits.*
Clicking the packet plays a 3 s build (Nan's hands, drawn simply as two pink rounded mitts, do the work):
- **Span:** from `riverRun[0] − 50` to `riverRun[1] + 50`, at least 200px wide (widen it symmetrically if needed) and clamped inside the dish. Deck height: `deckY = min(top at both span ends) − 10`, clamped to at least 450.
- **Pillars:** chocolate fingers (`#5A3A22` with a lighter edge, 10px wide) every 52px across the span. Each stands from `top[i]` (or the riverbed, inside the jam) up to `deckY`. They're planted one by one with a tiny thunk.
- **Deck:** Rich Teas laid edge-on along `deckY` as a 14px beige bar `#D9B27A` with pin-prick dots. Under the deck between each pair of pillars there's a spandrel of biscuit with a **semicircular arch bitten out** (draw the biscuit-coloured band down to `deckY + 26` and subtract a semicircle of radius ≈ half the gap, centred at the gap's bottom). Each bite gets a crunch sound.
- Nan, chewing: **"Engineering."** Dot: **"She ate the arches. Nobody's writing THAT down."**
Then Nan puts out the **liquorice allsorts**: **"And his train."** Click the bag: a train sits on the deck at the left end. Engine: a black liquorice cylinder (a rolled bootlace, 36×20) with a pink coconut-wheel chimney. Two carriages: allsorts sandwiches (black, white, black, pink, black stripes, 24×20). Black bootlace rails run the length of the deck.

### Step 9: Sheep and sun
Nan puts out a saucer with **5 marzipan sheep** (cream ovals `#F5EEDC`, black head dots, stick legs, 22×16) and a tub with **one glacé cherry** on a cocktail stick: **"Five sheep, as ordered. And the last glacé cherry for his sun."**
Hint: *Pick up a sheep and put it on a hill.* (Small second line: *Or click Dot to eat whatever you're holding.*)
- **Placing a sheep:** click on the dish at any x outside `riverRun` and not under the deck pillars' 6px. It settles on `top[i] − green[i]` with a tiny bounce, facing a random way. Clicking over the jam: it bounces back to the saucer and Nan says **"Sheep can't swim, love."**
- **Eating a sheep:** ledger I `1 sheep` (then `2 sheep`, and so on). Nan, the first time: **"Four sheep. Well. One's wandered."** After that: **"Another's wandered."** If all five are eaten: **"No sheep. It's a very quiet valley."**
- **Placing the cherry:** click anywhere in the dish above the surface. A cocktail stick plants into the nearest surface below the click, and the cherry sits at the click height (min y 400, so it can stand proud of the dish rim). It glows faintly. Nan: **"There's his sun."**
- **Eating the cherry:** ledger I `the sun`. Nan: **"Overcast, then. Very true to life."** She spoons a whipped-cream cloud onto a cocktail stick and plants it at the dish centre, high: a white cloud of 3 overlapping circles. (The cloud replaces the sun everywhere, including the crouch view.)

The step ends when the saucer and tub are empty.

### Step 10: The fold
Nan wipes her hands on her pinny, goes to the fridge (walk animation 1 s: her figure slides left behind the counter, then back), and peels off the strawberry magnet. **"Right. Last thing on his list. He said not to show you till the end."**
The order slip reading panel opens on its own, and the flap **unfolds** (it flips down over 0.6 s, with a paper sound). Revealed beneath, in the same wobbly capitals:

> 7. DOT.
> UNDER THE BRIDGE. HOLDING MY HAND LIKE WHEN THE TRAIN WENT OVER AND I WASNT SCARED BECAUSE SHE SAID ITS ONLY LOUD.
> DONT TELL DOT

Beside it is a crayon drawing of two stick figures, one big and one small, a line joining their hands, and a bridge above them. Both figures are smiling enormously.

Hold for 3 s. Then Dot (pencil): **"..."** and, 2 s later: **"I did say that."**
Nan: **"He asked for you. On his cake. Marzipan's on the side. Do the both of you."**
The panel closes when the player clicks.

### Step 11: Marzipan Dot and Pip
Nan puts out two marzipan lumps: one tinted red, one tinted yellow.
Hint: *Click each lump to shape it.*
Click a lump: a 1 s modelling animation (the lump squashes and stretches, with three soft pats) turns it into a small figure, about 26px tall:
- **Dot**: red cardigan, brown bob (cocoa-dusted), arm out.
- **Pip**: smaller (20px), yellow anorak, hood up, arm out.
Hint: *Stand them under the bridge, on the riverbank. Close together.*
Pick up a figure and click inside the bridge's span on dry ground (not jam). It stands on the surface there. If the two are within **28px** of each other, **their hands join**: draw a short line between their outstretched arms, with a soft "pat" and a single quiet piano note (A4, sine with a slow decay). If they're further apart, they just stand there, and the hint changes to *Closer. They're holding hands.* The player can re-pick and move them. You can't eat these: clicking Dot while holding one shows a tiny shake and the label "not these".
Hands joined → Nan: **"There."** Pause 1.5 s. Nan: **"Now get down to his height and look. That's how he'll see it."**
Hint: *Press S or ↓ to crouch down* (and a cream button under Dot: **"Crouch down"**).

### Step 12: The crouch: the cake becomes a landscape
On crouch, the whole kitchen slides **up** out of frame over 1.2 s (as if the eye drops). The counter edge rises past, and then a **separate eye-level scene** is rendered from the same data:
- **Background**: the window's view scaled to fill the screen: warm sky gradient, the fog pines in soft layers, the lawn, the tyre swing small and empty. Overlay a pale vignette of the window frame at the edges, faintly, so you know you're still in the kitchen.
- **The cake landscape** across the full width: map dish columns 0..679 to screen x 0..1600 (scale ≈ 2.35). Put the dish floor at screen y 900 and scale heights by the same factor, so top 470 → about 900 − (590 − 470) × 2.35 = 618 for an uncut hill. The sponge is the land, with the green layer on top, the jam river with a strong glossy highlight reflecting the sky colour, the chocolate-finger pillars and the bitten Rich Tea arches **silhouetted dark against the bright sky**, the sheep, the sun (cherry: a red disc glowing with a soft halo, low in the sky, on its stick) or the cream cloud, the two marzipan children small under an arch, hands joined.
- The front glass of the dish runs along the bottom 30px as a pale blue-green band with one highlight, so it's still a cake.
- 1.5 s after arriving: **Nan's finger** (a big soft pink fingertip with a ring, entering from the right edge) pushes the train along the deck from left to right over **6 s**. Make chuffs (filtered noise pulses) speeding up slightly. As the train passes directly over the two children, there's a two-note whistle (D5 then B4, sine and triangle, 0.35 s each). The deck shadow passes over them.
- When the train reaches the far side: Dot (pencil, centred near the top): **"It's only loud."**
- Hint: *Press W or ↑ to stand up.* (Auto-stand after 5 s idle.) The button reads **"Stand up"**.

Standing reverses the slide.

### Step 13: Reckon the ledger
The ledger opens to its large size in the centre, by itself. Dot rules a pencil line under both columns (an animated stroke with a scratch sound). Totals are written beneath, in numbers and tallies.
Above the choices, in pencil: **"Last line. Reckon it."**
Three choices, drawn as pencil-written options with hover underlines:
- **A**, *In Pip's column: "asked for me."* The line is written in PIP's column and the total ticks up. Dot: **"That's {P} to him. He can have it."**
- **B**, *In my column: "got asked for."* Written in I. Dot: **"{D} to me. Still losing. Doesn't feel like it."**
- **C**, *Rule it off: "ACCOUNT CLOSED."* A diagonal pencil line crosses both columns, with "ACCOUNT CLOSED, Pip's birthday" written across it at an angle. Dot: **"Nan says you close the books on birthdays. I just made that up. It's true now."**

({P} and {D} are the totals after the line is added.)

After the choice (1.5 s), the book shrinks back. Nan: **"Go and fetch him in, then."** And, depending on what was eaten (pick the first that applies):
- any sheep eaten: **"Wipe your mouth first. You've got sheep on you."**
- the cherry eaten: **"And wipe your mouth. You've gone red as a pillar box."**
- the spoon or the offcuts eaten: **"And brush yourself down. Crumbs."**
- nothing eaten: **"Go on. He's been on that swing an hour."**

## The way onward

The window glows (warm pulsing outline). Through it Pip is on the tyre swing again. A label tag hangs below the window: **"Knock for Pip  →  (E)"**. Hint: *Knock on the window for Pip.*
Click the window or press E: three knocks (wooden thuds, 0.25 s apart, with the glass rattling slightly). Pip stops swinging, turns, and runs toward the house, getting bigger. Over 1.2 s the kitchen warms toward white-gold, then:
- `branches.say("Pip's coming in. Tea's at five.")`
- `branches.carry([...])` (below)
- `branches.onward()`

The arc earns this. The onward only exists after the reckoning, at about 3.5 minutes. It can't be missed: a glowing window, a label, a hint, and a key.

**Carried items:** `["Dot's ledger (" + choiceText + ")", "Pip's cake order (unfolded)", "the strawberry magnet"]`, where choiceText is `"asked for me" | "got asked for" | "account closed"`. If Dot ate anything, also add `"crumbs on Dot"`. `branches.carried` coming in is ignored (this is the house's first space), but never let it throw.

## Sound (WebAudio, synthesised, no files)

Start the AudioContext on the first click. Keep the master gain low (0.5).
- **Room tone:** the wall clock ticks (a short click every 1 s, alternating pitch 1800/1600 Hz, very quiet). Distant birds outside every 6–12 s (two quick sine chirps, 2.5–4 kHz). Rarely, a far-off "WHEEE" from Pip on the swing: a formant-ish sawtooth glide, very quiet. Skip that if it's fiddly.
- **Egg crack:** a noise burst of 40 ms through a bandpass at 2 kHz plus a click.
- **Stir:** a lowpassed noise "slosh", 120 ms.
- **Pour:** filtered noise swell.
- **Timer wind:** a ratchet of 8 clicks. Then a tick at 4 Hz during the bake. **Ding:** 1320 Hz + 2640 Hz sines, 1.2 s decay.
- **Mash:** a low squelch (noise through a lowpass at 400 Hz with a quick pitch-down sine). **Bubbles:** short 120–200 Hz sine blips.
- **Knife:** a short high-pass noise "shhk".
- **Sieve:** a continuous quiet hiss while moving.
- **Crunch:** noise burst with random bandpass.
- **Pats:** 60 ms lowpassed thumps.
- **Pencil:** fine high-passed noise scratches as each letter writes.
- **Nom:** two soft low thumps.
- **Train:** chuff pulses plus the whistle.
- **Knocks:** 3 × a 140 Hz sine thump with fast decay, plus noise.
- **Hands join:** one sine note at 440 Hz with a 2 s decay, quiet. This is the only "musical" sound in the space; keep it that way.

## Colour summary

Wallpaper `#F2D98A` / daisies `#D9822B` · Formica `#A8D5C2` · cupboards `#E8E0CC` · fridge `#EFE8D8` · Nan's pinny `#7A9CC6` · Dot's cardigan `#C0392B` · Pip's anorak `#F2C230` · batter `#E2B84A` → `#F3DFA2` · sponge `#E8B45A`, crust `#C98A3A` · jam `#5B1A3A` / highlight `#A33A6A` · coconut `#7DBB4A` / `#A6D96A` · Rich Tea `#D9B27A` · chocolate `#5A3A22` · liquorice `#1A1414` · allsort pink `#F3A6C0`, yellow `#F6D55C` · marzipan `#F5EEDC` · cherry `#D7263D` · glass `rgba(190,225,220,0.35)` · sky `#F7E7C6` → `#E9C9A0` · pines `#6E7F86` · ink `#3B2A1E` · pencil `#5A5A5A`.

Everything should be drawn with soft rounded shapes and 2px darker outlines (about 30% darker than the fill), like a 1970s picture book. Nothing photographic.

## Robustness notes

- Never stall: every step has a hint, a pulsing target after 20 s idle, and the fallbacks above (carving and sieving have time-outs; bake-and-jam waits for both).
- No fail states. Nothing can be lost permanently except what Dot eats, and eating always has a graceful substitute (the cloud) or simply leaves fewer sheep.
- Clicks during a scripted animation are ignored (except skipping the intro).
- Wrap all text to its bubble width. Keep bubbles inside the 1600×900 frame.
- If anything throws in the render loop, catch it, log it, and keep drawing the kitchen, so the screen is never blank.

## First thing to try

Click **Pip's order on the fridge** (it pulses, and Nan has just told you about it).
