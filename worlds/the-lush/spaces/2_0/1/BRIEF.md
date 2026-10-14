You are making space 1 of a house in Branches: a dream someone wanders through. Endless streets of identical liminal houses; inside each, beyond its entry hall, a chain of spaces you make, each continuing the last like a dream continues.

This house has no idea yet: give it one (in space.json, "idea"). An idea is a feeling or theme with a point of view, not a room type: "the difficult emotions of futility, desperation and love in the song Jolene", "vast empty sand where light is the only thing to pay attention to", "The Brothers Karamazov: faith and doubt at one long kitchen table". Draw it from the dreamer's muses and compass, as a variation on the neighbours.

The house is at 2,0 in the world The Lush.


WHAT TO MAKE
A complete, self-contained web page in THIS folder: index.html plus any other files you want (scripts, shaders, generated textures, sounds made with WebAudio). There are no required building blocks and no fixed visual language: a three.js world, a 2D canvas, a text adventure, a shader you drift through, a field of sound, an impossible architecture. Make the medium serve the idea. It should feel like the next moment of a dream: continue the previous space, then transform it.

TIME
The next space takes about 180s to make, and starts being made the moment this one is entered. Design this one to hold a curious person for at least 5 minutes before the way onward can be taken: a maze solvable in about that time, a slow reveal, a puzzle, a long walk toward something, a thing that unfolds as they pay attention. The way onward should be found, not given, and should not appear before then.

THE CONTRACT (the only rules)
- It runs full screen in a sandboxed iframe (scripts and pointer lock allowed; no access to the app, cookies or storage beyond memory).
- To go onward, call branches.onward() (or parent.postMessage({branches: "onward"}, "*")). To go back to the house, branches.back().
- branches.say("text") shows a line of text over the space.
- These keys are reserved by the game; don't use them: Esc (back to the house), = and - (warmer, colder), G (feedback), ? (keys), ` (the workshop). Esc must never be trapped.
- Use only files in this folder and this server: three.js is at /vendor/three.module.js (import * as THREE from "/vendor/three.module.js"); /kit/space.js is an optional first-person helper (read it at /Users/msainz/Projects/branches/client/kit/space.js if you want it). No other network.
- Write space.json: {"name": "a name", "summary": "2-3 sentences: what this space is, how it continues the idea, how the way onward is found", "minutes": how long it should hold someone, "idea": {"name": "...", "premise": "2-4 sentences"}}.
- Test your logic by reading it carefully; you can't open a browser. Keep it robust: a blank screen is the worst outcome.
- Previous spaces are in ../<n>/ if you want to read them.
