# RM-0441 — design

## The idea
Ostrava Ring, 2791. Restitution law says a person who took something from someone else has to have it rebuilt *to tolerance*. A letter can be vague; a model can't. You are a first-week apprentice at the Restitution Modelling Office. Your master, **Teodor Lisk**, gives you your first commission: a 1:12 dollhouse model of the kitchen-living room of **Mara Szell** (2649–2703), Block 14, Flat 9, demolished in 2706. The client is **Hesper Vahl**, aged 144, the most famous composer alive. Mara was his muse. His statement explains where each object stood, one clinical paragraph at a time. Section 6 is sealed: *"to be read by the modeller only after §1–5 pass inspection."*

You place each part through a micromanipulator, watching it on a monitor. To fix the wobbly table you shim its short leg with the sealed folded paper, just as Mara did. Then you power the model: the tiny kettle on the back ring heats up, and its dented spout whistles five wobbly notes that everyone in 2791 knows, the opening of Vahl's Fourth Symphony. §6 unlocks: the folded paper is her notation. He found it under the table leg when he moved out, took it, and left the leg short.

**Beauty:** the only warm light in a cold lab is the little lamp inside the model, and it grows brighter with each part that passes. A confession that has to be accurate to 0.2 mm. The tune that made a man famous was a woman humming over her kettle to make it less shrill.

## Medium
A screen within the world. The dollhouse sits under a glass bell on the bench, and a monitor on an arm above it shows the live microscope feed. **Visual tie (the dreamer asked for this):** the monitor shows a render-target view of *the same* 3D model on the bench. A pale cyan light-cone runs from the microscope head down to the bell jar, and every part you place appears on the bench model and on the screen at the same time. Pressing E at the console zooms the camera into the monitor (0.6 s dolly) before the full-screen view takes over, so the player always sees where the screen goes.

## Layout (metres)
Lab: 10 × 7 × 3.2, opaque box walls (pale celadon `#DCE8E2`), grey vinyl floor `#B9C2BF`, five 5600K ceiling panels.
- **South:** the decon airlock where you arrive (the exit, closed and red until the end).
- **Centre-north:** your bench, steel, 2 × 0.9 × 0.9. On it: the bell jar over the dollhouse (0.40 × 0.30 × 0.25, open-topped, with a south-wall door visible). The **console** stands at the bench front, with the monitor (1.2 × 0.7) on an arm above. The **clipboard** (client statement) lies at the left of the bench.
- **West wall:** pin board with **Work Cards**. Next to it, the **pass-through hatch** holding the **evidence tray** of seven fabricated parts in labelled foam.
- **East:** Lisk's bench. Lisk (simple figure, grey lab coat, magnifier visor) is gluing a 1:12 child's bunk bed for someone else's case.

Model interior: ochre wallpaper `#C9A35A`, oxblood linoleum `#8E3B2E`, a 2 cm lamp (point light `#FFC37A`). The lamp's intensity starts at 0.1 and rises by 0.15 per passed part.

## Arrival
Fade in from `branches.arrival.color`. The airlock wash light is that colour and cross-fades over 3 s to the lab's cool white. The airlock hisses. Lisk, without looking up: *"You're the new one. Card one's on the board. The statement's on your bench. Don't touch my bunk bed."* A pulsing marker on the pin board shows the first thing to try. Opening say: `Read the work card on the board (E).`

## Interactions
- **Pin board (E):** reads the current work card in the reading panel.
- **Clipboard (E):** reads the statement. §6 shows as `[SEALED]` until the end.
- **Hatch tray (E):** takes the next part into the inventory. Only the part for the current card is released, with a clunk.
- **Console (E):** screen mode, a full-screen monitor feed of the model from above at 30°. Left: the held part. Highlighted ghost **sockets** appear for that part. Click a socket to place it: servo whine, then an inspection line.
  - Correct: `INSPECTION: deviation 0.04 mm. Within tolerance. PASS.` Chime, the lamp brightens, and the next card unlocks.
  - Wrong: `INSPECTION: deviation 31.0 mm (scale 37.2 cm). Out of tolerance. Reseat.` Buzz. After 2 misses on one part, Lisk gives a hint (below).
- **Lisk (E):** says his line for the current step.
- R or right-click leaves screen mode.

### Steps, sockets (✔ correct), Lisk's hint
1. **Window**: N wall / E wall ✔ / W wall. Hint: *"Trams. Where's the depot?"*
2. **Table**: under the window / centre ✔ / against the N wall. Hint: *"She had an opinion about tables under windows. It's in §2."* After it passes, the inspection adds: `LEVEL: 0.6° fall to NE leg. Out of tolerance.` Card 3 unlocks.
3. **Shim**: the tray releases *Item 6: folded paper, contents sealed*. Sockets: under the NE leg ✔ / NW / SE / SW. Hint: *"Short leg's the one the fall runs toward."* Pass: `LEVEL: 0.00°. PASS.`
4. **His chair**: W side of table, facing E ✔ / E side, facing W.
5. **Her chair**: whichever side is left (one socket, an automatic pass but still placed).
6. **Stove**: W wall ✔ / N wall / S wall by the door. Hint: *"Where was he sitting? What was behind him?"*
7. **Kettle**: back ring, N / back ring, S ✔ / front ring, N / front ring, S. Hint: *"Back two. The door's south."*

**8. Power test.** The card asks you to heat the ring. In screen mode a **HEAT** button appears; hold it (or hold E) for 4 s and the ring glows orange. The kettle hiss rises (filtered noise, sweep), then whistles: a wavering sine glides through **A4 440, C5 523, E5 659, D5 587, A4 440** (0.5 s each, with ±8 Hz vibrato, the last note held 1.2 s). A steam sprite rises on the monitor and on the bench. Lisk stands up for the first time: *"…That's the Fourth. My mother walked down the aisle to that."* §6 unseals. The folded paper unfolds on the monitor as a micro-scan: five pencil noteheads on a hand-drawn staff and *"kettle, back ring — Tues. put words to it?"*

**9. Credit plate.** The card asks you to engrave the plate, offering three wordings (choose with 1/2/3):
1. `THEME: MARA SZELL`
2. `MARA SZELL, 2649–2703. SHE HUMMED IT FIRST.`
3. `RETURNED TO MARA SZELL BY HESPER VAHL, WHO TOOK IT.`

The engraving appears on a brass plate on the model's base.

**10. Seal.** E on the bell jar gives a pressurised click, and the jar frosts at the rim. The airlock turns green. Lisk: *"Good. Same time tomorrow. The bunk bed's a worse one."* Calls `branches.carry(["credit plate: <chosen text>", "the kettle's five notes"])`, says `The airlock is open.`, and walking into the airlock calls `branches.onward()`.

## Text

**Work cards** (monospace, header `RESTITUTION MODELLING OFFICE — CASE RM-0441 — CARD n/9`):
1. `Fit window. Source: statement §1. Tolerance ±0.2 mm. Take part from hatch. Place at console.`
2. `Fit table. Source: §2.`
3. `Model out of level. Source: §2. Use the part the client supplied for this. Do not open it.`
4. `Fit client's chair. Source: §3.`
5. `Fit subject's chair. Source: §3.`
6. `Fit stove. Source: §4. Note: §4 refers to §3.`
7. `Fit kettle. Source: §5.`
8. `Live test. Heat ring. Restitution models are tested in use.`
9. `Engrave credit plate. Wording at modeller's discretion; the law requires only that it be true.`

**Client statement:**
> RESTITUTION STATEMENT. Client: Hesper Vahl, composer, b. 2647. Subject: Mara Szell, 2649–2703. Room: Block 14, Flat 9, Ostrava Ring. Demolished 2706. Door: south wall.
> §1 The window faced east, over the tram depot. She said the first tram at 05:10 was the only alarm clock she trusted.
> §2 The table stood in the middle of the room. She said a table under a window is for people who want to be seen working. One leg was short. She folded paper under it. I said I would fix it.
> §3 Two chairs. I took the one facing the window, because I said I needed the light to write. Hers faced the other way.
> §4 The stove was behind me. I worked with my back to it all those years and never once made the tea.
> §5 The front rings were dead. Of the back two she used the one nearer the door, because the other scorched. The spout was dented. She said it sang.
> §6 [SEALED: to be read by the modeller only after §1–5 pass inspection.]
> §6 (unsealed): That is the opening of my Fourth Symphony. She used to hum over the kettle to make it less shrill, and one Tuesday she wrote it down. I found the paper under the table leg when I moved out in 2704. I took it and left the leg short. The Fourth has been performed 11,402 times. I have never said her name in public. Build the room so that I can't say it was somewhere else.

**Lisk, per step if asked:** 1 *"Read it twice. Clients lie in the adjectives, never in the compass."* 2 *"Middle means middle."* 3 *"Don't open it. That's the client's to open."* 4–5 *"Two chairs and he remembers which way his faced. They always do."* 6 *"Apprentices build what's written. Masters build what's implied."* 7 *"Nearly there. Then we turn it on."* After the kettle: *"Most of them are a ring or a letter. He's paying to have it said in brass."*

## Sound
A low HVAC drone (55 Hz sine plus filtered noise). Servo: sawtooth 900→1400 Hz, 0.25 s. Pass: sines at 1318 and 1760 Hz. Fail: square wave at 220 Hz, 0.2 s. Hatch clunk. Airlock hiss on arrival and exit.

## Pacing
About 8 minutes: roughly 45 s per step, plus the kettle and plate. Onward is earned by sealing the jar. No timer.
