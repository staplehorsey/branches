// Branches client: walk an endless graph of worlds joined by doors.
import * as THREE from 'three';
import { Host, Live, identity, saveIdentity } from './net.js';
import { Outdoor, time } from './outdoor.js';
import { buildInterior, tintBranch } from './interior.js';
import { DoorPortal, PortalRenderer } from './portals.js';
import { Player, isTyping } from './player.js';
import { Others } from './avatars.js';
import { disposeTree } from './geo.js';
import * as ui from './ui.js';
import { CELL, HOUSE_D, DOOR_W, DOOR_H, pocketY, houseCenter, cellOf, OUTDOOR_MIN_Y } from './layout.js';

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
const outdoorRoot = new THREE.Group();
const interiorRoot = new THREE.Group();
const peopleRoot = new THREE.Group();
scene.add(outdoorRoot, interiorRoot, peopleRoot);
const fog = new THREE.Fog('#ffffff', 10, 100);
scene.fog = fog;

const portals = new PortalRenderer(renderer);
const player = new Player(canvas, camera);
scene.add(player.setAvatar(identity().color));

const S = {
  host: null,
  world: null,
  manifest: null,
  biome: null,
  outdoor: null,
  live: null,
  others: new Others(peopleRoot, identity().player),
  interiors: new Map(),
  loading: new Set(),
  manifests: new Map(),
  themes: [],
  inside: null,
  chamber: -1,
  cellKey: null,
  traveling: false,
  started: false,
  unread: 0,
  lastSend: 0,
  lastPoll: 0,
  generation: 0,
};

// ---------------------------------------------------------------- spaces

function spaceOf(p) {
  if (p.y > OUTDOOR_MIN_Y) return 'out';
  for (const [k, it] of S.interiors) if (p.y > it.origin.y - 2 && p.y < it.origin.y + 13) return k;
  return 'void';
}

const _sunDir = new THREE.Vector3();
function prepare(space, eye) {
  const out = space === 'out';
  outdoorRoot.visible = out;
  for (const [k, it] of S.interiors) it.wrap.visible = k === space;
  if (out && S.biome) {
    const b = S.biome;
    fog.color.set(b.fog.color);
    fog.near = b.fog.near;
    fog.far = Math.min(b.fog.far, 4 * CELL + 14);
    scene.background = fog.color;
    hemi.color.set(b.hemi.sky);
    hemi.groundColor.set(b.hemi.ground);
    hemi.intensity = b.hemi.intensity;
    sun.color.set(b.sun.color);
    sun.intensity = b.sun.intensity;
    _sunDir.copy(S.outdoor.sunDir);
    for (const l of points) l.intensity = 0;
    S.outdoor.follow(eye);
  } else {
    const it = S.interiors.get(space);
    if (!it) return;
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
  if (space === 'out') return S.outdoor ? S.outdoor.collidersNear(pos.x, pos.z, []) : [];
  return S.interiors.get(space)?.built.colliders || [];
}

// ---------------------------------------------------------------- interiors

function signature(room) {
  return [room.theme, room.chambers.length, room.features.length, room.portals.length, !!room.building].join(':');
}

async function loadInterior(x, z, force = false) {
  const key = `${x},${z}`;
  if (S.loading.has(key)) return;
  S.loading.add(key);
  const gen = S.generation;
  try {
    const room = await S.host.room(S.world, x, z);
    if (gen !== S.generation) return;
    const have = S.interiors.get(key);
    if (have && !force && signature(have.room) === signature(room)) {
      have.room = room;
      return;
    }
    buildFor(x, z, room);
  } catch (e) {
    console.warn('room', key, e);
  } finally {
    S.loading.delete(key);
  }
}

function buildFor(x, z, room) {
  const key = `${x},${z}`;
  const old = S.interiors.get(key);
  const py = pocketY(x, z);
  const c = houseCenter(x, z);
  const origin = new THREE.Vector3(c.x, py, c.z);
  const built = buildInterior({ room, origin, biome: S.biome });
  const wrap = new THREE.Group();
  wrap.add(built.group);
  const ext = new DoorPortal({
    pos: new THREE.Vector3(c.x, DOOR_H / 2, c.z + HOUSE_D / 2),
    normal: new THREE.Vector3(0, 0, 1),
    w: DOOR_W,
    h: DOOR_H,
    offset: new THREE.Vector3(0, py, 0),
    color: built.theme.glow,
    space: 'out',
  });
  const int = new DoorPortal({
    pos: new THREE.Vector3(c.x, py + DOOR_H / 2, c.z + HOUSE_D / 2),
    normal: new THREE.Vector3(0, 0, -1),
    w: DOOR_W,
    h: DOOR_H,
    offset: new THREE.Vector3(0, -py, 0),
    color: S.biome.sky.horizon,
    space: key,
  });
  ext.partner = int;
  int.partner = ext;
  outdoorRoot.add(ext.mesh);
  wrap.add(int.mesh);
  interiorRoot.add(wrap);
  S.interiors.set(key, { x, z, room, built, wrap, origin, ext, int });
  if (old) disposeInterior(old);
  for (const d of built.branchDoors) {
    destination(d.portal.target)
      .then((m) => tintBranch(d, m?.generator?.params?.sky))
      .catch(() => {});
  }
}

function disposeInterior(it) {
  interiorRoot.remove(it.wrap);
  it.ext.mesh.removeFromParent();
  it.ext.dispose();
  it.int.dispose();
  disposeTree(it.wrap);
}

function homeCell() {
  if (S.inside) {
    const it = S.interiors.get(S.inside);
    if (it) return { x: it.x, z: it.z };
  }
  return cellOf(player.pos.x, player.pos.z);
}

function manageInteriors() {
  const h = homeCell();
  for (let dx = -1; dx <= 1; dx++)
    for (let dz = -1; dz <= 1; dz++) {
      const key = `${h.x + dx},${h.z + dz}`;
      if (!S.interiors.has(key)) loadInterior(h.x + dx, h.z + dz);
    }
  for (const [k, it] of S.interiors) {
    if (k === S.inside) continue;
    if (Math.max(Math.abs(it.x - h.x), Math.abs(it.z - h.z)) > 2) {
      disposeInterior(it);
      S.interiors.delete(k);
    }
  }
}

// ---------------------------------------------------------------- travel

function resolve(target) {
  let origin = S.host?.origin || location.origin;
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

async function destination(target) {
  const t = resolve(target);
  const key = `${t.origin}|${t.world}`;
  if (!S.manifests.has(key)) S.manifests.set(key, new Host(t.origin).world(t.world).then((r) => r.manifest));
  return S.manifests.get(key);
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function travel(target, { arrive = true } = {}) {
  if (S.traveling) return;
  S.traveling = true;
  try {
    const t = resolve(target);
    const manifest = await destination(target);
    $('fade').style.background = manifest.generator.params?.sky?.horizon || '#ffffff';
    $('fade').classList.add('on');
    await sleep(700);
    await enterWorld(new Host(t.origin), manifest, t.x, t.z, { arrive });
    await sleep(150);
  } catch (e) {
    ui.toast('That door does not open', e.message);
    // Step back from the doorway so we do not loop into it.
    player.pos.addScaledVector(new THREE.Vector3(Math.sin(player.yaw), 0, Math.cos(player.yaw)), 1.2);
  } finally {
    $('fade').classList.remove('on');
    S.traveling = false;
  }
}

function teardown() {
  S.generation++;
  S.live?.close();
  S.live = null;
  S.others.clear();
  for (const it of S.interiors.values()) disposeInterior(it);
  S.interiors.clear();
  S.outdoor?.dispose();
  S.outdoor = null;
  outdoorRoot.clear();
  S.inside = null;
  S.chamber = -1;
  S.cellKey = null;
}

async function enterWorld(host, manifest, x, z, { arrive }) {
  teardown();
  S.host = host;
  S.world = manifest.id;
  S.manifest = manifest;
  S.biome = manifest.generator.params;
  if (manifest.generator.kind !== 'liminal-houses@1') ui.toast('Unfamiliar world', `This client renders it as houses (${manifest.generator.kind}).`);
  S.outdoor = new Outdoor(outdoorRoot, manifest);
  const c = houseCenter(x, z);
  if (arrive) player.place(c.x, 0, c.z + HOUSE_D / 2 + 1.0, Math.PI);
  else player.place(c.x - 0.7, 0, c.z + CELL / 2 - 1.3, -0.1);
  $('world-name').textContent = manifest.name;
  document.title = `${manifest.name} · Branches`;
  ui.banner(manifest.name);
  if (S.started) connect();
  if (!S.themes.length) host.get('/api/themes').then((t) => (S.themes = t.themes)).catch(() => {});
  setUrl(x, z);
  manageInteriors();
}

function setUrl(x, z) {
  const path = `/w/${encodeURIComponent(S.world)}/${x},${z}`;
  const url = S.host.origin === location.origin ? path : `/?at=${encodeURIComponent(S.host.origin + path)}`;
  if (location.pathname + location.search !== url) history.replaceState(null, '', url);
  $('address').textContent = `${S.host.origin === location.origin ? '' : new URL(S.host.origin).host + ' · '}${S.world} · ${x},${z}`;
}

async function fetchChunk(cx, cz) {
  const gen = S.generation;
  try {
    const { cells } = await S.host.chunk(S.world, cx - 5, cz - 5, cx + 5, cz + 5);
    if (gen !== S.generation) return;
    for (const s of cells) S.outdoor.setSummary(s);
  } catch {}
}

// ---------------------------------------------------------------- live

function connect() {
  S.live?.close();
  S.others.clear();
  S.live = new Live(S.host, S.world, {
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
      if (m.summary) S.outdoor?.setSummary(m.summary);
      const it = S.interiors.get(m.addr);
      if (it) {
        const [x, z] = m.addr.split(',').map(Number);
        loadInterior(x, z, !!m.grew);
      }
      if (m.grew && S.inside === m.addr) ui.banner('the house grew');
    },
    log(m) {
      if (S.inside === m.addr && m.entry.kind !== 'visit' && m.entry.player !== identity().player) ui.chatLine('', '', `${m.entry.who} wrote in the visitor log: “${m.entry.text}”`, true);
    },
    notify(m) {
      S.unread++;
      ui.setUnread(S.unread);
      ui.toast('Something grew', m.n.text, () => travel(`/w/${m.n.world}/${m.n.x},${m.n.z}`));
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

function interact() {
  const eye = player.eye;
  if (S.inside) {
    const it = S.interiors.get(S.inside);
    if (it && it.built.guestbook.distanceTo(eye) < 2.6) return openLog(it.x, it.z);
  } else {
    const c = cellOf(player.pos.x, player.pos.z);
    const h = houseCenter(c.x, c.z);
    if (Math.hypot(player.pos.x - (h.x + 1.4), player.pos.z - (h.z + CELL / 2 - 2.1)) < 2.6) return openLog(c.x, c.z);
  }
}

function openLog(x, z) {
  ui.openRoomPanel({ host: S.host, world: S.world, x, z, onTravel: travel, themes: S.themes });
}

function admire() {
  if (!S.inside || !S.live) return;
  S.live.send({ t: 'admire', room: S.inside, ch: Math.max(0, S.chamber) });
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
      canvas.requestPointerLock?.();
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
  if (e.code === 'KeyL') ui.openWorldsPanel({ host: S.host, currentWorld: S.world, onTravel: travel });
  if (e.code === 'KeyN') openInbox();
  if (e.code === 'KeyH') travel(`${location.origin}/w/the-lush/0,0`, { arrive: false });
});

function openInbox() {
  ui.openInbox({ host: S.host, onTravel: travel, onRead: () => ((S.unread = 0), ui.setUnread(0)) });
}
$('bell').onclick = openInbox;
$('map-btn').onclick = () => ui.openWorldsPanel({ host: S.host, currentWorld: S.world, onTravel: travel });

// Touch: a stick for walking, drag anywhere else to look.
if (matchMedia('(pointer: coarse)').matches) {
  document.body.classList.add('touch');
  const stick = $('stick'), knob = $('stick-knob');
  stick.addEventListener('pointermove', (e) => {
    if (!e.pressure) return;
    const r = stick.getBoundingClientRect();
    const dx = (e.clientX - r.left - r.width / 2) / (r.width / 2), dy = (e.clientY - r.top - r.height / 2) / (r.height / 2);
    player.touch = { x: Math.max(-1, Math.min(1, dx)), y: Math.max(-1, Math.min(1, -dy)), active: true };
    knob.style.transform = `translate(${player.touch.x * 40}px, ${-player.touch.y * 40}px)`;
  });
  stick.addEventListener('pointerup', () => ((player.touch = { x: 0, y: 0, active: false }), (knob.style.transform = '')));
  let last = null;
  canvas.addEventListener('pointerdown', (e) => (last = [e.clientX, e.clientY]));
  canvas.addEventListener('pointermove', (e) => {
    if (!last) return;
    player.yaw -= (e.clientX - last[0]) * 0.005;
    player.pitch = Math.max(-1.4, Math.min(1.4, player.pitch - (e.clientY - last[1]) * 0.005));
    last = [e.clientX, e.clientY];
  });
  canvas.addEventListener('pointerup', () => (last = null));
}

// ---------------------------------------------------------------- loop

const timer = new THREE.Timer();
let t = 0;

function updateHud() {
  const it = S.inside && S.interiors.get(S.inside);
  if (it) {
    const room = it.room;
    const ch = it.built.chambers[Math.max(0, S.chamber)];
    $('room-line').textContent = `${room.claim?.title || room.theme_name} · ${ch?.name || ''}`;
    $('growth').classList.add('on');
    $('growth-bar').style.width = `${room.building ? 100 : Math.min(100, (room.attention / Math.max(1, room.next_growth_at)) * 100)}%`;
    const nearBook = it.built.guestbook.distanceTo(player.eye) < 2.6;
    ui.hint(nearBook ? 'E · visitor log' : room.building ? 'the architect is building…' : 'F · more of this');
  } else {
    $('room-line').textContent = '';
    $('growth').classList.remove('on');
    const c = cellOf(player.pos.x, player.pos.z);
    const h = houseCenter(c.x, c.z);
    const nearBox = Math.hypot(player.pos.x - (h.x + 1.4), player.pos.z - (h.z + CELL / 2 - 2.1)) < 2.6;
    ui.hint(nearBox ? 'E · visitor log for this house' : '');
  }
  $('online').textContent = `● ${S.others.count() + 1}`;
}

function frame() {
  requestAnimationFrame(frame);
  timer.update();
  const dt = Math.min(timer.getDelta(), 0.05);
  if (!S.outdoor) return;
  step(dt);
  const all = [];
  for (const d of S.interiors.values()) all.push(d.ext, d.int);
  portals.render(scene, camera, all, prepare, spaceOf);
}

// Advance the world by dt seconds (rendering is separate).
function step(dt) {
  t += dt;
  time.value = t;

  player.enabled = S.started && !ui.isPanelOpen() && document.activeElement !== chatInput && !S.traveling;
  const before = player.eye;
  const space = spaceOf(before);
  const it = S.interiors.get(space);
  if (it) {
    const ch = it.built.chambers[it.built.chamberAt(player.pos.x, player.pos.z)];
    player.ceiling = it.origin.y + (ch?.h ?? 3.2);
  } else player.ceiling = Infinity;
  player.update(dt, collidersFor(space, player.pos));
  const after = player.eye;

  // Doors between the street and the pockets below.
  outer: for (const d of S.interiors.values()) {
    for (const p of [d.ext, d.int]) {
      if (p.space === space && p.crossed(before, after)) {
        player.translate(p.offset);
        break outer;
      }
    }
  }
  // Doors to other worlds.
  if (it && !S.traveling) {
    for (const d of it.built.branchDoors) {
      const sa = before.clone().sub(d.pos).dot(d.normal), sb = after.clone().sub(d.pos).dot(d.normal);
      if (sa >= 0 && sb < 0 && Math.abs(after.z - d.pos.z) < d.w / 2 + 0.1) {
        travel(d.portal.target);
        break;
      }
    }
  }

  const now = spaceOf(player.eye);
  const wasInside = S.inside;
  S.inside = now !== 'out' && now !== 'void' ? now : null;
  if (S.inside !== wasInside) {
    S.chamber = -1;
    if (S.inside) {
      const r = S.interiors.get(S.inside).room;
      ui.banner(r.claim?.title || r.theme_name);
    }
  }
  if (S.inside) {
    const cur = S.interiors.get(S.inside);
    const ch = cur.built.chamberAt(player.pos.x, player.pos.z);
    if (ch !== S.chamber) {
      if (S.chamber >= 0 && ch > 0) ui.banner(cur.built.chambers[ch].name);
      S.chamber = ch;
    }
    if (performance.now() - S.lastPoll > 8000) {
      S.lastPoll = performance.now();
      loadInterior(cur.x, cur.z);
    }
  }
  player.updateCamera(collidersFor(spaceOf(player.eye), player.pos));

  const home = homeCell();
  const key = `${home.x},${home.z}`;
  if (key !== S.cellKey) {
    S.cellKey = key;
    manageInteriors();
    fetchChunk(home.x, home.z);
    setUrl(home.x, home.z);
  }
  const anchor = S.inside ? new THREE.Vector3(houseCenter(home.x, home.z).x, 0, houseCenter(home.x, home.z).z) : player.pos;
  S.outdoor.update(anchor, dt);
  S.outdoor.animate(t);
  for (const d of S.interiors.values()) for (const a of d.built.anims) a(t, dt);
  S.others.update(dt, t);
  updateHud();

  if (S.live && performance.now() - S.lastSend > 100) {
    S.lastSend = performance.now();
    S.live.send({ t: 'move', p: [player.pos.x, player.pos.y, player.pos.z].map((v) => Math.round(v * 100) / 100), ry: player.yaw, room: S.inside, ch: S.inside ? Math.max(0, S.chamber) : null });
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
  let start = params.get('at') || (location.pathname.startsWith('/w/') ? location.pathname : '/w/the-lush/0,0');
  let target;
  try {
    target = resolve(start);
  } catch {
    target = resolve('/w/the-lush/0,0');
    start = '/w/the-lush/0,0';
  }
  const me = identity();
  $('name').value = me.name;
  $('color').value = me.color;
  try {
    const manifest = await destination(start);
    $('intro-world').textContent = manifest.name;
    $('intro-tagline').textContent = manifest.tagline;
    document.body.style.background = manifest.generator.params?.fog?.color || '';
    await enterWorld(new Host(target.origin), manifest, target.x, target.z, { arrive: false });
  } catch (e) {
    $('intro-world').textContent = 'No world here';
    $('intro-tagline').textContent = e.message;
  }
  frame();
  $('enter').onclick = () => {
    const id = identity();
    saveIdentity({ ...id, name: $('name').value.trim() || id.name, color: $('color').value });
    scene.remove(player.avatar);
    scene.add(player.setAvatar($('color').value));
    $('intro').hidden = true;
    $('hud').hidden = false;
    S.started = true;
    connect();
    canvas.requestPointerLock?.();
    ui.chatLine('', '', 'Walk up to any house. Go inside. Stay a while.', true);
  };
}

boot();

// Exposed for debugging and automated checks.
window.branches = { S, player, travel, camera, renderer, step: (dt, n = 1) => { for (let i = 0; i < n; i++) step(dt); } };
