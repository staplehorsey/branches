// Branches client: walk an endless graph of worlds joined by doors.
//
// Everything is one continuous physical space. Several worlds ("realms")
// are loaded into the same scene at once, stacked far apart vertically, and
// every door is a live window onto the door it is linked to. Walking through
// one carries your body to the other side: no fades, no loading screens.
import * as THREE from 'three';
import { Host, Live, identity, saveIdentity } from './net.js';
import { Outdoor, time } from './outdoor.js';
import { buildInterior } from './interior.js';
import { DoorPortal, PortalRenderer, link } from './portals.js';
import { Player, isTyping } from './player.js';
import { Others } from './avatars.js';
import { disposeTree } from './geo.js';
import * as ui from './ui.js';
import { CELL, HOUSE_D, DOOR_W, DOOR_H, pocketY, houseCenter, cellOf } from './layout.js';

const $ = (id) => document.getElementById(id);
const canvas = $('view');
const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, powerPreference: 'high-performance' });
renderer.setPixelRatio(Math.min(devicePixelRatio, 1.75));
renderer.toneMapping = THREE.ACESFilmicToneMapping;
renderer.toneMappingExposure = 1.05;

const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(70, 1, 0.05, 600);
const hemi = new THREE.HemisphereLight();
const sun = new THREE.DirectionalLight();
const points = [0, 1, 2, 3].map(() => new THREE.PointLight('#ffffff', 0, 11, 1.3));
scene.add(hemi, sun, sun.target, ...points);
const peopleRoot = new THREE.Group();
scene.add(peopleRoot);
const fog = new THREE.Fog('#ffffff', 10, 100);
scene.fog = fog;

const portals = new PortalRenderer(renderer);
const player = new Player(canvas, camera);
scene.add(player.setAvatar(identity().color));

// Realms sit this far apart vertically; far beyond the camera's reach.
const SLOT_GAP = 1500;

const S = {
  realms: new Map(), // key -> realm
  spaces: new Map(), // space key -> interior
  primary: null, // the realm your body is in
  live: null,
  others: new Others(peopleRoot, identity().player),
  manifests: new Map(),
  pairing: new Set(),
  themes: [],
  inside: null, // interior you are in
  chamber: -1,
  cellKey: null,
  started: false,
  waking: false,
  unread: 0,
  lastSend: 0,
  lastPoll: 0,
};

// ---------------------------------------------------------------- addresses

function resolve(target, origin) {
  let path = target;
  if (/^https?:\/\//.test(target)) {
    const u = new URL(target);
    origin = u.origin;
    path = u.pathname;
  }
  const m = path.match(/^\/w\/([^/]+)(?:\/(-?\d+),(-?\d+))?/);
  if (!m) throw new Error('not a Branches address');
  return { origin, world: decodeURIComponent(m[1]), x: m[2] ? Number(m[2]) : 0, z: m[3] ? Number(m[3]) : 0 };
}

const realmKey = (origin, world) => `${origin}|${world}`;

async function manifestFor(origin, world) {
  const key = realmKey(origin, world);
  if (!S.manifests.has(key)) {
    const p = new Host(origin).world(world).then((r) => r.manifest);
    p.catch(() => S.manifests.delete(key));
    S.manifests.set(key, p);
  }
  return S.manifests.get(key);
}

// ---------------------------------------------------------------- realms

function realmFor(origin, manifest) {
  const key = realmKey(origin, manifest.id);
  let r = S.realms.get(key);
  if (r) return r;
  const used = new Set([...S.realms.values()].map((x) => x.slot));
  let slot = 0;
  while (used.has(slot)) slot++;
  const outRoot = new THREE.Group();
  const worldRoot = new THREE.Group();
  outRoot.add(worldRoot);
  scene.add(outRoot);
  r = {
    key,
    host: new Host(origin),
    origin,
    manifest,
    biome: manifest.generator.params,
    slot,
    base: slot * SLOT_GAP,
    outKey: `${key}@out`,
    outRoot,
    worldRoot,
    outdoor: null,
    interiors: new Map(),
    loading: new Map(),
    inject: new Map(),
  };
  S.realms.set(key, r);
  return r;
}

function disposeRealm(r) {
  for (const it of r.interiors.values()) disposeInterior(it);
  r.interiors.clear();
  r.outdoor?.dispose();
  r.outdoor = null;
  scene.remove(r.outRoot);
  S.realms.delete(r.key);
}

function realmOfSpace(space) {
  const it = S.spaces.get(space);
  if (it) return it.realm;
  for (const r of S.realms.values()) if (r.outKey === space) return r;
  return null;
}

// ---------------------------------------------------------------- spaces

function spaceOf(p) {
  for (const r of S.realms.values()) {
    if (p.y > r.base - 20 && p.y < r.base + 600) return r.outKey;
    for (const it of r.interiors.values()) if (p.y > it.origin.y - 2 && p.y < it.origin.y + 13) return it.key;
  }
  return 'void';
}

const _sunDir = new THREE.Vector3();
function prepare(space, eye) {
  for (const r of S.realms.values()) {
    r.outRoot.visible = r.outKey === space;
    for (const it of r.interiors.values()) it.wrap.visible = it.key === space;
  }
  const it = S.spaces.get(space);
  const r = realmOfSpace(space);
  if (!it && r?.outdoor) {
    const b = r.biome;
    fog.color.set(b.fog.color);
    fog.near = b.fog.near;
    fog.far = Math.min(b.fog.far, 4 * CELL + 14);
    scene.background = fog.color;
    hemi.color.set(b.hemi.sky);
    hemi.groundColor.set(b.hemi.ground);
    hemi.intensity = b.hemi.intensity;
    sun.color.set(b.sun.color);
    sun.intensity = b.sun.intensity;
    _sunDir.copy(r.outdoor.sunDir);
    for (const l of points) l.intensity = 0;
    r.outdoor.follow(eye);
  } else if (it) {
    const env = it.built.theme.env;
    fog.color.set(env.fog);
    fog.near = env.near;
    fog.far = env.far;
    scene.background = fog.color;
    hemi.color.set(env.sky);
    hemi.groundColor.set(env.ground);
    hemi.intensity = env.hemi;
    sun.color.set(env.key);
    sun.intensity = env.keyI * 1.4;
    _sunDir.set(0.35, 1, 0.25).normalize();
    const near = it.built.lights.map((l) => [l, l.pos.distanceToSquared(eye)]).sort((a, b) => a[1] - b[1]);
    points.forEach((p, i) => {
      const l = near[i]?.[0];
      p.intensity = l ? l.intensity * 5 : 0;
      if (l) {
        p.position.copy(l.pos);
        p.color.copy(l.color);
      }
    });
  }
  sun.target.position.copy(eye);
  sun.position.copy(eye).addScaledVector(_sunDir, 50);
}

function collidersFor(space, pos) {
  const it = S.spaces.get(space);
  if (it) {
    const out = it.built.colliders.slice();
    for (const d of [it.int, ...it.branch]) if (!d.open && !d.sealed) out.push(d.block);
    return out;
  }
  const r = realmOfSpace(space);
  return r?.outdoor ? r.outdoor.collidersNear(pos.x, pos.z, []) : [];
}

function allDoors() {
  const out = [];
  for (const r of S.realms.values())
    for (const it of r.interiors.values()) {
      out.push(it.int, ...it.branch);
      if (it.ext) out.push(it.ext);
    }
  return out;
}

// ---------------------------------------------------------------- interiors

function signature(room) {
  return [room.theme, room.chambers.length, room.features.length, room.portals.map((p) => p.id + p.sealed).join(','), !!room.building].join(':');
}

function withInjected(r, x, z, room) {
  const extra = r.inject.get(`${x},${z}`) || [];
  const missing = extra.filter((p) => !room.portals.some((q) => q.id === p.id || sameAddress(q.target, r.origin, p.target, r.origin)));
  return missing.length ? { ...room, portals: [...room.portals, ...missing] } : room;
}

function loadInterior(r, x, z, force = false) {
  const addr = `${x},${z}`;
  if (r.loading.has(addr)) return r.loading.get(addr);
  const job = (async () => {
    try {
      const room = withInjected(r, x, z, await r.host.room(r.manifest.id, x, z));
      if (!S.realms.has(r.key)) return null;
      const have = r.interiors.get(addr);
      if (have && !force && signature(have.room) === signature(room)) {
        have.room = room;
        return have;
      }
      return buildFor(r, x, z, room);
    } catch (e) {
      console.warn('room', addr, e);
      return null;
    } finally {
      r.loading.delete(addr);
    }
  })();
  r.loading.set(addr, job);
  return job;
}

function buildFor(r, x, z, room) {
  const addr = `${x},${z}`;
  const old = r.interiors.get(addr);
  const c = houseCenter(x, z);
  const origin = new THREE.Vector3(c.x, r.base + pocketY(x, z), c.z);
  const built = buildInterior({ room, origin, biome: r.biome });
  const key = `${r.key}@${addr}`;
  const wrap = new THREE.Group();
  wrap.add(built.group);
  const int = new DoorPortal({
    pos: new THREE.Vector3(c.x, origin.y + DOOR_H / 2, c.z + HOUSE_D / 2),
    normal: new THREE.Vector3(0, 0, -1),
    w: DOOR_W,
    h: DOOR_H,
    color: r.biome.sky.horizon,
    space: key,
  });
  wrap.add(int.mesh);
  const branch = built.branchDoors.map((d) => {
    const door = new DoorPortal({ pos: d.pos, normal: d.normal, w: d.w, h: d.h, color: built.theme.glow, space: key, sealed: d.sealed });
    door.info = d;
    wrap.add(door.mesh);
    tint(door, r);
    return door;
  });
  scene.add(wrap);
  if (old) disposeInterior(old);
  const it = { realm: r, x, z, addr, key, room, built, wrap, origin, int, branch, ext: null };
  r.interiors.set(addr, it);
  S.spaces.set(key, it);
  if (r.outdoor) attachExterior(it);
  return it;
}

function attachExterior(it) {
  if (it.ext) return;
  const r = it.realm;
  const c = houseCenter(it.x, it.z);
  it.ext = new DoorPortal({
    pos: new THREE.Vector3(c.x, r.base + DOOR_H / 2, c.z + HOUSE_D / 2),
    normal: new THREE.Vector3(0, 0, 1),
    w: DOOR_W,
    h: DOOR_H,
    color: it.built.theme.glow,
    space: r.outKey,
  });
  r.outRoot.add(it.ext.mesh);
  link(it.ext, it.int);
}

function disposeInterior(it) {
  scene.remove(it.wrap);
  for (const d of [it.int, ...it.branch]) d.dispose();
  it.ext?.dispose();
  it.ext = null;
  disposeTree(it.wrap);
  if (S.spaces.get(it.key) === it) S.spaces.delete(it.key);
  if (it.realm.interiors.get(it.addr) === it) it.realm.interiors.delete(it.addr);
}

function tint(door, r) {
  let t;
  try {
    t = resolve(door.info.portal.target, r.origin);
  } catch {
    return;
  }
  manifestFor(t.origin, t.world)
    .then((m) => m?.generator?.params?.sky && door.setColor(m.generator.params.sky.horizon))
    .catch(() => {});
}

function sameAddress(a, aOrigin, b, bOrigin) {
  try {
    const p = resolve(a, aOrigin), q = resolve(b, bOrigin);
    return p.origin === q.origin && p.world === q.world && p.x === q.x && p.z === q.z;
  } catch {
    return false;
  }
}

// Make the far side of a branch door real: load its world and the house it
// leads into, find the door that leads back, and join the two.
async function pair(it, door) {
  const id = `${it.key}#${door.info.portal.id}`;
  if (S.pairing.has(id) || door.partner) return;
  S.pairing.add(id);
  try {
    const t = resolve(door.info.portal.target, it.realm.origin);
    const manifest = await manifestFor(t.origin, t.world);
    if (!S.spaces.has(it.key) || door.partner) return;
    const dest = realmFor(t.origin, manifest);
    const here = `/w/${encodeURIComponent(it.realm.manifest.id)}/${it.x},${it.z}`;
    const back = (room) => room.portals.findIndex((p) => sameAddress(p.target, dest.origin, here, it.realm.origin));
    let other = await loadInterior(dest, t.x, t.z);
    if (!other) return;
    if (back(other.room) < 0) {
      // The far side has no door back yet (another host, or an older room):
      // the door you walked through stays behind you all the same.
      const absolute = dest.origin === it.realm.origin ? here : it.realm.origin + here;
      dest.inject.set(`${t.x},${t.z}`, [...(dest.inject.get(`${t.x},${t.z}`) || []), { id: `back-${door.info.portal.id}`, slot: 0, target: absolute, label: it.realm.manifest.name, by: 'return', at: 0 }]);
      other = buildFor(dest, t.x, t.z, withInjected(dest, t.x, t.z, other.room));
    }
    const j = other.branch.findIndex((d) => sameAddress(d.info.portal.target, dest.origin, here, it.realm.origin));
    if (j >= 0 && S.spaces.has(it.key) && !door.partner) link(door, other.branch[j]);
  } catch (e) {
    console.warn('door', e);
  } finally {
    S.pairing.delete(id);
  }
}

function homeCell() {
  if (S.inside && S.inside.realm === S.primary) return { x: S.inside.x, z: S.inside.z };
  const b = S.primary.base;
  return cellOf(player.pos.x, player.pos.z, b);
}

// Keep what you can reach: the houses around you, and whatever lies on the
// far side of the doors in the room you are standing in.
function manage() {
  const r = S.primary;
  if (!r) return;
  const h = homeCell();
  for (let dx = -1; dx <= 1; dx++)
    for (let dz = -1; dz <= 1; dz++) if (!r.interiors.has(`${h.x + dx},${h.z + dz}`)) loadInterior(r, h.x + dx, h.z + dz);

  const keep = new Set();
  if (S.inside) {
    keep.add(S.inside);
    for (const d of S.inside.branch) {
      if (!d.sealed && !d.partner) pair(S.inside, d);
      if (d.partner) keep.add(S.spaces.get(d.partner.space));
    }
  }
  for (const realm of [...S.realms.values()]) {
    for (const it of [...realm.interiors.values()]) {
      if (keep.has(it)) continue;
      const near = realm === r && Math.max(Math.abs(it.x - h.x), Math.abs(it.z - h.z)) <= 2;
      if (!near) disposeInterior(it);
    }
    if (realm !== r && realm.interiors.size === 0 && realm.loading.size === 0) disposeRealm(realm);
  }
}

// Your body is now in realm r: it becomes the world you are "in".
function promote(r) {
  const old = S.primary;
  S.primary = r;
  if (old && old !== r && old.outdoor) {
    for (const it of old.interiors.values()) {
      it.ext?.dispose();
      it.ext = null;
    }
    old.outdoor.dispose();
    old.outdoor = null;
  }
  if (!r.outdoor) r.outdoor = new Outdoor(r.worldRoot, r.manifest, r.base);
  for (const it of r.interiors.values()) attachExterior(it);
  S.others.clear();
  S.others.offsetY = r.base;
  S.cellKey = null;
  $('world-name').textContent = r.manifest.name;
  document.title = `${r.manifest.name} · Branches`;
  if (old && old !== r) ui.banner(r.manifest.name);
  if (S.started) connect();
  if (!S.themes.length) r.host.get('/api/themes').then((t) => (S.themes = t.themes)).catch(() => {});
}

function setUrl(x, z) {
  const r = S.primary;
  const path = `/w/${encodeURIComponent(r.manifest.id)}/${x},${z}`;
  const url = r.origin === location.origin ? path : `${location.pathname.startsWith('/w/') ? '/' : location.pathname}?at=${encodeURIComponent(r.origin + path)}`;
  try {
    if (location.protocol.startsWith('http') && location.pathname + location.search !== url) history.replaceState(null, '', url);
  } catch {}
  $('address').textContent = `${r.origin === location.origin ? '' : new URL(r.origin).host + ' · '}${r.manifest.id} · ${x},${z}`;
}

async function fetchChunk(cx, cz) {
  const r = S.primary;
  try {
    const { cells } = await r.host.chunk(r.manifest.id, cx - 5, cz - 5, cx + 5, cz + 5);
    if (S.primary !== r || !r.outdoor) return;
    for (const s of cells) r.outdoor.setSummary(s);
  } catch {}
}

// Arrive somewhere by address alone (first load, or waking up at home).
async function arrive(target) {
  const t = resolve(target, location.origin);
  const manifest = await manifestFor(t.origin, t.world);
  for (const r of [...S.realms.values()]) disposeRealm(r);
  S.inside = null;
  const r = realmFor(t.origin, manifest);
  promote(r);
  const c = houseCenter(t.x, t.z);
  player.place(c.x - 0.7, r.base, c.z + CELL / 2 - 1.3, -0.1);
  ui.banner(manifest.name);
  return manifest;
}

// The one non-physical move: close your eyes and wake up at home.
async function wake() {
  if (S.waking) return;
  S.waking = true;
  $('fade').style.background = '#000';
  $('fade').classList.add('on');
  await new Promise((res) => setTimeout(res, 450));
  try {
    await arrive(`${location.origin}/w/the-lush/0,0`);
  } catch (e) {
    ui.toast('Still dreaming', e.message);
  }
  $('fade').classList.remove('on');
  S.waking = false;
}

// ---------------------------------------------------------------- live

function connect() {
  const r = S.primary;
  S.live?.close();
  S.others.clear();
  S.live = new Live(r.host, r.manifest.id, {
    welcome(m) {
      S.others.sync(m.peers);
      S.unread = m.unread;
      ui.setUnread(S.unread);
    },
    join(m) {
      if (m.id === identity().player) return;
      S.others.upsert(m);
      ui.chatLine('', '', `${m.name} arrived`, true);
    },
    leave(m) {
      S.others.remove(m.id);
    },
    peers(m) {
      S.others.sync(m.list.filter((p) => p.id !== identity().player));
    },
    chat(m) {
      ui.chatLine(m.name, m.color, m.text);
      S.others.say(m.id, m.text);
    },
    room(m) {
      if (S.primary !== r) return;
      if (m.summary) r.outdoor?.setSummary(m.summary);
      if (r.interiors.has(m.addr)) {
        const [x, z] = m.addr.split(',').map(Number);
        loadInterior(r, x, z, !!m.grew);
      }
      if (m.grew && S.inside?.realm === r && S.inside.addr === m.addr) ui.banner('the house grew');
    },
    log(m) {
      if (S.inside?.addr === m.addr && m.entry.kind !== 'visit' && m.entry.player !== identity().player) ui.chatLine('', '', `${m.entry.who} wrote in the visitor log: “${m.entry.text}”`, true);
    },
    notify(m) {
      S.unread++;
      ui.setUnread(S.unread);
      ui.toast('Something grew', m.n.text);
    },
    admired(m) {
      ui.toast('The architect noticed', m.tags.length ? `more ${m.tags.join(', ')}` : '');
    },
    error(m) {
      ui.toast('Hmm', m.error);
    },
  });
}

// ---------------------------------------------------------------- input

function mailboxNear() {
  const b = S.primary.base;
  const c = cellOf(player.pos.x, player.pos.z, b);
  const h = houseCenter(c.x, c.z);
  return Math.hypot(player.pos.x - (h.x + 1.4), player.pos.z - (h.z + CELL / 2 - 2.1)) < 2.6 ? c : null;
}

function interact() {
  const r = S.primary;
  if (S.inside) {
    if (S.inside.built.guestbook.distanceTo(player.eye) < 2.6) openLog(S.inside.realm, S.inside.x, S.inside.z);
  } else {
    const c = mailboxNear();
    if (c) openLog(r, c.x, c.z);
  }
}

function openLog(r, x, z) {
  ui.openRoomPanel({ host: r.host, world: r.manifest.id, x, z, themes: S.themes });
}

function admire() {
  if (!S.inside || !S.live) return;
  S.live.send({ t: 'admire', room: S.inside.addr, ch: Math.max(0, S.chamber) });
  ui.hint('♥');
}

const chatInput = $('chat-input');
addEventListener('keydown', (e) => {
  if (!S.started) return;
  if (e.code === 'Escape') {
    if (ui.isPanelOpen()) ui.closePanel();
    chatInput.blur();
    return;
  }
  if (e.code === 'Enter') {
    if (document.activeElement === chatInput) {
      const text = chatInput.value.trim();
      if (text && S.live) S.live.send({ t: 'chat', text });
      chatInput.value = '';
      chatInput.blur();
      player.lock();
    } else if (!isTyping(e) && !ui.isPanelOpen()) {
      document.exitPointerLock?.();
      chatInput.focus();
      e.preventDefault();
    }
    return;
  }
  if (isTyping(e) || ui.isPanelOpen()) return;
  if (e.code === 'KeyE') interact();
  if (e.code === 'KeyF') admire();
  if (e.code === 'KeyV') player.thirdPerson = !player.thirdPerson;
  if (e.code === 'KeyL') openWorlds();
  if (e.code === 'KeyN') openInbox();
  if (e.code === 'KeyH') wake();
});

function openWorlds() {
  ui.openWorldsPanel({ host: S.primary.host, currentWorld: S.primary.manifest.id });
}
function openInbox() {
  ui.openInbox({ host: S.primary.host, onRead: () => ((S.unread = 0), ui.setUnread(0)) });
}
$('bell').onclick = openInbox;
$('map-btn').onclick = openWorlds;

// Touch: a stick for walking, drag anywhere else to look.
if (matchMedia('(pointer: coarse)').matches) {
  document.body.classList.add('touch');
  const stick = $('stick'), knob = $('stick-knob');
  stick.addEventListener('pointermove', (e) => {
    if (!e.pressure) return;
    const rect = stick.getBoundingClientRect();
    const dx = (e.clientX - rect.left - rect.width / 2) / (rect.width / 2), dy = (e.clientY - rect.top - rect.height / 2) / (rect.height / 2);
    player.touch = { x: Math.max(-1, Math.min(1, dx)), y: Math.max(-1, Math.min(1, -dy)), active: true };
    knob.style.transform = `translate(${player.touch.x * 40}px, ${-player.touch.y * 40}px)`;
  });
  stick.addEventListener('pointerup', () => ((player.touch = { x: 0, y: 0, active: false }), (knob.style.transform = '')));
}

// ---------------------------------------------------------------- loop

const timer = new THREE.Timer();
let t = 0;

function updateHud() {
  const it = S.inside;
  if (it) {
    const room = it.room;
    const ch = it.built.chambers[Math.max(0, S.chamber)];
    $('room-line').textContent = `${room.claim?.title || room.theme_name} · ${ch?.name || ''}`;
    $('growth').classList.add('on');
    $('growth-bar').style.width = `${room.building ? 100 : Math.min(100, (room.attention / Math.max(1, room.next_growth_at)) * 100)}%`;
    const nearBook = it.built.guestbook.distanceTo(player.eye) < 2.6;
    const door = it.branch.find((d) => d.info.pos.distanceTo(player.eye) < 3);
    ui.hint(nearBook ? 'E · visitor log' : door ? (door.sealed ? 'this door does not open from this side' : door.partner ? `through here: ${door.info.portal.label}` : 'the far side is still arriving…') : room.building ? 'the architect is building…' : 'F · more of this');
  } else {
    $('room-line').textContent = '';
    $('growth').classList.remove('on');
    ui.hint(mailboxNear() ? 'E · visitor log for this house' : '');
  }
  $('online').textContent = `● ${S.others.count() + 1}`;
}

function frame() {
  requestAnimationFrame(frame);
  timer.update();
  const dt = Math.min(timer.getDelta(), 0.05);
  if (!S.primary) return;
  step(dt);
  portals.render(scene, camera, allDoors(), prepare, spaceOf);
}

// Advance the world by dt seconds (rendering is separate).
function step(dt) {
  t += dt;
  time.value = t;

  player.enabled = S.started && !ui.isPanelOpen() && document.activeElement !== chatInput && !S.waking;
  const before = player.eye;
  const space = spaceOf(before);
  const here = S.spaces.get(space);
  if (here) {
    const ch = here.built.chambers[here.built.chamberAt(player.pos.x, player.pos.z)];
    player.ceiling = here.origin.y + (ch?.h ?? 3.2);
  } else player.ceiling = Infinity;
  player.update(dt, collidersFor(space, player.pos));
  const after = player.eye;

  // Doors: if you walked through one, your body is now on the other side.
  for (const d of allDoors()) {
    if (d.space === space && d.crossed(before, after)) {
      player.carry(d.M);
      break;
    }
  }

  const now = spaceOf(player.eye);
  const nowRealm = realmOfSpace(now);
  if (nowRealm && nowRealm !== S.primary) promote(nowRealm);
  const prevInside = S.inside;
  S.inside = S.spaces.get(now) || null;
  if (S.inside !== prevInside) {
    S.chamber = -1;
    if (S.inside && (!prevInside || prevInside.addr !== S.inside.addr)) {
      const r = S.inside.room;
      ui.banner(r.claim?.title || r.theme_name);
    }
    manage();
  }
  if (S.inside) {
    const ch = S.inside.built.chamberAt(player.pos.x, player.pos.z);
    if (ch !== S.chamber) {
      if (S.chamber >= 0 && ch > 0) ui.banner(S.inside.built.chambers[ch].name);
      S.chamber = ch;
    }
    if (performance.now() - S.lastPoll > 8000) {
      S.lastPoll = performance.now();
      loadInterior(S.inside.realm, S.inside.x, S.inside.z);
      manage();
    }
  }
  player.updateCamera(collidersFor(spaceOf(player.eye), player.pos));

  const r = S.primary;
  const home = homeCell();
  const key = `${home.x},${home.z}`;
  if (key !== S.cellKey) {
    S.cellKey = key;
    manage();
    fetchChunk(home.x, home.z);
    setUrl(home.x, home.z);
  }
  const c = houseCenter(home.x, home.z);
  r.outdoor.update(S.inside ? new THREE.Vector3(c.x, 0, c.z) : player.pos, dt);
  r.outdoor.animate(t);
  for (const realm of S.realms.values()) for (const it of realm.interiors.values()) for (const a of it.built.anims) a(t, dt);
  S.others.update(dt, t);
  updateHud();

  if (S.live && performance.now() - S.lastSend > 100) {
    S.lastSend = performance.now();
    const inPrimary = S.inside?.realm === r;
    S.live.send({
      t: 'move',
      p: [player.pos.x, player.pos.y - r.base, player.pos.z].map((v) => Math.round(v * 100) / 100),
      ry: player.yaw,
      room: inPrimary ? S.inside.addr : null,
      ch: inPrimary ? Math.max(0, S.chamber) : null,
    });
  }
}

function resize() {
  renderer.setSize(innerWidth, innerHeight, false);
  camera.aspect = innerWidth / innerHeight;
  camera.updateProjectionMatrix();
  portals.resize();
}
addEventListener('resize', resize);

// ---------------------------------------------------------------- boot

async function boot() {
  resize();
  const params = new URLSearchParams(location.search);
  const start = params.get('at') || (location.pathname.startsWith('/w/') ? location.pathname : '/w/the-lush/0,0');
  const me = identity();
  $('name').value = me.name;
  $('color').value = me.color;
  try {
    let manifest;
    try {
      manifest = await arrive(start);
    } catch {
      manifest = await arrive('/w/the-lush/0,0');
    }
    $('intro-world').textContent = manifest.name;
    $('intro-tagline').textContent = manifest.tagline;
  } catch (e) {
    $('intro-world').textContent = 'No world here';
    $('intro-tagline').textContent = e.message;
  }
  frame();
  const enter = () => {
    if (S.started) return;
    const id = identity();
    saveIdentity({ ...id, name: $('name').value.trim() || id.name, color: $('color').value });
    scene.remove(player.avatar);
    scene.add(player.setAvatar($('color').value));
    $('intro').classList.add('gone');
    setTimeout(() => ($('intro').hidden = true), 900);
    $('hud').hidden = false;
    S.started = true;
    connect();
    player.lock();
    ui.chatLine('', '', 'Walk up to any house. Go inside. Stay a while.', true);
  };
  $('enter').onclick = enter;
  $('name').addEventListener('keydown', (e) => e.code === 'Enter' && enter());
}

boot();

// Exposed for debugging and automated checks.
window.branches = { S, player, camera, renderer, wake, step: (dt, n = 1) => { for (let i = 0; i < n; i++) step(dt); } };
