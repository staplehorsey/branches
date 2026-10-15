// A local copy of /kit/space.js with three small additions: read() takes an
// on-close callback, setLabel() changes a thing's label, and the camera lives
// in the scene so things can be parented to it (held objects).
import * as THREE from '/vendor/three.module.js';

export function createSpace({ eye = 1.6, speed = 3, fov = 70, background = '#000000', reach = 2.6 } = {}) {
  const renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
  renderer.setSize(innerWidth, innerHeight);
  document.body.style.cssText += ';margin:0;overflow:hidden;font-family:Georgia,serif;background:#000';
  document.body.appendChild(renderer.domElement);
  const scene = new THREE.Scene();
  scene.background = new THREE.Color(background);
  const camera = new THREE.PerspectiveCamera(fov, innerWidth / innerHeight, 0.05, 2000);
  scene.add(camera);
  const player = { position: new THREE.Vector3(0, 0, 0), yaw: 0, pitch: 0, eye, speed, enabled: true };
  const walls = [];
  const frames = [];
  const keys = new Set();
  const things = new Map();
  const ui = (css, html = '') => {
    const d = document.createElement('div');
    d.style.cssText = 'position:fixed;pointer-events:none;color:#fff;text-shadow:0 1px 4px rgba(0,0,0,.7);' + css;
    d.innerHTML = html;
    document.body.appendChild(d);
    return d;
  };
  const cross = ui('left:50%;top:50%;width:6px;height:6px;margin:-3px;border-radius:50%;background:rgba(255,255,255,.8)');
  const hint = ui('left:0;right:0;bottom:22%;text-align:center;font-size:17px');
  const bag = ui('left:14px;bottom:14px;font-size:14px;max-width:40vw');
  const panel = ui('left:50%;top:50%;transform:translate(-50%,-50%);max-width:min(560px,86vw);max-height:70vh;overflow:auto;padding:22px 26px;border-radius:14px;background:rgba(250,246,238,.97);color:#222;text-shadow:none;font-size:17px;line-height:1.55;white-space:pre-wrap;display:none;pointer-events:auto;z-index:50');

  const items = [...(window.branches?.carried || [])];
  const drawBag = () => (bag.innerHTML = items.length ? `<b>carrying</b><br>${items.map((i) => `· ${i}`).join('<br>')}` : '');
  const inventory = {
    items,
    add(i) {
      items.push(String(i));
      drawBag();
      window.branches?.carry(items);
    },
    remove(i) {
      const k = items.indexOf(String(i));
      if (k >= 0) items.splice(k, 1);
      drawBag();
      window.branches?.carry(items);
    },
    rename(a, b) {
      const k = items.indexOf(String(a));
      if (k >= 0) items[k] = String(b);
      drawBag();
      window.branches?.carry(items);
    },
    has: (i) => items.includes(String(i)),
  };
  drawBag();
  if (items.length) window.branches?.carry(items);

  let reading = false;
  let onClose = null;
  const read = (title, text, cb) => {
    panel.innerHTML = '';
    const h = document.createElement('div');
    h.style.cssText = 'font-size:20px;font-style:italic;margin-bottom:10px';
    h.textContent = title;
    const b = document.createElement('div');
    b.textContent = text;
    const f = document.createElement('div');
    f.style.cssText = 'margin-top:14px;font-size:13px;opacity:.6';
    f.textContent = 'E or click to close';
    panel.append(h, b, f);
    panel.style.display = 'block';
    reading = true;
    onClose = cb || null;
  };
  const closeRead = () => {
    if (!reading) return;
    panel.style.display = 'none';
    reading = false;
    const cb = onClose;
    onClose = null;
    if (cb) cb();
  };
  panel.addEventListener('click', closeRead);

  const ray = new THREE.Raycaster();
  let looking = null;
  const look = () => {
    ray.setFromCamera(new THREE.Vector2(0, 0), camera);
    ray.far = reach;
    const hits = ray.intersectObjects([...things.keys()].filter((o) => o.visible !== false), true);
    for (const h of hits) {
      let o = h.object;
      while (o && !things.has(o)) o = o.parent;
      if (o) return o;
    }
    return null;
  };

  addEventListener('resize', () => {
    renderer.setSize(innerWidth, innerHeight);
    camera.aspect = innerWidth / innerHeight;
    camera.updateProjectionMatrix();
  });
  addEventListener('keydown', (e) => {
    keys.add(e.code);
    if (e.code === 'KeyE') {
      if (reading) return closeRead();
      if (looking) {
        const t = things.get(looking);
        const out = t.use?.(looking);
        if (typeof out === 'string' && things.get(looking)) things.get(looking).label = out;
      }
    }
  });
  addEventListener('keyup', (e) => keys.delete(e.code));
  addEventListener('blur', () => keys.clear());
  renderer.domElement.addEventListener('click', () => renderer.domElement.requestPointerLock?.());
  addEventListener('mousemove', (e) => {
    if (document.pointerLockElement !== renderer.domElement) return;
    player.yaw -= e.movementX * 0.0022;
    player.pitch = Math.max(-1.45, Math.min(1.45, player.pitch - e.movementY * 0.0022));
  });
  const blocked = (x, z) => walls.some((w) => x > w.x0 - 0.25 && x < w.x1 + 0.25 && z > w.z0 - 0.25 && z < w.z1 + 0.25);
  let last = performance.now();
  const t0 = last;
  renderer.setAnimationLoop(() => {
    const now = performance.now();
    const dt = Math.min(0.05, (now - last) / 1000);
    last = now;
    if (player.enabled && !reading) {
      const f = (keys.has('KeyW') || keys.has('ArrowUp') ? 1 : 0) - (keys.has('KeyS') || keys.has('ArrowDown') ? 1 : 0);
      const r = (keys.has('KeyD') || keys.has('ArrowRight') ? 1 : 0) - (keys.has('KeyA') || keys.has('ArrowLeft') ? 1 : 0);
      if (f || r) {
        const v = (player.speed * (keys.has('ShiftLeft') || keys.has('ShiftRight') ? 2.2 : 1) * dt) / Math.hypot(f, r);
        const dx = (-Math.sin(player.yaw) * f + Math.cos(player.yaw) * r) * v;
        const dz = (-Math.cos(player.yaw) * f - Math.sin(player.yaw) * r) * v;
        if (!blocked(player.position.x + dx, player.position.z)) player.position.x += dx;
        if (!blocked(player.position.x, player.position.z + dz)) player.position.z += dz;
      }
    }
    try {
      for (const fn of frames) fn(dt, (now - t0) / 1000);
    } catch (err) {
      console.error(err);
    }
    camera.position.set(player.position.x, player.position.y + player.eye, player.position.z);
    camera.rotation.set(player.pitch, player.yaw, 0, 'YXZ');
    camera.updateMatrixWorld(true);
    looking = reading ? null : look();
    hint.textContent = looking ? `E · ${things.get(looking).label}` : '';
    cross.style.background = looking ? '#ffd27a' : 'rgba(255,255,255,.8)';
    renderer.render(scene, camera);
  });

  function room({ x = 0, z = 0, w = 10, d = 8, h = 3, wall = '#cfc6b8', floor = '#5a4636', ceiling = '#e8e2d8', doors = [], thickness = 0.2 } = {}) {
    const g = new THREE.Group();
    const mat = (c) => (c instanceof THREE.Material ? c : new THREE.MeshStandardMaterial({ color: c, roughness: 0.85 }));
    const slab = (sx, sy, sz, px, py, pz, m) => {
      const b = new THREE.Mesh(new THREE.BoxGeometry(sx, sy, sz), mat(m));
      b.position.set(px, py, pz);
      g.add(b);
      return b;
    };
    slab(w, 0.1, d, x, -0.05, z, floor);
    if (ceiling) slab(w, 0.1, d, x, h + 0.05, z, ceiling);
    const side = (name, len, cx, cz, alongX) => {
      const gaps = doors.filter((o) => o.side === name).map((o) => [o.at - (o.w || 1.2) / 2, o.at + (o.w || 1.2) / 2]).sort((a, b) => a[0] - b[0]);
      let start = -len / 2;
      for (const [a, b] of [...gaps, [len / 2, len / 2]]) {
        if (a - start > 0.01) {
          const mid = (start + a) / 2, l = a - start;
          if (alongX) {
            slab(l, h, thickness, cx + mid, h / 2, cz, wall);
            walls.push({ x0: cx + start, z0: cz - thickness / 2, x1: cx + a, z1: cz + thickness / 2 });
          } else {
            slab(thickness, h, l, cx, h / 2, cz + mid, wall);
            walls.push({ x0: cx - thickness / 2, z0: cz + start, x1: cx + thickness / 2, z1: cz + a });
          }
        }
        start = b;
      }
    };
    side('n', w, x, z - d / 2, true);
    side('s', w, x, z + d / 2, true);
    side('w', d, x - w / 2, z, false);
    side('e', d, x + w / 2, z, false);
    scene.add(g);
    return g;
  }

  return {
    THREE,
    renderer,
    scene,
    camera,
    player,
    walls,
    keys,
    inventory,
    room,
    read,
    isReading: () => reading,
    onFrame: (fn) => frames.push(fn),
    thing(obj, { label = 'use', use } = {}) {
      things.set(obj, { label, use });
      return obj;
    },
    setLabel(obj, label) {
      const t = things.get(obj);
      if (t) t.label = label;
    },
    forget: (obj) => things.delete(obj),
    say: (t, s) => window.branches?.say(t, s),
    onward: () => window.branches?.onward(),
  };
}
