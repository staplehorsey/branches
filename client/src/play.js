// What the architect learns from, measured as people play:
//  - gaze: which feature or thing sits under your view, and for how long,
//  - paths: the shape of how you wander (straight or winding, fast or slow,
//    new ground or old), which grows new worlds,
//  - exploration: the cells you have walked, for your own map.
import * as THREE from 'three';
import { CELL } from './layout.js';

const _ray = new THREE.Vector3();
const _dir = new THREE.Vector3();

// Slab test: distance along the ray to an AABB with y bounds, or Infinity.
function hit(o, d, b, maxT) {
  let t0 = 0, t1 = maxT;
  for (const [oo, dd, lo, hi] of [[o.x, d.x, b.x0, b.x1], [o.y, d.y, b.y0, b.y1], [o.z, d.z, b.z0, b.z1]]) {
    if (Math.abs(dd) < 1e-8) {
      if (oo < lo || oo > hi) return Infinity;
      continue;
    }
    let a = (lo - oo) / dd, c = (hi - oo) / dd;
    if (a > c) [a, c] = [c, a];
    t0 = Math.max(t0, a);
    t1 = Math.min(t1, c);
    if (t0 > t1) return Infinity;
  }
  return t0;
}

function load(key, fallback) {
  try {
    const v = localStorage.getItem(key);
    return v == null ? fallback : JSON.parse(v);
  } catch {
    return fallback;
  }
}
function save(key, value) {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {}
}

export class Signals {
  constructor() {
    this.looks = new Map(); // room -> Map(id -> secs)
    this.lastFlush = performance.now();
    this.samples = [];
    this.lastSample = 0;
    this.lastPath = performance.now();
    this.explored = new Map(); // world key -> Set("x,z")
    this.looking = null;
  }

  // Which feature or thing the view rests on, within 9 m.
  gaze(camera, interior, dt) {
    this.looking = null;
    if (!interior) return;
    camera.getWorldPosition(_ray);
    camera.getWorldDirection(_dir);
    let best = 9, id = null;
    for (const g of interior.built.gaze) {
      const d = hit(_ray, _dir, g.box, best);
      if (d < best) {
        best = d;
        id = g.id;
      }
    }
    if (!id) return;
    this.looking = id;
    const room = interior.addr;
    if (!this.looks.has(room)) this.looks.set(room, new Map());
    const m = this.looks.get(room);
    m.set(id, (m.get(id) || 0) + dt);
  }

  flush(live) {
    if (!live || performance.now() - this.lastFlush < 5000) return;
    this.lastFlush = performance.now();
    for (const [room, m] of this.looks) {
      const items = {};
      for (const [id, secs] of m) if (secs >= 0.3) items[id] = Math.round(secs * 10) / 10;
      if (Object.keys(items).length) live.send({ t: 'look', room, items });
    }
    this.looks.clear();
  }

  // Sample the walk twice a second; send its shape every 20 s or so.
  path(pos, now, live, worldKey) {
    if (now - this.lastSample > 500) {
      this.lastSample = now;
      this.samples.push({ x: pos.x, z: pos.z, t: now });
      if (this.samples.length > 160) this.samples.shift();
      this.explore(worldKey, pos);
    }
    if (!live || now - this.lastPath < 20000 || this.samples.length < 12) return;
    const sig = this.signature();
    if (!sig) return;
    this.lastPath = now;
    live.send({ t: 'path', sig });
  }

  signature() {
    const s = this.samples.slice(-120);
    let dist = 0, turn = 0, moving = 0, prevH = null, revisit = 0;
    const seen = new Set();
    for (let i = 1; i < s.length; i++) {
      const dx = s[i].x - s[i - 1].x, dz = s[i].z - s[i - 1].z;
      const d = Math.hypot(dx, dz);
      const cell = `${Math.round(s[i].x / 4)},${Math.round(s[i].z / 4)}`;
      if (seen.has(cell)) revisit++;
      seen.add(cell);
      if (d < 0.2) continue;
      dist += d;
      moving += (s[i].t - s[i - 1].t) / 1000;
      const h = Math.atan2(dz, dx);
      if (prevH !== null) turn += Math.abs(Math.atan2(Math.sin(h - prevH), Math.cos(h - prevH)));
      prevH = h;
    }
    if (dist < 15) return null;
    const a = s[0], b = s[s.length - 1];
    const disp = Math.hypot(b.x - a.x, b.z - a.z);
    const r = (v) => Math.round(v * 1000) / 1000;
    return { turn: r(turn / dist), straight: r(Math.min(1, disp / dist)), speed: r(dist / Math.max(1, moving)), revisit: r(revisit / s.length), heading: r(Math.atan2(b.z - a.z, b.x - a.x)) };
  }

  explore(worldKey, pos) {
    let set = this.explored.get(worldKey);
    if (!set) {
      set = new Set(load(`branches.explored.${worldKey}`, []));
      this.explored.set(worldKey, set);
    }
    const k = `${Math.round(pos.x / CELL)},${Math.round(pos.z / CELL)}`;
    if (!set.has(k)) {
      set.add(k);
      if (set.size < 20000) save(`branches.explored.${worldKey}`, [...set]);
    }
  }

  exploredIn(worldKey) {
    if (!this.explored.has(worldKey)) this.explored.set(worldKey, new Set(load(`branches.explored.${worldKey}`, [])));
    return this.explored.get(worldKey);
  }
}

// A soft tone for bells, lanterns and the cat.
let audio = null;
export function tone(freq, dur = 0.6) {
  try {
    audio ||= new (window.AudioContext || window.webkitAudioContext)();
    const o = audio.createOscillator(), g = audio.createGain();
    o.type = 'sine';
    o.frequency.value = freq;
    const now = audio.currentTime;
    g.gain.setValueAtTime(0.0001, now);
    g.gain.exponentialRampToValueAtTime(0.18, now + 0.02);
    g.gain.exponentialRampToValueAtTime(0.0001, now + dur);
    o.connect(g).connect(audio.destination);
    o.start(now);
    o.stop(now + dur + 0.05);
  } catch {}
}
