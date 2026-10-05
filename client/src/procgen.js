// An exact port of server/src/procgen.rs. Any client must generate the same
// untouched room as the reference host for the same address, so doors pair
// up whether a room was stored by a host or generated in the browser.
// 64-bit arithmetic uses BigInt; see the Rust source for the reference.

const M64 = (1n << 64n) - 1n;

export function splitmix(x) {
  x = (x + 0x9e3779b97f4a7c15n) & M64;
  let z = x;
  z = ((z ^ (z >> 30n)) * 0xbf58476d1ce4e5b9n) & M64;
  z = ((z ^ (z >> 27n)) * 0x94d049bb133111ebn) & M64;
  return z ^ (z >> 31n);
}

const asU64 = (n) => BigInt.asUintN(64, BigInt(n));

export function hash(seed, x, z, salt) {
  return splitmix(asU64(seed) ^ splitmix(asU64(x) ^ splitmix(asU64(z) ^ splitmix(asU64(salt)))));
}

export class Rng {
  constructor(seed) {
    this.s = splitmix(asU64(seed));
  }
  next() {
    this.s = splitmix(this.s);
    return this.s;
  }
  f32() {
    return Number(this.next() >> 40n) / 2 ** 24;
  }
  pick(items) {
    return items[Number(this.next() % BigInt(items.length))];
  }
}

export function strHash(s) {
  let h = 0xcbf29ce484222325n;
  for (const b of new TextEncoder().encode(s)) h = ((h ^ BigInt(b)) * 0x100000001b3n) & M64;
  return h;
}

export const TAGS = ['water', 'plants', 'light', 'books', 'art', 'cozy', 'sky', 'creatures', 'music', 'stone', 'snow', 'neon'];

export const THEMES = [
  { id: 'poolrooms', name: 'Poolrooms', tags: ['water', 'light', 'stone'] },
  { id: 'moss-library', name: 'Moss Library', tags: ['books', 'plants', 'cozy'] },
  { id: 'cloud-nursery', name: 'Cloud Nursery', tags: ['sky', 'cozy', 'light'] },
  { id: 'sunset-terrarium', name: 'Sunset Terrarium', tags: ['plants', 'light', 'stone'] },
  { id: 'night-aquarium', name: 'Night Aquarium', tags: ['water', 'creatures', 'neon'] },
  { id: 'vapor-mall', name: 'Vapor Mall', tags: ['neon', 'plants', 'art'] },
  { id: 'tea-garden', name: 'Tea Garden', tags: ['water', 'plants', 'stone'] },
  { id: 'fern-cathedral', name: 'Fern Cathedral', tags: ['plants', 'stone', 'light'] },
  { id: 'arcade-after-hours', name: 'Arcade After Hours', tags: ['neon', 'music', 'light'] },
  { id: 'snowglobe-den', name: 'Snowglobe Den', tags: ['snow', 'cozy', 'light'] },
  { id: 'citrus-kitchen', name: 'Citrus Kitchen', tags: ['plants', 'light', 'cozy'] },
  { id: 'velvet-theatre', name: 'Velvet Theatre', tags: ['music', 'art', 'cozy'] },
];

const parseAddr = (s) => {
  const m = /^\s*(-?\d+)\s*,\s*(-?\d+)\s*$/.exec(s || '');
  return m ? [Number(m[1]), Number(m[2])] : null;
};

// Rust compares Strings bytewise; for the ASCII ids we use this is the same
// as JavaScript's code-unit comparison.
const lt = (a, b) => a < b;

export function defaultRoom(world, others, x, z) {
  const seed = Number(hash(world.seed, x, z, 1) & 0xffffffffn);
  const theme = THEMES[Number(hash(world.seed, x, z, 2) % BigInt(THEMES.length))];
  const rng = new Rng(seed);
  const p = world.portal_policy;
  const sealed = p.mode === 'closed' || (p.mode === 'allowlist' && !p.allow_local);
  const mySpawn = parseAddr(world.spawn);
  const portals = [];
  // Hub worlds are pre-wired to each other; grown worlds hang off their doors.
  for (const o of others.filter((o) => world.hub !== false && o.hub !== false)) {
    const theirs = parseAddr(o.spawn);
    let target, slot;
    if (mySpawn && mySpawn[0] === x && mySpawn[1] === z) {
      if (!theirs) continue;
      target = theirs;
      slot = 0;
    } else {
      if (theirs && theirs[0] === x && theirs[1] === z) continue;
      const [a, b] = lt(world.id, o.id) ? [world.id, o.id] : [o.id, world.id];
      const pair = splitmix(strHash(a) ^ splitmix(strHash(b)));
      const h = hash(Number(pair & 0xffffffffn), x, z, 0xb0b);
      if (h % 9n !== 0n) continue;
      target = [x, z];
      slot = Number((h >> 8n) % 3n);
    }
    portals.push({ id: `pair-${o.id}`, slot, target: `/w/${o.id}/${target[0]},${target[1]}`, label: o.name, by: 'generator', at: 0, sealed });
  }
  // BTreeMap order matters only for display; values must match.
  const weights = {};
  for (const t of theme.tags) weights[t] = Math.fround(1 + Math.fround(rng.f32() * 0.2));
  return {
    x,
    z,
    seed,
    theme: theme.id,
    chambers: [{ name: 'Entry Hall', tag: theme.tags[0], seed, by: 'generator', at: 0 }],
    features: [],
    portals,
    log: [],
    claim: null,
    attention: 0,
    weights,
    investors: {},
    building: null,
    last_visit: {},
  };
}
