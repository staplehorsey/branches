// Interior themes. Every house has the same shell; its theme decides the
// palette, surfaces, light and the three tags its entry hall is built from.
import * as THREE from 'three';
import { canvasTexture } from './geo.js';
import { Rand } from './rng.js';

// Each pattern paints one square tile; `scale` is the tile's size in metres.
const PATTERNS = {
  plain(ctx, s, [a, b]) {
    ctx.fillStyle = a;
    ctx.fillRect(0, 0, s, s);
    noise(ctx, s, b || a, 0.05);
  },
  tiles(ctx, s, [a, b]) {
    ctx.fillStyle = a;
    ctx.fillRect(0, 0, s, s);
    noise(ctx, s, b, 0.03);
    ctx.strokeStyle = b;
    ctx.lineWidth = s * 0.025;
    const n = 4;
    for (let i = 0; i <= n; i++) {
      const p = (i * s) / n;
      line(ctx, p, 0, p, s);
      line(ctx, 0, p, s, p);
    }
  },
  checker(ctx, s, [a, b]) {
    const n = 2;
    for (let i = 0; i < n; i++)
      for (let j = 0; j < n; j++) {
        ctx.fillStyle = (i + j) % 2 ? b : a;
        ctx.fillRect((i * s) / n, (j * s) / n, s / n, s / n);
      }
    noise(ctx, s, '#000', 0.02);
  },
  stripes(ctx, s, [a, b]) {
    ctx.fillStyle = a;
    ctx.fillRect(0, 0, s, s);
    ctx.fillStyle = b;
    for (let i = 0; i < 4; i++) ctx.fillRect((i * s) / 4, 0, s / 10, s);
    ctx.globalAlpha = 0.5;
    for (let i = 0; i < 4; i++) ctx.fillRect((i * s) / 4 + s / 7, 0, s / 40, s);
    ctx.globalAlpha = 1;
  },
  damask(ctx, s, [a, b]) {
    ctx.fillStyle = a;
    ctx.fillRect(0, 0, s, s);
    ctx.fillStyle = b;
    const m = (x, y, r) => {
      ctx.beginPath();
      ctx.moveTo(x, y - r);
      ctx.quadraticCurveTo(x + r * 0.8, y, x, y + r);
      ctx.quadraticCurveTo(x - r * 0.8, y, x, y - r);
      ctx.fill();
    };
    for (const [x, y] of [[0.25, 0.25], [0.75, 0.75], [0.75, 0.25], [0.25, 0.75]]) m(x * s, y * s, s * ((x + y) % 1 ? 0.06 : 0.12));
  },
  wood(ctx, s, [a, b]) {
    const r = new Rand(7);
    const n = 6;
    for (let i = 0; i < n; i++) {
      ctx.fillStyle = shade(a, r.range(-0.08, 0.08));
      ctx.fillRect(0, (i * s) / n, s, s / n);
      ctx.fillStyle = b;
      ctx.fillRect(0, (i * s) / n, s, s * 0.006);
      ctx.fillRect(r.range(0, s), (i * s) / n, s * 0.006, s / n);
    }
    noise(ctx, s, b, 0.04);
  },
  carpet(ctx, s, [a, b, c]) {
    ctx.fillStyle = a;
    ctx.fillRect(0, 0, s, s);
    const r = new Rand(11);
    ctx.lineWidth = s * 0.012;
    for (let i = 0; i < 14; i++) {
      ctx.strokeStyle = i % 2 ? b : c || b;
      ctx.beginPath();
      const x = r.range(0, s), y = r.range(0, s);
      ctx.moveTo(x, y);
      for (let k = 0; k < 3; k++) ctx.lineTo(x + r.range(-s / 6, s / 6), y + r.range(-s / 6, s / 6));
      ctx.stroke();
    }
    noise(ctx, s, '#000', 0.05);
  },
  tatami(ctx, s, [a, b]) {
    ctx.fillStyle = a;
    ctx.fillRect(0, 0, s, s);
    ctx.globalAlpha = 0.18;
    ctx.fillStyle = b;
    for (let i = 0; i < s; i += 4) ctx.fillRect(0, i, s, 1);
    ctx.globalAlpha = 1;
    ctx.fillStyle = b;
    ctx.fillRect(0, 0, s, s * 0.03);
    ctx.fillRect(0, s / 2, s, s * 0.03);
    ctx.fillRect(s / 2, 0, s * 0.03, s / 2);
  },
  shoji(ctx, s, [a, b]) {
    ctx.fillStyle = a;
    ctx.fillRect(0, 0, s, s);
    noise(ctx, s, '#a08060', 0.03);
    ctx.fillStyle = b;
    for (let i = 0; i <= 3; i++) ctx.fillRect((i * s) / 3 - s * 0.01, 0, s * 0.02, s);
    for (let i = 0; i <= 4; i++) ctx.fillRect(0, (i * s) / 4 - s * 0.01, s, s * 0.02);
  },
  stone(ctx, s, [a, b]) {
    const r = new Rand(3);
    ctx.fillStyle = b;
    ctx.fillRect(0, 0, s, s);
    const rows = 4;
    for (let j = 0; j < rows; j++) {
      let x = j % 2 ? -s / 6 : 0;
      while (x < s) {
        const w = r.range(s / 5, s / 3);
        ctx.fillStyle = shade(a, r.range(-0.06, 0.06));
        ctx.fillRect(x + 2, (j * s) / rows + 2, w - 4, s / rows - 4);
        x += w;
      }
    }
    noise(ctx, s, '#000', 0.04);
  },
  moss(ctx, s, [a, b]) {
    ctx.fillStyle = a;
    ctx.fillRect(0, 0, s, s);
    const r = new Rand(5);
    for (let i = 0; i < 700; i++) {
      ctx.fillStyle = r.chance(0.5) ? b : shade(a, -0.1);
      ctx.globalAlpha = 0.35;
      ctx.beginPath();
      ctx.arc(r.range(0, s), r.range(0, s), r.range(2, 9), 0, 7);
      ctx.fill();
    }
    ctx.globalAlpha = 1;
  },
  velvet(ctx, s, [a, b]) {
    const g = ctx.createLinearGradient(0, 0, s, 0);
    for (let i = 0; i <= 8; i++) g.addColorStop(i / 8, i % 2 ? a : b);
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, s, s);
  },
  sand(ctx, s, [a, b]) {
    ctx.fillStyle = a;
    ctx.fillRect(0, 0, s, s);
    noise(ctx, s, b, 0.12);
  },
};

function line(ctx, x0, y0, x1, y1) {
  ctx.beginPath();
  ctx.moveTo(x0, y0);
  ctx.lineTo(x1, y1);
  ctx.stroke();
}

function noise(ctx, s, color, alpha) {
  const r = new Rand(Math.floor(s * 13 + alpha * 1000));
  ctx.fillStyle = color;
  ctx.globalAlpha = alpha;
  for (let i = 0; i < 900; i++) ctx.fillRect(r.range(0, s), r.range(0, s), 2, 2);
  ctx.globalAlpha = 1;
}

export function shade(hex, amt) {
  const c = new THREE.Color(hex);
  const hsl = {};
  c.getHSL(hsl);
  c.setHSL(hsl.h, hsl.s, Math.min(1, Math.max(0, hsl.l + amt)));
  return '#' + c.getHexString();
}

const cache = new Map();
// A material for a surface spec: { p: pattern, c: [colors], scale: metres }.
export function surface(spec, { emissive = 0 } = {}) {
  const key = JSON.stringify(spec) + emissive;
  if (cache.has(key)) return cache.get(key);
  const map = canvasTexture(256, (ctx, s) => PATTERNS[spec.p](ctx, s, spec.c));
  map.repeat.set(1 / (spec.scale || 1), 1 / (spec.scale || 1));
  map.userData.shared = true;
  const m = new THREE.MeshStandardMaterial({ map, roughness: spec.rough ?? 0.85, metalness: 0 });
  if (emissive) {
    m.emissive = new THREE.Color(spec.c[0]);
    m.emissiveMap = map;
    m.emissiveIntensity = emissive;
  }
  m.userData.shared = true;
  cache.set(key, m);
  return m;
}

export const THEMES = {
  poolrooms: {
    name: 'Poolrooms',
    wall: { p: 'tiles', c: ['#eef7f6', '#c3dfe0'], scale: 1, rough: 0.35 },
    floor: { p: 'tiles', c: ['#e3f2f3', '#b2d6d9'], scale: 1, rough: 0.3 },
    ceiling: { p: 'tiles', c: ['#f6fcfc', '#d8ecec'], scale: 1 },
    trim: '#ffffff',
    accent: ['#3fc9d6', '#a6f0ea', '#ffffff', '#8fe3ff'],
    glow: '#f0fffb',
    water: '#45d3dc',
    env: { sky: '#f2ffff', ground: '#9ad8d8', hemi: 1.5, key: '#ffffff', keyI: 1.0, fog: '#d6f1f1', near: 5, far: 46 },
    height: 3.4,
    tags: ['water', 'light', 'stone'],
  },
  'moss-library': {
    name: 'Moss Library',
    wall: { p: 'wood', c: ['#4a3426', '#2e2018'], scale: 2.2 },
    floor: { p: 'moss', c: ['#3e5a2a', '#6b8a3a'], scale: 2.5 },
    ceiling: { p: 'wood', c: ['#2f221a', '#1f1610'], scale: 2.5 },
    trim: '#c9a66b',
    accent: ['#7a2e2e', '#2e4a7a', '#c9a66b', '#3f6b3a', '#d9c7a0'],
    glow: '#ffcf8a',
    water: '#3a6a5a',
    env: { sky: '#ffe2b5', ground: '#2a3a1e', hemi: 1.0, key: '#ffd9a0', keyI: 0.7, fog: '#2b2a1f', near: 3, far: 32 },
    height: 4.4,
    tags: ['books', 'plants', 'cozy'],
  },
  'cloud-nursery': {
    name: 'Cloud Nursery',
    wall: { p: 'stripes', c: ['#ffe6ee', '#ffd2e1'], scale: 1.6 },
    floor: { p: 'plain', c: ['#cfe6ff', '#bcd8f6'], scale: 2 },
    ceiling: { p: 'plain', c: ['#dcecff', '#c8defa'], scale: 2 },
    trim: '#ffffff',
    accent: ['#ffb3c9', '#b3d7ff', '#fff3b0', '#ffffff', '#d7c2ff'],
    glow: '#fff6e0',
    water: '#9fd6ff',
    env: { sky: '#ffffff', ground: '#ffd6e6', hemi: 1.6, key: '#fff4f0', keyI: 0.8, fog: '#f3e8f4', near: 6, far: 46 },
    height: 3.6,
    tags: ['sky', 'cozy', 'light'],
  },
  'sunset-terrarium': {
    name: 'Sunset Terrarium',
    wall: { p: 'damask', c: ['#f6c39b', '#e8a477'], scale: 1.2 },
    floor: { p: 'sand', c: ['#e6c08f', '#c99a62'], scale: 2 },
    ceiling: { p: 'plain', c: ['#ffd7b0', '#f2c08f'], scale: 2 },
    trim: '#b35f3a',
    accent: ['#e8713c', '#7fae5a', '#ffd166', '#c2553a', '#5a8a4a'],
    glow: '#ffb070',
    water: '#5ab8b0',
    env: { sky: '#ffcf9e', ground: '#a2603a', hemi: 1.3, key: '#ff9a5c', keyI: 1.2, fog: '#f2b48a', near: 5, far: 40 },
    height: 4.0,
    tags: ['plants', 'light', 'stone'],
  },
  'night-aquarium': {
    name: 'Night Aquarium',
    wall: { p: 'tiles', c: ['#10294a', '#1b4370'], scale: 1, rough: 0.4 },
    floor: { p: 'plain', c: ['#0a1a30', '#0f2440'], scale: 2, rough: 0.3 },
    ceiling: { p: 'plain', c: ['#081426', '#0b1a30'], scale: 2 },
    trim: '#2a5a8a',
    accent: ['#5ff2ff', '#ff7be5', '#7b8cff', '#9bffd6'],
    glow: '#5ff2ff',
    water: '#1f6fd0',
    env: { sky: '#3a6aa8', ground: '#05101e', hemi: 0.8, key: '#6fb8ff', keyI: 0.5, fog: '#06182c', near: 3, far: 30 },
    height: 4.2,
    tags: ['water', 'creatures', 'neon'],
  },
  'vapor-mall': {
    name: 'Vapor Mall',
    wall: { p: 'stripes', c: ['#ffd8ef', '#c8f4f1'], scale: 2 },
    floor: { p: 'checker', c: ['#fbf6ff', '#c9b8ff'], scale: 2, rough: 0.25 },
    ceiling: { p: 'plain', c: ['#fff0fa', '#f6e0f2'], scale: 2 },
    trim: '#7ef7e9',
    accent: ['#ff71ce', '#01cdfe', '#05ffa1', '#b967ff', '#fffb96'],
    glow: '#ff9ee8',
    water: '#6ff0ff',
    env: { sky: '#ffe6fb', ground: '#b9a6ff', hemi: 1.4, key: '#ffffff', keyI: 0.8, fog: '#f4d9f6', near: 6, far: 52 },
    height: 4.8,
    tags: ['neon', 'plants', 'art'],
  },
  'tea-garden': {
    name: 'Tea Garden',
    wall: { p: 'shoji', c: ['#f4ecd8', '#7a5a3e'], scale: 1.8 },
    floor: { p: 'tatami', c: ['#d6d19a', '#6a6a32'], scale: 1.8 },
    ceiling: { p: 'wood', c: ['#c9a77a', '#8a6a4a'], scale: 2 },
    trim: '#5a3d2a',
    accent: ['#c23b22', '#3f6b3a', '#e8d8b0', '#2b2b2b', '#e88aa0'],
    glow: '#ffe6b0',
    water: '#4a8a7a',
    env: { sky: '#fff4dc', ground: '#6b7a3a', hemi: 1.2, key: '#fff0d0', keyI: 0.9, fog: '#e9e0c6', near: 5, far: 38 },
    height: 3.2,
    tags: ['water', 'plants', 'stone'],
  },
  'fern-cathedral': {
    name: 'Fern Cathedral',
    wall: { p: 'stone', c: ['#bdbcaa', '#8f8e7c'], scale: 2.4 },
    floor: { p: 'stone', c: ['#9a9d88', '#6e705f'], scale: 2.4 },
    ceiling: { p: 'plain', c: ['#c9c9b8', '#b0b09e'], scale: 3 },
    trim: '#e8e4d0',
    accent: ['#3f7a3a', '#5aa04a', '#e8c86a', '#7a5aa8', '#4aa0c8'],
    glow: '#fff2c4',
    water: '#5a9a8a',
    env: { sky: '#f0f6e8', ground: '#3a5a2a', hemi: 1.2, key: '#fff6d8', keyI: 1.1, fog: '#cfd6c0', near: 6, far: 52 },
    height: 7.5,
    tags: ['plants', 'stone', 'light'],
  },
  'arcade-after-hours': {
    name: 'Arcade After Hours',
    wall: { p: 'plain', c: ['#1d1236', '#2a1a4a'], scale: 2 },
    floor: { p: 'carpet', c: ['#160c2a', '#ff3fa4', '#3ff0ff'], scale: 2 },
    ceiling: { p: 'plain', c: ['#0c0818', '#140c24'], scale: 2 },
    trim: '#ff3fa4',
    accent: ['#ff3fa4', '#3ff0ff', '#ffe83f', '#7a3fff', '#3fff8a'],
    glow: '#ff6ad5',
    water: '#3f6fff',
    env: { sky: '#4a2a7a', ground: '#100818', hemi: 0.7, key: '#ff8ad8', keyI: 0.4, fog: '#120a22', near: 3, far: 30 },
    height: 3.6,
    tags: ['neon', 'music', 'light'],
  },
  'snowglobe-den': {
    name: 'Snowglobe Den',
    wall: { p: 'wood', c: ['#eadfce', '#cdbb9f'], scale: 2 },
    floor: { p: 'wood', c: ['#c49a6c', '#8a6440'], scale: 2 },
    ceiling: { p: 'wood', c: ['#f4ece0', '#d8ccb8'], scale: 2 },
    trim: '#a8483a',
    accent: ['#c8423a', '#2f6b4a', '#ffffff', '#d9b36a', '#7aa8d8'],
    glow: '#ffc98a',
    water: '#bfe6ff',
    env: { sky: '#ffffff', ground: '#c8d8e8', hemi: 1.3, key: '#fff0e0', keyI: 0.8, fog: '#eef2f6', near: 5, far: 40 },
    height: 3.8,
    tags: ['snow', 'cozy', 'light'],
  },
  'citrus-kitchen': {
    name: 'Citrus Kitchen',
    wall: { p: 'tiles', c: ['#fff6c8', '#f0d44a'], scale: 0.8, rough: 0.4 },
    floor: { p: 'checker', c: ['#ffffff', '#ffd23f'], scale: 1.2, rough: 0.4 },
    ceiling: { p: 'plain', c: ['#fffbe8', '#fff2c8'], scale: 2 },
    trim: '#3f9a4a',
    accent: ['#ffd23f', '#ff8c1a', '#3f9a4a', '#ffffff', '#ff5a5a'],
    glow: '#fff2b0',
    water: '#7fd8ff',
    env: { sky: '#fffbe6', ground: '#e8c84a', hemi: 1.6, key: '#ffffff', keyI: 1.2, fog: '#fff4d0', near: 6, far: 46 },
    height: 3.4,
    tags: ['plants', 'light', 'cozy'],
  },
  'velvet-theatre': {
    name: 'Velvet Theatre',
    wall: { p: 'velvet', c: ['#5a0f1e', '#7e1c2e'], scale: 2 },
    floor: { p: 'carpet', c: ['#3a0a14', '#d9a63a', '#7a1a2a'], scale: 2.5 },
    ceiling: { p: 'plain', c: ['#2a0810', '#3a0c18'], scale: 2 },
    trim: '#d9a63a',
    accent: ['#d9a63a', '#a8142a', '#f2e6c8', '#2a0810', '#ffd27a'],
    glow: '#ffd27a',
    water: '#6a1a3a',
    env: { sky: '#ffb08a', ground: '#2a0810', hemi: 0.9, key: '#ffcf8a', keyI: 0.6, fog: '#2a0a12', near: 3, far: 32 },
    height: 5.0,
    tags: ['music', 'art', 'cozy'],
  },
};

export function theme(id) {
  return THEMES[id] || THEMES.poolrooms;
}
