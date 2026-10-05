// Geometry and texture helpers shared by every generator.
import * as THREE from 'three';

const _c = new THREE.Color();

// Collects coloured primitives and merges them into one mesh, so a whole
// garden or house costs one draw call.
export class Batch {
  constructor() {
    this.parts = [];
  }
  add(geometry, matrix, color) {
    const g = geometry.index ? geometry.toNonIndexed() : geometry.clone();
    g.applyMatrix4(matrix);
    _c.set(color);
    const n = g.attributes.position.count;
    const col = new Float32Array(n * 3);
    for (let i = 0; i < n; i++) {
      col[i * 3] = _c.r;
      col[i * 3 + 1] = _c.g;
      col[i * 3 + 2] = _c.b;
    }
    g.setAttribute('color', new THREE.BufferAttribute(col, 3));
    this.parts.push(g);
    return this;
  }
  // Convenience: place a geometry at a position with optional rotation/scale.
  put(geometry, color, x, y, z, { rx = 0, ry = 0, rz = 0, sx = 1, sy = 1, sz = 1 } = {}) {
    const m = new THREE.Matrix4().compose(
      new THREE.Vector3(x, y, z),
      new THREE.Quaternion().setFromEuler(new THREE.Euler(rx, ry, rz)),
      new THREE.Vector3(sx, sy, sz),
    );
    return this.add(geometry, m, color);
  }
  build() {
    if (!this.parts.length) return null;
    let total = 0;
    for (const p of this.parts) total += p.attributes.position.count;
    const pos = new Float32Array(total * 3);
    const nor = new Float32Array(total * 3);
    const col = new Float32Array(total * 3);
    let o = 0;
    for (const p of this.parts) {
      pos.set(p.attributes.position.array, o * 3);
      if (p.attributes.normal) nor.set(p.attributes.normal.array, o * 3);
      col.set(p.attributes.color.array, o * 3);
      o += p.attributes.position.count;
      p.dispose();
    }
    const g = new THREE.BufferGeometry();
    g.setAttribute('position', new THREE.BufferAttribute(pos, 3));
    g.setAttribute('normal', new THREE.BufferAttribute(nor, 3));
    g.setAttribute('color', new THREE.BufferAttribute(col, 3));
    g.computeBoundingSphere();
    this.parts = [];
    return g;
  }
}

// Shared unit primitives.
export const G = {
  box: new THREE.BoxGeometry(1, 1, 1),
  cyl: new THREE.CylinderGeometry(0.5, 0.5, 1, 10),
  cyl6: new THREE.CylinderGeometry(0.5, 0.5, 1, 6),
  cone: new THREE.ConeGeometry(0.5, 1, 9),
  ball: new THREE.IcosahedronGeometry(0.5, 1),
  ballSmooth: new THREE.SphereGeometry(0.5, 16, 12),
  dome: new THREE.SphereGeometry(0.5, 16, 8, 0, Math.PI * 2, 0, Math.PI / 2),
  octa: new THREE.OctahedronGeometry(0.5, 0),
  torus: new THREE.TorusGeometry(0.5, 0.06, 8, 32),
  plane: new THREE.PlaneGeometry(1, 1),
};

// Canvas-backed textures, generated rather than downloaded.
export function canvasTexture(size, draw, { repeat = true } = {}) {
  const c = document.createElement('canvas');
  c.width = c.height = size;
  const ctx = c.getContext('2d');
  draw(ctx, size);
  const t = new THREE.CanvasTexture(c);
  t.colorSpace = THREE.SRGBColorSpace;
  if (repeat) t.wrapS = t.wrapT = THREE.RepeatWrapping;
  t.anisotropy = 4;
  return t;
}

export function softDot() {
  return canvasTexture(
    64,
    (ctx, s) => {
      const g = ctx.createRadialGradient(s / 2, s / 2, 0, s / 2, s / 2, s / 2);
      g.addColorStop(0, 'rgba(255,255,255,1)');
      g.addColorStop(0.35, 'rgba(255,255,255,0.55)');
      g.addColorStop(1, 'rgba(255,255,255,0)');
      ctx.fillStyle = g;
      ctx.fillRect(0, 0, s, s);
    },
    { repeat: false },
  );
}

// A text label that always faces the camera.
export function label(text, { color = '#ffffff', bg = 'rgba(20,20,30,0.45)', size = 0.32, font = 'italic 500 44px Georgia, serif' } = {}) {
  const c = document.createElement('canvas');
  const ctx = c.getContext('2d');
  ctx.font = font;
  const w = Math.ceil(ctx.measureText(text).width) + 48;
  c.width = w;
  c.height = 72;
  ctx.font = font;
  if (bg) {
    ctx.fillStyle = bg;
    const r = 30;
    ctx.beginPath();
    ctx.roundRect(0, 4, w, 64, r);
    ctx.fill();
  }
  ctx.fillStyle = color;
  ctx.textBaseline = 'middle';
  ctx.fillText(text, 24, 38);
  const t = new THREE.CanvasTexture(c);
  t.colorSpace = THREE.SRGBColorSpace;
  const m = new THREE.SpriteMaterial({ map: t, transparent: true, depthWrite: false, fog: false });
  const s = new THREE.Sprite(m);
  s.scale.set((size * w) / 72, size, 1);
  return s;
}

export function disposeTree(obj) {
  obj.traverse((o) => {
    if (o.geometry && !o.geometry.userData.shared) o.geometry.dispose();
    const mats = Array.isArray(o.material) ? o.material : o.material ? [o.material] : [];
    for (const m of mats) {
      if (m.userData?.shared) continue;
      for (const k of ['map', 'emissiveMap']) if (m[k] && !m[k].userData?.shared) m[k].dispose();
      m.dispose();
    }
  });
}
for (const g of Object.values(G)) g.userData.shared = true;

// Axis-aligned box collider. y0/y1 bound it vertically.
export function aabb(x0, z0, x1, z1, y0 = -1e4, y1 = 1e4) {
  return { x0: Math.min(x0, x1), x1: Math.max(x0, x1), z0: Math.min(z0, z1), z1: Math.max(z0, z1), y0, y1 };
}
