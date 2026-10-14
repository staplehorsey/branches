// An optional helper for spaces: a three.js renderer, scene and camera
// that fill the window, and first-person walking (WASD/arrows, mouse look
// with pointer lock, shift to go faster). Use any of it, or none.
//
//   import * as THREE from '/vendor/three.module.js';
//   import { createSpace } from '/kit/space.js';
//   const s = createSpace({ eye: 1.6, speed: 3 });
//   s.scene.add(...);
//   s.walls.push({ x0, z0, x1, z1 });          // boxes you can't walk into
//   s.onFrame((dt, t) => { ... });              // every frame
//   s.player.position                           // where you are (feet)
//   branches.onward()                           // when they find the way on
import * as THREE from '/vendor/three.module.js';

export function createSpace({ eye = 1.6, speed = 3, fov = 70, background = '#000000', bounds = null } = {}) {
  const renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
  renderer.setSize(innerWidth, innerHeight);
  document.body.style.margin = '0';
  document.body.style.overflow = 'hidden';
  document.body.appendChild(renderer.domElement);
  const scene = new THREE.Scene();
  scene.background = new THREE.Color(background);
  const camera = new THREE.PerspectiveCamera(fov, innerWidth / innerHeight, 0.05, 2000);
  const player = { position: new THREE.Vector3(0, 0, 0), yaw: 0, pitch: 0, eye, speed, enabled: true };
  const walls = [];
  const frames = [];
  const keys = new Set();
  addEventListener('resize', () => {
    renderer.setSize(innerWidth, innerHeight);
    camera.aspect = innerWidth / innerHeight;
    camera.updateProjectionMatrix();
  });
  addEventListener('keydown', (e) => keys.add(e.code));
  addEventListener('keyup', (e) => keys.delete(e.code));
  addEventListener('blur', () => keys.clear());
  renderer.domElement.addEventListener('click', () => renderer.domElement.requestPointerLock?.());
  addEventListener('mousemove', (e) => {
    if (document.pointerLockElement !== renderer.domElement) return;
    player.yaw -= e.movementX * 0.0022;
    player.pitch = Math.max(-1.45, Math.min(1.45, player.pitch - e.movementY * 0.0022));
  });
  const blocked = (x, z) => walls.some((w) => x > w.x0 - 0.25 && x < w.x1 + 0.25 && z > w.z0 - 0.25 && z < w.z1 + 0.25) || (bounds && (x < bounds.x0 || x > bounds.x1 || z < bounds.z0 || z > bounds.z1));
  let last = performance.now();
  const t0 = last;
  renderer.setAnimationLoop(() => {
    const now = performance.now();
    const dt = Math.min(0.05, (now - last) / 1000);
    last = now;
    if (player.enabled) {
      const f = (keys.has('KeyW') || keys.has('ArrowUp') ? 1 : 0) - (keys.has('KeyS') || keys.has('ArrowDown') ? 1 : 0);
      const r = (keys.has('KeyD') || keys.has('ArrowRight') ? 1 : 0) - (keys.has('KeyA') || keys.has('ArrowLeft') ? 1 : 0);
      if (f || r) {
        const v = player.speed * (keys.has('ShiftLeft') || keys.has('ShiftRight') ? 2.2 : 1) * dt / Math.hypot(f, r);
        const dx = (-Math.sin(player.yaw) * f + Math.cos(player.yaw) * r) * v;
        const dz = (-Math.cos(player.yaw) * f - Math.sin(player.yaw) * r) * v;
        if (!blocked(player.position.x + dx, player.position.z)) player.position.x += dx;
        if (!blocked(player.position.x, player.position.z + dz)) player.position.z += dz;
      }
    }
    camera.position.set(player.position.x, player.position.y + player.eye, player.position.z);
    camera.rotation.set(player.pitch, player.yaw, 0, 'YXZ');
    for (const fn of frames) fn(dt, (now - t0) / 1000);
    renderer.render(scene, camera);
  });
  return { THREE, renderer, scene, camera, player, walls, keys, onFrame: (fn) => frames.push(fn) };
}
