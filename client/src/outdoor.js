// The outside of a liminal-houses@1 world: an endless grid of identical
// houses in an overgrown suburb, streamed in around the player.
import * as THREE from 'three';
import { Batch, G, aabb, canvasTexture, softDot, label, disposeTree } from './geo.js';
import { Rand, hash32 } from './rng.js';
import { CELL, HOUSE_W, HOUSE_D, WALL_H, DOOR_W, DOOR_H } from './layout.js';
import { shade } from './themes.js';
import { makeVehicle } from './player.js';

export const time = { value: 0 };

const RADIUS = 4; // cells of props around the player
const GRASS_RADIUS = 2;

function smooth(t) {
  return t * t * (3 - 2 * t);
}

// Smooth value noise in [-1, 1].
function vnoise(seed, x, z) {
  const xi = Math.floor(x), zi = Math.floor(z);
  const u = smooth(x - xi), v = smooth(z - zi);
  const h = (a, b) => hash32(seed, a, b, 91) / 4294967296;
  const a = h(xi, zi) + (h(xi + 1, zi) - h(xi, zi)) * u;
  const b = h(xi, zi + 1) + (h(xi + 1, zi + 1) - h(xi, zi + 1)) * u;
  return (a + (b - a) * v) * 2 - 1;
}

// Shifts every placement of a batch up by a terrain height.
function lifted(batch, heightAt) {
  return {
    put(geo, color, x, y, z, opts) {
      return batch.put(geo, color, x, y + heightAt(x, z), z, opts);
    },
    add: (...a) => batch.add(...a),
  };
}
function raised(batch, dy) {
  return {
    put(geo, color, x, y, z, opts) {
      return batch.put(geo, color, x, y + dy, z, opts);
    },
    add: (...a) => batch.add(...a),
  };
}

function gableRoof(b, up = 0) {
  // Ridge along x. Built from triangles so it can be merged into the batch.
  const w = HOUSE_W / 2 + 0.45, eaveY = WALL_H + up, ridgeY = WALL_H + up + 2.1, d = HOUSE_D / 2 + 0.5;
  const v = [];
  const quad = (a, b2, c, d2) => v.push(...a, ...b2, ...c, ...a, ...c, ...d2);
  quad([-w, eaveY, d], [w, eaveY, d], [w, ridgeY, 0], [-w, ridgeY, 0]);
  quad([w, eaveY, -d], [-w, eaveY, -d], [-w, ridgeY, 0], [w, ridgeY, 0]);
  const g = new THREE.BufferGeometry();
  g.setAttribute('position', new THREE.Float32BufferAttribute(v, 3));
  g.computeVertexNormals();
  b.add(g, new THREE.Matrix4(), b.roofColor);
  g.dispose();
  // Gable ends, in wall colour.
  const gv = [];
  for (const sx of [-1, 1]) {
    const x = sx * (HOUSE_W / 2);
    if (sx > 0) gv.push(x, eaveY, HOUSE_D / 2, x, eaveY, -HOUSE_D / 2, x, ridgeY - 0.1, 0);
    else gv.push(x, eaveY, -HOUSE_D / 2, x, eaveY, HOUSE_D / 2, x, ridgeY - 0.1, 0);
  }
  const gg = new THREE.BufferGeometry();
  gg.setAttribute('position', new THREE.Float32BufferAttribute(gv, 3));
  gg.computeVertexNormals();
  b.add(gg, new THREE.Matrix4(), b.wallColor);
  gg.dispose();
}

// Each storey a house grows (as chambers are built) adds this much height.
const STOREY_H = 3;

// One shell geometry per world and height; every house uses it.
function buildHouse(biome, storeys = 1) {
  const up = (storeys - 1) * STOREY_H;
  const h = biome.house;
  const solid = new Batch();
  const glow = new Batch();
  solid.roofColor = h.roof;
  solid.wallColor = h.wall;
  const W = HOUSE_W / 2, D = HOUSE_D / 2, T = 0.2;
  const wall = h.wall, trim = h.trim;
  solid.put(G.box, wall, 0, WALL_H / 2, -D + T / 2, { sx: HOUSE_W, sy: WALL_H, sz: T });
  solid.put(G.box, wall, -W + T / 2, WALL_H / 2, 0, { sx: T, sy: WALL_H, sz: HOUSE_D });
  solid.put(G.box, wall, W - T / 2, WALL_H / 2, 0, { sx: T, sy: WALL_H, sz: HOUSE_D });
  const side = W - DOOR_W / 2;
  solid.put(G.box, wall, -(DOOR_W / 2 + side / 2), WALL_H / 2, D - T / 2, { sx: side, sy: WALL_H, sz: T });
  solid.put(G.box, wall, DOOR_W / 2 + side / 2, WALL_H / 2, D - T / 2, { sx: side, sy: WALL_H, sz: T });
  solid.put(G.box, wall, 0, (WALL_H + DOOR_H) / 2, D - T / 2, { sx: DOOR_W, sy: WALL_H - DOOR_H, sz: T });
  // Foundation, corner boards, fascia.
  solid.put(G.box, shade(wall, -0.25), 0, 0.12, 0, { sx: HOUSE_W + 0.12, sy: 0.24, sz: HOUSE_D + 0.12 });
  for (const sx of [-1, 1]) for (const sz of [-1, 1]) solid.put(G.box, trim, sx * (W - 0.05), WALL_H / 2, sz * (D - 0.05), { sx: 0.2, sy: WALL_H, sz: 0.2 });
  solid.put(G.box, trim, 0, WALL_H - 0.05, D + 0.02, { sx: HOUSE_W + 0.2, sy: 0.14, sz: 0.1 });
  // Storeys added by building: plain walls, a trim band, lit windows.
  for (let k = 1; k < storeys; k++) {
    const y0 = WALL_H + (k - 1) * STOREY_H, yc = y0 + STOREY_H / 2;
    solid.put(G.box, wall, 0, yc, -D + T / 2, { sx: HOUSE_W, sy: STOREY_H, sz: T });
    solid.put(G.box, wall, 0, yc, D - T / 2, { sx: HOUSE_W, sy: STOREY_H, sz: T });
    solid.put(G.box, wall, -W + T / 2, yc, 0, { sx: T, sy: STOREY_H, sz: HOUSE_D });
    solid.put(G.box, wall, W - T / 2, yc, 0, { sx: T, sy: STOREY_H, sz: HOUSE_D });
    solid.put(G.box, trim, 0, y0 + 0.05, 0, { sx: HOUSE_W + 0.16, sy: 0.12, sz: HOUSE_D + 0.16 });
    for (const sx of [-1, 1]) for (const sz of [-1, 1]) solid.put(G.box, trim, sx * (W - 0.05), yc, sz * (D - 0.05), { sx: 0.2, sy: STOREY_H, sz: 0.2 });
  }
  gableRoof(solid, up);
  solid.put(G.box, shade(h.roof, -0.15), 2.6, WALL_H + up + 2.0, -1.2, { sx: 0.8, sy: 1.6, sz: 0.8 });
  // Door frame and porch.
  for (const sx of [-1, 1]) solid.put(G.box, trim, sx * (DOOR_W / 2 + 0.07), DOOR_H / 2, D + 0.04, { sx: 0.14, sy: DOOR_H, sz: 0.1 });
  solid.put(G.box, trim, 0, DOOR_H + 0.08, D + 0.04, { sx: DOOR_W + 0.3, sy: 0.16, sz: 0.12 });
  solid.put(G.box, biome.path, 0, 0.1, D + 0.6, { sx: 2.4, sy: 0.2, sz: 1.2 });
  solid.put(G.box, h.door, 0, DOOR_H + 0.6, D + 0.5, { sx: 2.2, sy: 0.08, sz: 1.0, rx: 0.25 });
  glow.put(G.ballSmooth, biome.lamp, 0.95, DOOR_H + 0.2, D + 0.15, { sx: 0.16, sy: 0.22, sz: 0.16 });
  // Seen only when a door's live view is not loaded: a warm, recessed glow.
  glow.put(G.box, h.window, 0, DOOR_H / 2, D - 0.6, { sx: DOOR_W, sy: DOOR_H, sz: 0.05 });
  // Windows: front pair, sides, back. They glow, so the inside stays a secret.
  const win = (x, y, z, ry) => {
    glow.put(G.box, h.window, x, y, z, { sx: 1.6, sy: 1.2, sz: 0.04, ry });
    solid.put(G.box, trim, x, y, z, { sx: 0.06, sy: 1.25, sz: 0.08, ry });
    solid.put(G.box, trim, x, y, z, { sx: 1.65, sy: 0.06, sz: 0.08, ry });
    const ox = ry ? 0 : 1.05, oz = ry ? 1.05 : 0;
    for (const s of [-1, 1]) solid.put(G.box, h.door, x + ox * s, y, z + (ry ? oz * s : 0), { sx: ry ? 0.06 : 0.45, sy: 1.3, sz: ry ? 0.45 : 0.06 });
    solid.put(G.box, trim, x, y - 0.66, z, { sx: ry ? 0.2 : 1.9, sy: 0.08, sz: ry ? 1.9 : 0.2 });
  };
  win(-3, 1.6, D + 0.03, 0);
  win(3, 1.6, D + 0.03, 0);
  win(-W - 0.03, 1.6, 0, Math.PI / 2);
  win(W + 0.03, 1.6, 0, Math.PI / 2);
  win(-2.5, 1.6, -D - 0.03, 0);
  win(2.5, 1.6, -D - 0.03, 0);
  for (let k = 1; k < storeys; k++) {
    const y = WALL_H + (k - 1) * STOREY_H + 1.55;
    for (const x of [-3, 0, 3]) win(x, y, D + 0.03, 0);
    win(-W - 0.03, y, 0, Math.PI / 2);
    win(W + 0.03, y, 0, Math.PI / 2);
    for (const x of [-2.5, 2.5]) win(x, y, -D - 0.03, 0);
  }
  const colliders = [
    [-W, -D, W, -D + T],
    [-W, -D, -W + T, D],
    [W - T, -D, W, D],
    [-W, D - T - 0.05, -DOOR_W / 2, D + 0.1],
    [DOOR_W / 2, D - T - 0.05, W, D + 0.1],
  ];
  return { solid: solid.build(), glow: glow.build(), colliders, top: WALL_H + up + 2.9, chimney: [2.6, WALL_H + up + 2.9, -1.2] };
}

// Every place has its door in the same spot (centre front, facing +z), so
// the pocket below and the portals work the same for all of them.
const DZ = HOUSE_D / 2;

function doorGlow(glow, biome) {
  glow.put(G.box, biome.house.window, 0, DOOR_H / 2, DZ - 0.6, { sx: DOOR_W, sy: DOOR_H, sz: 0.05 });
}

// A narrow tower: tall, a conical roof, slit windows.
function buildTower(biome) {
  const h = biome.house;
  const solid = new Batch();
  const glow = new Batch();
  const W = 3.2, back = -2.4, T = 0.25, HT = 8.5;
  const zc = (DZ + back) / 2, depth = DZ - back;
  solid.put(G.box, h.wall, 0, HT / 2, back + T / 2, { sx: W * 2, sy: HT, sz: T });
  solid.put(G.box, h.wall, -W + T / 2, HT / 2, zc, { sx: T, sy: HT, sz: depth });
  solid.put(G.box, h.wall, W - T / 2, HT / 2, zc, { sx: T, sy: HT, sz: depth });
  const side = W - DOOR_W / 2;
  solid.put(G.box, h.wall, -(DOOR_W / 2 + side / 2), HT / 2, DZ - T / 2, { sx: side, sy: HT, sz: T });
  solid.put(G.box, h.wall, DOOR_W / 2 + side / 2, HT / 2, DZ - T / 2, { sx: side, sy: HT, sz: T });
  solid.put(G.box, h.wall, 0, (HT + DOOR_H) / 2, DZ - T / 2, { sx: DOOR_W, sy: HT - DOOR_H, sz: T });
  solid.put(G.box, shade(h.wall, -0.25), 0, 0.15, zc, { sx: W * 2 + 0.3, sy: 0.3, sz: depth + 0.3 });
  solid.put(G.cone, h.roof, 0, HT + 1.9, zc, { sx: W * 2.9, sy: 3.8, sz: depth * 1.45, ry: Math.PI / 4 });
  for (let k = 0; k < 3; k++) {
    for (const sx of [-1, 1]) glow.put(G.box, h.window, sx * 1.6, 3.2 + k * 2, DZ + 0.03, { sx: 0.35, sy: 1.1, sz: 0.04 });
    glow.put(G.box, h.window, -W - 0.03, 3.2 + k * 2, zc, { sx: 0.04, sy: 1.1, sz: 0.35 });
    glow.put(G.box, h.window, W + 0.03, 3.2 + k * 2, zc, { sx: 0.04, sy: 1.1, sz: 0.35 });
  }
  for (const sx of [-1, 1]) solid.put(G.box, h.trim, sx * (DOOR_W / 2 + 0.07), DOOR_H / 2, DZ + 0.04, { sx: 0.14, sy: DOOR_H, sz: 0.1 });
  solid.put(G.box, h.trim, 0, DOOR_H + 0.08, DZ + 0.04, { sx: DOOR_W + 0.3, sy: 0.16, sz: 0.12 });
  solid.put(G.box, biome.path, 0, 0.1, DZ + 0.6, { sx: 2.2, sy: 0.2, sz: 1.2 });
  glow.put(G.ballSmooth, biome.lamp, 0.95, DOOR_H + 0.2, DZ + 0.15, { sx: 0.16, sy: 0.22, sz: 0.16 });
  doorGlow(glow, biome);
  const colliders = [
    [-W, back, W, back + T],
    [-W, back, -W + T, DZ],
    [W - T, back, W, DZ],
    [-W, DZ - T - 0.05, -DOOR_W / 2, DZ + 0.1],
    [DOOR_W / 2, DZ - T - 0.05, W, DZ + 0.1],
  ];
  return { solid: solid.build(), glow: glow.build(), colliders, top: HT + 4.2 };
}

// A hill with a door in its face: mossy stones, a lintel, warm light.
function buildCave(biome) {
  const solid = new Batch();
  const glow = new Batch();
  const stone = shade(biome.path, -0.28);
  const moss = biome.hedge;
  const r = new Rand(4242);
  for (let k = 0; k < 9; k++) {
    const x = r.range(-4.5, 4.5), z = r.range(-5, 1.5);
    const sx = r.range(4, 7), sy = r.range(3, 5.5), sz = r.range(4, 6);
    solid.put(G.ball, k % 3 ? moss : stone, x, sy * 0.25, z, { sx, sy, sz, ry: r.range(0, 3) });
  }
  // The rock face around the door.
  for (const sx of [-1, 1]) {
    solid.put(G.ball, stone, sx * 1.6, 1.4, DZ - 0.2, { sx: 2.2, sy: 3.4, sz: 1.4 });
    solid.put(G.ball, stone, sx * 3.6, 1.0, DZ - 0.8, { sx: 3.2, sy: 2.8, sz: 2.4 });
  }
  solid.put(G.box, stone, 0, DOOR_H + 0.35, DZ - 0.05, { sx: DOOR_W + 1.4, sy: 0.7, sz: 0.9 });
  for (const sx of [-1, 1]) solid.put(G.box, shade(stone, 0.06), sx * (DOOR_W / 2 + 0.2), DOOR_H / 2, DZ - 0.05, { sx: 0.4, sy: DOOR_H, sz: 0.9 });
  solid.put(G.box, biome.path, 0, 0.08, DZ + 0.6, { sx: 2.4, sy: 0.16, sz: 1.2 });
  for (const sx of [-1, 1]) glow.put(G.ballSmooth, biome.lamp, sx * 1.4, 0.5, DZ + 0.5, { sx: 0.2, sy: 0.26, sz: 0.2 });
  doorGlow(glow, biome);
  const colliders = [
    [-6.5, -7, -DOOR_W / 2, DZ + 0.1],
    [DOOR_W / 2, -7, 6.5, DZ + 0.1],
    [-DOOR_W / 2, -7, DOOR_W / 2, DZ - 0.75],
  ];
  return { solid: solid.build(), glow: glow.build(), colliders, top: 5.2 };
}

// A freestanding stone arch on a little plaza: the door is all there is.
function buildArch(biome) {
  const solid = new Batch();
  const glow = new Batch();
  const stone = shade(biome.house.trim, -0.12);
  solid.put(G.cyl, shade(biome.path, -0.05), 0, 0.06, DZ - 1, { sx: 7, sy: 0.12, sz: 6 });
  for (const sx of [-1, 1]) {
    solid.put(G.box, stone, sx * (DOOR_W / 2 + 0.3), (DOOR_H + 0.5) / 2, DZ - 0.25, { sx: 0.6, sy: DOOR_H + 0.5, sz: 0.6 });
    solid.put(G.box, shade(stone, -0.1), sx * (DOOR_W / 2 + 0.3), 0.15, DZ - 0.25, { sx: 0.8, sy: 0.3, sz: 0.8 });
    solid.put(G.box, stone, sx * 3.2, 1.2, DZ - 2.6, { sx: 0.7, sy: 2.4, sz: 0.5, ry: sx * 0.3 });
  }
  solid.put(G.box, stone, 0, DOOR_H + 0.55, DZ - 0.25, { sx: DOOR_W + 1.4, sy: 0.5, sz: 0.7 });
  solid.put(G.box, shade(stone, 0.08), 0, DOOR_H + 0.9, DZ - 0.25, { sx: DOOR_W + 0.6, sy: 0.2, sz: 0.5 });
  glow.put(G.octa, biome.lamp, 0, DOOR_H + 1.3, DZ - 0.25, { sx: 0.3, sy: 0.45, sz: 0.3 });
  doorGlow(glow, biome);
  const colliders = [
    [-DOOR_W / 2 - 0.6, DZ - 0.55, -DOOR_W / 2, DZ + 0.05],
    [DOOR_W / 2, DZ - 0.55, DOOR_W / 2 + 0.6, DZ + 0.05],
    // Behind the doorway: you can only go through it from the front.
    [-DOOR_W / 2, DZ - 0.75, DOOR_W / 2, DZ - 0.6],
  ];
  return { solid: solid.build(), glow: glow.build(), colliders, top: DOOR_H + 2.2 };
}

const PLACES = { house: buildHouse, tower: buildTower, cave: buildCave, arch: buildArch };

function tree(b, glow, biome, r, x, z, scale = 1) {
  const f = biome.foliage;
  const trunk = biome.trunk;
  const s = scale * r.range(0.8, 1.25);
  switch (biome.trees) {
    case 'pine': {
      const h = r.range(6, 10) * s;
      b.put(G.cyl, trunk, x, h * 0.15, z, { sx: 0.4 * s, sy: h * 0.3, sz: 0.4 * s });
      for (let k = 0; k < 4; k++) {
        const y = h * (0.25 + k * 0.18);
        const rad = (2.6 - k * 0.55) * s;
        b.put(G.cone, r.pick(f), x, y + rad * 0.6, z, { sx: rad * 2, sy: rad * 1.6, sz: rad * 2, ry: r.range(0, 3) });
      }
      break;
    }
    case 'blossom': {
      const h = r.range(2.2, 3) * s;
      b.put(G.cyl, trunk, x, h / 2, z, { sx: 0.3 * s, sy: h, sz: 0.3 * s, rz: r.range(-0.1, 0.1) });
      for (let k = 0; k < 9; k++) {
        b.put(G.ball, r.pick(f), x + r.range(-1.6, 1.6) * s, h + r.range(-0.2, 1.4) * s, z + r.range(-1.6, 1.6) * s, { sx: r.range(1.1, 1.8) * s, sy: r.range(0.8, 1.3) * s, sz: r.range(1.1, 1.8) * s });
      }
      for (let k = 0; k < 10; k++) b.put(G.cyl, f[0], x + r.range(-2.5, 2.5), 0.02, z + r.range(-2.5, 2.5), { sx: 0.12, sy: 0.01, sz: 0.12 });
      break;
    }
    case 'palm': {
      let px = x, py = 0, pz = z;
      const lean = r.range(0, Math.PI * 2);
      const segs = 7;
      for (let k = 0; k < segs; k++) {
        const len = 0.9 * s;
        const tilt = 0.05 + k * 0.035;
        px += Math.cos(lean) * Math.sin(tilt) * len;
        pz += Math.sin(lean) * Math.sin(tilt) * len;
        b.put(G.cyl, k % 2 ? trunk : shade(trunk, -0.08), px, py + len / 2, pz, { sx: 0.28 * s, sy: len, sz: 0.28 * s, rz: -Math.cos(lean) * tilt, rx: Math.sin(lean) * tilt });
        py += len * Math.cos(tilt);
      }
      for (let k = 0; k < 8; k++) {
        const a = (k / 8) * Math.PI * 2;
        b.put(G.cone, r.pick(f), px + Math.cos(a) * 1.2 * s, py + 0.1, pz + Math.sin(a) * 1.2 * s, { sx: 0.5 * s, sy: 2.8 * s, sz: 0.12 * s, rz: Math.PI / 2 + 0.35, ry: -a });
      }
      for (let k = 0; k < 3; k++) b.put(G.ballSmooth, '#6a4a2a', px + r.range(-0.2, 0.2), py - 0.2, pz + r.range(-0.2, 0.2), { sx: 0.3, sy: 0.3, sz: 0.3 });
      break;
    }
    case 'mushroom': {
      const h = r.range(1.5, 4) * s;
      const cap = r.range(1.4, 2.6) * s;
      const c = r.pick(f);
      b.put(G.cyl, trunk, x, h / 2, z, { sx: 0.35 * s, sy: h, sz: 0.35 * s });
      glow.put(G.dome, c, x, h - 0.1, z, { sx: cap * 2, sy: cap * 0.9, sz: cap * 2 });
      glow.put(G.cyl, shade(c, -0.25), x, h - 0.08, z, { sx: cap * 1.9, sy: 0.05, sz: cap * 1.9 });
      for (let k = 0; k < 5; k++) {
        const a = r.range(0, 7), d = r.range(0.6, 1.8);
        const mh = r.range(0.2, 0.5);
        b.put(G.cyl, trunk, x + Math.cos(a) * d, mh / 2, z + Math.sin(a) * d, { sx: 0.07, sy: mh, sz: 0.07 });
        glow.put(G.dome, r.pick(f), x + Math.cos(a) * d, mh, z + Math.sin(a) * d, { sx: 0.3, sy: 0.15, sz: 0.3 });
      }
      break;
    }
    default: {
      const h = r.range(2, 3) * s;
      b.put(G.cyl, trunk, x, h / 2, z, { sx: 0.32 * s, sy: h, sz: 0.32 * s });
      for (let k = 0; k < 6; k++) {
        b.put(G.ball, r.pick(f), x + r.range(-1.1, 1.1) * s, h + r.range(0.2, 1.8) * s, z + r.range(-1.1, 1.1) * s, { sx: r.range(1.5, 2.3) * s, sy: r.range(1.3, 2) * s, sz: r.range(1.5, 2.3) * s });
      }
    }
  }
  return { x, z, r: 0.35 * s };
}

function flowers(b, biome, r, x0, z0, x1, z1, n) {
  for (let i = 0; i < n; i++) {
    const x = r.range(x0, x1), z = r.range(z0, z1);
    const h = r.range(0.2, 0.45);
    b.put(G.cyl6, '#3f7a3a', x, h / 2, z, { sx: 0.025, sy: h, sz: 0.025 });
    b.put(G.ball, r.pick(biome.flowers), x, h, z, { sx: 0.18, sy: 0.12, sz: 0.18 });
  }
}

function makeGrassGeometry(biome) {
  const g = new THREE.BufferGeometry();
  const base = new THREE.Color(biome.grass.base), tip = new THREE.Color(biome.grass.tip);
  const mid = base.clone().lerp(tip, 0.5);
  g.setAttribute('position', new THREE.Float32BufferAttribute([-0.05, 0, 0, 0.05, 0, 0, 0.03, 0.5, 0, -0.03, 0.5, 0, 0.0, 1, 0], 3));
  g.setAttribute('normal', new THREE.Float32BufferAttribute([0, 1, 0.3, 0, 1, 0.3, 0, 1, 0.3, 0, 1, 0.3, 0, 1, 0.3], 3));
  g.setAttribute('color', new THREE.Float32BufferAttribute([...base.toArray(), ...base.toArray(), ...mid.toArray(), ...mid.toArray(), ...tip.toArray()], 3));
  g.setIndex([0, 1, 2, 0, 2, 3, 3, 2, 4]);
  return g;
}

function makeGrassMaterial() {
  const m = new THREE.MeshLambertMaterial({ vertexColors: true, side: THREE.DoubleSide });
  m.onBeforeCompile = (shader) => {
    shader.uniforms.uTime = time;
    // Light both faces of a blade the same way.
    shader.fragmentShader = shader.fragmentShader.replace('#include <normal_fragment_begin>', '#include <normal_fragment_begin>\n normal = normalize( vNormal );');
    shader.vertexShader =
      'uniform float uTime;\n' +
      shader.vertexShader.replace(
        '#include <begin_vertex>',
        `#include <begin_vertex>
        vec4 ip = instanceMatrix * vec4(0.0, 0.0, 0.0, 1.0);
        float sway = sin(uTime * 1.6 + ip.x * 0.31 + ip.z * 0.17) * 0.5 + sin(uTime * 2.7 + ip.x * 0.7) * 0.2;
        transformed.x += sway * position.y * position.y * 0.35;
        transformed.z += sway * position.y * position.y * 0.25;`,
      );
  };
  return m;
}

const parseAddr = (s) => {
  const m = /^\s*(-?\d+)\s*,\s*(-?\d+)\s*$/.exec(s || '');
  return m ? [Number(m[1]), Number(m[2])] : [0, 0];
};

export class Outdoor {
  // `base` lifts the whole world: several worlds share one scene, stacked.
  // `manifest._versions` (newest first) and `_ring` let far bands of the
  // world show the biome as it was at older versions.
  constructor(root, manifest, base = 0) {
    this.root = root;
    this.base = base;
    root.position.y = base;
    this.manifest = manifest;
    this.biome = manifest.generator.params;
    this.seed = manifest.seed >>> 0;
    this.spawn = parseAddr(manifest.spawn);
    this.ring = manifest._ring || 6;
    this.bands = [this.biome, ...(manifest._versions || []).map((v) => (v.biome && v.biome.sky ? v.biome : this.biome))];
    const t = this.biome.terrain || {};
    this.amp = t.amp ?? 2;
    this.scale = t.scale ?? 60;
    this.pads = new Map();
    this.cells = new Map();
    this.summaries = new Map();
    this.places = new Map();
    this.grassGeos = new Map();
    this.racks = new Map();
    this.solidMat = new THREE.MeshLambertMaterial({ vertexColors: true, flatShading: true });
    this.groundMat = new THREE.MeshLambertMaterial({ vertexColors: true });
    this.glowMat = new THREE.MeshBasicMaterial({ vertexColors: true });
    this.shadowMat = new THREE.MeshBasicMaterial({ color: '#000000', transparent: true, opacity: 0.16, depthWrite: false });
    this.grassMat = makeGrassMaterial();
    this.waterMat = new THREE.MeshStandardMaterial({ color: this.biome.water, emissive: new THREE.Color(this.biome.water), emissiveIntensity: this.biome.night ? 0.5 : 0.15, roughness: 0.05, metalness: 0.2 });
    for (const m of [this.solidMat, this.groundMat, this.glowMat, this.shadowMat, this.grassMat, this.waterMat]) m.userData.shared = true;
    this.queue = [];
    this.center = null;
    this.band = 0;
    this.buildUnderlay();
    this.buildSky();
    this.buildMotes();
  }

  // ------------------------------------------------------------ terrain

  rawHeight(x, z) {
    if (this.amp <= 0) return 0;
    const s = this.scale;
    const n = vnoise(this.seed, x / s, z / s) + 0.45 * vnoise(this.seed + 1, (x / s) * 2.3, (z / s) * 2.3) + 0.18 * vnoise(this.seed + 2, (x / s) * 5.1, (z / s) * 5.1);
    return (this.amp * n) / 1.63;
  }

  // Each place sits on a level pad; the land eases into it.
  padAt(cx, cz) {
    const k = `${cx},${cz}`;
    if (!this.pads.has(k)) this.pads.set(k, Math.round(this.rawHeight(cx * CELL, cz * CELL) * 20) / 20);
    return this.pads.get(k);
  }

  heightAt(x, z) {
    const cx = Math.round(x / CELL), cz = Math.round(z / CELL);
    const lx = x - cx * CELL, lz = z - cz * CELL;
    const pad = this.padAt(cx, cz);
    const dx = Math.max(0, Math.abs(lx) - 6.5), dz = Math.max(0, lz > 0 ? lz - 10 : -lz - 7);
    const k = smooth(Math.min(1, Math.hypot(dx, dz) / 4));
    return pad + (this.rawHeight(x, z) - pad) * k;
  }

  // ------------------------------------------------------------ versions

  bandOf(cx, cz) {
    const n = this.bands.length;
    if (n === 1) return 0;
    const d = Math.max(Math.abs(cx - this.spawn[0]), Math.abs(cz - this.spawn[1]));
    return Math.min(n - 1, Math.floor((n * d) / (d + this.ring)));
  }

  biomeAt(cx, cz) {
    return this.bands[this.bandOf(cx, cz)];
  }

  placeOf(cx, cz, biome) {
    const w = biome.places || { house: 1 };
    const kinds = Object.keys(PLACES).filter((k) => (w[k] || 0) > 0);
    if (!kinds.length) return 'house';
    const total = kinds.reduce((s2, k) => s2 + w[k], 0);
    let roll = (hash32(this.seed, cx, cz, 515) / 4294967296) * total;
    for (const k of kinds) if ((roll -= w[k]) <= 0) return k;
    return kinds[0];
  }

  // Untouched addresses are all the same house; a house that has grown
  // gets its own shape (by the biome's mix of places) and a storey for
  // every couple of chambers.
  shapeOf(cx, cz, biome) {
    const g = this.summaries.get(`${cx},${cz}`)?.growth || 0;
    if (g <= 0) return { kind: 'house', storeys: 1, grown: false };
    const kind = this.placeOf(cx, cz, biome);
    return { kind, storeys: kind === 'house' ? 1 + Math.min(8, Math.floor((g + 1) / 2)) : 1, grown: true };
  }

  place(band, kind, storeys = 1) {
    const key = `${band}:${kind}:${storeys}`;
    if (!this.places.has(key)) {
      const p = PLACES[kind](this.bands[band], storeys);
      for (const g of [p.solid, p.glow]) if (g) g.userData.shared = true;
      this.places.set(key, p);
    }
    return this.places.get(key);
  }

  // ------------------------------------------------------------ sky & air

  buildUnderlay() {
    const b = this.biome;
    this.ground = new THREE.Mesh(new THREE.PlaneGeometry(CELL * 30, CELL * 30), new THREE.MeshLambertMaterial({ color: shade(b.ground, -0.08) }));
    this.ground.rotation.x = -Math.PI / 2;
    this.ground.position.y = -this.amp - 0.6;
    this.root.add(this.ground);
  }

  buildSky() {
    this.sunDir = new THREE.Vector3();
    const mat = new THREE.ShaderMaterial({
      side: THREE.BackSide,
      depthWrite: false,
      fog: false,
      uniforms: {
        top: { value: new THREE.Color() },
        horizon: { value: new THREE.Color() },
        bottom: { value: new THREE.Color() },
        sunDir: { value: this.sunDir },
        sunColor: { value: new THREE.Color() },
        night: { value: 0 },
        uTime: time,
      },
      vertexShader: 'varying vec3 vDir; void main(){ vDir = normalize(position); gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0); }',
      fragmentShader: `uniform vec3 top, horizon, bottom, sunDir, sunColor; uniform float night, uTime; varying vec3 vDir;
        float h21(vec2 p){ return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
        void main(){
          vec3 d = normalize(vDir);
          float y = d.y;
          vec3 c = y > 0.0 ? mix(horizon, top, pow(y, 0.55)) : mix(horizon, bottom, pow(-y, 0.4));
          float sd = max(dot(d, normalize(sunDir)), 0.0);
          c += sunColor * (pow(sd, 900.0) * 3.0 + pow(sd, 12.0) * 0.25);
          if (night > 0.5 && y > 0.0) {
            vec2 g = floor(vec2(atan(d.z, d.x) * 180.0, y * 260.0));
            float st = step(0.995, h21(g));
            c += vec3(st) * (0.6 + 0.4 * sin(uTime * 2.0 + h21(g + 3.0) * 30.0)) * smoothstep(0.05, 0.4, y);
          }
          gl_FragColor = vec4(c, 1.0);
          #include <tonemapping_fragment>
          #include <colorspace_fragment>
        }`,
    });
    this.sky = new THREE.Mesh(new THREE.SphereGeometry(400, 32, 16), mat);
    this.sky.frustumCulled = false;
    this.sky.renderOrder = -1;
    this.root.add(this.sky);
    this.setSky(this.biome);
  }

  // The sky, light and air of whichever version band you are standing in.
  setSky(b) {
    this.biome = b;
    const u = this.sky.material.uniforms;
    u.top.value.set(b.sky.top);
    u.horizon.value.set(b.sky.horizon);
    u.bottom.value.set(b.sky.bottom);
    u.sunColor.value.set(b.sun.color);
    u.night.value = b.night ? 1 : 0;
    this.sunDir.set(Math.cos(b.sun.azimuth) * Math.cos(b.sun.elevation), Math.sin(b.sun.elevation), Math.sin(b.sun.azimuth) * Math.cos(b.sun.elevation));
    if (this.motes) {
      this.motes.material.color.set(b.motes);
      this.motes.material.size = b.night ? 0.16 : 0.07;
    }
  }

  buildMotes() {
    const n = 380;
    const pos = new Float32Array(n * 3);
    const r = new Rand(this.seed);
    for (let i = 0; i < n; i++) {
      pos[i * 3] = r.range(-30, 30);
      pos[i * 3 + 1] = r.range(0.3, 7);
      pos[i * 3 + 2] = r.range(-30, 30);
    }
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.BufferAttribute(pos, 3));
    const mat = new THREE.PointsMaterial({ color: this.biome.motes, size: this.biome.night ? 0.16 : 0.07, map: softDot(), transparent: true, depthWrite: false, blending: THREE.AdditiveBlending, opacity: 0.9 });
    this.motes = new THREE.Points(geo, mat);
    this.motes.frustumCulled = false;
    this.root.add(this.motes);
  }

  // Called before each render from a camera standing outdoors.
  follow(cam) {
    this.sky.position.set(cam.x, cam.y - this.base, cam.z);
  }

  update(player, dt) {
    const cx = Math.round(player.x / CELL), cz = Math.round(player.z / CELL);
    this.ground.position.x = cx * CELL;
    this.ground.position.z = cz * CELL;
    const band = this.bandOf(cx, cz);
    if (band !== this.band) {
      this.band = band;
      this.setSky(this.bands[band]);
    }
    const a = this.motes.geometry.attributes.position.array;
    const t = time.value;
    const py = player.y - this.base;
    for (let i = 0; i < a.length; i += 3) {
      a[i] += Math.sin(t * 0.3 + i) * dt * 0.3;
      a[i + 1] += Math.sin(t * 0.5 + i * 0.7) * dt * 0.15;
      const dx = a[i] - player.x, dz = a[i + 2] - player.z;
      if (dx > 30) a[i] -= 60;
      if (dx < -30) a[i] += 60;
      if (dz > 30) a[i + 2] -= 60;
      if (dz < -30) a[i + 2] += 60;
      if (a[i + 1] - py > 9) a[i + 1] -= 9;
      if (a[i + 1] - py < -1) a[i + 1] += 9;
    }
    this.motes.geometry.attributes.position.needsUpdate = true;
    if (this.biome.night) this.motes.material.opacity = 0.65 + Math.sin(t * 2.3) * 0.3;

    const key = `${cx},${cz}`;
    if (key !== this.center) {
      this.center = key;
      const want = new Set();
      for (let dx = -RADIUS; dx <= RADIUS; dx++)
        for (let dz = -RADIUS; dz <= RADIUS; dz++) {
          const k = `${cx + dx},${cz + dz}`;
          want.add(k);
          if (!this.cells.has(k) && !this.queue.includes(k)) this.queue.push(k);
        }
      for (const [k, cell] of this.cells) {
        const [x, z] = k.split(',').map(Number);
        if (Math.max(Math.abs(x - cx), Math.abs(z - cz)) > RADIUS + 1) {
          this.root.remove(cell.group);
          if (cell.grass) {
            this.root.remove(cell.grass);
            cell.grass.dispose();
          }
          disposeTree(cell.group);
          this.cells.delete(k);
          this.racks.delete(k);
        } else {
          this.setGrass(cell, Math.max(Math.abs(x - cx), Math.abs(z - cz)) <= GRASS_RADIUS);
        }
      }
      this.queue = this.queue.filter((k) => want.has(k));
      const dist = (k) => {
        const [x, z] = k.split(',').map(Number);
        return Math.abs(x - cx) + Math.abs(z - cz);
      };
      this.queue.sort((a2, b2) => dist(a2) - dist(b2));
    }
    // Build a couple of cells per frame to keep things smooth.
    for (let i = 0; i < 2 && this.queue.length; i++) {
      const k = this.queue.shift();
      const [x, z] = k.split(',').map(Number);
      const cell = this.buildCell(x, z);
      this.cells.set(k, cell);
      this.setGrass(cell, Math.max(Math.abs(x - cx), Math.abs(z - cz)) <= GRASS_RADIUS);
    }
  }

  get loading() {
    return this.queue.length;
  }

  // ------------------------------------------------------------ cells

  groundMesh(cx, cz, b) {
    const H = CELL / 2, N = 32;
    const g = new THREE.PlaneGeometry(CELL, CELL, N, N);
    g.rotateX(-Math.PI / 2);
    const pos = g.attributes.position;
    const col = new Float32Array(pos.count * 3);
    const grass = new THREE.Color(b.ground), path = new THREE.Color(b.path), edge = new THREE.Color(shade(b.path, -0.04));
    const r = new Rand(hash32(this.seed, cx, cz, 12));
    const c = new THREE.Color();
    for (let i = 0; i < pos.count; i++) {
      const lx = pos.getX(i), lz = pos.getZ(i);
      pos.setY(i, this.heightAt(cx * CELL + lx, cz * CELL + lz));
      const street = Math.abs(lx) > H - 1.5 || Math.abs(lz) > H - 1.5;
      const walk = Math.abs(lx) < 0.7 && lz > HOUSE_D / 2 + 0.8 && lz < H - 1.5;
      if (street) c.copy(Math.abs(lx) > H - 1.5 && Math.abs(lz) > H - 1.5 ? edge : path);
      else if (walk) c.copy(path);
      else c.copy(grass).offsetHSL(0, 0, r.range(-0.03, 0.03));
      col[i * 3] = c.r;
      col[i * 3 + 1] = c.g;
      col[i * 3 + 2] = c.b;
    }
    g.setAttribute('color', new THREE.BufferAttribute(col, 3));
    g.computeVertexNormals();
    return new THREE.Mesh(g, this.groundMat);
  }

  buildCell(cx, cz) {
    const band = this.bandOf(cx, cz);
    const b = this.bands[band];
    const shape = this.shapeOf(cx, cz, b);
    // Liminal: every untouched lawn is the same lawn.
    const r = new Rand(shape.grown ? hash32(this.seed, cx, cz, 77) : hash32(this.seed, 0, 0, 77 + band));
    const group = new THREE.Group();
    const ox = cx * CELL, oz = cz * CELL;
    group.position.set(ox, 0, oz);
    const solidB = new Batch();
    const glowB = new Batch();
    const shadowB = new Batch();
    const h = (lx, lz) => this.heightAt(ox + lx, oz + lz);
    const solid = lifted(solidB, h);
    const glow = lifted(glowB, h);
    const shadow = lifted(shadowB, (x, z) => h(x, z) + 0.02);
    const colliders = [];
    const col = (x0, z0, x1, z1) => colliders.push(aabb(ox + x0, oz + z0, ox + x1, oz + z1, this.base - 30, this.base + 60));
    const H = CELL / 2;
    const pad = this.padAt(cx, cz);

    group.add(this.groundMesh(cx, cz, b));

    // The place with the door: same door, many shapes.
    const kind = shape.kind;
    const pl = this.place(band, kind, shape.storeys);
    const body = new THREE.Mesh(pl.solid, this.solidMat);
    body.position.y = pad;
    group.add(body);
    if (pl.glow) {
      const gl = new THREE.Mesh(pl.glow, this.glowMat);
      gl.position.y = pad;
      group.add(gl);
    }
    for (const c of pl.colliders) col(...c);
    if (kind === 'house' || kind === 'tower') shadowB.put(G.box, '#000', 0, pad + 0.02, -0.5, { sx: kind === 'house' ? HOUSE_W + 1.2 : 7.4, sy: 0.01, sz: kind === 'house' ? HOUSE_D + 1.2 : 8 });

    // Mailbox at the end of the walk.
    const my = h(1.4, H - 2.1);
    solidB.put(G.box, b.house.trim, 1.4, my + 0.55, H - 2.1, { sx: 0.1, sy: 1.1, sz: 0.1 });
    solidB.put(G.box, b.house.door, 1.4, my + 1.15, H - 2.1, { sx: 0.3, sy: 0.3, sz: 0.5 });
    col(1.25, H - 2.35, 1.55, H - 1.85);

    // Hedges along the side yards, in short runs that follow the land.
    for (const sx of [-1, 1]) {
      const x = sx * 8.2;
      let z = -H + 1.8;
      while (z < H - 3) {
        const len = r.range(2.5, 6);
        const end = Math.min(z + len, H - 3);
        if (r.chance(0.82)) {
          for (let zz = z; zz < end - 0.1; zz += 1.5) {
            const seg = Math.min(1.5, end - zz), mid = zz + seg / 2;
            solid.put(G.box, b.hedge, x, 0.6, mid, { sx: 0.9, sy: 1.2, sz: seg + 0.05 });
            solid.put(G.ball, shade(b.hedge, r.range(-0.04, 0.06)), x + r.range(-0.15, 0.15), 1.15, mid, { sx: 1, sy: 0.55, sz: seg + 0.2 });
          }
          if (r.chance(0.35)) flowers(solid, b, r, x - 0.5, z, x + 0.5, end, Math.floor((end - z) * 2));
          col(x - 0.5, z, x + 0.5, end);
          shadow.put(G.box, '#000', x + 0.4, 0, (z + end) / 2, { sx: 1.6, sy: 0.01, sz: end - z + 0.4 });
        }
        z = end + r.range(1.2, 2.2);
      }
    }

    // Flower beds along the front.
    if (kind === 'house' || kind === 'tower') {
      for (const sx of [-1, 1]) {
        solid.put(G.box, shade(b.ground, -0.12), sx * 3, 0.06, HOUSE_D / 2 + 0.6, { sx: kind === 'house' ? 3.6 : 1.6, sy: 0.12, sz: 0.9 });
        flowers(solid, b, r, sx * 3 - 1.6, HOUSE_D / 2 + 0.25, sx * 3 + 1.6, HOUSE_D / 2 + 0.95, kind === 'house' ? 14 : 6);
      }
    } else {
      flowers(solid, b, r, -5, HOUSE_D / 2 + 0.5, 5, HOUSE_D / 2 + 3, 18);
    }

    // Trees.
    const trees = [];
    const spots = [
      [r.range(-6, 6), r.range(-H + 2.5, -HOUSE_D / 2 - 2.5)],
      [r.range(-6, 6), r.range(-H + 2.5, -HOUSE_D / 2 - 2.5)],
      [r.chance(0.5) ? -10.3 : 10.3, r.range(-8, 6)],
      [r.chance(0.5) ? -5.5 : 5.5, r.range(7.5, 9)],
    ];
    spots.forEach(([x, z], i) => {
      if (i > 1 && r.chance(0.45)) return;
      if (i === 1 && Math.abs(spots[0][0] - x) < 3.5) return;
      if (kind === 'cave' && i < 2 && Math.abs(x) < 7) return;
      const y = h(x, z);
      trees.push(tree(raised(solidB, y), raised(glowB, y), b, r, x, z, i === 3 ? 0.7 : 1));
    });
    for (const t of trees) {
      col(t.x - t.r, t.z - t.r, t.x + t.r, t.z + t.r);
      shadow.put(G.cyl, '#000', t.x + 0.6, 0.005, t.z + 0.3, { sx: 4.5, sy: 0.01, sz: 4.5 });
    }

    // A pond in some back yards, where the land allows.
    if (shape.grown && r.chance(b.ponds ?? 0.2) && kind !== 'cave') {
      const px = r.chance(0.5) ? -4.5 : 4.5, pz = -H + 4.2;
      const py = Math.min(h(px - 2, pz), h(px + 2, pz), h(px, pz - 1.5), h(px, pz + 1.5));
      const pond = new THREE.Mesh(G.cyl, this.waterMat);
      pond.scale.set(4.4, 0.05, 3.2);
      pond.position.set(px, py + 0.04, pz);
      group.add(pond);
      for (let k = 0; k < 14; k++) {
        const a = (k / 14) * Math.PI * 2;
        solid.put(G.ball, shade(b.path, r.range(-0.15, 0)), px + Math.cos(a) * 2.25, 0.06, pz + Math.sin(a) * 1.65, { sx: 0.6, sy: 0.3, sz: 0.5 });
      }
      for (let k = 0; k < 5; k++) solidB.put(G.cyl, '#4f9a4a', px + r.range(-1.4, 1.4), py + 0.08, pz + r.range(-1, 1), { sx: 0.5, sy: 0.02, sz: 0.5 });
      col(px - 2.2, pz - 1.6, px + 2.2, pz + 1.6);
    }

    // A lamp post on the corner.
    const lx = H - 1.9, lz = H - 1.9;
    solid.put(G.cyl, '#3a3a3a', lx, 1.7, lz, { sx: 0.12, sy: 3.4, sz: 0.12 });
    solid.put(G.cyl, '#3a3a3a', lx, 3.45, lz, { sx: 0.5, sy: 0.1, sz: 0.5 });
    glow.put(G.ballSmooth, b.lamp, lx, 3.2, lz, { sx: 0.42, sy: 0.5, sz: 0.42 });
    col(lx - 0.12, lz - 0.12, lx + 0.12, lz + 0.12);
    if (b.night) glow.put(G.cyl, b.lamp, lx, 0.03, lz, { sx: 3.5, sy: 0.005, sz: 3.5 });

    // Bikes to borrow at some corners.
    if (hash32(this.seed, cx, cz, 808) % 3 === 0) {
      const bx = lx - 2.4, bz = H - 1.2;
      const by = h(bx, bz);
      solidB.put(G.box, '#3a3a3a', bx, by + 0.4, bz, { sx: 1.8, sy: 0.06, sz: 0.06 });
      for (const ex of [-0.9, 0.9]) solidB.put(G.box, '#3a3a3a', bx + ex, by + 0.2, bz, { sx: 0.06, sy: 0.4, sz: 0.06 });
      const colors = b.flowers || ['#ff8fb1'];
      for (let k = 0; k < 2; k++) {
        const bike = makeVehicle('bike', colors[Math.abs(cx * 7 + cz * 3 + k) % colors.length]);
        bike.position.set(bx - 0.45 + k * 0.9, by, bz - 0.05);
        bike.rotation.y = Math.PI / 2;
        group.add(bike);
      }
      this.racks.set(`${cx},${cz}`, new THREE.Vector3(ox + bx, this.base + by, oz + bz));
      col(bx - 1, bz - 0.4, bx + 1, bz + 0.4);
    }

    const sg = solidB.build();
    if (sg) group.add(new THREE.Mesh(sg, this.solidMat));
    const gg = glowB.build();
    if (gg) group.add(new THREE.Mesh(gg, this.glowMat));
    const shg = shadowB.build();
    if (shg) {
      const m = new THREE.Mesh(shg, this.shadowMat);
      m.renderOrder = 1;
      group.add(m);
    }
    this.root.add(group);
    const cell = { x: cx, z: cz, band, kind, storeys: shape.storeys, grown: shape.grown, chimney: pl.chimney, top: pl.top + pad, group, colliders, grass: null, extras: null };
    this.applySummary(cell);
    return cell;
  }

  bikeNear(pos) {
    for (const p of this.racks.values()) if (Math.hypot(p.x - pos.x, p.z - pos.z) < 2.4) return p;
    return null;
  }

  grassGeo(band) {
    if (!this.grassGeos.has(band)) {
      const g = makeGrassGeometry(this.bands[band]);
      g.userData.shared = true;
      this.grassGeos.set(band, g);
    }
    return this.grassGeos.get(band);
  }

  setGrass(cell, on) {
    if (on && !cell.grass) {
      const b = this.bands[cell.band];
      const r = new Rand(hash32(this.seed, cell.x, cell.z, 3));
      const n = Math.floor(2600 * (b.grass.density ?? 1));
      const mesh = new THREE.InstancedMesh(this.grassGeo(cell.band), this.grassMat, Math.max(1, n));
      const m = new THREE.Matrix4(), q = new THREE.Quaternion(), e = new THREE.Euler();
      const H = CELL / 2;
      const clearX = cell.kind === 'cave' ? 6.5 : cell.kind === 'arch' ? 2 : HOUSE_W / 2 + 0.4;
      let placed = 0;
      for (let i = 0; i < n * 2 && placed < n; i++) {
        const x = r.range(-H, H), z = r.range(-H, H);
        if (Math.abs(x) < clearX && z < HOUSE_D / 2 + 1.4 && z > -7) continue;
        if (Math.abs(x) > H - 1.6 || Math.abs(z) > H - 1.6) continue;
        if (Math.abs(x) < 0.9 && z > 0) continue;
        const gh = (b.grass.height ?? 0.5) * r.range(0.45, 1.25);
        m.compose(new THREE.Vector3(x, this.heightAt(cell.x * CELL + x, cell.z * CELL + z), z), q.setFromEuler(e.set(0, r.range(0, Math.PI), 0)), new THREE.Vector3(r.range(0.7, 1.2), gh, 1));
        mesh.setMatrixAt(placed++, m);
      }
      mesh.count = placed;
      mesh.position.set(cell.x * CELL, 0, cell.z * CELL);
      mesh.computeBoundingSphere();
      cell.grass = mesh;
      this.root.add(mesh);
    } else if (!on && cell.grass) {
      this.root.remove(cell.grass);
      cell.grass.dispose();
      cell.grass = null;
    }
  }

  setSummary(s) {
    const k = `${s.x},${s.z}`;
    this.summaries.set(k, s);
    const cell = this.cells.get(k);
    if (!cell) return;
    const sh = this.shapeOf(s.x, s.z, this.bands[cell.band]);
    if (sh.kind === cell.kind && sh.storeys === cell.storeys && sh.grown === cell.grown) return this.applySummary(cell);
    // It grew: build the address again, taller.
    const grass = !!cell.grass;
    this.root.remove(cell.group);
    if (cell.grass) {
      this.root.remove(cell.grass);
      cell.grass.dispose();
    }
    disposeTree(cell.group);
    this.racks.delete(k);
    const fresh = this.buildCell(s.x, s.z);
    this.cells.set(k, fresh);
    this.setGrass(fresh, grass);
  }

  // Claimed places get a name; places being built get chimney smoke.
  applySummary(cell) {
    const s = this.summaries.get(`${cell.x},${cell.z}`);
    if (cell.extras) {
      cell.group.remove(cell.extras);
      disposeTree(cell.extras);
      cell.extras = null;
    }
    if (!s) return;
    const extras = new THREE.Group();
    const pad = this.padAt(cell.x, cell.z);
    if (s.claimed) {
      const l = label(s.claimed, { size: 0.42 });
      l.position.set(0, cell.top + 0.6, HOUSE_D / 2 + 0.6);
      extras.add(l);
    }
    if (s.building || s.growth > 0) {
      const puffs = [];
      const mat = new THREE.SpriteMaterial({ map: softDot(), color: s.building ? '#ffffff' : this.bands[cell.band].lamp, transparent: true, opacity: 0.5, depthWrite: false });
      for (let i = 0; i < (s.building ? 6 : 2); i++) {
        const p = new THREE.Sprite(mat);
        p.userData.phase = i / (s.building ? 6 : 2);
        extras.add(p);
        puffs.push(p);
      }
      const top = cell.chimney ? pad + cell.chimney[1] : cell.top;
      extras.userData.update = (t) => {
        for (const p of puffs) {
          const k = (t * 0.12 + p.userData.phase) % 1;
          p.position.set((cell.chimney ? cell.chimney[0] : 0) + Math.sin(k * 6) * 0.3, top + k * 4, cell.chimney ? cell.chimney[2] : 0);
          const sc = 0.6 + k * 1.8;
          p.scale.set(sc, sc, 1);
        }
      };
    }
    cell.group.add(extras);
    cell.extras = extras;
  }

  animate(t) {
    for (const cell of this.cells.values()) cell.extras?.userData.update?.(t);
  }

  collidersNear(x, z, out) {
    const cx = Math.round(x / CELL), cz = Math.round(z / CELL);
    for (let dx = -1; dx <= 1; dx++)
      for (let dz = -1; dz <= 1; dz++) {
        const c = this.cells.get(`${cx + dx},${cz + dz}`);
        if (c) out.push(...c.colliders);
      }
    return out;
  }

  dispose() {
    for (const cell of this.cells.values()) {
      if (cell.grass) cell.grass.dispose();
      disposeTree(cell.group);
    }
    this.cells.clear();
    for (const p of this.places.values()) {
      p.solid?.dispose();
      p.glow?.dispose();
    }
    for (const g of this.grassGeos.values()) g.dispose();
    this.root.clear();
  }
}
