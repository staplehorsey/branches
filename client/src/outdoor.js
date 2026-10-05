// The outside of a liminal-houses@1 world: an endless grid of identical
// houses in an overgrown suburb, streamed in around the player.
import * as THREE from 'three';
import { Batch, G, aabb, canvasTexture, softDot, label, disposeTree } from './geo.js';
import { Rand, hash32 } from './rng.js';
import { CELL, HOUSE_W, HOUSE_D, WALL_H, DOOR_W, DOOR_H } from './layout.js';
import { shade } from './themes.js';

export const time = { value: 0 };

const RADIUS = 4; // cells of props around the player
const GRASS_RADIUS = 2;

function gableRoof(b) {
  // Ridge along x. Built from triangles so it can be merged into the batch.
  const w = HOUSE_W / 2 + 0.45, eaveY = WALL_H, ridgeY = WALL_H + 2.1, d = HOUSE_D / 2 + 0.5;
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

// One shell geometry per world; every house uses it.
function buildHouse(biome) {
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
  gableRoof(solid);
  solid.put(G.box, shade(h.roof, -0.15), 2.6, WALL_H + 2.0, -1.2, { sx: 0.8, sy: 1.6, sz: 0.8 });
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
  // Mailbox at the end of the walk.
  solid.put(G.box, trim, 1.4, 0.55, CELL / 2 - 2.1, { sx: 0.1, sy: 1.1, sz: 0.1 });
  solid.put(G.box, h.door, 1.4, 1.15, CELL / 2 - 2.1, { sx: 0.3, sy: 0.3, sz: 0.5 });
  const colliders = [
    [-W, -D, W, -D + T],
    [-W, -D, -W + T, D],
    [W - T, -D, W, D],
    [-W, D - T - 0.05, -DOOR_W / 2, D + 0.1],
    [DOOR_W / 2, D - T - 0.05, W, D + 0.1],
    [1.25, CELL / 2 - 2.35, 1.55, CELL / 2 - 1.85],
  ];
  return { solid: solid.build(), glow: glow.build(), colliders };
}

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

export class Outdoor {
  constructor(root, manifest) {
    this.root = root;
    this.biome = manifest.generator.params;
    this.seed = manifest.seed >>> 0;
    this.cells = new Map();
    this.summaries = new Map();
    this.house = buildHouse(this.biome);
    this.solidMat = new THREE.MeshLambertMaterial({ vertexColors: true, flatShading: true });
    this.glowMat = new THREE.MeshBasicMaterial({ vertexColors: true });
    this.shadowMat = new THREE.MeshBasicMaterial({ color: '#000000', transparent: true, opacity: 0.18, depthWrite: false });
    this.grassGeo = makeGrassGeometry(this.biome);
    this.grassMat = makeGrassMaterial();
    this.waterMat = new THREE.MeshStandardMaterial({ color: this.biome.water, emissive: new THREE.Color(this.biome.water), emissiveIntensity: this.biome.night ? 0.5 : 0.15, roughness: 0.05, metalness: 0.2 });
    for (const m of [this.solidMat, this.glowMat, this.shadowMat, this.grassMat, this.waterMat]) m.userData.shared = true;
    this.queue = [];
    this.center = null;
    this.buildGround();
    this.buildSky();
    this.buildMotes();
  }

  buildGround() {
    const b = this.biome;
    const tex = canvasTexture(256, (ctx, s) => {
      ctx.fillStyle = b.ground;
      ctx.fillRect(0, 0, s, s);
      const r = new Rand(1);
      for (let i = 0; i < 2500; i++) {
        ctx.fillStyle = r.chance(0.5) ? shade(b.ground, 0.06) : shade(b.ground, -0.06);
        ctx.globalAlpha = 0.5;
        ctx.fillRect(r.range(0, s), r.range(0, s), 3, 3);
      }
    });
    tex.repeat.set(120, 120);
    this.ground = new THREE.Mesh(new THREE.PlaneGeometry(CELL * 30, CELL * 30), new THREE.MeshLambertMaterial({ map: tex }));
    this.ground.rotation.x = -Math.PI / 2;
    this.root.add(this.ground);
  }

  buildSky() {
    const s = this.biome.sky;
    const sun = this.biome.sun;
    const dir = new THREE.Vector3(Math.cos(sun.azimuth) * Math.cos(sun.elevation), Math.sin(sun.elevation), Math.sin(sun.azimuth) * Math.cos(sun.elevation));
    this.sunDir = dir;
    const mat = new THREE.ShaderMaterial({
      side: THREE.BackSide,
      depthWrite: false,
      fog: false,
      uniforms: {
        top: { value: new THREE.Color(s.top) },
        horizon: { value: new THREE.Color(s.horizon) },
        bottom: { value: new THREE.Color(s.bottom) },
        sunDir: { value: dir },
        sunColor: { value: new THREE.Color(sun.color) },
        night: { value: this.biome.night ? 1 : 0 },
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
    this.sky.position.copy(cam);
  }

  update(player, dt) {
    const cx = Math.round(player.x / CELL), cz = Math.round(player.z / CELL);
    this.ground.position.set(cx * CELL, 0, cz * CELL);
    const a = this.motes.geometry.attributes.position.array;
    const t = time.value;
    for (let i = 0; i < a.length; i += 3) {
      a[i] += Math.sin(t * 0.3 + i) * dt * 0.3;
      a[i + 1] += Math.sin(t * 0.5 + i * 0.7) * dt * 0.15;
      let dx = a[i] - player.x, dz = a[i + 2] - player.z;
      if (dx > 30) a[i] -= 60;
      if (dx < -30) a[i] += 60;
      if (dz > 30) a[i + 2] -= 60;
      if (dz < -30) a[i + 2] += 60;
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
          disposeTree(cell.group);
          this.cells.delete(k);
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

  buildCell(cx, cz) {
    const b = this.biome;
    const r = new Rand(hash32(this.seed, cx, cz, 77));
    const group = new THREE.Group();
    const ox = cx * CELL, oz = cz * CELL;
    group.position.set(ox, 0, oz);
    const solid = new Batch();
    const glow = new Batch();
    const shadow = new Batch();
    const colliders = [];
    const col = (x0, z0, x1, z1) => colliders.push(aabb(ox + x0, oz + z0, ox + x1, oz + z1, -5, 20));
    const H = CELL / 2;

    // House.
    const house = new THREE.Mesh(this.house.solid, this.solidMat);
    const houseGlow = new THREE.Mesh(this.house.glow, this.glowMat);
    house.geometry.userData.shared = houseGlow.geometry.userData.shared = true;
    group.add(house, houseGlow);
    for (const c of this.house.colliders) col(...c);
    shadow.put(G.box, '#000', 0, 0.02, 0, { sx: HOUSE_W + 1.2, sy: 0.01, sz: HOUSE_D + 1.2 });

    // Streets on this cell's +x and +z edges, and the front walk.
    solid.put(G.box, b.path, 0, 0.015, H, { sx: CELL - 3, sy: 0.03, sz: 3 });
    solid.put(G.box, shade(b.path, -0.03), H, 0.02, 0, { sx: 3, sy: 0.04, sz: CELL + 3 });
    solid.put(G.box, b.path, 0, 0.012, (HOUSE_D / 2 + H - 1.5) / 2 + 0.6, { sx: 1.4, sy: 0.024, sz: H - 1.5 - HOUSE_D / 2 - 1 });

    // Hedges along the side yards, with gaps to slip through.
    for (const sx of [-1, 1]) {
      const x = sx * 8.2;
      let z = -H + 1.8;
      while (z < H - 3) {
        const len = r.range(2.5, 6);
        const end = Math.min(z + len, H - 3);
        if (r.chance(0.82)) {
          const mid = (z + end) / 2, l = end - z;
          solid.put(G.box, b.hedge, x, 0.6, mid, { sx: 0.9, sy: 1.2, sz: l });
          for (let k = 0; k < l / 0.9; k++) solid.put(G.ball, shade(b.hedge, r.range(-0.04, 0.06)), x + r.range(-0.15, 0.15), 1.15, z + 0.45 + k * 0.9, { sx: 1, sy: 0.55, sz: 1 });
          if (r.chance(0.35)) flowers(solid, b, r, x - 0.5, z, x + 0.5, end, Math.floor(l * 2));
          col(x - 0.5, z, x + 0.5, end);
          shadow.put(G.box, '#000', x + 0.4, 0.02, mid, { sx: 1.6, sy: 0.01, sz: l + 0.4 });
        }
        z = end + r.range(1.2, 2.2);
      }
    }

    // Flower beds along the front of the house.
    for (const sx of [-1, 1]) {
      solid.put(G.box, shade(b.ground, -0.12), sx * 3, 0.06, HOUSE_D / 2 + 0.6, { sx: 3.6, sy: 0.12, sz: 0.9 });
      flowers(solid, b, r, sx * 3 - 1.7, HOUSE_D / 2 + 0.25, sx * 3 + 1.7, HOUSE_D / 2 + 0.95, 14);
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
      trees.push(tree(solid, glow, b, r, x, z, i === 3 ? 0.7 : 1));
    });
    for (const t of trees) {
      col(t.x - t.r, t.z - t.r, t.x + t.r, t.z + t.r);
      shadow.put(G.cyl, '#000', t.x + 0.6, 0.025, t.z + 0.3, { sx: 4.5, sy: 0.01, sz: 4.5 });
    }

    // A pond in some back yards.
    if (r.chance(b.ponds)) {
      const px = r.chance(0.5) ? -4.5 : 4.5, pz = -H + 4.2;
      const pond = new THREE.Mesh(G.cyl, this.waterMat);
      pond.scale.set(4.4, 0.05, 3.2);
      pond.position.set(px, 0.03, pz);
      group.add(pond);
      for (let k = 0; k < 14; k++) {
        const a = (k / 14) * Math.PI * 2;
        solid.put(G.ball, shade(b.path, r.range(-0.15, 0)), px + Math.cos(a) * 2.25, 0.06, pz + Math.sin(a) * 1.65, { sx: 0.6, sy: 0.25, sz: 0.5 });
      }
      for (let k = 0; k < 5; k++) solid.put(G.cyl, '#4f9a4a', px + r.range(-1.4, 1.4), 0.07, pz + r.range(-1, 1), { sx: 0.5, sy: 0.02, sz: 0.5 });
      col(px - 2.2, pz - 1.6, px + 2.2, pz + 1.6);
    }

    // A lamp post on the corner.
    const lx = H - 1.9, lz = H - 1.9;
    solid.put(G.cyl, '#3a3a3a', lx, 1.7, lz, { sx: 0.12, sy: 3.4, sz: 0.12 });
    solid.put(G.cyl, '#3a3a3a', lx, 3.45, lz, { sx: 0.5, sy: 0.1, sz: 0.5 });
    glow.put(G.ballSmooth, b.lamp, lx, 3.2, lz, { sx: 0.42, sy: 0.5, sz: 0.42 });
    col(lx - 0.12, lz - 0.12, lx + 0.12, lz + 0.12);
    if (b.night) glow.put(G.cyl, b.lamp, lx, 0.03, lz, { sx: 3.5, sy: 0.005, sz: 3.5 });

    const sg = solid.build();
    if (sg) group.add(new THREE.Mesh(sg, this.solidMat));
    const gg = glow.build();
    if (gg) group.add(new THREE.Mesh(gg, this.glowMat));
    const shg = shadow.build();
    if (shg) {
      const m = new THREE.Mesh(shg, this.shadowMat);
      m.renderOrder = 1;
      group.add(m);
    }
    this.root.add(group);
    const cell = { x: cx, z: cz, group, colliders, grass: null, extras: null };
    this.applySummary(cell);
    return cell;
  }

  setGrass(cell, on) {
    if (on && !cell.grass) {
      const b = this.biome;
      const r = new Rand(hash32(this.seed, cell.x, cell.z, 3));
      const n = Math.floor(2600 * b.grass.density);
      const mesh = new THREE.InstancedMesh(this.grassGeo, this.grassMat, n);
      mesh.geometry.userData.shared = true;
      const m = new THREE.Matrix4(), q = new THREE.Quaternion(), e = new THREE.Euler();
      const H = CELL / 2;
      let placed = 0;
      for (let i = 0; i < n * 2 && placed < n; i++) {
        const x = r.range(-H, H), z = r.range(-H, H);
        if (Math.abs(x) < HOUSE_W / 2 + 0.4 && Math.abs(z) < HOUSE_D / 2 + 1.4) continue;
        if (z > H - 1.6 || x > H - 1.6) continue;
        if (Math.abs(x) < 0.9 && z > 0) continue;
        const h = b.grass.height * r.range(0.45, 1.25);
        m.compose(new THREE.Vector3(x, 0, z), q.setFromEuler(e.set(0, r.range(0, Math.PI), 0)), new THREE.Vector3(r.range(0.7, 1.2), h, 1));
        mesh.setMatrixAt(placed++, m);
      }
      mesh.count = placed;
      mesh.position.set(cell.x * CELL, 0, cell.z * CELL);
      mesh.frustumCulled = false;
      mesh.computeBoundingSphere();
      mesh.frustumCulled = true;
      cell.grass = mesh;
      this.root.add(mesh);
    } else if (!on && cell.grass) {
      this.root.remove(cell.grass);
      cell.grass.dispose();
      cell.grass = null;
    }
  }

  setSummary(s) {
    this.summaries.set(`${s.x},${s.z}`, s);
    const cell = this.cells.get(`${s.x},${s.z}`);
    if (cell) this.applySummary(cell);
  }

  // Claimed houses get a name; houses being built get chimney smoke.
  applySummary(cell) {
    const s = this.summaries.get(`${cell.x},${cell.z}`);
    if (cell.extras) {
      cell.group.remove(cell.extras);
      disposeTree(cell.extras);
      cell.extras = null;
    }
    if (!s) return;
    const extras = new THREE.Group();
    if (s.claimed) {
      const l = label(s.claimed, { size: 0.42 });
      l.position.set(0, WALL_H + 2.9, HOUSE_D / 2 + 0.6);
      extras.add(l);
      const flag = new THREE.Mesh(G.box, new THREE.MeshLambertMaterial({ color: this.biome.house.door }));
      flag.scale.set(0.04, 0.25, 0.35);
      flag.position.set(1.55, 1.45, CELL / 2 - 2.1);
      extras.add(flag);
    }
    if (s.building || s.growth > 0) {
      const puffs = [];
      const mat = new THREE.SpriteMaterial({ map: softDot(), color: s.building ? '#ffffff' : this.biome.lamp, transparent: true, opacity: 0.5, depthWrite: false });
      for (let i = 0; i < (s.building ? 6 : 2); i++) {
        const p = new THREE.Sprite(mat);
        p.userData.phase = i / (s.building ? 6 : 2);
        extras.add(p);
        puffs.push(p);
      }
      extras.userData.update = (t) => {
        for (const p of puffs) {
          const k = (t * 0.12 + p.userData.phase) % 1;
          p.position.set(2.6 + Math.sin(k * 6) * 0.3, WALL_H + 2.9 + k * 4, -1.2);
          const sc = 0.6 + k * 1.8;
          p.scale.set(sc, sc, 1);
        }
        mat.opacity = 0.5;
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
    this.house.solid.dispose();
    this.house.glow.dispose();
    this.root.clear();
  }
}
