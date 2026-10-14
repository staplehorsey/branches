You are making space 1 of a house in Branches, a game someone plays to wander through ideas: endless streets of identical houses; inside each, beyond its entry hall, a chain of spaces, each one a self-contained web page that you write.

WHERE THIS SPACE IS
The game keeps an atlas of every idea explored so far and picked these coordinates for you (a new house: as far as possible from everything explored, leaning toward what was warm):
  era: early modern
  feeling: rage
  medium: text and choices
  register: austere
  scale: a street
  source: a religious practice
  structure: journey with stations
  verb: assemble a machine
Synthesise ONE concrete, specific idea that sits exactly at these coordinates. Not a vibe: a situation with people, objects and stakes. These coordinates are the brief; don't drift back toward the familiar.

Nearest ideas already explored (be clearly unlike them):
(none yet: this is the first)

This is the house's first space: its idea is born here (put it in space.json as "idea").

Neighbouring houses' ideas (this street should feel like wandering through a dream: a neighbour is a close variation, further away the ideas drift):
- 3 houses away at 2,0: Kept at Golden Hour: A sealed glass garden where the sun has been stopped just above the horizon so that nothing inside will ever have to end. Whoever put the glass over it did it out of tenderness. The plants have grown toward that one light for so long they have forgotten there are others; the dreamer is the first thing in here that still remembers dusk. Every space asks the same thing: is keeping a kindness, and what does it cost the kept.

THE SPIRIT
This is an invitation, not a ticket. Make something you'd be glad to have made, and enjoy making it. Go looking for beauty wherever these coordinates lead, and appreciate it in whatever way you can: there is beauty in everything if you pay close enough attention, even in a fax machine (the warm curl of thermal paper, the screeching handshake, a machine's patience waiting for a dial tone, a cover sheet someone decorated). Find the one thing here that is genuinely lovely and build toward it, so the player gets to discover it too. Take pleasure in a mechanism that clicks just right, a colour that lands, a line of dialogue that's true, a joke that's kind. Beauty here is attention: specific, earned, often funny, never vague mist.

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
