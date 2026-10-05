// A Branches host that runs inside the page. Used when the client is served
// without a server (a static page, a file, an embed): the same worlds, doors,
// visitor logs and architect, for one player, saved in localStorage.
// It mirrors the Rust reference host closely; see server/src for the source
// of truth.
import { WORLDS, THEMES, TAGS } from './offline-data.js';
import { hash32, Rand } from './rng.js';

export const LOCAL_ORIGIN = 'local://branches';
const KEY = 'branches.local-host.v1';
const PACE = 25; // seconds of attention for a first growth

const now = () => Date.now();
const newId = () => Math.random().toString(16).slice(2, 14);
const addr = (x, z) => `${x},${z}`;
const parseAddr = (s) => {
  const m = /^(-?\d+),(-?\d+)$/.exec(s || '');
  return m ? [Number(m[1]), Number(m[2])] : null;
};
const strHash = (s) => {
  let h = 0x811c9dc5;
  for (let i = 0; i < s.length; i++) h = Math.imul(h ^ s.charCodeAt(i), 0x01000193);
  return h >>> 0;
};

const KEYWORDS = {
  water: ['water', 'pool', 'fountain', 'ocean', 'swim', 'rain', 'river', 'lake', 'bath', 'wet', 'sea'],
  plants: ['plant', 'garden', 'green', 'moss', 'fern', 'tree', 'flower', 'jungle', 'lush', 'vine', 'leaf', 'leaves', 'bloom'],
  light: ['light', 'bright', 'glow', 'sun', 'lamp', 'candle', 'window', 'golden'],
  books: ['book', 'library', 'read', 'poem', 'story', 'shelf', 'shelves', 'archive'],
  art: ['art', 'painting', 'sculpture', 'statue', 'gallery', 'mural', 'portrait', 'frame'],
  cozy: ['cozy', 'cosy', 'sofa', 'couch', 'bed', 'pillow', 'warm', 'blanket', 'chair', 'sit', 'rest', 'nap'],
  sky: ['sky', 'cloud', 'star', 'moon', 'skylight', 'air', 'float', 'heaven'],
  creatures: ['fish', 'jelly', 'bird', 'butterfl', 'cat', 'creature', 'animal', 'firefl', 'koi', 'moth'],
  music: ['music', 'song', 'piano', 'sound', 'record', 'radio', 'dance', 'sing', 'chime'],
  stone: ['stone', 'column', 'marble', 'arch', 'temple', 'ruin', 'stair', 'pillar', 'statue'],
  snow: ['snow', 'ice', 'winter', 'cold', 'frost', 'crystal'],
  neon: ['neon', 'arcade', 'synth', 'vapor', 'retro', 'pink', 'cyber', 'disco'],
};
const tagsIn = (text) => Object.entries(KEYWORDS).filter(([, ws]) => ws.some((w) => text.toLowerCase().includes(w))).map(([t]) => t);

const ADJ = ['Quiet', 'Drowsy', 'Endless', 'Soft', 'Hollow', 'Golden', 'Sunken', 'Lantern', 'Velvet', 'Lucid', 'Gentle', 'Forgotten', 'Humming', 'Pale', 'Tender', 'Still', 'Second', 'Inner', 'Slow', 'Lower'];
const NOUNS = {
  water: ['Cistern', 'Baths', 'Tide Room', 'Wading Hall', 'Lagoon'],
  plants: ['Conservatory', 'Greenhouse', 'Fernery', 'Arbor', 'Hothouse'],
  light: ['Atrium', 'Sunroom', 'Lantern Hall', 'Solarium'],
  books: ['Reading Room', 'Archive', 'Stacks', 'Study'],
  art: ['Gallery', 'Studio', 'Salon'],
  cozy: ['Parlor', 'Den', 'Nook', 'Lounge'],
  sky: ['Observatory', 'Cloud Loft', 'Skyroom'],
  creatures: ['Menagerie', 'Aviary', 'Aquarium'],
  music: ['Music Room', 'Listening Room', 'Ballroom'],
  stone: ['Colonnade', 'Cloister', 'Rotunda'],
  snow: ['Frost Hall', 'Winter Room', 'Ice House'],
  neon: ['Arcade', 'Afterglow', 'Night Market'],
};

const theme = (id) => THEMES.find((t) => t.id === id);
const world = (id) => WORLDS.find((w) => w.id === id);

function defaultRoom(w, x, z) {
  const seed = hash32(w.seed, x, z, 1);
  const th = THEMES[hash32(w.seed, x, z, 2) % THEMES.length];
  const p = w.portal_policy;
  const sealed = p.mode === 'closed' || (p.mode === 'allowlist' && !p.allow_local);
  const mySpawn = parseAddr(w.spawn);
  const portals = [];
  for (const o of WORLDS) {
    if (o.id === w.id) continue;
    const theirs = parseAddr(o.spawn);
    let target, slot;
    if (mySpawn && mySpawn[0] === x && mySpawn[1] === z) {
      target = theirs;
      slot = 0;
    } else {
      if (theirs && theirs[0] === x && theirs[1] === z) continue;
      const [a, b] = [w.id, o.id].sort();
      const h = hash32(strHash(a), strHash(b), x, z, 0xb0b);
      if (h % 9 !== 0) continue;
      target = [x, z];
      slot = (h >>> 8) % 3;
    }
    portals.push({ id: `pair-${o.id}`, slot, target: `/w/${o.id}/${target[0]},${target[1]}`, label: o.name, by: 'generator', at: 0, sealed });
  }
  return {
    x, z, seed, theme: th.id,
    chambers: [{ name: 'Entry Hall', tag: th.tags[0], seed, by: 'generator', at: 0 }],
    features: [], portals, log: [], claim: null, attention: 0,
    weights: Object.fromEntries(th.tags.map((t) => [t, 1])),
    investors: {}, building: null, last_visit: {},
  };
}

class LocalHost {
  constructor() {
    let saved = null;
    try {
      saved = JSON.parse(localStorage.getItem(KEY));
    } catch {}
    this.data = saved || { rooms: {}, logs: {}, player: null };
    this.lives = new Set();
    this.dirty = false;
    setInterval(() => this.tick(), 1000);
    setInterval(() => this.save(), 4000);
  }
  save() {
    if (!this.dirty) return;
    this.dirty = false;
    try {
      localStorage.setItem(KEY, JSON.stringify(this.data));
    } catch {}
  }
  touch() {
    this.dirty = true;
  }
  emit(msg) {
    for (const l of this.lives) l.handlers[msg.t]?.(msg);
  }
  threshold(growth) {
    return PACE * (1 + 0.6 * growth);
  }
  room(w, x, z) {
    return this.data.rooms[w]?.[addr(x, z)] || defaultRoom(world(w), x, z);
  }
  roomMut(w, x, z) {
    const rooms = (this.data.rooms[w] ||= {});
    this.touch();
    return (rooms[addr(x, z)] ||= defaultRoom(world(w), x, z));
  }
  player(id, secret) {
    const p = this.data.player;
    if (!p || p.id !== id || p.secret !== secret) throw new Error('unknown player or bad secret');
    return p;
  }
  summary(r) {
    return { x: r.x, z: r.z, seed: r.seed, theme: r.theme, growth: r.chambers.length - 1, claimed: r.claim?.title ?? null, portals: r.portals.length, building: !!r.building };
  }
  desire(r) {
    const out = {};
    const add = (t, v) => (out[t] = (out[t] || 0) + v);
    const rt = Object.values(r.weights).reduce((a, b) => a + b, 0) || 1;
    for (const [t, w] of Object.entries(r.weights)) add(t, (0.55 * w) / rt);
    const p = this.data.player;
    if (p && r.investors[p.id]) {
      const pt = Object.values(p.prefs).reduce((a, b) => a + b, 0) || 1;
      for (const [t, w] of Object.entries(p.prefs)) add(t, (0.45 * w) / pt);
    }
    return out;
  }
  view(w, r) {
    const p = this.data.player;
    const leaning = Object.entries(this.desire(r)).sort((a, b) => b[1] - a[1]).slice(0, 3).map(([t]) => t);
    return {
      world: w, x: r.x, z: r.z, seed: r.seed, theme: r.theme, theme_name: theme(r.theme)?.name || r.theme,
      chambers: r.chambers, features: r.features, portals: r.portals, log: r.log.slice(-60).reverse(), claim: r.claim,
      growth: r.chambers.length - 1, attention: Math.round(r.attention), next_growth_at: Math.round(this.threshold(r.chambers.length - 1)),
      building: r.building, leaning,
      investors: Object.entries(r.investors).map(([id, s]) => ({ name: id === p?.id ? p.name : 'someone', seconds: Math.round(s) })),
    };
  }
  chamberTags(r, ch) {
    const tags = new Set([r.chambers[ch]?.tag].filter(Boolean));
    if (ch === 0) for (const t of theme(r.theme)?.tags || []) tags.add(t);
    for (const f of r.features) if (f.chamber === ch) tags.add(f.tag);
    return [...tags];
  }
  bump(map, t, by) {
    map[t] = Math.min(50, (map[t] || 0) + by);
  }

  // ------------------------------------------------------------ HTTP-ish

  async get(path) {
    const u = new URL(path, 'http://x');
    let m;
    if (path === '/.well-known/branches.json') return { protocol: 'branches/0.1', software: 'branches-in-page', worlds: WORLDS.map((w) => ({ id: w.id, name: w.name })) };
    if (u.pathname === '/api/themes') return { tags: TAGS, themes: THEMES };
    if (u.pathname === '/api/worlds')
      return WORLDS.map((w) => ({ id: w.id, name: w.name, tagline: w.tagline, online: this.lives.size ? 1 : 0, rooms_grown: Object.values(this.data.rooms[w.id] || {}).filter((r) => r.chambers.length > 1).length, portal_policy: w.portal_policy }));
    if ((m = u.pathname.match(/^\/api\/worlds\/([^/]+)$/))) {
      const w = world(decodeURIComponent(m[1]));
      if (!w) throw new Error('no such world');
      return { manifest: w };
    }
    if ((m = u.pathname.match(/^\/api\/worlds\/([^/]+)\/chunk$/))) {
      const w = decodeURIComponent(m[1]);
      if (!world(w)) throw new Error('no such world');
      const q = (k) => Number(u.searchParams.get(k));
      const cells = [];
      for (let x = q('x0'); x <= q('x1'); x++) for (let z = q('z0'); z <= q('z1'); z++) cells.push(this.summary(this.room(w, x, z)));
      return { cells };
    }
    if ((m = u.pathname.match(/^\/api\/worlds\/([^/]+)\/rooms\/(-?\d+)\/(-?\d+)$/))) {
      const w = decodeURIComponent(m[1]);
      if (!world(w)) throw new Error('no such world');
      return this.view(w, this.room(w, Number(m[2]), Number(m[3])));
    }
    if ((m = u.pathname.match(/^\/api\/players\/([^/]+)\/inbox$/))) {
      const p = this.player(m[1], u.searchParams.get('secret'));
      return { inbox: p.inbox.slice().reverse(), prefs: p.prefs };
    }
    throw new Error('not found');
  }

  async send(method, path, body) {
    let m;
    if ((m = path.match(/^\/api\/players\/([^/]+)\/inbox\/read$/))) {
      this.player(m[1], body.secret).inbox.forEach((n) => (n.read = true));
      this.touch();
      return { ok: true };
    }
    m = path.match(/^\/api\/worlds\/([^/]+)\/rooms\/(-?\d+)\/(-?\d+)(\/.*)?$/);
    if (!m) throw new Error('not found');
    const w = decodeURIComponent(m[1]), x = Number(m[2]), z = Number(m[3]), rest = m[4] || '';
    if (!world(w)) throw new Error('no such world');
    const p = this.player(body.player, body.secret);
    const entry = (kind, text, who = p.name) => {
      const e = { id: newId(), kind, who, player: p.id, text, at: now() };
      r.log.push(e);
      if (r.log.length > 300) r.log.shift();
      return e;
    };
    const r = this.roomMut(w, x, z);
    const mine = r.claim?.owner === p.id;
    const owner = () => {
      if (!mine) throw new Error("only the address's owner can do that; claim it first");
    };
    const changed = (extra = {}) => this.emit({ t: 'room', addr: addr(x, z), summary: this.summary(r), rebuild: true, ...extra });
    if (method === 'POST' && rest === '/log') {
      const text = String(body.text || '').trim().slice(0, 400);
      if (!text) throw new Error('empty text');
      const kind = ['request', 'praise'].includes(body.kind) ? body.kind : 'note';
      let heard = [];
      if (kind === 'request') {
        heard = tagsIn(text);
        for (const t of heard) this.bump(r.weights, t, 3);
        r.attention += 6;
      }
      const e = entry(kind, text);
      return { ok: true, entry: e, architect_heard: heard };
    }
    if (method === 'POST' && rest === '/claim') {
      if (r.claim) throw new Error('already claimed');
      const title = String(body.title || `${p.name}'s house`).slice(0, 60);
      r.claim = { owner: p.id, owner_name: p.name, title, since: now(), outbound: {} };
      entry('claim', `claimed this address as “${title}”`);
      changed();
      return { ok: true };
    }
    if (method === 'PATCH' && rest === '') {
      owner();
      if (body.theme) {
        if (!theme(body.theme)) throw new Error('unknown theme');
        r.theme = body.theme;
      }
      if (body.title) r.claim.title = String(body.title).slice(0, 60);
      if (body.outbound) r.claim.outbound = body.outbound;
      changed();
      return { ok: true };
    }
    if (method === 'POST' && rest === '/portals') {
      owner();
      const target = String(body.target || '').trim();
      const pol = world(w).portal_policy;
      if (pol.mode === 'closed') throw new Error('this world is closed to new outbound portals');
      if (r.claim.outbound?.closed) throw new Error('the owner has closed this address to new portals');
      const local = target.match(/^\/w\/([^/]+)\/(-?\d+),(-?\d+)$/);
      if (!local && !/^https?:\/\//.test(target)) throw new Error('target must be /w/<world>/<x>,<z> or an http(s) URL');
      if (local && !world(local[1])) throw new Error('no such local world');
      if (!local && pol.mode === 'allowlist') throw new Error("the world's allowlist does not include that destination");
      const built = r.portals.filter((q) => q.by !== 'generator' && q.by !== 'return').length;
      if (built >= pol.max_per_room || (r.claim.outbound?.max != null && built >= r.claim.outbound.max)) throw new Error('this room has reached its portal limit');
      const portal = { id: newId(), slot: Math.min(Number(body.slot) || 0, r.chambers.length - 1), target, label: String(body.label || 'Somewhere else').slice(0, 60), by: p.name, at: now(), sealed: false };
      r.portals.push(portal);
      entry('portal', `opened a door to ${portal.label} (${target})`);
      if (local) {
        const t = this.roomMut(local[1], Number(local[2]), Number(local[3]));
        t.portals.push({ id: `ret-${portal.id}`, slot: 0, target: `/w/${w}/${x},${z}`, label: world(w).name, by: 'return', at: now(), sealed: world(local[1]).portal_policy.mode === 'closed' });
      }
      changed();
      return { ok: true, portal };
    }
    if (method === 'DELETE' && rest.startsWith('/portals/')) {
      owner();
      const id = rest.slice('/portals/'.length);
      const gone = r.portals.find((q) => q.id === id);
      if (!gone) throw new Error('no such portal');
      r.portals = r.portals.filter((q) => q !== gone);
      const local = gone.target.match(/^\/w\/([^/]+)\/(-?\d+),(-?\d+)$/);
      const back = local && this.data.rooms[local[1]]?.[`${local[2]},${local[3]}`];
      if (back) back.portals = back.portals.filter((q) => q.id !== `ret-${id}`);
      changed();
      return { ok: true };
    }
    if (method === 'POST' && rest === '/features') {
      owner();
      if (!TAGS.includes(body.tag)) throw new Error('unknown tag');
      const f = { id: newId(), tag: body.tag, chamber: Math.min(body.chamber ?? r.chambers.length - 1, r.chambers.length - 1), seed: hash32(now(), 7), by: `agent:${p.name}`, at: now() };
      r.features.push(f);
      changed();
      return { ok: true, feature: f };
    }
    throw new Error('not found');
  }

  // ------------------------------------------------------------ live

  hello(live, m) {
    let p = this.data.player;
    if (!p || p.id !== m.player) p = this.data.player = { id: m.player, secret: m.secret, name: m.name, color: m.color, prefs: {}, inbox: [] };
    if (p.secret !== m.secret) return live.handlers.error?.({ error: 'that player id belongs to someone else' });
    p.name = m.name;
    p.color = m.color;
    this.touch();
    live.me = p.id;
    live.handlers.welcome?.({ you: p.id, peers: [], unread: p.inbox.filter((n) => !n.read).length });
  }

  move(live, m) {
    const t = now();
    const dt = Math.min(1, (t - (live.lastMove || t)) / 1000);
    live.lastMove = t;
    const a = parseAddr(m.room);
    if (!a || !live.me) return;
    const r = this.roomMut(live.world, a[0], a[1]);
    const p = this.data.player;
    r.attention += dt;
    r.investors[p.id] = (r.investors[p.id] || 0) + dt;
    for (const tag of this.chamberTags(r, m.ch || 0)) {
      this.bump(r.weights, tag, 0.01 * dt);
      this.bump(p.prefs, tag, 0.01 * dt);
    }
    if (live.room !== m.room) {
      live.room = m.room;
      if (!r.last_visit[p.id] || t - r.last_visit[p.id] > 3600e3) {
        r.last_visit[p.id] = t;
        r.log.push({ id: newId(), kind: 'visit', who: p.name, player: p.id, text: 'stopped by', at: t });
      }
    }
  }

  admire(live, m) {
    const a = parseAddr(m.room);
    if (!a) return;
    const r = this.roomMut(live.world, a[0], a[1]);
    const p = this.data.player;
    const tags = this.chamberTags(r, m.ch || 0);
    r.attention += 4;
    for (const t of tags) {
      this.bump(r.weights, t, 0.6);
      this.bump(p.prefs, t, 0.6);
    }
    live.handlers.admired?.({ addr: m.room, tags });
  }

  // The architect: start builds where attention crossed the line, finish
  // builds whose time has come.
  tick() {
    const t = now();
    for (const [w, rooms] of Object.entries(this.data.rooms)) {
      for (const r of Object.values(rooms)) {
        const growth = r.chambers.length - 1;
        if (!r.building && r.attention >= this.threshold(growth)) {
          r.building = { started: t, ready_at: t + (8 + (hash32(r.seed, t) % 14)) * 1000 };
          this.touch();
          this.emitWorld(w, { t: 'room', addr: addr(r.x, r.z), summary: this.summary(r) });
        } else if (r.building && t >= r.building.ready_at) {
          this.build(w, r, t);
        }
      }
    }
  }

  build(w, r, t) {
    const rand = new Rand(hash32(r.seed, r.chambers.length, t));
    const desire = Object.entries(this.desire(r));
    const sample = (novelty) => {
      if (rand.f() < novelty || !desire.length) return rand.pick(TAGS);
      const total = desire.reduce((s, [, v]) => s + v ** 1.5, 0);
      let roll = rand.f() * total;
      for (const [tag, v] of desire) if ((roll -= v ** 1.5) <= 0) return tag;
      return desire[desire.length - 1][0];
    };
    const last = r.chambers[r.chambers.length - 1].at;
    const requested = r.log.filter((e) => e.kind === 'request' && e.at > last).slice(-2).flatMap((e) => tagsIn(e.text));
    const tag = requested.length && rand.f() < 0.5 ? rand.pick(requested) : sample(0.12);
    const features = [tag, ...Array.from({ length: 1 + rand.int(0, 2) }, () => sample(0.2))];
    const name = `The ${rand.pick(ADJ)} ${rand.pick(NOUNS[tag] || ['Room'])}`;
    const idx = r.chambers.length;
    r.chambers.push({ name, tag, seed: hash32(r.seed, idx, 99), by: 'reference-heuristic@1', at: t });
    for (const f of features) r.features.push({ id: newId(), tag: f, chamber: idx, seed: rand.int(0, 2 ** 31), by: 'reference-heuristic@1', at: t });
    if (idx > 1 && rand.f() < 0.6) r.features.push({ id: newId(), tag: sample(0.1), chamber: rand.int(0, idx - 1), seed: rand.int(0, 2 ** 31), by: 'reference-heuristic@1', at: t });
    r.attention = Math.max(0, r.attention - this.threshold(idx - 1));
    r.building = null;
    r.log.push({ id: newId(), kind: 'growth', who: 'architect', text: `The architect built ${name} (${[...new Set(features)].join(', ')}).`, at: t });
    this.touch();
    const p = this.data.player;
    if (p && (r.investors[p.id] || 0) >= 15) {
      const n = { id: newId(), at: t, world: w, x: r.x, z: r.z, text: `${name} grew in ${world(w).name} at ${r.x},${r.z}.`, read: false };
      p.inbox.push(n);
      if (p.inbox.length > 50) p.inbox.shift();
      this.emit({ t: 'notify', n });
    }
    this.emitWorld(w, { t: 'room', addr: addr(r.x, r.z), summary: this.summary(r), grew: true });
  }

  emitWorld(w, msg) {
    for (const l of this.lives) if (l.world === w) l.handlers[msg.t]?.(msg);
  }
}

let instance = null;
export function localHost() {
  return (instance ||= new LocalHost());
}

// Same shape as net.js's Live, without a socket.
export class LocalLive {
  constructor(world, handlers) {
    this.host = localHost();
    this.world = world;
    this.handlers = handlers;
    this.host.lives.add(this);
    this.ready = false;
  }
  send(msg) {
    const h = this.host;
    if (msg.t === 'hello') {
      h.hello(this, msg);
      this.ready = true;
    } else if (!this.ready) return;
    else if (msg.t === 'move') h.move(this, msg);
    else if (msg.t === 'admire') h.admire(this, msg);
    else if (msg.t === 'chat') this.handlers.chat?.({ id: this.me, name: h.data.player?.name, color: h.data.player?.color, text: String(msg.text).slice(0, 280) });
  }
  close() {
    this.host.lives.delete(this);
  }
}
