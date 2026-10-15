# SZ-2690-M — Her Statement, in Notes

## The idea
Restitution Modelling Office, Translation Bay 4, a grey morning in 2791. Hesper Vahl died on Thursday night in Room 9 of Lantern House, six days after he signed receipt RD-9. Under Statute §31(4), a claimant's death opens the **counter-file**: the muse's own statement. Mara Szell's is one notebook of stave paper, logged by the 2690 clearance crew as *"music, not statement"* and left untranslated for 101 years. She was a music copyist. She wrote it the morning after the landing, in a private note-cipher keyed to her kettle.

You decode it on the bay's wall screen, one clause at a time. Her notation drops octave marks, so every note could be one of up to four letters, and each clause keeps one word the machine can't settle. You settle it by checking the dollhouse model on your desk, which was returned from Lantern House under its glass hood. It's the flat you built, to 0.2 mm tolerance, from *his* guilt. Now it turns out to be evidence for *her* confession.

**The beauty:** because the cipher maps letters to notes, her confession is literally a melody. The Fourth Symphony came off a mass-produced kettle whistle. The first thing Mara Szell ever composed is the statement you're translating, and when you file it the screen plays it, a tune no one has heard in 101 years. Small extra: her ambiguous words collide like puns in the cipher. TAKE and MAKE are both spelled F-A-D-E, and the machine offers SORRY or LORRY with a straight face.

## Medium
A screen within the world: a first-person room with one big wall screen. Walk to the desk and press **E** on the screen. The camera dollies in until the screen fills the view and an HTML overlay framed as the screen bezel takes over. **Q** or the on-screen `STEP BACK` button returns to the room (Esc is reserved). The screen's MODEL tab is a micromanipulator feed. It switches to a second camera looking down into the dollhouse, shown inside the bezel, and you click hotspots there. Every switch must stay visibly tied to the room: bezel frame, a `FEED: HOOD CAM 2` label, and the amber hood lamp in shot.

## Layout (metres)
- Room 6 × 5, ceiling 3. Chalk-white box walls (#e9e7e2), slate floor (#59606a). Door behind the player (closed, unremarkable).
- Desk 2.0 × 0.8 at 0.75 high, centred against the north wall.
- Wall screen 2.4 × 1.35, centred above the desk, bottom edge at 1.0. Pale phosphor (#cfe3df) on #0f1416. When idle it shows the file header and a blinking `E: OPEN FILE`.
- **Glass hood** on the desk's left: 0.6 × 0.4 × 0.35, holding the dollhouse (kitchen + hall + the landing annex from space 2, door open). A small amber lamp (#f2b45a) hangs inside, the only warm light. Around the base is a black band: `RETURNED — CLAIMANT DECEASED`.
- **Notebook** on the desk's right, in a clear evidence sleeve. Real size, 21 × 15 cm.
- **Carried items** lie on the desk front: receipt RD-9 ("H. V."), a card reading `kettle: A C E D A`, and the credit plate shown sealed inside the annex. If `branches.carried` is empty, place them anyway.
- East wall: a tall narrow window showing a snowed flat roof and a white sky. Cool daylight comes in from there, with low ambient light.

**Arrival:** a full-screen veil in `branches.arrival.color` fades over about 3 s as the room resolves. You face the desk from 3 m away, and the screen's blinking line is the brightest thing. `branches.say("A file is open on the wall screen. The model is back on your desk.")`

## Objects (E in room)
- **Notebook:** reading panel. *"Szell, M. Stave paper, 14 leaves, one used. Dated 12.XI.2690 in her hand. Five lines of notes, each beginning with the copyist's mark ‡. 2690 clearance log: 'Music, not statement. No action.'"*
- **Receipt RD-9:** *"Delivery receipt RD-9. Model SZ-2690 and Grief Annex, received. H. V."* The letters wander.
- **Kettle card:** *"kettle: A C E D A"*. The record from the last room.
- **Hood:** *"Returned under seal. Room 9 asked that the annex door stay open. It has."*
- **Window:** *"Snow on the archive roof. Nobody has walked on it."*

## Screen flow
**1. Brandt's note** (on open):
> Vahl died Thursday night. Room 9 says he asked them to leave the model open.
> Under §31(4) his death opens her counter-file. SZ-2690-M: one notebook, untranslated 101 years. In 2690 they logged it as music. It isn't music. It's her.
> Key is probably the kettle. File by end of shift. — I. Brandt, Supervisor

**2. KEY.** The screen shows the cipher grid scrambled, plus seven note buttons A–G and `ENTER KEY (5 NOTES)`. Each button plays its pitch. Entering A C E D A unscrambles the grid with a tick-sweep:
```
      A  B  C  D  E  F  G
row1  A  B  C  D  E  F  G
row2  H  I  J  K  L  M  N
row3  O  P  Q  R  S  T  U
row4  V  W  X  Y  Z
```
Below it: *"Key accepted. Notation: French cryptogram, octave marks omitted. Each note = up to four letters. Ambiguities will be flagged."* A wrong key gives *"Key rejected."* After two wrong tries: *"Hint: the notebook's first five notes are unlabelled. So is the kettle."*

**3. CLAUSES I–V.** Tabs I–V unlock in order. Each tab shows a stave with her note heads (each word's letters mapped to the column note) and a `DECODE` button. Pressing it plays the clause's notes (0.12 s each) while the letters resolve under them. One word stays flagged as two candidate buttons with confidence bars. The statute line says *"§31(5): Clauses citing physical fact must be verified against the model."* Click `MODEL` to open the feed. The relevant hotspot pulses faintly. Click it to get a tolerance note (below). Back on the clause, the supported candidate reads `SUPPORTED`. Choosing correctly locks the word in, marks the tab ✓, and plays a soft two-note confirm. A wrong choice gives the rejection line and no penalty.

| # | Clause | Flag (notes) | Hotspot → tolerance note | Wrong-choice line |
|---|---|---|---|---|
| I | THAT I HEARD YOU ON THE FOURTH STAIR AND KNEW IT WAS **LATE**. | LATE / SAFE (E A F E) | Kitchen clock: *"Hands 23:41. Battery removed by clearance crew 2690; hands not moved since."* | SAFE: *"Not supported. Clock reads 23:41."* |
| II | THAT I MOVED THE KETTLE TO THE FRONT RING SO YOU WOULD HEAR IT AND KNOW I WAS **NEAR**. | NEAR / GLAD (G E A D) | Stove rings: *"Back ring: wear 0.4 mm (daily use). Front ring, 30 cm nearer the hall wall: wear nil, one scorch, 61 mm: kettle base."* | GLAD: *"Not verifiable. §31 accepts hopes as context, not as evidence."* |
| III | THAT THE TUNE IS NOT MINE. IT IS STAMPED ON THE BOTTOM OF THE KETTLE. I AM NOT **SORRY** YOU TOOK IT. | SORRY / LORRY (E A D D D) | Kettle (lifts on needle, tilts): *"Base stamp: VESTRY & DOLE · MODEL 4 'SONGBIRD' · WHISTLE A–C–E–D–A."* Then a line on screen: *"Model 4. His Fourth."* | LORRY: *"Rejected: grammar. No lorry in model."* |
| IV | THAT I STOOD AT THE STOVE UNTIL IT BOILED DRY, AND DID NOT GO TO THE **DOOR**. | DOOR / ROAD (D A A D) | Floor before stove: *"Wear patch 0.2 mm, one pair of shoes, facing the hall. Kettle interior: scale ring, boiled dry."* Window curtain: *"Undisturbed."* | ROAD: *"Not supported. Curtain undisturbed; she did not go to the window."* |
| V | THAT I WOULD HAVE OPENED IT. THE TUNE IS YOURS NOW. **LOVE / SAVE** IT. — M. | LOVE / SAVE (E A A E), 50/50 | MODEL shows nothing pulsing: *"No evidence in model."* | (both accepted) |

Clause V's screen line: *"§31(6): Where the record is silent, the translator chooses, and signs."* After the player chooses, a `SIGN` button appears. Pressing it writes `Translator, Bay 4` in a ruled box beside a faint `H. V.`, then: *"Your reading: LOVE. Alternative noted in margin: SAVE."* (or the reverse).

**4. FILE.** A `FILE TO PUBLIC RECORD` button appears. Pressing it shows the full five-clause translation on screen and plays the whole statement as one melody, about 30 s. The letter being sung highlights, the hood lamp brightens slightly on each clause, and the model kettle gives a faint wisp. Then:
> SZ-2690-M — TRANSLATED. Public record updated 09:12.
> Statement length: 214 notes.
> Archive note: first item on record composed by Szell, M.

Brandt, 4 s later:
> Read it twice. Went to the window. Go home. — I.B.

Then `CLOSE FILE`. It calls `branches.carry(["translation SZ-2690-M (signed: Translator, Bay 4)", "her statement's melody", "receipt RD-9"])` and `branches.onward()` after a 2 s fade to the room's white daylight.

## Sound
WebAudio only. A low room hum (55 Hz, very quiet). Notes are sine tones with a breathy noise edge and a slight upward pitch scoop, like a kettle whistle, at A4 C5 E5 D5 A4 range (map A–G to A4–G5). Decode ticks are short clicks. Start audio on first click.

## Pacing
About 1 min in the room, 1 min on the key, roughly 1 min per clause, then 1 min of filing and listening, so about 7 min. Onward is earned only after filing.

## First thing to try
The blinking `E: OPEN FILE` on the wall screen. If the player wanders for 25 s without opening it: `branches.say("The file is on the wall screen. The key is on the desk.")`
