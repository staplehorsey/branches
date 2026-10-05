// Things with a story: characters to talk to, notes and story pages to read,
// little games. Architects place them (from the story kit or their own
// imagination); every kind here is data the client knows how to bring to
// life, and anything else still shows up as a curious, labelled object.
import * as THREE from 'three';
import { G, label } from './geo.js';
import { Rand } from './rng.js';

const SKIN = ['#f3d2b8', '#e0b393', '#c68f6b', '#8d5a3f', '#5c3a27', '#f0c9a6'];
const PITCH = { low: 261.6, middle: 392.0, high: 587.3 };

function store(key, fallback) {
  try {
    const v = localStorage.getItem(key);
    return v == null ? fallback : JSON.parse(v);
  } catch {
    return fallback;
  }
}
function keep(key, value) {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {}
}

function mat(color, opts = {}) {
  return new THREE.MeshStandardMaterial({ color, roughness: 0.7, ...opts });
}

function hat(kind, color) {
  const g = new THREE.Group();
  const m = mat(color);
  const add = (geo, x, y, z, sx, sy, sz, rx = 0) => {
    const h = new THREE.Mesh(geo, m);
    h.position.set(x, y, z);
    h.scale.set(sx, sy, sz);
    h.rotation.x = rx;
    g.add(h);
  };
  switch (kind) {
    case 'wide':
      add(G.cyl, 0, 0, 0, 0.62, 0.03, 0.62);
      add(G.cyl, 0, 0.09, 0, 0.32, 0.18, 0.32);
      break;
    case 'tall':
      add(G.cyl, 0, 0, 0, 0.44, 0.03, 0.44);
      add(G.cyl, 0, 0.2, 0, 0.3, 0.4, 0.3);
      break;
    case 'cap':
      add(G.dome, 0, -0.02, 0, 0.42, 0.3, 0.42);
      add(G.box, 0, 0, -0.2, 0.3, 0.02, 0.18);
      break;
    case 'visor':
      add(G.box, 0, 0, -0.18, 0.36, 0.02, 0.2);
      add(G.torus, 0, 0, 0, 0.4, 0.4, 0.5, Math.PI / 2);
      break;
    case 'straw':
      add(G.cyl, 0, 0, 0, 0.8, 0.025, 0.8);
      add(G.dome, 0, 0, 0, 0.36, 0.28, 0.36);
      break;
    case 'beret':
      add(G.cyl, 0.04, 0.02, 0, 0.44, 0.08, 0.42);
      break;
    case 'beanie':
      add(G.dome, 0, -0.04, 0, 0.42, 0.42, 0.42);
      add(G.ballSmooth, 0, 0.18, 0, 0.1, 0.1, 0.1);
      break;
    case 'bowler':
      add(G.cyl, 0, 0, 0, 0.5, 0.025, 0.5);
      add(G.dome, 0, 0, 0, 0.34, 0.34, 0.34);
      break;
    case 'scarf':
      add(G.torus, 0, -0.3, 0, 0.5, 0.5, 0.9, Math.PI / 2);
      break;
    case 'net':
      add(G.cyl, 0.32, 0.3, 0, 0.02, 0.9, 0.02);
      add(G.torus, 0.32, 0.78, 0, 0.36, 0.36, 0.36);
      break;
  }
  return g;
}

function character(t, ctx, out) {
  const d = t.data || {};
  const look = d.look || {};
  const r = new Rand(t.seed >>> 0);
  const s = ctx.spot('floor', 0.9, 0.9);
  if (!s) return;
  const g = new THREE.Group();
  const coat = look.coat || r.pick(ctx.th.accent);
  const body = new THREE.Mesh(new THREE.CylinderGeometry(0.2, 0.34, 1.15, 14), mat(coat));
  body.position.y = 0.6;
  const head = new THREE.Mesh(G.ballSmooth, mat(r.pick(SKIN), { roughness: 0.55 }));
  head.scale.setScalar(0.36);
  head.position.y = 1.38;
  const accent = new THREE.Mesh(G.torus, mat(look.accent || '#ffffff'));
  accent.rotation.x = Math.PI / 2;
  accent.scale.set(0.34, 0.34, 0.6);
  accent.position.y = 1.16;
  const eyes = new THREE.MeshBasicMaterial({ color: '#1a1a22' });
  for (const sx of [-1, 1]) {
    const e = new THREE.Mesh(G.ballSmooth, eyes);
    e.scale.setScalar(0.04);
    e.position.set(sx * 0.07, 1.41, 0.16);
    g.add(e);
  }
  const h = hat(look.hat || 'none', look.accent || coat);
  h.position.y = 1.55;
  g.add(body, head, accent, h);
  g.position.set(s.x, 0, s.z);
  g.rotation.y = r.range(-0.6, 0.6) + Math.atan2(-(s.x - (ctx.x0 + ctx.x1) / 2), -(s.z - (ctx.z0 + ctx.z1) / 2)) + Math.PI;
  ctx.group.add(g);
  const tag = label(d.name || 'someone', { size: 0.2 });
  tag.position.set(s.x, 2.0, s.z);
  ctx.group.add(tag);
  const ph = r.range(0, 6);
  ctx.anim((time) => {
    body.scale.y = 1 + Math.sin(time * 1.4 + ph) * 0.015;
    head.position.y = 1.38 + Math.sin(time * 1.4 + ph) * 0.01;
  });
  ctx.collide(s, 0.7, 0.7);
  const lines = Array.isArray(d.lines) && d.lines.length ? d.lines : ['…'];
  out.push({
    id: t.id,
    kind: 'character',
    pos: ctx.world(s.x, 1.2, s.z),
    radius: 2.4,
    hint: `talk to ${d.name || 'them'}`,
    use(api) {
      const key = `branches.beat.${t.id}`;
      const beat = store(key, 0);
      keep(key, beat + 1);
      api.say(g, lines[beat % lines.length], d.name);
      api.touch(t.id);
    },
  });
}

function note(t, ctx, out) {
  const d = t.data || {};
  const s = ctx.spot('wall', 0.7, 0.4);
  if (!s) return;
  const f = ctx.frame(s);
  f.put(ctx.solid, G.box, ctx.th.trim, 0, 0.45, 0, { sx: 0.35, sy: 0.9, sz: 0.3 });
  f.put(ctx.solid, G.box, ctx.th.trim, 0, 0.93, 0.02, { sx: 0.55, sy: 0.05, sz: 0.4, rx: -0.4 });
  f.put(ctx.glow, G.box, d.thread ? '#fff2c8' : '#fffaf0', 0, 0.97, 0.04, { sx: 0.34, sy: 0.01, sz: 0.26, rx: -0.4 });
  const title = d.title || 'A note';
  const tag = label(d.thread ? `${title} · ${(d.page ?? 0) + 1}/${d.of ?? '?'}` : title, { size: 0.16 });
  const p = ctx.world(s.x, 1.45, s.z);
  tag.position.copy(p).sub(ctx.originV);
  ctx.group.add(tag);
  ctx.collide(s, 0.5, 0.4);
  out.push({
    id: t.id,
    kind: 'note',
    pos: ctx.world(s.x, 1, s.z),
    radius: 2.0,
    hint: `read “${title}”`,
    use(api) {
      api.read(title, d.text || '', d.thread ? `page ${(d.page ?? 0) + 1} of ${d.of ?? '?'}` : '');
      api.touch(t.id);
    },
  });
}

function lanterns(t, ctx, out) {
  const d = t.data || {};
  const n = Math.max(2, Math.min(6, d.count || 3));
  const key = `branches.lit.${t.id}`;
  const lit = new Set(store(key, []));
  const glass = [];
  for (let i = 0; i < n; i++) {
    const s = ctx.spot(i % 2 ? 'corner' : 'floor', 0.5, 0.5);
    if (!s) continue;
    const post = new THREE.Mesh(G.cyl, mat('#2a2a2a', { metalness: 0.4 }));
    post.scale.set(0.05, 1.1, 0.05);
    post.position.set(s.x, 0.55, s.z);
    const m = new THREE.MeshStandardMaterial({ color: '#c8c0a8', emissive: new THREE.Color('#ffc86a'), emissiveIntensity: lit.has(i) ? 2.2 : 0.05, roughness: 0.3, transparent: true, opacity: 0.9 });
    const lamp = new THREE.Mesh(G.ballSmooth, m);
    lamp.scale.set(0.26, 0.32, 0.26);
    lamp.position.set(s.x, 1.25, s.z);
    const cap = new THREE.Mesh(G.cone, mat('#2a2a2a'));
    cap.scale.set(0.3, 0.14, 0.3);
    cap.position.set(s.x, 1.47, s.z);
    ctx.group.add(post, lamp, cap);
    glass.push(m);
    ctx.collide(s, 0.25, 0.25);
    out.push({
      id: t.id,
      kind: 'lantern',
      pos: ctx.world(s.x, 1.2, s.z),
      radius: 1.8,
      hint: lit.has(i) ? 'already lit' : 'light the lamp',
      use(api) {
        if (lit.has(i)) return;
        lit.add(i);
        keep(key, [...lit]);
        m.emissiveIntensity = 2.2;
        this.hint = 'already lit';
        api.tone(523 + i * 66, 0.4);
        if (lit.size >= glass.length) api.touch(t.id, `lit every lamp here. ${d.reward || ''}`.trim());
        else api.touch(t.id);
      },
    });
  }
  ctx.anim((time) => glass.forEach((m, i) => lit.has(i) && (m.emissiveIntensity = 2 + Math.sin(time * 3 + i) * 0.25)));
}

function bells(t, ctx, out) {
  const d = t.data || {};
  const order = Array.isArray(d.order) && d.order.length ? d.order : ['middle', 'high', 'low'];
  const s = ctx.spot('wall', 2.0, 0.5);
  if (!s) return;
  const f = ctx.frame(s);
  f.put(ctx.solid, G.box, ctx.th.trim, -0.9, 1.1, 0, { sx: 0.08, sy: 2.2, sz: 0.08 });
  f.put(ctx.solid, G.box, ctx.th.trim, 0.9, 1.1, 0, { sx: 0.08, sy: 2.2, sz: 0.08 });
  f.put(ctx.solid, G.box, ctx.th.trim, 0, 2.2, 0, { sx: 1.9, sy: 0.08, sz: 0.08 });
  const notes = ['low', 'middle', 'high'];
  let played = [];
  const swing = [];
  notes.forEach((n, i) => {
    const size = 0.42 - i * 0.08;
    const g = new THREE.Group();
    const bell = new THREE.Mesh(G.cone, new THREE.MeshStandardMaterial({ color: '#d9a63a', metalness: 0.8, roughness: 0.25 }));
    bell.scale.set(size, size * 1.1, size);
    bell.position.y = -size * 0.6;
    g.add(bell);
    const p = f.world(-0.55 + i * 0.55, 2.15, 0);
    g.position.copy(p).sub(ctx.originV);
    ctx.group.add(g);
    swing.push({ g, t: -10 });
    out.push({
      id: t.id,
      kind: 'bell',
      pos: f.world(-0.55 + i * 0.55, 1.6, 0),
      radius: 1.6,
      hint: `ring the ${n} bell`,
      use(api) {
        api.tone(PITCH[n], 1.4);
        swing[i].t = performance.now() / 1000;
        played.push(n);
        played = played.slice(-order.length);
        if (played.join() === order.join()) {
          api.touch(t.id, 'rang the bells in the order they like');
          played = [];
          setTimeout(() => order.forEach((o, k) => setTimeout(() => api.tone(PITCH[o] * 2, 0.8), k * 220)), 400);
        } else api.touch(t.id);
      },
    });
  });
  ctx.anim(() => {
    const now = performance.now() / 1000;
    for (const s2 of swing) {
      const k = now - s2.t;
      s2.g.rotation.z = k < 2 ? Math.sin(k * 12) * 0.35 * Math.exp(-k * 2) : 0;
    }
  });
  if (d.hint) {
    const tag = label(d.hint, { size: 0.15 });
    tag.position.copy(f.world(0, 2.55, 0)).sub(ctx.originV);
    ctx.group.add(tag);
  }
  ctx.collide(s, 2.0, 0.5);
}

function cat(t, ctx, out) {
  const d = t.data || {};
  const r = new Rand(t.seed >>> 0);
  const g = new THREE.Group();
  const fur = mat(r.pick(['#3a3a40', '#d9a066', '#f2efe8', '#7a7a80']));
  const body = new THREE.Mesh(G.ballSmooth, fur);
  body.scale.set(0.28, 0.24, 0.5);
  body.position.y = 0.22;
  const head = new THREE.Mesh(G.ballSmooth, fur);
  head.scale.setScalar(0.22);
  head.position.set(0, 0.38, 0.28);
  g.add(body, head);
  for (const sx of [-1, 1]) {
    const ear = new THREE.Mesh(G.cone, fur);
    ear.scale.set(0.08, 0.12, 0.08);
    ear.position.set(sx * 0.07, 0.52, 0.28);
    g.add(ear);
  }
  const tail = new THREE.Mesh(G.cyl, fur);
  tail.scale.set(0.04, 0.4, 0.04);
  tail.position.set(0, 0.36, -0.3);
  tail.rotation.x = -0.6;
  g.add(tail);
  ctx.group.add(g);
  const cx = (ctx.x0 + ctx.x1) / 2, cz = (ctx.z0 + ctx.z1) / 2;
  const rx = (ctx.x1 - ctx.x0) / 2 - 1.2, rz = (ctx.z1 - ctx.z0) / 2 - 1.2;
  const ph = r.range(0, 6);
  const item = {
    id: t.id,
    kind: 'cat',
    pos: new THREE.Vector3(),
    radius: 1.6,
    hint: `pet ${d.name || 'the cat'}`,
    use(api) {
      api.say(g, d.says || `${d.name || 'The cat'} purrs.`, d.name || 'the cat');
      api.tone(180, 0.3);
      api.touch(t.id);
    },
  };
  ctx.anim((time) => {
    const a = time * 0.12 + ph;
    const x = cx + Math.cos(a) * rx * 0.8, z = cz + Math.sin(a * 1.3) * rz * 0.8;
    g.rotation.y = Math.atan2(x - g.position.x, z - g.position.z);
    g.position.set(x, 0, z);
    tail.rotation.z = Math.sin(time * 3) * 0.3;
    item.pos.copy(ctx.world(x, 0.4, z));
  });
  out.push(item);
}

function curious(t, ctx, out) {
  const d = t.data || {};
  const s = ctx.spot('floor', 0.8, 0.8);
  if (!s) return;
  const m = new THREE.Mesh(G.octa, new THREE.MeshStandardMaterial({ color: ctx.th.glow, emissive: new THREE.Color(ctx.th.glow), emissiveIntensity: 0.8 }));
  m.scale.setScalar(0.5);
  m.position.set(s.x, 1.2, s.z);
  ctx.group.add(m);
  ctx.anim((time) => (m.rotation.y = time * 0.6));
  const name = d.name || d.title || t.kind;
  const tag = label(name, { size: 0.18 });
  tag.position.set(s.x, 1.85, s.z);
  ctx.group.add(tag);
  out.push({
    id: t.id,
    kind: t.kind,
    pos: ctx.world(s.x, 1.2, s.z),
    radius: 2,
    hint: `look closer at ${name}`,
    use(api) {
      api.read(name, d.text || d.description || d.says || JSON.stringify(d, null, 1).slice(0, 400), t.kind);
      api.touch(t.id);
    },
  });
}

const KINDS = { character, note, lanterns, bells, cat };

export function buildThing(t, ctx, out) {
  (KINDS[t.kind] || curious)(t, ctx, out);
}
