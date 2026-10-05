// What the architect can build. One builder per tag; every builder dresses
// itself in the room's theme so the same tag reads differently per house.
import * as THREE from 'three';
import { G, canvasTexture, softDot } from './geo.js';
import { shade } from './themes.js';
import { Rand } from './rng.js';

const _m = new THREE.Matrix4();
const _l = new THREE.Matrix4();
const _q = new THREE.Quaternion();
const _e = new THREE.Euler();
const _v = new THREE.Vector3();
const _s = new THREE.Vector3();

// Places parts relative to a spot {x, y, z, ry} returned by ctx.spot().
export function frame(spot) {
  const base = new THREE.Matrix4().compose(
    new THREE.Vector3(spot.x, spot.y || 0, spot.z),
    new THREE.Quaternion().setFromEuler(new THREE.Euler(0, spot.ry || 0, 0)),
    new THREE.Vector3(1, 1, 1),
  );
  return {
    put(batch, geo, color, x, y, z, { rx = 0, ry = 0, rz = 0, sx = 1, sy = 1, sz = 1 } = {}) {
      _l.compose(_v.set(x, y, z), _q.setFromEuler(_e.set(rx, ry, rz)), _s.set(sx, sy, sz));
      _m.multiplyMatrices(base, _l);
      batch.add(geo, _m, color);
    },
    world(x, y, z) {
      return new THREE.Vector3(x, y, z).applyMatrix4(base);
    },
    base,
  };
}

const GREENS = ['#3f7a3a', '#5aa04a', '#2f6b3f', '#78b85a', '#4a8a5a'];

const waterTex = (() => {
  let t;
  return () => {
    if (t) return t;
    t = canvasTexture(256, (ctx, s) => {
      ctx.fillStyle = '#808080';
      ctx.fillRect(0, 0, s, s);
      const r = new Rand(9);
      ctx.strokeStyle = '#ffffff';
      for (let i = 0; i < 40; i++) {
        ctx.globalAlpha = r.range(0.15, 0.5);
        ctx.lineWidth = r.range(1, 4);
        ctx.beginPath();
        const y = r.range(0, s);
        ctx.moveTo(0, y);
        for (let x = 0; x <= s; x += 16) ctx.lineTo(x, y + Math.sin(x * 0.05 + i) * 10);
        ctx.stroke();
      }
    });
    t.userData.shared = true;
    return t;
  };
})();

function waterSurface(ctx, w, d, x, y, z) {
  const map = waterTex().clone();
  map.repeat.set(w / 2, d / 2);
  const mat = new THREE.MeshStandardMaterial({
    color: ctx.th.water,
    map,
    emissive: new THREE.Color(ctx.th.water),
    emissiveIntensity: 0.35,
    roughness: 0.08,
    metalness: 0.1,
    transparent: true,
    opacity: 0.88,
  });
  const mesh = new THREE.Mesh(G.plane, mat);
  mesh.rotation.x = -Math.PI / 2;
  mesh.scale.set(w, d, 1);
  mesh.position.set(x, y, z);
  ctx.group.add(mesh);
  ctx.anim((t) => {
    map.offset.set(t * 0.03, t * 0.05);
    mat.emissiveIntensity = 0.3 + Math.sin(t * 1.3) * 0.08;
  });
  return mesh;
}

export const FEATURES = {
  water(ctx) {
    const r = ctx.rand;
    if (r.chance(0.55)) {
      const w = r.range(2.6, 4.2), d = r.range(1.8, 3);
      const s = ctx.spot('floor', w + 0.4, d + 0.4);
      if (!s) return;
      const f = frame(s);
      const rim = ctx.th.trim;
      f.put(ctx.solid, G.box, rim, 0, 0.17, d / 2 + 0.1, { sx: w + 0.4, sy: 0.34, sz: 0.2 });
      f.put(ctx.solid, G.box, rim, 0, 0.17, -d / 2 - 0.1, { sx: w + 0.4, sy: 0.34, sz: 0.2 });
      f.put(ctx.solid, G.box, rim, w / 2 + 0.1, 0.17, 0, { sx: 0.2, sy: 0.34, sz: d });
      f.put(ctx.solid, G.box, rim, -w / 2 - 0.1, 0.17, 0, { sx: 0.2, sy: 0.34, sz: d });
      f.put(ctx.solid, G.box, shade(ctx.th.water, -0.25), 0, 0.02, 0, { sx: w, sy: 0.04, sz: d });
      const c = f.world(0, 0.27, 0);
      const surf = waterSurface(ctx, w, d, c.x, c.y, c.z);
      surf.rotation.z = -(s.ry || 0);
      ctx.collide(s, w + 0.4, d + 0.4);
      ctx.light(c.x, 0.8, c.z, ctx.th.water, 0.6);
    } else {
      const s = ctx.spot('floor', 2, 2);
      if (!s) return;
      const f = frame(s);
      f.put(ctx.solid, G.cyl, ctx.th.trim, 0, 0.25, 0, { sx: 1.9, sy: 0.5, sz: 1.9 });
      f.put(ctx.solid, G.cyl, shade(ctx.th.trim, -0.1), 0, 0.9, 0, { sx: 0.25, sy: 1.3, sz: 0.25 });
      f.put(ctx.solid, G.cyl, ctx.th.trim, 0, 1.55, 0, { sx: 0.9, sy: 0.15, sz: 0.9 });
      f.put(ctx.glow, G.ballSmooth, ctx.th.water, 0, 1.68, 0, { sx: 0.35, sy: 0.25, sz: 0.35 });
      const c = f.world(0, 0.5, 0);
      waterSurface(ctx, 1.6, 1.6, c.x, 0.51, c.z);
      ctx.collide(s, 2, 2);
    }
  },

  plants(ctx) {
    const r = ctx.rand;
    const n = r.int(2, 4);
    for (let i = 0; i < n; i++) {
      const tall = ctx.h > 4.5 && r.chance(0.5);
      const s = ctx.spot(r.chance(0.6) ? 'wall' : 'corner', tall ? 1.8 : 1, tall ? 1.8 : 1);
      if (!s) continue;
      const f = frame(s);
      const pot = r.pick(ctx.th.accent);
      f.put(ctx.solid, G.cyl, pot, 0, 0.3, 0, { sx: 0.6, sy: 0.6, sz: 0.6 });
      f.put(ctx.solid, G.cyl, '#3a2a1a', 0, 0.58, 0, { sx: 0.55, sy: 0.04, sz: 0.55 });
      if (tall) {
        const top = Math.min(ctx.h - 0.6, 5.5);
        f.put(ctx.solid, G.cyl, '#6a4a32', 0, top / 2, 0, { sx: 0.14, sy: top, sz: 0.14 });
        for (let k = 0; k < 7; k++) {
          f.put(ctx.solid, G.ball, r.pick(GREENS), r.range(-0.8, 0.8), top - r.range(0, 1.2), r.range(-0.8, 0.8), { sx: r.range(0.9, 1.5), sy: r.range(0.7, 1.1), sz: r.range(0.9, 1.5) });
        }
      } else {
        for (let k = 0; k < 9; k++) {
          const a = (k / 9) * Math.PI * 2;
          f.put(ctx.solid, G.cone, r.pick(GREENS), Math.cos(a) * 0.18, 0.95, Math.sin(a) * 0.18, { rz: Math.cos(a) * 0.6, rx: -Math.sin(a) * 0.6, sx: 0.22, sy: r.range(0.7, 1.1), sz: 0.06 });
        }
      }
      ctx.collide(s, 0.6, 0.6);
    }
    // Vines from the ceiling.
    const vines = r.int(3, 8);
    for (let i = 0; i < vines; i++) {
      const s = ctx.spot('wall', 0.3, 0.3, { soft: true });
      if (!s) continue;
      const len = r.range(0.8, Math.min(2.4, ctx.h - 1.2));
      const f = frame(s);
      f.put(ctx.solid, G.box, r.pick(GREENS), 0, ctx.h - len / 2, 0, { sx: 0.05, sy: len, sz: 0.05 });
      for (let k = 0; k < 5; k++) f.put(ctx.solid, G.ball, r.pick(GREENS), r.range(-0.06, 0.06), ctx.h - (k / 5) * len, 0, { sx: 0.16, sy: 0.12, sz: 0.16 });
    }
  },

  light(ctx) {
    const r = ctx.rand;
    const pendants = r.int(2, 5);
    for (let i = 0; i < pendants; i++) {
      const s = ctx.spot('floor', 0.5, 0.5, { soft: true, center: true });
      if (!s) continue;
      const drop = r.range(0.8, Math.min(2.2, ctx.h - 2.1));
      const f = frame(s);
      f.put(ctx.solid, G.cyl, '#222222', 0, ctx.h - drop / 2, 0, { sx: 0.02, sy: drop, sz: 0.02 });
      f.put(ctx.glow, G.ballSmooth, ctx.th.glow, 0, ctx.h - drop - 0.12, 0, { sx: 0.32, sy: 0.32, sz: 0.32 });
      const p = f.world(0, ctx.h - drop - 0.3, 0);
      ctx.light(p.x, p.y, p.z, ctx.th.glow, 1.0);
    }
    const s = ctx.spot('wall', 0.7, 0.7);
    if (s) {
      const f = frame(s);
      f.put(ctx.solid, G.cyl, ctx.th.trim, 0, 0.03, 0, { sx: 0.5, sy: 0.06, sz: 0.5 });
      f.put(ctx.solid, G.cyl, '#333333', 0, 0.85, 0, { sx: 0.04, sy: 1.7, sz: 0.04 });
      f.put(ctx.glow, G.cone, ctx.th.glow, 0, 1.75, 0, { sx: 0.6, sy: 0.4, sz: 0.6, rx: Math.PI });
      const p = f.world(0, 1.6, 0);
      ctx.light(p.x, p.y, p.z, ctx.th.glow, 0.8);
    }
    // A shaft of light from a skylight.
    const sh = ctx.spot('floor', 1.4, 1.4, { soft: true, center: true });
    if (sh) {
      const mat = new THREE.MeshBasicMaterial({ color: ctx.th.glow, transparent: true, opacity: 0.05, blending: THREE.AdditiveBlending, depthWrite: false, side: THREE.FrontSide });
      const m = new THREE.Mesh(new THREE.CylinderGeometry(0.6, 1.0, ctx.h, 16, 1, true), mat);
      m.position.set(sh.x, ctx.h / 2, sh.z);
      ctx.group.add(m);
      ctx.glow.put(G.plane, ctx.th.glow, sh.x, ctx.h - 0.01, sh.z, { rx: Math.PI / 2, sx: 1.3, sy: 1.3 });
      ctx.anim((t) => (mat.opacity = 0.05 + Math.sin(t * 0.7 + sh.x) * 0.015));
    }
  },

  books(ctx) {
    const r = ctx.rand;
    const shelves = r.int(1, 3);
    for (let i = 0; i < shelves; i++) {
      const w = r.range(1.8, 3);
      const h = Math.min(ctx.h - 0.4, r.range(2.2, 3.4));
      const s = ctx.spot('wall', w, 0.45);
      if (!s) continue;
      const f = frame(s);
      const wood = shade(ctx.th.trim, -0.35);
      f.put(ctx.solid, G.box, wood, 0, h / 2, -0.2, { sx: w, sy: h, sz: 0.04 });
      f.put(ctx.solid, G.box, wood, -w / 2, h / 2, 0, { sx: 0.05, sy: h, sz: 0.42 });
      f.put(ctx.solid, G.box, wood, w / 2, h / 2, 0, { sx: 0.05, sy: h, sz: 0.42 });
      const rows = Math.floor(h / 0.45);
      for (let row = 0; row <= rows; row++) {
        const y = row * (h / rows);
        f.put(ctx.solid, G.box, wood, 0, y, 0, { sx: w, sy: 0.04, sz: 0.42 });
        if (row === rows) break;
        let x = -w / 2 + 0.06;
        while (x < w / 2 - 0.12) {
          const bw = r.range(0.04, 0.09), bh = r.range(0.24, 0.38);
          if (r.chance(0.08)) {
            x += 0.15;
            continue;
          }
          f.put(ctx.solid, G.box, r.pick(ctx.th.accent), x + bw / 2, y + 0.02 + bh / 2, 0, { sx: bw, sy: bh, sz: 0.3, rz: r.chance(0.1) ? 0.2 : 0 });
          x += bw + 0.005;
        }
      }
      ctx.collide(s, w, 0.45);
    }
    const st = ctx.spot('floor', 0.6, 0.6);
    if (st) {
      const f = frame(st);
      let y = 0;
      for (let k = 0; k < r.int(4, 9); k++) {
        const bh = r.range(0.05, 0.09);
        f.put(ctx.solid, G.box, r.pick(ctx.th.accent), r.range(-0.03, 0.03), y + bh / 2, 0, { sx: 0.32, sy: bh, sz: 0.24, ry: r.range(-0.3, 0.3) });
        y += bh;
      }
    }
  },

  art(ctx) {
    const r = ctx.rand;
    const n = r.int(2, 4);
    for (let i = 0; i < n; i++) {
      const w = r.range(0.9, 1.8), h = r.range(0.8, 1.5);
      const s = ctx.spot('wall', w + 0.2, 0.1, { soft: true });
      if (!s) continue;
      const f = frame(s);
      const y = Math.min(1.7, ctx.h - h / 2 - 0.4);
      f.put(ctx.solid, G.box, ctx.th.trim, 0, y, -0.02, { sx: w + 0.14, sy: h + 0.14, sz: 0.06 });
      const tex = painting(r, ctx.th.accent);
      const mesh = new THREE.Mesh(G.plane, new THREE.MeshBasicMaterial({ map: tex }));
      mesh.applyMatrix4(new THREE.Matrix4().compose(f.world(0, y, 0.015), new THREE.Quaternion().setFromEuler(new THREE.Euler(0, s.ry || 0, 0)), new THREE.Vector3(w, h, 1)));
      ctx.group.add(mesh);
    }
    const sc = ctx.spot('floor', 1, 1);
    if (sc) {
      const f = frame(sc);
      f.put(ctx.solid, G.box, ctx.th.trim, 0, 0.5, 0, { sx: 0.7, sy: 1, sz: 0.7 });
      let y = 1;
      for (let k = 0; k < 3; k++) {
        const g = r.pick([G.ballSmooth, G.octa, G.torus, G.box]);
        const sz = r.range(0.3, 0.55);
        f.put(ctx.solid, g, r.pick(ctx.th.accent), 0, y + sz / 2, 0, { sx: sz, sy: sz, sz, ry: r.range(0, 3), rx: g === G.torus ? Math.PI / 2 : 0 });
        y += sz * 0.9;
      }
      ctx.collide(sc, 0.8, 0.8);
    }
  },

  cozy(ctx) {
    const r = ctx.rand;
    const s = ctx.spot('wall', 2.4, 1);
    if (s) {
      const f = frame(s);
      const c = r.pick(ctx.th.accent);
      f.put(ctx.solid, G.box, c, 0, 0.25, 0, { sx: 2.2, sy: 0.4, sz: 0.9 });
      f.put(ctx.solid, G.box, shade(c, -0.08), 0, 0.65, -0.35, { sx: 2.2, sy: 0.6, sz: 0.22 });
      f.put(ctx.solid, G.box, shade(c, -0.05), -1.05, 0.5, 0, { sx: 0.2, sy: 0.45, sz: 0.9 });
      f.put(ctx.solid, G.box, shade(c, -0.05), 1.05, 0.5, 0, { sx: 0.2, sy: 0.45, sz: 0.9 });
      for (let k = 0; k < 3; k++) f.put(ctx.solid, G.box, r.pick(ctx.th.accent), -0.6 + k * 0.6, 0.6, -0.15, { sx: 0.45, sy: 0.4, sz: 0.14, rz: r.range(-0.2, 0.2) });
      // A rug in front.
      f.put(ctx.solid, G.cyl, r.pick(ctx.th.accent), 0, 0.01, 1.4, { sx: 3, sy: 0.02, sz: 2 });
      ctx.collide(s, 2.4, 1);
    }
    const a = ctx.spot('corner', 1.1, 1.1);
    if (a) {
      const f = frame(a);
      const c = r.pick(ctx.th.accent);
      f.put(ctx.solid, G.box, c, 0, 0.25, 0, { sx: 0.9, sy: 0.4, sz: 0.9 });
      f.put(ctx.solid, G.box, c, 0, 0.7, -0.38, { sx: 0.9, sy: 0.6, sz: 0.16 });
      f.put(ctx.solid, G.cyl, ctx.th.trim, 0.85, 0.3, 0, { sx: 0.45, sy: 0.6, sz: 0.45 });
      f.put(ctx.glow, G.cone, ctx.th.glow, 0.85, 0.85, 0, { sx: 0.36, sy: 0.3, sz: 0.36 });
      const p = f.world(0.85, 0.9, 0);
      ctx.light(p.x, p.y, p.z, ctx.th.glow, 0.5);
      ctx.collide(a, 1.1, 1.1);
    }
  },

  sky(ctx) {
    const r = ctx.rand;
    const clouds = new THREE.Group();
    const n = r.int(3, 6);
    for (let i = 0; i < n; i++) {
      const s = ctx.spot('floor', 1.5, 1.5, { soft: true, center: true });
      if (!s) continue;
      const cloud = new THREE.Group();
      const mat = new THREE.MeshStandardMaterial({ color: '#ffffff', emissive: new THREE.Color(ctx.th.glow), emissiveIntensity: 0.35, roughness: 1 });
      for (let k = 0; k < 6; k++) {
        const m = new THREE.Mesh(G.ballSmooth, mat);
        const sc = r.range(0.5, 1.0);
        m.scale.set(sc * 1.3, sc * 0.8, sc);
        m.position.set(r.range(-0.7, 0.7), r.range(-0.1, 0.15), r.range(-0.4, 0.4));
        cloud.add(m);
      }
      cloud.position.set(s.x, ctx.h - r.range(0.7, 1.3), s.z);
      clouds.add(cloud);
      const ph = r.range(0, 6);
      ctx.anim((t) => (cloud.position.y = ctx.h - 0.9 + Math.sin(t * 0.4 + ph) * 0.12));
    }
    ctx.group.add(clouds);
    for (let i = 0; i < r.int(5, 10); i++) {
      const s = ctx.spot('floor', 0.3, 0.3, { soft: true, center: true });
      if (!s) continue;
      const drop = r.range(0.4, 1.4);
      ctx.solid.put(G.cyl, '#dddddd', s.x, ctx.h - drop / 2, s.z, { sx: 0.01, sy: drop, sz: 0.01 });
      ctx.glow.put(G.octa, r.pick(['#fff3b0', '#ffffff', ctx.th.glow]), s.x, ctx.h - drop - 0.1, s.z, { sx: 0.18, sy: 0.24, sz: 0.06 });
    }
  },

  creatures(ctx) {
    const r = ctx.rand;
    const n = r.int(3, 6);
    for (let i = 0; i < n; i++) {
      const s = ctx.spot('floor', 0.8, 0.8, { soft: true, center: true });
      if (!s) continue;
      const color = r.pick(ctx.th.accent);
      const jelly = new THREE.Group();
      const bell = new THREE.Mesh(G.dome, new THREE.MeshBasicMaterial({ color, transparent: true, opacity: 0.75 }));
      bell.scale.set(0.5, 0.4, 0.5);
      jelly.add(bell);
      const strandMat = new THREE.MeshBasicMaterial({ color, transparent: true, opacity: 0.45 });
      for (let k = 0; k < 5; k++) {
        const st = new THREE.Mesh(G.cyl, strandMat);
        st.scale.set(0.015, r.range(0.4, 0.8), 0.015);
        st.position.set(Math.cos(k) * 0.12, -0.3, Math.sin(k) * 0.12);
        jelly.add(st);
      }
      const y0 = r.range(1.2, Math.max(1.4, ctx.h - 1));
      jelly.position.set(s.x, y0, s.z);
      ctx.group.add(jelly);
      const ph = r.range(0, 6);
      ctx.anim((t) => {
        jelly.position.y = y0 + Math.sin(t * 0.8 + ph) * 0.25;
        bell.scale.y = 0.4 + Math.sin(t * 2.4 + ph) * 0.05;
      });
      ctx.light(s.x, y0, s.z, color, 0.35);
    }
    // A loop of small fish.
    const school = new THREE.Group();
    const center = ctx.spot('floor', 3, 3, { soft: true, center: true });
    if (center) {
      const fishMat = new THREE.MeshBasicMaterial({ color: r.pick(ctx.th.accent) });
      for (let k = 0; k < 9; k++) {
        const fish = new THREE.Mesh(G.cone, fishMat);
        fish.scale.set(0.08, 0.28, 0.05);
        fish.rotation.z = Math.PI / 2;
        const holder = new THREE.Group();
        holder.add(fish);
        fish.position.x = 1.2 + (k % 3) * 0.2;
        holder.rotation.y = (k / 9) * Math.PI * 2;
        holder.position.y = 1.4 + (k % 4) * 0.25;
        school.add(holder);
      }
      school.position.set(center.x, 0, center.z);
      ctx.group.add(school);
      ctx.anim((t) => (school.rotation.y = t * 0.35));
    }
  },

  music(ctx) {
    const r = ctx.rand;
    const s = ctx.spot('wall', 2, 1.6);
    if (s) {
      const f = frame(s);
      const body = r.chance(0.5) ? '#141414' : shade(ctx.th.trim, -0.3);
      f.put(ctx.solid, G.box, body, 0, 0.75, -0.1, { sx: 1.8, sy: 0.5, sz: 1.2 });
      f.put(ctx.solid, G.box, body, 0, 1.25, -0.6, { sx: 1.8, sy: 0.6, sz: 0.2 });
      f.put(ctx.solid, G.box, '#f8f8f0', 0, 1.02, 0.45, { sx: 1.6, sy: 0.05, sz: 0.25 });
      for (let k = 0; k < 15; k++) f.put(ctx.solid, G.box, '#111111', -0.7 + k * 0.1, 1.06, 0.4, { sx: 0.04, sy: 0.04, sz: 0.14 });
      for (const lx of [-0.8, 0.8]) f.put(ctx.solid, G.box, body, lx, 0.25, 0.3, { sx: 0.08, sy: 0.5, sz: 0.08 });
      f.put(ctx.solid, G.box, shade(body, 0.1), 0, 0.25, 1.0, { sx: 0.8, sy: 0.5, sz: 0.35 });
      ctx.collide(s, 2, 1.6);
    }
    for (let i = 0; i < 2; i++) {
      const sp = ctx.spot('corner', 0.7, 0.7);
      if (!sp) continue;
      const f = frame(sp);
      f.put(ctx.solid, G.box, '#1e1e24', 0, 0.7, 0, { sx: 0.6, sy: 1.4, sz: 0.5 });
      f.put(ctx.glow, G.cyl, r.pick(ctx.th.accent), 0, 0.95, 0.26, { rx: Math.PI / 2, sx: 0.36, sy: 0.02, sz: 0.36 });
      f.put(ctx.glow, G.cyl, r.pick(ctx.th.accent), 0, 0.4, 0.26, { rx: Math.PI / 2, sx: 0.22, sy: 0.02, sz: 0.22 });
      ctx.collide(sp, 0.6, 0.5);
    }
    // Wind chimes that sway.
    const ch = ctx.spot('floor', 0.6, 0.6, { soft: true, center: true });
    if (ch) {
      const chimes = new THREE.Group();
      const mat = new THREE.MeshStandardMaterial({ color: ctx.th.trim, metalness: 0.7, roughness: 0.3 });
      for (let k = 0; k < 6; k++) {
        const c = new THREE.Mesh(G.cyl, mat);
        const len = 0.3 + k * 0.08;
        c.scale.set(0.035, len, 0.035);
        c.position.set(Math.cos(k) * 0.15, -0.3 - len / 2, Math.sin(k) * 0.15);
        chimes.add(c);
      }
      chimes.position.set(ch.x, ctx.h - 0.3, ch.z);
      ctx.group.add(chimes);
      ctx.anim((t) => (chimes.rotation.z = Math.sin(t * 1.1) * 0.08));
    }
  },

  stone(ctx) {
    const r = ctx.rand;
    const pairs = r.int(1, 3);
    const stone = shade(ctx.th.trim, -0.05);
    for (let i = 0; i < pairs * 2; i++) {
      const s = ctx.spot('wall', 0.8, 0.8);
      if (!s) continue;
      const f = frame(s);
      f.put(ctx.solid, G.box, stone, 0, 0.15, 0, { sx: 0.75, sy: 0.3, sz: 0.75 });
      f.put(ctx.solid, G.cyl, stone, 0, ctx.h / 2, 0, { sx: 0.5, sy: ctx.h - 0.5, sz: 0.5 });
      f.put(ctx.solid, G.box, stone, 0, ctx.h - 0.15, 0, { sx: 0.75, sy: 0.3, sz: 0.75 });
      ctx.collide(s, 0.75, 0.75);
    }
    const st = ctx.spot('floor', 1.2, 1.2);
    if (st) {
      const f = frame(st);
      f.put(ctx.solid, G.box, stone, 0, 0.45, 0, { sx: 0.9, sy: 0.9, sz: 0.9 });
      f.put(ctx.solid, G.ballSmooth, '#f2f0e6', 0, 1.35, 0, { sx: 0.5, sy: 0.9, sz: 0.4 });
      f.put(ctx.solid, G.ballSmooth, '#f2f0e6', 0, 1.95, 0, { sx: 0.32, sy: 0.36, sz: 0.32 });
      ctx.collide(st, 1, 1);
    }
  },

  snow(ctx) {
    const r = ctx.rand;
    for (let i = 0; i < r.int(2, 4); i++) {
      const s = ctx.spot('corner', 1.4, 1.4);
      if (!s) continue;
      const f = frame(s);
      f.put(ctx.solid, G.ballSmooth, '#ffffff', 0, 0, 0, { sx: 1.6, sy: 0.6, sz: 1.4 });
      if (r.chance(0.6)) {
        for (let k = 0; k < 3; k++) f.put(ctx.solid, G.cone, k % 2 ? '#2f6b4a' : '#255a3e', 0, 0.6 + k * 0.45, 0, { sx: 0.9 - k * 0.22, sy: 0.7, sz: 0.9 - k * 0.22 });
        f.put(ctx.solid, G.cone, '#ffffff', 0, 1.75, 0, { sx: 0.25, sy: 0.25, sz: 0.25 });
      }
    }
    for (let i = 0; i < r.int(3, 6); i++) {
      const s = ctx.spot('floor', 0.4, 0.4, { soft: true });
      if (!s) continue;
      ctx.glow.put(G.octa, r.pick(['#bff4ff', '#e6fbff', '#9fe6ff']), s.x, r.range(0.25, 0.5), s.z, { sx: 0.25, sy: r.range(0.5, 0.9), sz: 0.25, ry: r.range(0, 2) });
    }
    // Falling snow inside the chamber.
    const count = 500;
    const pos = new Float32Array(count * 3);
    for (let i = 0; i < count; i++) {
      pos[i * 3] = r.range(ctx.x0, ctx.x1);
      pos[i * 3 + 1] = r.range(0, ctx.h);
      pos[i * 3 + 2] = r.range(ctx.z0, ctx.z1);
    }
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.BufferAttribute(pos, 3));
    const pts = new THREE.Points(geo, new THREE.PointsMaterial({ color: '#ffffff', size: 0.06, map: softDot(), transparent: true, depthWrite: false }));
    ctx.group.add(pts);
    ctx.anim((t, dt) => {
      const a = geo.attributes.position.array;
      for (let i = 0; i < count; i++) {
        a[i * 3 + 1] -= dt * (0.25 + (i % 7) * 0.04);
        a[i * 3] += Math.sin(t + i) * dt * 0.05;
        if (a[i * 3 + 1] < 0) a[i * 3 + 1] = ctx.h;
      }
      geo.attributes.position.needsUpdate = true;
    });
  },

  neon(ctx) {
    const r = ctx.rand;
    for (let i = 0; i < r.int(2, 4); i++) {
      const s = ctx.spot('wall', 1.2, 0.1, { soft: true });
      if (!s) continue;
      const f = frame(s);
      const c = r.pick(ctx.th.accent);
      const y = Math.min(2, ctx.h - 0.8);
      if (r.chance(0.5)) f.put(ctx.glow, G.torus, c, 0, y, 0.05, { sx: 1.1, sy: 1.1, sz: 1.1 });
      else for (let k = 0; k < 4; k++) f.put(ctx.glow, G.box, c, -0.45 + k * 0.3, y + (k % 2 ? 0.15 : -0.15), 0.05, { sx: 0.38, sy: 0.05, sz: 0.05, rz: k % 2 ? -0.8 : 0.8 });
      const p = f.world(0, y, 0.4);
      ctx.light(p.x, p.y, p.z, c, 0.6);
    }
    // Light strips where floor meets wall.
    ctx.glow.put(G.box, r.pick(ctx.th.accent), ctx.x0 + 0.06, 0.04, (ctx.z0 + ctx.z1) / 2, { sx: 0.04, sy: 0.04, sz: ctx.z1 - ctx.z0 - 0.4 });
    ctx.glow.put(G.box, r.pick(ctx.th.accent), ctx.x1 - 0.06, 0.04, (ctx.z0 + ctx.z1) / 2, { sx: 0.04, sy: 0.04, sz: ctx.z1 - ctx.z0 - 0.4 });
    const cab = ctx.spot('wall', 0.9, 0.9);
    if (cab) {
      const f = frame(cab);
      const c = r.pick(ctx.th.accent);
      f.put(ctx.solid, G.box, '#1a1430', 0, 0.9, 0, { sx: 0.8, sy: 1.8, sz: 0.7 });
      f.put(ctx.glow, G.box, c, 0, 1.3, 0.36, { sx: 0.6, sy: 0.45, sz: 0.01 });
      f.put(ctx.glow, G.box, shade(c, 0.2), 0, 1.72, 0.3, { sx: 0.78, sy: 0.14, sz: 0.1 });
      ctx.collide(cab, 0.8, 0.7);
    }
  },
};

function painting(r, palette) {
  const kind = r.int(0, 3);
  const cols = [r.pick(palette), r.pick(palette), r.pick(palette), shade(r.pick(palette), 0.25)];
  const seed = r.int(0, 1e9);
  const t = canvasTexture(
    128,
    (ctx, s) => {
      const q = new Rand(seed);
      const g = ctx.createLinearGradient(0, 0, 0, s);
      g.addColorStop(0, cols[3]);
      g.addColorStop(1, cols[0]);
      ctx.fillStyle = g;
      ctx.fillRect(0, 0, s, s);
      if (kind === 0) {
        ctx.fillStyle = cols[1];
        ctx.beginPath();
        ctx.arc(s * 0.5, s * 0.55, s * 0.22, 0, 7);
        ctx.fill();
        ctx.fillStyle = cols[2];
        ctx.fillRect(0, s * 0.68, s, s * 0.32);
      } else if (kind === 1) {
        for (let i = 0; i < 6; i++) {
          ctx.fillStyle = cols[i % 3];
          ctx.globalAlpha = 0.8;
          ctx.fillRect(q.range(0, s * 0.7), q.range(0, s * 0.7), q.range(10, 50), q.range(10, 50));
        }
      } else if (kind === 2) {
        for (let i = 0; i < 5; i++) {
          ctx.strokeStyle = cols[i % 3];
          ctx.lineWidth = 6;
          ctx.beginPath();
          ctx.arc(s / 2, s / 2, 10 + i * 11, 0, 7);
          ctx.stroke();
        }
      } else {
        for (let i = 0; i < 4; i++) {
          ctx.fillStyle = cols[i % 3];
          ctx.beginPath();
          ctx.moveTo(0, s * (0.5 + i * 0.12));
          for (let x = 0; x <= s; x += 8) ctx.lineTo(x, s * (0.5 + i * 0.12) + Math.sin(x * 0.05 + i) * 8);
          ctx.lineTo(s, s);
          ctx.lineTo(0, s);
          ctx.fill();
        }
      }
    },
    { repeat: false },
  );
  return t;
}
