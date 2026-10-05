// Builds a house's interior from its room record: an entry hall the same
// size as the shell, then a chain of chambers the architect has grown,
// each further from the front door, ending at a misty frontier.
import * as THREE from 'three';
import { Batch, G, aabb, label } from './geo.js';
import { FEATURES } from './features.js';
import { theme as themeOf, surface, shade } from './themes.js';
import { Rand } from './rng.js';
import { HOUSE_W, HOUSE_D, DOOR_W, DOOR_H, ARCH_W, ARCH_H } from './layout.js';

const UP = new THREE.Vector3(0, 1, 0);
const solidMat = new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.75 });
const glowMat = new THREE.MeshBasicMaterial({ vertexColors: true });
solidMat.userData.shared = glowMat.userData.shared = true;

const BRANCH_W = 1.3;
const BRANCH_H = 2.3;

// Chamber rectangles, deterministic from the room's chamber seeds.
export function layoutChambers(room, th) {
  const out = [];
  out.push({ x0: -HOUSE_W / 2, x1: HOUSE_W / 2, z0: -HOUSE_D / 2, z1: HOUSE_D / 2, h: th.height, frontX: 0 });
  for (let i = 1; i < room.chambers.length; i++) {
    const prev = out[i - 1];
    const r = new Rand(room.chambers[i].seed);
    const archX = r.range(prev.x0 + 1.8, prev.x1 - 1.8);
    const W = r.range(8, 15);
    const D = r.range(8, 13);
    const tag = room.chambers[i].tag;
    const lift = tag === 'sky' || tag === 'stone' || tag === 'plants' ? 1.6 : 1.25;
    const H = Math.min(11, Math.max(th.height, r.range(th.height, th.height * lift)));
    const cx = archX + r.range(-(W / 2 - 1.8), W / 2 - 1.8);
    prev.backX = archX;
    out.push({ x0: cx - W / 2, x1: cx + W / 2, z1: prev.z0, z0: prev.z0 - D, h: H, frontX: archX });
  }
  const last = out[out.length - 1];
  last.backX = new Rand(room.seed ^ (room.chambers.length * 7919)).range(last.x0 + 1.8, last.x1 - 1.8);
  return out;
}

// A wall plane with door-shaped notches cut from its bottom edge.
function wallGeometry(length, height, holes) {
  const s = new THREE.Shape();
  s.moveTo(-length / 2, 0);
  for (const h of [...holes].sort((a, b) => a.u - b.u)) {
    s.lineTo(h.u - h.w / 2, 0);
    s.lineTo(h.u - h.w / 2, h.h);
    s.lineTo(h.u + h.w / 2, h.h);
    s.lineTo(h.u + h.w / 2, 0);
  }
  s.lineTo(length / 2, 0);
  s.lineTo(length / 2, height);
  s.lineTo(-length / 2, height);
  return new THREE.ShapeGeometry(s);
}

export function buildInterior({ room, origin, biome, frontier = true }) {
  const th = themeOf(room.theme);
  const group = new THREE.Group();
  group.position.set(origin.x, origin.y, origin.z);
  const solid = new Batch();
  const glow = new Batch();
  const colliders = [];
  const lights = [];
  const anims = [];
  const branchDoors = [];
  const chambers = layoutChambers(room, th);
  const wallMat = surface(th.wall);
  const floorMat = surface(th.floor);
  const ceilMat = surface(th.ceiling);

  const addCollider = (x0, z0, x1, z1) => colliders.push(aabb(origin.x + x0, origin.z + z0, origin.x + x1, origin.z + z1, origin.y - 1, origin.y + 12));

  function wall(cx, cz, nx, nz, length, height, holes) {
    const n = new THREE.Vector3(nx, 0, nz);
    const xAxis = new THREE.Vector3().crossVectors(UP, n);
    const local = holes.map((h) => ({ u: (h.x - cx) * xAxis.x + (h.z - cz) * xAxis.z, w: h.w, h: h.h }));
    const mesh = new THREE.Mesh(wallGeometry(length, height, local), wallMat);
    mesh.matrixAutoUpdate = false;
    mesh.matrix.makeBasis(xAxis, UP, n).setPosition(cx, 0, cz);
    group.add(mesh);
    // Baseboards and colliders between the holes.
    const sorted = [...local].sort((a, b) => a.u - b.u);
    let start = -length / 2;
    const segs = [];
    for (const h of sorted) {
      segs.push([start, h.u - h.w / 2]);
      start = h.u + h.w / 2;
    }
    segs.push([start, length / 2]);
    for (const [a, b] of segs) {
      if (b - a < 0.01) continue;
      const mid = (a + b) / 2;
      const px = cx + xAxis.x * mid, pz = cz + xAxis.z * mid;
      const len = b - a;
      const along = Math.abs(xAxis.x) > 0.5;
      solid.put(G.box, th.trim, px + nx * 0.02, 0.07, pz + nz * 0.02, { sx: along ? len : 0.04, sy: 0.14, sz: along ? 0.04 : len });
      const ax = cx + xAxis.x * a, az = cz + xAxis.z * a, bx = cx + xAxis.x * b, bz = cz + xAxis.z * b;
      // Thick behind the face (-n), thin in front of it.
      const back = 0.25, front = 0.02;
      addCollider(
        Math.min(ax, bx) - (nx > 0 ? back : nx < 0 ? front : 0),
        Math.min(az, bz) - (nz > 0 ? back : nz < 0 ? front : 0),
        Math.max(ax, bx) + (nx < 0 ? back : nx > 0 ? front : 0),
        Math.max(az, bz) + (nz < 0 ? back : nz > 0 ? front : 0),
      );
    }
    // Frames around the holes.
    for (const h of local) {
      const px = cx + xAxis.x * h.u, pz = cz + xAxis.z * h.u;
      const along = Math.abs(xAxis.x) > 0.5;
      const jx = along ? h.w / 2 + 0.06 : 0, jz = along ? 0 : h.w / 2 + 0.06;
      for (const sgn of [-1, 1]) solid.put(G.box, th.trim, px + jx * sgn, h.h / 2, pz + jz * sgn, { sx: along ? 0.12 : 0.18, sy: h.h, sz: along ? 0.18 : 0.12 });
      solid.put(G.box, th.trim, px, h.h + 0.06, pz, { sx: along ? h.w + 0.24 : 0.18, sy: 0.12, sz: along ? 0.18 : h.w + 0.24 });
    }
  }

  function slab(c, y, mat, down) {
    const w = c.x1 - c.x0, d = c.z1 - c.z0;
    const s = new THREE.Shape();
    s.moveTo(c.x0, -c.z1);
    s.lineTo(c.x1, -c.z1);
    s.lineTo(c.x1, -c.z0);
    s.lineTo(c.x0, -c.z0);
    const m = new THREE.Mesh(new THREE.ShapeGeometry(s), mat);
    m.rotation.x = down ? Math.PI / 2 : -Math.PI / 2;
    if (down) m.scale.y = -1;
    m.position.y = y;
    group.add(m);
    return { w, d };
  }

  const portalsBySlot = new Map();
  for (const p of room.portals || []) {
    if (!portalsBySlot.has(p.slot)) portalsBySlot.set(p.slot, []);
    portalsBySlot.get(p.slot).push(p);
  }

  chambers.forEach((c, i) => {
    const data = room.chambers[i];
    slab(c, 0, floorMat, false);
    slab(c, c.h, ceilMat, true);
    const midX = (c.x0 + c.x1) / 2, midZ = (c.z0 + c.z1) / 2;
    const frontHoles = i === 0 ? [{ x: 0, z: c.z1, w: DOOR_W, h: DOOR_H }] : [{ x: c.frontX, z: c.z1, w: ARCH_W, h: ARCH_H }];
    wall(midX, c.z1, 0, -1, c.x1 - c.x0, c.h, frontHoles);
    const isLast = i === chambers.length - 1;
    const backHoles = isLast && !frontier ? [] : [{ x: c.backX, z: c.z0, w: ARCH_W, h: ARCH_H }];
    wall(midX, c.z0, 0, 1, c.x1 - c.x0, c.h, backHoles);

    // Branch doors: portals to elsewhere in the graph, on the side walls.
    const doors = portalsBySlot.get(i) || [];
    const left = [], right = [];
    doors.slice(0, 6).forEach((p, k) => {
      const side = k % 2 === 0 ? left : right;
      const z = c.z1 - 2.2 - Math.floor(k / 2) * 2.6;
      if (z - BRANCH_W / 2 < c.z0 + 0.6) return;
      side.push({ p, z });
    });
    wall(c.x0, midZ, 1, 0, c.z1 - c.z0, c.h, left.map((d) => ({ x: c.x0, z: d.z, w: BRANCH_W, h: BRANCH_H })));
    wall(c.x1, midZ, -1, 0, c.z1 - c.z0, c.h, right.map((d) => ({ x: c.x1, z: d.z, w: BRANCH_W, h: BRANCH_H })));
    for (const [list, x, nx] of [[left, c.x0, 1], [right, c.x1, -1]]) {
      for (const d of list) {
        branchDoors.push({
          portal: d.p,
          local: new THREE.Vector3(x, BRANCH_H / 2, d.z),
          pos: new THREE.Vector3(origin.x + x, origin.y + BRANCH_H / 2, origin.z + d.z),
          normal: new THREE.Vector3(nx, 0, 0),
          w: BRANCH_W,
          h: BRANCH_H,
          chamber: i,
        });
        const shimmer = branchSurface(d.p.target);
        shimmer.scale.set(0.4, BRANCH_H, BRANCH_W);
        shimmer.position.set(x - nx * 0.2, BRANCH_H / 2, d.z);
        group.add(shimmer);
        branchDoors[branchDoors.length - 1].mesh = shimmer;
        anims.push((t) => (shimmer.material.uniforms.uTime.value = t));
        lights.push({ pos: new THREE.Vector3(origin.x + x + nx * 0.8, origin.y + 1.5, origin.z + d.z), color: shimmer.material.uniforms.a.value, intensity: 0.6 });
        const tag = label(`→ ${d.p.label}`, { size: 0.26 });
        tag.position.set(x + nx * 0.3, BRANCH_H + 0.45, d.z);
        group.add(tag);
      }
    }

    // Occupancy grid for placing features away from the walking path.
    const occ = makeGrid(c);
    occ.blockPath(i === 0 ? 0 : c.frontX, c.backX ?? midX);
    for (const d of left) occ.block(c.x0, d.z - 1, c.x0 + 2, d.z + 1);
    for (const d of right) occ.block(c.x1 - 2, d.z - 1, c.x1, d.z + 1);
    if (i === 0) {
      occ.block(1.6, 1.6, 3.6, 3.8); // guestbook
      occ.block(-HOUSE_W / 2, HOUSE_D / 2 - 1.5, HOUSE_W / 2, HOUSE_D / 2);
    }
    const rand = new Rand((data?.seed ?? room.seed) ^ 0x51ed);
    const ctx = {
      group,
      solid,
      glow,
      th,
      rand,
      h: c.h,
      x0: c.x0,
      x1: c.x1,
      z0: c.z0,
      z1: c.z1,
      spot: (kind, w, d, opts) => occ.spot(rand, kind, w, d, opts),
      collide: (s, w, d) => {
        const swap = Math.abs(Math.sin(s.ry || 0)) > 0.5;
        const hw = (swap ? d : w) / 2, hd = (swap ? w : d) / 2;
        addCollider(s.x - hw, s.z - hd, s.x + hw, s.z + hd);
      },
      light: (x, y, z, color, intensity = 1) => lights.push({ pos: new THREE.Vector3(origin.x + x, origin.y + y, origin.z + z), color: new THREE.Color(color), intensity }),
      anim: (fn) => anims.push(fn),
    };
    // Entry hall is dressed from the theme's own tags; later chambers from
    // what the architect built there.
    const tags = i === 0 ? th.tags.slice() : [];
    for (const f of room.features || []) if (f.chamber === i) tags.push(f.tag);
    tags.forEach((tag, k) => {
      const build = FEATURES[tag];
      if (!build) return;
      ctx.rand = new Rand(((data?.seed ?? 1) + k * 7907) ^ hashTag(tag));
      build(ctx);
    });
    // Soft ambient fill so every chamber has a lamp somewhere.
    ctx.light(midX, c.h - 0.5, midZ, th.glow, 0.5);
    // Ceiling glow panel, the classic liminal fluorescent.
    if (th.env.hemi > 1.1) glow.put(G.box, shade(th.glow, 0.05), midX, c.h - 0.02, midZ, { sx: Math.min(3, (c.x1 - c.x0) / 3), sy: 0.03, sz: Math.min(1.2, (c.z1 - c.z0) / 6) });
  });

  // The front windows, matching the shell's, glowing with outside light.
  const daylight = biome?.sky?.horizon || '#fff6e0';
  for (const wx of [-3, 3]) {
    glow.put(G.box, daylight, wx, 1.6, HOUSE_D / 2 - 0.02, { sx: 1.6, sy: 1.2, sz: 0.02 });
    solid.put(G.box, th.trim, wx, 1.6, HOUSE_D / 2 - 0.04, { sx: 0.06, sy: 1.2, sz: 0.03 });
    solid.put(G.box, th.trim, wx, 1.6, HOUSE_D / 2 - 0.04, { sx: 1.6, sy: 0.06, sz: 0.03 });
    solid.put(G.box, th.trim, wx, 0.98, HOUSE_D / 2 - 0.08, { sx: 1.8, sy: 0.06, sz: 0.14 });
  }

  // The guestbook: every room keeps a visitor log.
  const gb = new THREE.Vector3(2.6, 0, 2.6);
  solid.put(G.box, shade(th.trim, -0.25), gb.x, 0.5, gb.z, { sx: 0.5, sy: 1.0, sz: 0.4 });
  solid.put(G.box, shade(th.trim, -0.2), gb.x, 1.05, gb.z, { sx: 0.75, sy: 0.08, sz: 0.55, rx: -0.35 });
  glow.put(G.box, '#fff8e8', gb.x, 1.11, gb.z + 0.02, { sx: 0.6, sy: 0.02, sz: 0.4, rx: -0.35 });
  addCollider(gb.x - 0.4, gb.z - 0.35, gb.x + 0.4, gb.z + 0.35);
  const gbLabel = label('visitor log · E', { size: 0.2 });
  gbLabel.position.set(gb.x, 1.6, gb.z);
  group.add(gbLabel);
  lights.push({ pos: new THREE.Vector3(origin.x + gb.x, origin.y + 1.5, origin.z + gb.z), color: new THREE.Color('#fff2d0'), intensity: 0.5 });

  // The frontier: where the next chamber will be built.
  const last = chambers[chambers.length - 1];
  let frontierMist = null;
  if (frontier) {
    const fx = last.backX, fz = last.z0;
    const mistMat = new THREE.ShaderMaterial({
      transparent: true,
      depthWrite: false,
      uniforms: { uTime: { value: 0 }, uColor: { value: new THREE.Color(th.glow) }, uBuilding: { value: room.building ? 1 : 0 } },
      vertexShader: 'varying vec2 vUv; void main(){ vUv=uv; gl_Position=projectionMatrix*modelViewMatrix*vec4(position,1.0); }',
      fragmentShader: `uniform float uTime; uniform vec3 uColor; uniform float uBuilding; varying vec2 vUv;
        void main(){
          float n = sin(vUv.y*14.0 - uTime*1.3 + sin(vUv.x*9.0+uTime)*1.5)*0.5+0.5;
          float edge = smoothstep(0.0,0.25,vUv.x)*smoothstep(1.0,0.75,vUv.x)*smoothstep(1.0,0.7,vUv.y);
          float a = (0.55 + n*0.35) * edge;
          vec3 c = mix(uColor, vec3(1.0), 0.35 + n*0.3);
          if (uBuilding > 0.5) c = mix(c, vec3(1.0,0.85,0.5), 0.4 + 0.3*sin(uTime*4.0));
          gl_FragColor = vec4(c, a);
        }`,
    });
    frontierMist = new THREE.Mesh(G.plane, mistMat);
    frontierMist.scale.set(ARCH_W, ARCH_H, 1);
    frontierMist.position.set(fx, ARCH_H / 2, fz - 0.05);
    group.add(frontierMist);
    anims.push((t) => (mistMat.uniforms.uTime.value = t));
    addCollider(fx - ARCH_W / 2, fz - 0.3, fx + ARCH_W / 2, fz + 0.05);
    const text = room.building ? 'under construction…' : 'the architect is listening';
    const fl = label(text, { size: 0.22 });
    fl.position.set(fx, ARCH_H + 0.5, fz + 0.2);
    group.add(fl);
    if (room.building) {
      for (let k = 0; k < 4; k++) {
        solid.put(G.box, '#c8a050', fx - ARCH_W / 2 + 0.2 + k * 0.6, ARCH_H / 2 + 0.2, fz + 0.4, { sx: 0.06, sy: ARCH_H + 0.4, sz: 0.06 });
        solid.put(G.box, '#c8a050', fx, 0.6 + k * 0.6, fz + 0.4, { sx: ARCH_W + 0.4, sy: 0.06, sz: 0.06 });
      }
    }
    lights.push({ pos: new THREE.Vector3(origin.x + fx, origin.y + 1.6, origin.z + fz + 0.8), color: new THREE.Color(th.glow), intensity: 0.7 });
  }

  const sg = solid.build();
  if (sg) group.add(new THREE.Mesh(sg, solidMat));
  const gg = glow.build();
  if (gg) group.add(new THREE.Mesh(gg, glowMat));

  return {
    group,
    theme: th,
    colliders,
    lights,
    anims,
    branchDoors,
    chambers: chambers.map((c, i) => ({ ...c, name: room.chambers[i]?.name || '' })),
    guestbook: new THREE.Vector3(origin.x + gb.x, origin.y + 1, origin.z + gb.z),
    chamberAt(x, z) {
      const lx = x - origin.x, lz = z - origin.z;
      for (let i = chambers.length - 1; i >= 0; i--) {
        const c = chambers[i];
        if (lx >= c.x0 - 0.2 && lx <= c.x1 + 0.2 && lz >= c.z0 && lz <= c.z1 + 0.3) return i;
      }
      return 0;
    },
  };
}

// The surface of a door to another world: a slow, bright swirl.
function branchSurface(target) {
  const h = hashTag(target);
  const a = new THREE.Color().setHSL(((h >>> 0) % 360) / 360, 0.75, 0.7);
  const b = new THREE.Color().setHSL(((h >>> 9) % 360) / 360, 0.6, 0.85);
  const mat = new THREE.ShaderMaterial({
    side: THREE.DoubleSide,
    uniforms: { uTime: { value: 0 }, a: { value: a }, b: { value: b } },
    vertexShader: 'varying vec3 vP; void main(){ vP = position; gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0); }',
    fragmentShader: `uniform float uTime; uniform vec3 a, b; varying vec3 vP;
      void main(){
        vec2 p = vP.zy * vec2(1.0, 2.0);
        float r = length(p);
        float ang = atan(p.y, p.x);
        float s = sin(ang * 3.0 + r * 9.0 - uTime * 1.5) * 0.5 + 0.5;
        vec3 c = mix(a, b, s) + vec3(1.0) * smoothstep(0.35, 0.0, r) * 0.6;
        gl_FragColor = vec4(c, 1.0);
        #include <colorspace_fragment>
      }`,
  });
  return new THREE.Mesh(G.box, mat);
}

// Once the destination's look is known, the door takes on its sky.
export function tintBranch(door, sky) {
  const u = door.mesh?.material.uniforms;
  if (!u || !sky) return;
  u.a.value.set(sky.horizon);
  u.b.value.set(sky.top);
}

function hashTag(tag) {
  let h = 0;
  for (let i = 0; i < tag.length; i++) h = (h * 31 + tag.charCodeAt(i)) | 0;
  return h;
}

// Coarse occupancy grid over one chamber.
function makeGrid(c) {
  const step = 0.5;
  const nx = Math.max(1, Math.ceil((c.x1 - c.x0) / step));
  const nz = Math.max(1, Math.ceil((c.z1 - c.z0) / step));
  const cells = new Uint8Array(nx * nz);
  const idx = (x, z) => {
    const i = Math.floor((x - c.x0) / step), j = Math.floor((z - c.z0) / step);
    return i < 0 || j < 0 || i >= nx || j >= nz ? -1 : j * nx + i;
  };
  function each(x0, z0, x1, z1, fn) {
    for (let x = x0; x < x1; x += step / 2)
      for (let z = z0; z < z1; z += step / 2) {
        const k = idx(x, z);
        if (k >= 0 && fn(k) === false) return false;
      }
    return true;
  }
  const free = (x0, z0, x1, z1) => x0 >= c.x0 && x1 <= c.x1 && z0 >= c.z0 && z1 <= c.z1 && each(x0, z0, x1, z1, (k) => cells[k] === 0);
  const block = (x0, z0, x1, z1) => each(x0, z0, x1, z1, (k) => void (cells[k] = 1));
  return {
    block,
    blockPath(fx, bx) {
      const mz = (c.z0 + c.z1) / 2;
      const hw = 1.3;
      block(fx - hw, mz, fx + hw, c.z1);
      block(Math.min(fx, bx) - hw, mz - hw, Math.max(fx, bx) + hw, mz + hw);
      block(bx - hw, c.z0, bx + hw, mz);
    },
    spot(r, kind, w, d, { soft = false, center = false } = {}) {
      for (let attempt = 0; attempt < 30; attempt++) {
        let x, z, ry;
        if (kind === 'wall') {
          const side = r.int(0, 2);
          if (side === 0) {
            ry = Math.PI / 2;
            x = c.x0 + d / 2 + 0.05;
            z = r.range(c.z0 + w / 2 + 0.3, c.z1 - w / 2 - 0.3);
          } else if (side === 1) {
            ry = -Math.PI / 2;
            x = c.x1 - d / 2 - 0.05;
            z = r.range(c.z0 + w / 2 + 0.3, c.z1 - w / 2 - 0.3);
          } else {
            ry = 0;
            z = c.z0 + d / 2 + 0.05;
            x = r.range(c.x0 + w / 2 + 0.3, c.x1 - w / 2 - 0.3);
          }
        } else if (kind === 'corner') {
          const sx = r.chance(0.5) ? 1 : -1, sz = r.chance(0.5) ? 1 : -1;
          x = sx > 0 ? c.x1 - w / 2 - 0.15 : c.x0 + w / 2 + 0.15;
          z = sz > 0 ? c.z1 - d / 2 - 0.15 : c.z0 + d / 2 + 0.15;
          ry = sx > 0 ? -Math.PI / 2 : Math.PI / 2;
        } else {
          const m = center ? 1.2 : 0.4;
          x = r.range(c.x0 + w / 2 + m, c.x1 - w / 2 - m);
          z = r.range(c.z0 + d / 2 + m, c.z1 - d / 2 - m);
          ry = r.pick([0, Math.PI / 2, Math.PI, -Math.PI / 2]);
        }
        if (!isFinite(x) || !isFinite(z)) return null;
        if (soft) return { x, z, ry };
        const swap = Math.abs(Math.sin(ry)) > 0.5;
        const hw = (swap ? d : w) / 2, hd = (swap ? w : d) / 2;
        if (free(x - hw, z - hd, x + hw, z + hd)) {
          block(x - hw, z - hd, x + hw, z + hd);
          return { x, z, ry };
        }
      }
      return null;
    },
  };
}
