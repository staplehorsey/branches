// The one forced primitive: the player. Same body, same controls, in every
// world, so crossing a door never changes who you are.
import * as THREE from 'three';
import { label } from './geo.js';

const RADIUS = 0.3;
const EYE = 1.6;

export function makeAvatar(color, name) {
  const g = new THREE.Group();
  const c = new THREE.Color(color);
  const bodyMat = new THREE.MeshStandardMaterial({ color: c, roughness: 0.6 });
  const body = new THREE.Mesh(new THREE.CapsuleGeometry(0.28, 0.7, 6, 14), bodyMat);
  body.position.y = 0.66;
  const head = new THREE.Mesh(new THREE.SphereGeometry(0.24, 18, 14), new THREE.MeshStandardMaterial({ color: c.clone().lerp(new THREE.Color('#ffffff'), 0.45), roughness: 0.5 }));
  head.position.y = 1.42;
  const eyeMat = new THREE.MeshBasicMaterial({ color: '#1a1a22' });
  for (const sx of [-1, 1]) {
    const eye = new THREE.Mesh(new THREE.SphereGeometry(0.035, 8, 6), eyeMat);
    eye.position.set(sx * 0.08, 1.45, -0.21);
    g.add(eye);
  }
  const halo = new THREE.Mesh(new THREE.TorusGeometry(0.2, 0.025, 8, 28), new THREE.MeshBasicMaterial({ color: c.clone().lerp(new THREE.Color('#ffffff'), 0.6) }));
  halo.rotation.x = Math.PI / 2;
  halo.position.y = 1.82;
  g.add(body, head, halo);
  if (name) {
    const tag = label(name, { size: 0.24 });
    tag.position.y = 2.15;
    g.add(tag);
    g.userData.tag = tag;
  }
  g.userData.halo = halo;
  g.userData.body = body;
  return g;
}

const _ray = new THREE.Vector3();

// Slab test of a ray against an AABB collider; returns hit distance or Infinity.
function rayBox(o, d, b, maxT) {
  let t0 = 0, t1 = maxT;
  for (const [oo, dd, lo, hi] of [[o.x, d.x, b.x0, b.x1], [o.y, d.y, b.y0, b.y1], [o.z, d.z, b.z0, b.z1]]) {
    if (Math.abs(dd) < 1e-8) {
      if (oo < lo || oo > hi) return Infinity;
      continue;
    }
    let a = (lo - oo) / dd, c = (hi - oo) / dd;
    if (a > c) [a, c] = [c, a];
    t0 = Math.max(t0, a);
    t1 = Math.min(t1, c);
    if (t0 > t1) return Infinity;
  }
  return t0;
}

export class Player {
  constructor(dom, camera) {
    this.dom = dom;
    this.camera = camera;
    this.pos = new THREE.Vector3();
    this.vel = new THREE.Vector3();
    this.yaw = 0;
    this.pitch = 0;
    this.floorY = 0;
    this.ceiling = Infinity;
    this.grounded = true;
    this.thirdPerson = false;
    this.keys = new Set();
    this.enabled = false;
    this.boom = 3.4;
    this.avatar = null;
    this.walkPhase = 0;

    dom.addEventListener('click', () => {
      if (this.enabled && document.pointerLockElement !== dom) dom.requestPointerLock?.();
    });
    document.addEventListener('mousemove', (e) => {
      if (document.pointerLockElement !== dom) return;
      this.yaw -= e.movementX * 0.0022;
      this.pitch = Math.max(-1.45, Math.min(1.45, this.pitch - e.movementY * 0.0022));
    });
    addEventListener('keydown', (e) => {
      if (isTyping(e)) return;
      this.keys.add(e.code);
    });
    addEventListener('keyup', (e) => this.keys.delete(e.code));
    addEventListener('blur', () => this.keys.clear());
    this.touch = { x: 0, y: 0, active: false };
  }

  setAvatar(color, name) {
    if (this.avatar) this.avatar.removeFromParent();
    this.avatar = makeAvatar(color, null);
    this.avatar.visible = this.thirdPerson;
    return this.avatar;
  }

  get eye() {
    return new THREE.Vector3(this.pos.x, this.pos.y + EYE, this.pos.z);
  }

  place(x, y, z, yaw) {
    this.pos.set(x, y, z);
    this.vel.set(0, 0, 0);
    this.floorY = y;
    if (yaw !== undefined) this.yaw = yaw;
    this.pitch = 0;
  }

  translate(offset) {
    this.pos.add(offset);
    this.floorY += offset.y;
  }

  update(dt, colliders) {
    const k = this.keys;
    let fx = 0, fz = 0;
    if (this.enabled) {
      if (k.has('KeyW') || k.has('ArrowUp')) fz += 1;
      if (k.has('KeyS') || k.has('ArrowDown')) fz -= 1;
      if (k.has('KeyA') || k.has('ArrowLeft')) fx -= 1;
      if (k.has('KeyD') || k.has('ArrowRight')) fx += 1;
      if (this.touch.active) {
        fx += this.touch.x;
        fz += this.touch.y;
      }
    }
    const len = Math.hypot(fx, fz);
    if (len > 1) {
      fx /= len;
      fz /= len;
    }
    const speed = k.has('ShiftLeft') || k.has('ShiftRight') ? 7.5 : 3.8;
    const sin = Math.sin(this.yaw), cos = Math.cos(this.yaw);
    const wx = (-sin * fz + cos * fx) * speed;
    const wz = (-cos * fz - sin * fx) * speed;
    const a = 1 - Math.exp(-dt * 10);
    this.vel.x += (wx - this.vel.x) * a;
    this.vel.z += (wz - this.vel.z) * a;
    this.vel.y -= 22 * dt;
    if (this.enabled && k.has('Space') && this.grounded) {
      this.vel.y = 7;
      this.grounded = false;
    }
    this.pos.addScaledVector(this.vel, dt);
    if (this.pos.y <= this.floorY) {
      this.pos.y = this.floorY;
      this.vel.y = 0;
      this.grounded = true;
    }
    // Two passes of push-out against nearby boxes.
    for (let pass = 0; pass < 2; pass++) {
      for (const b of colliders) {
        if (this.pos.y + 1.7 < b.y0 || this.pos.y > b.y1) continue;
        const cx = Math.max(b.x0, Math.min(this.pos.x, b.x1));
        const cz = Math.max(b.z0, Math.min(this.pos.z, b.z1));
        let dx = this.pos.x - cx, dz = this.pos.z - cz;
        const d2 = dx * dx + dz * dz;
        if (d2 >= RADIUS * RADIUS) continue;
        if (d2 < 1e-10) {
          // Centre inside the box: leave by the nearest face.
          const opts = [[this.pos.x - b.x0, -1, 0], [b.x1 - this.pos.x, 1, 0], [this.pos.z - b.z0, 0, -1], [b.z1 - this.pos.z, 0, 1]].sort((p, q) => p[0] - q[0]);
          const [dist, ox, oz] = opts[0];
          this.pos.x += ox * (dist + RADIUS);
          this.pos.z += oz * (dist + RADIUS);
          continue;
        }
        const d = Math.sqrt(d2);
        this.pos.x = cx + (dx / d) * RADIUS;
        this.pos.z = cz + (dz / d) * RADIUS;
      }
    }
    const moving = Math.hypot(this.vel.x, this.vel.z);
    this.walkPhase += moving * dt * 2.2;
  }

  updateCamera(colliders) {
    const cam = this.camera;
    cam.rotation.set(this.pitch, this.yaw, 0, 'YXZ');
    const head = new THREE.Vector3(this.pos.x, this.pos.y + EYE, this.pos.z);
    if (this.avatar) {
      this.avatar.visible = this.thirdPerson;
      this.avatar.position.copy(this.pos);
      this.avatar.rotation.y = this.yaw;
      this.avatar.userData.body.position.y = 0.66 + Math.abs(Math.sin(this.walkPhase)) * 0.05;
    }
    if (!this.thirdPerson) {
      head.y += Math.sin(this.walkPhase * 2) * 0.025;
      cam.position.copy(head);
      return;
    }
    const target = head.clone().add(new THREE.Vector3(0, 0.15, 0));
    const back = new THREE.Vector3(0, 0, 1).applyEuler(cam.rotation).normalize();
    let dist = this.boom;
    for (const b of colliders) {
      const t = rayBox(target, back, b, dist);
      if (t < dist) dist = Math.max(0.35, t - 0.25);
    }
    cam.position.copy(target).addScaledVector(back, dist);
    if (cam.position.y > this.ceiling - 0.25) cam.position.y = this.ceiling - 0.25;
    if (cam.position.y < this.floorY + 0.25) cam.position.y = this.floorY + 0.25;
  }
}

export function isTyping(e) {
  const t = e.target;
  return t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable);
}
