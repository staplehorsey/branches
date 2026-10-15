# Nothing After 1948

## The idea

Delhi, Saturday 31 January 1981. Richard Attenborough has wanted to make *Gandhi* for nearly twenty years, and this morning he shoots the funeral on Rajpath, the great avenue between the Viceroy's palace and India Gate, on the 33rd anniversary of the real one. Around 300,000 people turned up, most of them unpaid volunteers who answered a newspaper notice, many of whom stood on this road in 1948. Eleven camera crews. There is no rehearsing a crowd that size. Realistically you get one good take.

You are a gate assistant at **Sector C, Checkpoint 4**, a trestle table on the north lawn. Your job is a ritual in steps, and the core of it is sorting. Each person who comes through has to be turned from 1981 back into 1948. You take off the watches, sunglasses, plastic and nylon, and then you **classify** each person by how they'll read on camera: FRONT (white, and close enough for the lens to count buttons), MIDDLE (pale from fifty yards), BACK (everyone else who came). Nobody goes home.

**The lovely thing:** to step into 1948, every one of these people hands you their time. Their wristwatches go into a big yellow Dalda ghee tin that ticks like a field of crickets, and each owner gets a brass token on a string. Yours goes in first. For the rest of the morning there's no clock in the game. You get the time only from a loudhailer. After the take you give everyone their time back, one token at a time. The last watch is yours, and when you buckle it on it beeps the hour: **10:00, SA 31 1**. It's 1981 again.

**Thread from space 1:** last night you found a thing you'd wanted badly, a grave two and a half miles down, and stood silent over the cable. Here the question "what do you owe it?" gets a working answer: a proper send-off, done carefully, with every detail right, and then giving people back what you borrowed. The tow cable becomes the **drag ropes of the gun carriage**. The thermal print (dark, deep, found) is joined by a **Polaroid** of the living front row that develops in your hand in the sun. Mrs Kamla Arora, who was nine in 1948, looks at your print and says, "So it was found. Then it's a grave too."

**Why first-person 3D:** the scale. You stand at a small table, and behind you is an avenue two kilometres long, packed with people in white, fading into the winter fog towards India Gate. The epic register comes from walking among them: a single small job inside an enormous one.

---

## Layout (metres; +x = east toward India Gate, road along x, north lawn is +z)

- **Ground:** a 2400 × 800 plane, dull winter grass `#6f7d4a`.
- **Rajpath:** a road strip x −1200…+1200, z −15…+15, red-gravel `#b08f78`, with low pale kerbs (0.15 high, 0.4 wide) at z = ±15.
- **Canals:** water strips z +88…+94 and −88…−94 running the whole length, colour `#7d9aa0`, slightly reflective (metalness 0.3, roughness 0.2). They read as long silver lines in the fog.
- **India Gate:** at x = +700, z = 0. Built from boxes: two piers 10.5 × 42 × 10 with a top beam, giving an arch opening 9 wide × 25 high. Buff sandstone `#d9c6a0`. At x = +760 there's a small domed stone canopy (4 posts, a 6 × 6 roof, a small dome).
- **Rashtrapati Bhavan:** at x = −1150, on a raised plinth 6 high. A long body 220 × 22 × 30, red-sandstone base `#a5523c` and cream upper `#e6d6b8`, with a dome (hemisphere r = 18 on a drum r = 18, h = 8). North and South Blocks: boxes 160 × 26 × 40 at x = −900, z = ±90, the same two-tone colours.
- **The crowd:** InstancedMesh figures. Each figure is a body (cylinder r 0.22, h 1.15, base at y 0) plus a head (sphere r 0.12 at y 1.35). There are about 9,000 instances on both lawns, x −450…+450, z ±16…±80, packed on a jittered 0.7 m grid. Colours: 75% white `#f3efe4`, 15% pale (cream `#e8dcc0`, fawn `#c9b48e`, grey `#b9b9b4`), 10% colour (maroon, navy, mustard, a little marigold). Beyond x ±450, out to India Gate and the palace, use long low "crowd band" planes (height 1.6, double-sided) textured with a canvas of dense white dots on dull green, so the crowd goes all the way to the horizon. Fog does the rest.
  - Keep empty: Sector C (below) except for its own fill, and the **aisle** x +9.5…+12 through Sector C so the player can walk from the table to the rope.
  - **Idle motion:** every frame, nudge about 400 random instances by a tiny head bob or sway. When "Quiet" is called, this stops section by section (see ACTION).
- **Camera towers:** scaffold towers (4 poles plus cross bars, a 2 × 2 platform at 12 m, a box camera, a seated figure) on the south side at z = −22, every 70 m from x = −350 to +350 (11 towers, as on the day). The **nearest tower** is at x = +20, z = −22. A figure stands on it with a cone loudhailer. That's **Mr Chopra, the sector AD**. His lines play as `say()` text prefixed with **📣**, plus a short loudhailer squelch.

### Sector C (the player's world, about 24 × 50 m)

- Bounded by **bamboo poles** (cylinders r 0.05, h 2.0, `#c8a86a`) every 3 m along x = −12 and x = +12 from z = 16 to 64, joined by **rope** at 1.0 m height. These are the player's walls: add `s.walls` for the bamboo lines and a wall along z = 16 at the road rope, so the player can't walk onto the road until the end. Also add a wall at z = 76 behind the queue.
- **Road rope:** the front edge at z = 16, a thick jute rope on short posts, with chalked marks on the grass.
- **Pens:** three zones painted on the grass with a lime chalk line and big chalk letters lying flat (canvas-textured planes):
  - FRONT z 16…19.5
  - MIDDLE z 19.5…36
  - BACK z 36…56
- **Gaps (the "spots"):** circles of chalk, 0.6 r, where sorted people will stand:
  - FRONT: (−4, 17.2), (0, 17.2), (+4, 17.2)
  - MIDDLE: (−5, 26), (+3, 30)
  - BACK: (−6, 44), (+2, 46) (the Malhotra sisters share this one and stand side by side)
  - Your own mark: **"AD C4"** chalked at (+10.8, 17.0) at the end of the aisle, by the rope.
- **Fill:** instanced figures fill all three pens, avoiding the spots and the aisle. FRONT is all white. MIDDLE is mostly white and pale. BACK is mixed.
- **The checkpoint**, at the back of the sector:
  - **Trestle table:** a 2.4 × 0.8 top at height 0.76, centred (0, 62), long side along x. A white cloth with a printed sign hanging on the front: **SECTOR C · CHECKPOINT 4**.
  - On the table, left to right:
    - the **call sheet** on a clipboard (x −0.8)
    - the **Dalda tin** (x −0.2): a cylinder r 0.14, h 0.28, bright yellow `#f2c230` with a green band and a palm-tree sign drawn on a canvas label "DALDA vanaspati"
    - the **token board** (x +0.4): a 0.5 × 0.35 plank of nails with brass discs, which empty as tokens go out
    - the **cane basket "LOST TIME & PROPERTY — 1981"** (x +1.0): an open cylinder r 0.22, h 0.18, `#b88a4a`
  - **Blackboard** on an easel at (−2.2, 62.6), 1.2 × 0.9, facing the player's side (−z). Chalk text (see Documents).
  - **Three sorting boards** on bamboo poles at the table's west end, at (−3.4, 61), (−4.3, 61) and (−5.2, 61). Each is plywood 0.8 × 0.5 at height 1.6, facing the player, hand-painted:
    - **① FRONT**, white board, black letters
    - **② MIDDLE**, cream board
    - **③ BACK**, sky-blue board
  - **Wardrobe trunk** at (+2.6, 61.6): a tin trunk 0.9 × 0.5 × 0.5, blue `#3d5f8a`, lid open, with folded white khadi showing.
  - **Petal basket** (appears in Step 3) at (+2.6, 60.8): a flat round basket heaped with orange `#f29a1d` and yellow `#f6c33b` marigold petals (small instanced quads).
- **The queue:** behind the table, inside a rope lane x −0.8…+0.8, z 64…76. The people waiting stand at z = 64.0, 65.3, 66.6, …, facing −z. The one at the front of the queue steps to **(0, 63.3)**. That's "at the table".
- **Player start:** (+1.5, 0, 59.5), yaw facing **south toward the road**. The first view is the backs of a thousand white heads, the rope, the empty red road, towers in fog, and India Gate faint on the left. The hint makes you turn round.

---

## Light, colour, sound

**Light progresses by step.** Store a `light` value from 0 to 1 and ease it as steps complete.
- **Dawn (0):** fog `#e9e4d8`, near 20, far 260. Hemisphere light: sky `#f1ece0`, ground `#6f7d4a`, intensity 0.9. Low sun (directional, `#ffd9a8`, 0.6) from the east-south-east, very low.
- **Morning (1, by ACTION):** fog far 1400. Sun `#fff4e0` at 1.4 and higher. Sky background lerps from pearl `#e9e4d8` to winter blue-white `#dfe8ee`.

Shadows are optional; skip them on the crowd.

**Palette:** white khadi `#f3efe4`, sandstone `#d9c6a0` and `#a5523c`, marigold `#f29a1d`, Dalda yellow `#f2c230`, olive drab `#4b5320` (the weapons carrier), navy whites (the ratings' uniforms are white with navy collars).

**Sound (all WebAudio, synthesised, started on first click):**
- **Crowd murmur:** brown noise → bandpass 400 Hz (Q 0.7), gain 0.25, slowly amplitude-modulated (0.1–0.3 Hz LFOs). Add a second, higher murmur layer (bandpass 1200 Hz, gain 0.08).
- **Crows:** every 6–15 s, a "kaa" (sawtooth 650→480 Hz over 0.18 s through bandpass 900, gain 0.08, panned randomly).
- **Loudhailer:** before each 📣 line, a squelch (square 1800 Hz for 0.05 s, then bandpassed noise 0.15 s).
- **The tin:** when within 3 m of the table, many soft ticks: random 3 kHz clicks (5 ms decay), about 14 per second at very low gain, rising as watches go in. It sounds like crickets.
- **Casio beep:** a square wave at 2048 Hz, two 80 ms beeps, 60 ms apart.
- **Radio commentary (Step 2):** a positional source on the radio boy. Bandpass 300–3000 Hz noise, amplitude-modulated at a syllable rate (random 3–6 Hz gating), with a sine "carrier whine" at 7 kHz, very quiet, and a crowd-roar swell (noise burst through lowpass 1 kHz, 1.5 s) every 8–14 s. Gain falls with distance (1/(1+d·0.35)) and is panned by the angle to the player. Audible from about 25 m.
- **Silence (ACTION):** the crowd murmur gain ramps to 0.02 over 6 s. Then you hear only: rope-pullers' feet (soft noise thumps at 1.6 Hz), a wooden wheel creak (filtered sawtooth glide every 2.5 s), and one crow.
- **Polaroid:** a motor whirr (sawtooth 120 Hz through lowpass, 0.6 s) and a click.

---

## HUD

- **Top right, a small "call sheet" card**, paper-coloured, typed font, always visible. It lists the steps with a tick (✓) as each completes. The current step is in bold.
  ```
  SECTOR C · CHECKPOINT 4
  1  Your watch in the tin
  2  The gate (0/7 sorted)
  3  A radio in the sector
  4  Petals to the front row
  5  Take your mark (AD C4)
  6  ACTION
  7  Give back the time
  ```
- **No clock anywhere** until step 7. Mr Chopra announces the time.
- Keys hint on arrival (bottom centre, fades after 12 s): **"E: look closer / use · 1 2 3: sort the person at the table"**.

---

## The arc, step by step

### Arrival

Spawn facing the road. The carried **thermal print** is already in the inventory (from `branches.carried`). After 1.5 s:

> `say`: "Rajpath, Delhi. Saturday 31 January 1981, before seven. Fog. Behind you is your checkpoint: Sector C, table 4. The call sheet's on the table."

If the player hasn't turned toward the table after 10 s:

> `say`: "Turn round. Your table is behind you."

**Interactions available from the start:**
- **Call sheet** (E) opens the reading panel (text in Documents).
- **Blackboard** (E) opens the reading panel with the rules.

After the call sheet has been read (or 20 s have passed), the 📣 loudhailer says:

> 📣 "Gate ADs! Your own watches in the tin as well. You are in shot at the rope, and Mr Attenborough can see a digital watch from the moon."

### Step 1: your watch in the tin

**Dalda tin** (E), label "the Dalda tin, full of watches":

> `say`: "You unbuckle your Casio. 06:41, SA 31 1. Into the tin it goes, onto two hundred others, all ticking. Token C4-00 goes round your neck on a string."

What happens:
- Inventory gets **"token C4-00"**.
- The tin's ticks get louder.
- Step 1 is ticked off and the queue starts.
- The first person walks up to (0, 63.3).

From now on, E on the tin before the end says:

> "Two hundred watches, all set to different minutes. Nobody here knows exactly what time it is."

### Step 2: the gate (the sorting heart)

People come to the table one at a time. Each is a non-instanced figure: body cylinder plus head, with clothing colours as described, a slightly darker hair cap, and an optional cap or turban. Props are separate small meshes attached to the figure, and each prop is a kit `thing`.

**For the person at the table:**
- **E on the person** opens their card. A reading panel titled with their name gives a description and what they say. It shows only the first time; after that they say a short line.
- **E on an anachronistic prop** removes it:
  - Watches go into the tin. A brass disc leaves the token board and appears briefly in their hand. `say` "Token C4-0N."
  - Other things go into the cane basket, and the mesh moves to the basket.
- **Wardrobe trunk** (E): adds **"khadi shawl"** to the inventory (only one at a time).
  > "A white khadi shawl, handspun, smelling of the trunk."

  E on a person while holding the shawl drapes it over them. The body material turns white and a shawl mesh is added. The shawl only matters for Kamla. On anyone else:
  > "They're fine as they are. Save it."
- **Sorting:** E on a sorting board, or keys 1/2/3 while a person is at the table.
  - **If any anachronism remains:** they don't go. 📣 or a `say` from the person (see per-person lines), for example:
    > "Not with that on him. Look again."
  - **If the pen is wrong:** they go a few steps, then come back. 📣 gives the person-specific correction.
  - **If correct:** the person says their departure line. They walk (1.4 m/s) round the table's east end, down the aisle, then across to their chalk spot, and turn to face the road. Tick the counter. The next person steps up.

**Classification rule (also on the blackboard):**
- Anyone wearing anything after 1948 is fixed first. Nobody is sent away.
- **FRONT:** all white.
- **MIDDLE:** pale (cream, fawn, grey).
- **BACK:** any colour, or uniform.

#### The seven, in order

**1. Mr P. N. Raghunath**, 70, retired railway clerk. Thin, in a white khadi dhoti and kurta, with a white Gandhi cap.
- **Prop:** a black **Casio digital watch** on his left wrist (a small black box). It beeps faintly every 30 s until removed.
- **Card:**
  > Mr P. N. Raghunath, retired, Northern Railway. "In '48 I walked from Paharganj with my brother and we stood on this side. Back there, near the canal. You couldn't see anything, only hats. Today they wrote to me in the newspaper and I said: this time, front." He holds out his wrist before you ask. "My son sent it from Dubai. It beeps on the hour. I don't know how to stop it."
- **Watch removed:**
  > "Token C4-01. He looks at his bare wrist as if he's just been told some news."
- **Answer: FRONT.**
  - Departure: "Front. Thirty-three years." (He doesn't hurry.)
  - Wrong-pen correction (📣): "Gate four, that's a white khadi gentleman. He's FRONT."

**2. Vinod Sethi**, 19, a Delhi University student. White kurta-pyjama, big mirrored aviator sunglasses (props), a glass **Campa Cola** bottle in his hand (a brown-orange bottle, prop).
- **Card:**
  > Vinod Sethi, B.Com second year, Hansraj College. "My grandmother told me, you go, you stand there for her because her knees can't." He pushes the aviators up into his hair. "These are Ray-Bans. Well, they're from Palika Bazaar. Same thing."
- **Sunglasses removed:**
  > "Into the basket. Without them he looks about fourteen."
- **Campa removed:**
  > "He finishes it first. That seems fair."
- **Answer: FRONT.**
  - Departure: "If I'm in it, she has to come to the Odeon."
  - With an anachronism left: "Arre, the bottle. Sorry, sorry."

**3. Mrs Kamla Devi Arora**, 42. A **pink nylon sari** (the body is pink `#e88aa8` with a faint sheen) and a small round gold **wristwatch** (prop) on a thin leather strap.
- **Card:**
  > Mrs Kamla Devi Arora, Lajpat Nagar. "I was nine. My father put me on his shoulders on that kerb, there, by the road, and I could see the flowers on him going past. I didn't understand why all the men were crying. I wore my best sari." The sari is pink nylon, bought new for today. She knows. "Is it wrong?"
- **E on her watch** (the twist). It does **not** come off. It opens a panel titled **"Her watch"**:
  > A West End Watch Co. "Sowar", the steel case worn soft. The back is engraved: *R. L. Arora, 1946.* It's a winding watch, wound this morning, keeping good time. "My father's. He had it on that day."
  >
  > Made before 1948. It belongs here.

  The prop's label changes to "her father's watch (1946): it stays". If the player tries E on it again:
  > "It stays. It was here the first time."
- **Sorting her while she's still in pink:**
  - FRONT: 📣 "Gate four! Pink nylon in my front row? Wardrobe! There's a trunk next to you!"
  - MIDDLE or BACK: she goes, stops, and comes back. Her line: "Please. I can see the kerb from here. Please, the front."
- **With the shawl draped:**
  > "She pulls the white shawl round her so the pink is gone. 'Like my mother's,' she says."
- **Answer: FRONT** (only after the shawl).
  - Departure: "On the kerb. Thank you, beta."
- **If the player sorts her before looking at the watch:** that's allowed. The watch stays either way. (The point is that the attentive player discovers why.)

**4. Harbans Singh**, 58, taxi driver, Ambassador taxi number DLT 4471. White kurta under a heavy **fawn** shawl (the body is fawn), navy turban. Holds a **car key on a plastic keyring** shaped like a red heart (prop).
- **Card:**
  > Harbans Singh, taxi stand, Connaught Place. "I left the cab at Janpath with the meter off. In '48 I was on the roof of a bus, you could see the whole road all the way to the Gate. Never mind front for me, I am a tall man. Put me where I don't block a child."
- **Keyring removed:**
  > "A red plastic heart. 'My daughter's,' he says, handing it over like a passport."
- **Answer: MIDDLE.**
  - Departure: "I'll be the tall one."
  - Wrong-pen correction (📣): "Gate four, the fawn shawl. Fawn's MIDDLE, read your board."

**5. Joseph D'Souza**, 63, a Goan bandmaster now living in Delhi. A cream linen suit (cream body) with a **Polaroid SX-70** folding camera on a strap (a grey-silver box prop).
- **Card:**
  > Joseph D'Souza, bandmaster (retired), Brass and Reed, Daryaganj. "In '48 the band stood over there and played Abide With Me. His favourite hymn, you know, a Christian hymn. I had the cornet. Nobody asked us to play today; they have a recording." He lifts the camera. "Can I take just one? No? After. You'll keep it for me?"
- **Camera removed:**
  > "Into the basket, carefully, lens up. 'After,' he says, pointing at you."
- **Answer: MIDDLE.**
  - Departure: "Abide with me," hummed, flat, under his breath.
  - Wrong-pen correction for FRONT (📣): "Cream isn't white, gate four. Cream's MIDDLE."

**6. Meena and Rita Malhotra**, 11 and 9, sisters in maroon school sweaters over salwar kameez. Two small figures standing together, sorted as one unit. Rita has a **red plastic hair clip** (prop).
- **Card:**
  > Meena and Rita Malhotra, Class VI and Class IV, Mater Dei School. Meena does the talking. "Our dadi said it's history and we're allowed to miss the test. Rita, say namaste." Rita looks at your Dalda tin like it might contain sweets.
- **Clip removed:**
  > "Rita gives up the clip with enormous dignity."
- **Answer: BACK.**
  - Departure: "Can we see from the back?" Then, from Rita: "We can climb on Meena."
  - Wrong-pen correction for FRONT or MIDDLE (📣): "Maroon jumpers, gate four! Colour goes BACK."

**7. Constable Gurnam Singh Rana**, Delhi Police, off duty, still in **khaki** with a beret (khaki body `#a99a6b`). An **HMT Janata watch** on his wrist (prop).
- **Card:**
  > Constable G. S. Rana, Parliament Street Thana, off duty since six. "I came straight from the night shift. My father was a constable at this funeral, on that rope, holding back people like me. He said they pushed so hard his feet left the ground." He taps the watch. "Government issue. Well, government price."
- **Watch removed:**
  > "Token C4-02. 'You take care of that, it's a good one.'"
- **Answer: BACK.**
  - Departure: "Back is fine. Like my father, on a rope."
  - Wrong-pen correction (📣): "Gate four, that's 1981 police khaki! BACK, please!"

When all seven are sorted, step 2 is ticked off and 📣 says:

> 📣 "Gates are closed. It is seven-fifty. Thank you, Delhi, the whole road is full."

### Step 3: a radio in the sector

Twenty seconds later (or immediately after step 2):

> 📣 "Sector C! The sound man can hear CRICKET on his boom! There is a radio in your sector. Find it, please, before he cries."

The radio commentary sound starts from **Pappu**, about 10 years old. He's a small figure wrapped in a brown blanket, in the BACK pen at (+6, 50), a few metres from the aisle, sitting down (half height). He's not instanced and is a kit `thing`; his label just reads "a boy in a blanket". The player follows the sound down the aisle and among the figures. Crowd instances near (+6, 50) are left out in a 1.2 m radius so he can be reached from the aisle.

**E on Pappu:**
> `say`: "Inside the blanket: a Philips transistor the size of a brick, the commentary from Australia turned down to a whisper."
>
> Pappu: "Sir, Kapil is bowling. Just this over?"

A second E within 5 s, or the first E if the player waits:
> "He hands it over. You switch it off. Somewhere, Kapil Dev bowls in silence."

What happens:
- The radio sound stops.
- Inventory gets **"Pappu's transistor"**.
- 📣 "Thank you, C! The sound man has stopped crying."
- Step 3 is ticked off.

The transistor goes into the cane basket automatically the next time the player is within 3 m of the table, or they can E the basket.

**Hint after 60 s:**
> 📣 "Listen for it, gate four. Somewhere in the back, near the aisle."

### Step 4: petals to the front row

> 📣 "Petals! Front row only. One handful each. Nobody throws until the carriage is level with them."

The **petal basket** appears beside the trunk.
- **E on the basket:** inventory gets **"marigold petals ×4"**.
  > "Four handfuls of marigold petals, cold and damp, smelling green."
- Walk down the aisle to the front row and **E on Raghunath, Vinod and Kamla**. Each gets a handful (a small orange cluster appears at their hand) and the count goes down by one. Their lines:
  - Raghunath: "Last time I had none. I threw my cap."
  - Vinod: "She said: throw it high, so it lands on him."
  - Kamla: "My father threw a whole garland and it hit a sailor."
- **One handful is left over: yours.**

**Optional, while Kamla is in the front row and you carry the thermal print:** an extra E on her (any time from Step 4 until ACTION) gives:

> You show her the print: a round boiler two and a half miles down, three shut furnace doors.
>
> Kamla: "What is it?"
>
> "A ship. We found it last night."
>
> "Who was looking?"
>
> "A lot of people. For a long time."
>
> She gives it back with both hands. "So it was found. Then it's a grave too. Be careful with it."

This works once and sets the flag `showedPrint`.

When three handfuls have been given, step 4 is ticked off.

### Step 5: take your mark

> 📣 "All ADs to your marks at the rope. Eight-twenty. We have the light."

The chalk **"AD C4"** mark at (+10.8, 17.0) brightens (an emissive pulse). Standing within 0.8 m of it, or pressing E on it, ticks step 5.

The fog has lifted by now (`light` at 0.85). From the rope you see the full avenue: the red road, the crowd banks running away both ways, eleven towers, India Gate clear in the east, the palace dome in the west.

### Step 6: ACTION

Two seconds after you take the mark, the loudhailer calls run down the line. Each is a `say` with a 📣, about 2.5 s apart:
1. "Quiet, please."
2. "QUIET, please, Delhi."
3. "Roll cameras."
4. "Speed."
5. "Background… action."

**The silence wave:** starting at x = −450 and moving east at 60 m/s, instanced figures stop bobbing as the wave passes them, and the murmur gain falls with it (fully quiet by 6 s).

**The procession** comes in from the west (spawn at x = −120 on the road, z = −4, moving +x at 2.4 m/s so it passes the player in about 50 s):
- Four lines of **ratings** in white with navy collars, 16 men per line (stylised figures), in two columns each side of the carriage, z = −8, −6.5 and −1.5, 0. They lean forward, hauling **drag ropes**: thick off-white cylinders r 0.035 from the men to the carriage. Their walk is a lean-forward bob.
- The **weapons carrier**, olive `#4b5320`: a low box body 4.5 × 1.2 × 2.2 on four dark wheels r 0.5.
- **The bier** on top: draped white, heaped with flowers (orange and white instanced quads), and a small white head shape. Keep it modest and dignified.
- Behind it, a few dozen walking figures in white.

**The front row throws** when the carriage reaches x within ±3 of each person. Petal particles: about 60 orange and yellow quads arc up 3–4 m and drift down onto the road over 3 s. The petals stay on the road (they're added to a static instanced "petals on the road" mesh that grows).

When the carriage is level with the player (x = +10.8 ± 4), the player can **E** to throw their own handful:
> "You throw. Most of it falls short, on the road, which is where most of everyone's fell."

That's optional. If the player doesn't throw, it stays in the inventory as "a handful of marigold petals".

**No text appears during the passing**, apart from one line as the carriage reaches the player:
> Beside you, Kamla says nothing at all. Her hand is on her father's watch.

The procession continues east and fades into the haze by x = +400 (remove it at +420). Then, 8 s of near-silence, then a crow. Then:
> 📣 "…Cut."

A beat, then:
> 📣 "Check the gate."

A longer beat (4 s), then:
> 📣 "Gate is clean. That's the one. Thank you. Thank you, everybody."

The murmur rises back over 4 s, louder than before. The instanced figures resume bobbing. Step 6 is ticked off. (The procession isn't reset. The road is left strewn with petals.)

### Step 7: give back the time

> 📣 "Sector C: watches and property back, by token, please. Gate ADs to your tables. It is… someone tell me what time it is."

Your seven (well, six people and the sisters) and Pappu walk back up the aisle and queue at the table, in this order: Raghunath, Vinod, Harbans, D'Souza, the sisters, Rana, Pappu. Kamla doesn't queue: she has nothing to collect, and she stays on the kerb at the front.

**E on each person at the table** gives back what was theirs. The meshes move from the tin or basket to them. Then they say a line and walk off north out of the sector, past the queue rope at z 76, and fade out:
- **Raghunath** (token C4-01, Casio): he puts it on. It beeps immediately. "Nine fifty-two. It has never been right before."
- **Vinod** (aviators, the empty Campa bottle): "She has to come to the Odeon now. Twice."
- **Harbans** (the heart keyring): "Meter's still off. Today, I'll drive anybody home free."
- **D'Souza** (Polaroid). He gets it back, unfolds it with a click, and points it at you. **Whirr, click.** A white-bordered square slides out. He waves it once, the way everybody does, and gives it to you.
  > "For you. Front row of C, and you, squinting. It takes a minute in this cold. Don't shake it, that's a superstition. I shake it anyway."

  Inventory gets **"a Polaroid, still developing"**. (See the Polaroid, below.)
- **The Malhotra sisters** (the hair clip): Rita takes it. Meena: "Was it history?" You nod. "Then we can tell Sister Agnes."
- **Rana** (token C4-02, the HMT): "Back on a rope tomorrow. Real one." He salutes, half joking.
- **Pappu** (the transistor, which the player must have put in the basket or still carries): he switches it on at once. Tinny commentary, crowd roar. "Sir! He got him! He got him!" He runs.

**Then your own watch.** When everyone has gone, the tin holds about 200 other people's watches (other gates will collect them) and one with your token. **E on the tin:**
> You find the tag C4-00. Your Casio. You buckle it on. It reads 09:59:56. You watch it.
>
> *Beep-beep.*
>
> 10:00. SA 31 1.

Play the Casio beep. A **digital watch readout** appears at the bottom-right of the HUD: green-grey LCD, showing the real elapsed time from 10:00 onward. The token leaves the inventory. Step 7 is ticked off.

📣 "That's a wrap on Rajpath. Clear the road gently, please."

### The Polaroid (developing)

The Polaroid can be read (E on it isn't possible from the kit's inventory, so do this): after receiving it, **press R** to hold it up. A small 256 × 300 canvas panel appears at the bottom centre: white border, image area. The image fades over 60 s from murky brown-green to a clear flat picture: three white figures at a rope (one with a white shawl and a glint of gold at the wrist), the red road strewn with orange, and on the edge of the frame half of a squinting face (yours) and a brass token on a string. Draw it with simple canvas shapes and a slight overexposure. Press R again to put it away. The `say` the first time:
> "The picture comes up out of brown the way the boiler came up out of the dark, only faster, and full of sun."

### The way onward

When step 7 is complete, the crowd begins to thin. Instanced figures within ±60 m of x = 0 drift north over 30 s, while the far ones stay. The **bamboo rope at the road edge in front of the AD mark is now down**: remove the wall at z = 16 and lower the rope mesh to the grass.

On the road at (+14, 0, −3) lies the **end of a drag rope**, dropped when the procession passed. It's thick jute, a 6 m piece coiled loosely, with a slow warm emissive pulse (`#f29a1d`, intensity 0.15–0.35) and petals around it. The other end runs off east down the road into the distance: a long thin cylinder along x out to +700, which fades into the haze toward India Gate.

> 📣 "Somebody pick up that rope, it's government property."

**E on the rope end:**
> You take up the rope. It's heavier than the cable was, and dry, and warm from sixty hands. It runs east down the empty road through the petals toward the Gate. You follow it.

Then:
```js
branches.carry([
  "a thermal print of the boiler, stamped 01:05",
  "a Polaroid of Sector C's front row, still coming up",
  "your Casio, back on your wrist, beeping the hour"
]);
```
(Add "a handful of marigold petals" if it was never thrown.) Then `branches.onward()` after 1.5 s.

### Pacing, hints, safety nets

- The expected time is about 8 minutes: arrival and step 1 about 1 min; the gate about 3.5 min; the radio about 1 min; petals and mark about 1 min; ACTION about 1.5 min; give-back about 1 min.
- If any step stalls for 75 s, 📣 gives a hint:
  - **Step 1:** "Gate four, your watch. The yellow tin."
  - **Step 2, with a person waiting:** "Gate four, take things off them, then use your boards: one, two, three." If the person waiting is Kamla and she hasn't been draped: "The trunk, gate four. White shawl."
  - **Step 4:** "Basket by your trunk. Then the aisle, down the east side."
  - **Step 5:** "Your chalk mark's at the end of the aisle, at the rope."
  - **Step 7:** "Your own watch is still in the tin, gate four."
- **Hard fallback:** at 12 minutes in the space, if step 7 isn't done, jump forward:
  > 📣 "Leave it, gate four, they'll find their watches."

  Auto-complete the remaining steps (skip the procession if it hasn't happened), give the player their Casio, and show the rope. It must always be possible to leave.
- The person at the table can't wander off. People only move when sorted.
- If the player is far from the table during step 2, the next person waits patiently.

---

## Documents

**The call sheet** (reading panel, title: "CALL SHEET: GANDHI"):
```
INDO-BRITISH FILMS LTD                         CALL SHEET
"GANDHI"                                 Sat. 31 January 1981

SCENE      The funeral procession. Rajpath, Rashtrapati
           Bhavan to India Gate.
CROWD      Approx. 300,000. (Thank you, Delhi.)
CAMERAS    11 crews on towers, as plotted.
REHEARSAL  Not possible.
TAKES      As many as the light allows. Assume one.

GATE ADs — your morning:
  1. Your own watch in the tin. You are in shot.
  2. Open your gate. Make everyone 1948. Sort them:
     FRONT / MIDDLE / BACK. See blackboard.
  3. Keep your sector quiet. NO RADIOS.
  4. Petals: front row only, on the call.
  5. To your marks at the rope.
  6. ACTION. Stand still. Do not look at the camera.
  7. Give everything back. Every token. Every watch.

Nobody is to be sent home. Everyone who came is in the film.
                                                   — R.A.
```

**The blackboard** (reading panel, title: "Blackboard, Checkpoint 4", in chalk):
```
NOTHING AFTER 1948
  watches off (token)  ·  no dark glasses
  no plastic · no nylon · no radios · no cameras

① FRONT  — all WHITE. The lens can count your buttons.
② MIDDLE — PALE. Cream, fawn, grey. White from 50 yds.
③ BACK   — COLOUR. Jumpers, uniforms, saris, anyone.

If in doubt look at the wrist.
If it was here in '48, it stays.
```
(The last line is the hint for Kamla's watch, written in a different, smaller hand.)

---

## Object and thing list (for the build)

| Thing | Label | Use |
|---|---|---|
| call sheet | "the call sheet" | reading panel |
| blackboard | "the blackboard" | reading panel |
| Dalda tin | "the Dalda tin" | step 1; later lines; step 7 |
| sorting boards ①②③ | "① FRONT" etc. | sort the person at the table |
| wardrobe trunk | "the wardrobe trunk" | gives khadi shawl |
| cane basket | "LOST TIME & PROPERTY — 1981" | takes the transistor; flavour line |
| person at the table | their name | card, shawl, petals, print, give-back |
| props | "a Casio watch", "aviator sunglasses", "a Campa Cola", "a wristwatch" (Kamla), "a plastic heart keyring", "a Polaroid camera", "a red hair clip", "an HMT watch" | remove |
| Pappu | "a boy in a blanket" | the radio |
| petal basket | "marigold petals" | step 4 |
| AD C4 mark | "your mark: AD C4" | step 5 |
| rope end | "the end of the drag rope" | onward |

Cane basket E line (any time):
> "Sunglasses, a Campa bottle, a red plastic heart. 1981, waiting in a basket for 1948 to finish."

**State to track:**
- `step` (1–7, then 8 = rope)
- `queueIndex`
- per person: `{ propsLeft, draped, sorted, pen }`
- `hasShawl`, `petalsLeft`, `showedPrint`, `threwPetals`, `radioFound`
- `polaroidT0`, `light`
- `stepStartTime` (for hints)

## First thing to try

Turn round, read the call sheet, and put your own watch in the yellow tin.
