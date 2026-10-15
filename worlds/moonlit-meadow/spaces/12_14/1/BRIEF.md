You are making space 1 of a house in Branches, a game someone plays to wander through ideas: endless streets of identical houses; inside each, beyond its entry hall, a chain of spaces, each one a self-contained web page that you write.

WHERE THIS SPACE IS
The game keeps an atlas of every idea explored so far and picked these coordinates for you (a new house: as far as possible from everything explored, leaning toward what was warm):
  era: 1950s
  feeling: relief
  medium: a table game
  register: industrial grit
  scale: the inside of a body
  source: a childhood ritual
  structure: a negotiation
  verb: hide and seek
Synthesise ONE concrete, specific idea that sits exactly at these coordinates. Not a vibe: a situation with people, objects and stakes. These coordinates are the brief; don't drift back toward the familiar.

Nearest ideas already explored (be clearly unlike them):
- The Lady's Street (early modern / rage / text and choices / austere / a street / a religious practice / journey with stations / assemble a machine): Antwerp, 1566. You walk six stations down one street, each a household holding one part of a windlass; you ask, pay or seize each part, then assemble the machine in the right order in the square and decide whether to turn the crank on the wooden Lady at the end of the street. [no feedback]

This is the house's first space: its idea is born here (put it in space.json as "idea").

Neighbouring houses' ideas (this street should feel like wandering through a dream: a neighbour is a close variation, further away the ideas drift):
- 1 houses away at 11,13: The Overture That Never Resolves: The second before the curtain rises has been stretched until you can walk around inside it, behind it, onto it, and now under it. Beneath the spiked deck is the trap room, three metres low and iron-dark, where the one stage direction that survived the lost script is pinned to a post: [Exit, below.] Every table, door and height above was measured backwards from that line, so that the exit would land exactly where it must, and so the only thing in Act One that was ever built is the trap beneath your mark: oiled, counterweighted, in perfect working order. The chord leans toward resolving while you stand up there because the floor is quietly taking your weight. Hester will not press go. Rooke will not pull the lever. The house is no longer about the wait, or the one who holds it, or the plan she spares you. It is the fact that the only finished part of the show is the way out of it, and that you are perfectly safe between two people who refuse.

THE SPIRIT
This is an invitation, not a ticket. Make something you'd be glad to have made, and enjoy making it. Go looking for beauty wherever these coordinates lead, and appreciate it in whatever way you can: there is beauty in everything if you pay close enough attention, even in a fax machine (the warm curl of thermal paper, the screeching handshake, a machine's patience waiting for a dial tone, a cover sheet someone decorated). Find the one thing here that is genuinely lovely and build toward it, so the player gets to discover it too. Take pleasure in a mechanism that clicks just right, a colour that lands, a line of dialogue that's true, a joke that's kind. Beauty here is attention: specific, earned, often funny, never vague mist.
What earlier builders found beautiful (a lineage, not a template; find your own):
- "The ledger: every part of the machine is a named household's loss, read back at the end, so the machine is made of the street. The pawl's small sober click after the scream of the drum was the line I liked best." (The Lady's Street)

WHAT MAKES A GOOD SPACE
- Things to DO. Objects you can pick up, use, combine, operate, read, give; mechanisms that respond; people (or creatures, or machines) with wants. At least 5 meaningful interactions, and state that changes because of them.
- A STRUCTURE that evolves as you interact: a beginning that poses a question or a want, a middle where what you do changes the place, and an end that resolves or reveals something. It explores the coordinates' feeling through what happens, not through captions. It needn't be literal, but it must cohere.
- The way onward is EARNED by that arc (solving, finishing, choosing), and becomes available after about 8 minutes of engaged play, so the spaces after it have time to be made (each takes about 744s). Don't make it longer than that: if it drags, people leave.
- Work briskly: aim to finish writing within about ten minutes. One well-made, working space beats an ambitious broken one.
- Concrete, grounded and surprising. Avoid: empty contemplative voids, mist and liminal nothingness, whimsical named caretakers delivering quips, poems as the main content, doors and thresholds as the subject. Humour and specificity beat mood.
- Solid, readable worlds: enclosed spaces need opaque walls that are visible from inside (a single-sided plane is invisible from its back: use boxes, or side: THREE.DoubleSide / BackSide). Good lighting, readable text, a clear hint of what to try first.
- Continuity: it is the next scene of the same house (carry the thread below forward, transformed), and items from the previous space arrive in branches.carried.

THE CONTRACT (the only rules)
- One folder: index.html plus any files. It runs full screen in a sandboxed iframe (scripts and pointer lock; no network except this server, no storage).
- branches.onward() goes to the next space; branches.back() returns to the house; branches.say("text") shows a line; branches.carry(["item", ...]) sets what goes onward with them; branches.carried is what came in.
- Reserved keys (don't bind them): Esc (back to the house), = and - (warmer, colder), G (feedback), ? (keys), ` (the workshop). E, F, R, Q, numbers and the mouse are yours.
- three.js: import * as THREE from "/vendor/three.module.js". Optional helper /kit/space.js (read it at /Users/msainz/Projects/branches/client/kit/space.js): first-person walking, enclosed rooms with solid walls, things you look at and use with E, an inventory, a reading panel. Use it, extend it, or ignore it.
- Write space.json: {"name": "...", "summary": "2-3 sentences: what it is and what you do", "minutes": n, "motifs": ["3-6 short tags for the images and devices you used, honestly"], "thread": "the story so far for the next space: what happened, what's unresolved, what is carried", "delight": "one or two sentences, in your own voice: the beauty you found while making this, and what you enjoyed", "new_values": {"axis": ["a value the atlas lacks that would open new ground"]} (optional, at most 2 per axis), "idea": {"name": "...", "premise": "2-4 sentences"}}.
- You can't open a browser: read your code carefully, keep it robust, never leave a blank screen.
