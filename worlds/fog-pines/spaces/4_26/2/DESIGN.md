# The Landing Annex: design

## The idea
2791. The Restitution Modelling Office's apprentice (the player) delivers the finished model of Mara Szell's flat to **Hesper Vahl, 144**, in Room 9 of Lantern House, a long-care home. Restitution Statute §31 lets a recipient attach one **Grief Annex** to a delivered model: a single extra space, built to tolerance and dictated as a *Statement of Grief in Clauses*. Vahl's eyes are going and his hands shake, so you work the micromanipulator while he dictates. A wall screen in his room shows the model magnified.

The annex is **the landing outside her door**. On 11 November 2690 he stood there for forty minutes. He heard her humming the tune that became his Fourth Symphony, left flowers on the mat and didn't knock. She died that winter. Each clause he speaks gives you one part to build. The last clause is an omission, and by protocol a clause of omission is verified by performing it. So you knock on a door 7 cm tall. The model's kettle whistles A C E D A, the hinged door opens a crack, light falls on the tiny flowers, and for the first time on the record he says her name.

**The beauty:** grief run through a tolerance spec. He can't be vague about a stair tread or a light timer, and the clinical form keeps forcing small exact truths out of him. The knock is the act he couldn't do, finally done at 1:12 by someone else's hand. The door was always hinged: you built it that way in the last room.

## Medium
A first-person 3D room (kit/space.js) with a **screen within the world**: a 2.4 × 1.35 m wall monitor showing a live render-target camera trained on the model. **Visual tie (the dreamer asked for this):** pressing E at the manipulator console flies the player camera into the wall screen over 1.2 s until the screen fills the view. You are then "inside" the magnified feed, and the screen's bezel stays visible as a thin frame. E or Q flies you back out. In the room, the screen always shows what you're doing in the model.

## Layout (metres)
- Room 9: 6 × 5 × 2.8, built from boxes. Pale sage walls (#b9c4b0), warm grey floor, and one window at dusk (an emissive plane, #e9a46a fading to #6d7fa8) with blinds.
- **Wall screen**: north wall, centred, 1.2 m off the floor. Thin graphite bezel. A strip along the bottom shows the transcript (green-grey clinical type on near-black).
- **Rolling table**: centre, 1.4 × 0.7, with the model of Mara's flat on it (0.5 × 0.4 × 0.3, lamp lit, roofless). The **annex baseboard** (0.22 × 0.4, bare birch with a faint dashed outline grid) is clamped to its west side, against the flat's door wall. The **manipulator console** sits at the table's south edge: a brass-and-white unit with a joystick and two knobs, and its status LED glows amber.
- **Vahl**: in a high-backed chair at the east end of the table, facing the screen. A simple figure: blanket, cardigan, white hair, hands on the blanket that tremble slightly (an animated wobble). Face him and press E to talk.
- Bed against the south wall, a bedside clock, and a cup of water.
- **Room door**: west wall. Locked (red pip) until the end.

## Arrival
Open filled with `branches.arrival.color`, fading over 3 s into the dusk room. The screen already glows with the model's lamp-lit kitchen. Hint line: *"Mr Vahl is waiting by the screen. Speak to him (E)."*

## Operator view (the build mechanic)
- The camera shows the annex from a 3/4 angle at about 12× magnification. The manipulator tip is a fine steel needle with a crosshair, moved by mouse (pointer lock) or WASD. A **parts tray** strip runs along the screen bottom.
- Click a tray part to pick it up. A ghost follows the tip. R rotates it in 90° steps. A target outline (dashed cyan) appears on the baseboard for the current clause only.
- Readout: `DEVIATION 3.41 mm`. Within 3 model-mm and at the right rotation it snaps, and the readout shows `0.18 mm WITHIN TOLERANCE`. Click to place. The part settles with a soft tick, the clause gets a green ✓, and the transcript advances.
- Wrong rotation shows `ORIENTATION: NON-CONFORMANT`.

## The arc: five clauses
Talking to Vahl (E) starts the next clause. He speaks it, it appears in the transcript, and the part arrives in the tray. You then go to the console to build it.

**Opening (first E on Vahl):**
> VAHL: "You brought it. Put it where I can see it on the wall. I can't see things that small any more. I can barely see things that size."
> VAHL: "There's a form. There's always a form. Section 31. I'm allowed one more room. Not hers. Mine. The landing."
> SCREEN: `RD-31 STATEMENT OF GRIEF IN CLAUSES. Recipient: H. VAHL. Operator: Apprentice. Each clause must be verifiable in the annex. Begin.`

**Clause 1: the stairs.** Part: STAIR FLIGHT, 3 TREADS + LANDING.
> VAHL: "Clause one. I was on the landing outside her door on the evening of the eleventh of November, 2690. Third floor. The top stair creaked. Put the creak in."
> SCREEN: `Clause 1. Tolerance 0.2 mm. Creak: noted as acoustic property of tread 3.`
> On place: tread 3 creaks when the tip touches it, from now on. VAHL: "That's it. That's the one. I stood on it so it wouldn't creak again."

**Clause 2: the light.** Part: TIMER SWITCH (stairwell, 40-second).
> VAHL: "Clause two. The stairwell light was on a timer. It went out twice. Both times I pressed it on again instead of knocking."
> On place: the landing gets a tiny warm bulb. Clicking the switch with the tip lights it for 8 s (scaled) with a ticking sound, then it goes out. The protocol wants it pressed twice: `Verify: actuate ×2.` After the second press: ✓.
> VAHL: "Eighty seconds of light I bought rather than one knock. Write that down. — Don't write that down."
> SCREEN: `Written.`

**Clause 3: the pencil.** Part: PENCIL STUB, 4 mm.
> VAHL: "Clause three. Through the door I could hear her humming. Five notes, over and over, while the kettle came up. I wrote them on my cuff. Then I left the pencil on the newel post. I didn't need it. I wasn't going to forget."
> Target: the top of the newel post. On place: VAHL: "I never did forget. It's the only thing I haven't."

**Clause 4: the flowers.** Part: DOORMAT + PAPER CONE (CHRYSANTHEMUMS, VENDING).
> VAHL: "Clause four. Flowers. From the machine at Kessel Street station; it only did chrysanthemums. I left them on the mat, no card. She'd have known. I hope she knew. I never asked."
> SCREEN: `Clause 4 contains a hope. Hopes are not verifiable. Accepted as context.`
> On place: VAHL: "Yellow. They were yellow. Good."

**Clause 5: the omission.** No part. Tray shows: `NO PART. SEE PROTOCOL.`
> VAHL: "Clause five. I did not knock."
> SCREEN: `Clause 5 is a clause of omission. §31(4): an omission is verified by performing it in the annex. Operator may act on the recipient's behalf.`
> VAHL: "...Would you. Not loud. She hated loud. Three, the way you knock for someone you know."
> The flat's door shows a faint target. Each click of the tip on it is one knock: a soft wooden tock, and the screen camera shakes very slightly. **After three knocks**, the stairwell light comes on by itself and the kettle inside the flat whistles A–C–E–D–A (sine with a breathy whistle, rising pitch per note, 0.45 s each). The door, hinged as built, swings open 20°. Warm kitchen light spills across the mat and lands on the yellow flowers.
> Pause 2 s. VAHL (quietly): "Mara."
> SCREEN: `Recipient named the deceased: MARA SZELL. First entry on public record. Clause 5 verified.`

**Mounting the plate.** The tray now shows CREDIT PLATE. Its text comes from `branches.carried` (any item that includes "plate"); if there isn't one, use *"For Mara Szell, who hummed it first."* Target: the landing wall beside the door. On place:
> VAHL: "Leave the door like that. Open. Seal it open."
> SCREEN: `ANNEX SEALED. Delivery RD-9 accepted. Signature: ` followed by a drawn, shaky "H. V."
> Fly out of the screen automatically. Vahl's hands have stopped trembling. The console LED turns green. The room door's pip turns green, and the hint reads *"The delivery is signed. The door is open (E)."*

## Way onward
The room door (E): `branches.carry(["the credit plate (sealed in the annex)", "the kettle's five notes (A C E D A)", "delivery receipt RD-9, signed H. V."])`, then `branches.onward()`. It takes about 7–9 minutes.

## Other interactions
- Vahl between clauses: "Go on. I'm not going anywhere. Statistically." / "The form doesn't mind if you're slow. I mind a little."
- Water cup (E): "Lantern House water. Room temperature, by regulation."
- Clock (E): "18:41. Visiting hours end at 19:00. The form doesn't care."
- Window (E): "Dusk over the tram depot. The trams are still green."

## Light & sound
Room: warm ambient light plus a cool fill from the window. The screen is the brightest thing in the room. Inside the model the tones are tungsten amber. A quiet HVAC hum, a clock tick, the tray part tick, the stair creak, the timer tick and the knocks are all short WebAudio synths. The kettle is the only melodic sound.

## First thing to try
Speak to Vahl (E), then go to the console (E).
