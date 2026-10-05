// Everyone else who showed up.
import * as THREE from 'three';
import { makeAvatar } from './player.js';
import { label, disposeTree } from './geo.js';

export class Others {
  constructor(root, selfId) {
    this.root = root;
    this.selfId = selfId;
    this.map = new Map();
    this.offsetY = 0;
  }
  upsert(p) {
    if (p.id === this.selfId) return;
    let o = this.map.get(p.id);
    if (!o) {
      const g = makeAvatar(p.color || '#ffffff', p.name || 'wanderer');
      this.root.add(g);
      o = { g, target: new THREE.Vector3(), ry: 0, name: p.name, color: p.color, room: null, bubble: null, bubbleUntil: 0, seen: false };
      this.map.set(p.id, o);
    }
    if (p.p) {
      o.target.set(p.p[0], p.p[1] + this.offsetY, p.p[2]);
      o.ry = p.ry || 0;
      if (!o.seen) o.g.position.copy(o.target);
      o.seen = true;
    }
    o.room = p.room ?? null;
    return o;
  }
  remove(id) {
    const o = this.map.get(id);
    if (!o) return;
    this.root.remove(o.g);
    disposeTree(o.g);
    this.map.delete(id);
  }
  sync(list) {
    const ids = new Set();
    for (const p of list) {
      ids.add(p.id);
      this.upsert(p);
    }
    for (const id of [...this.map.keys()]) if (!ids.has(id)) this.remove(id);
  }
  say(id, text) {
    const o = this.map.get(id);
    if (!o) return;
    if (o.bubble) {
      o.g.remove(o.bubble);
      disposeTree(o.bubble);
    }
    o.bubble = label(text.length > 48 ? text.slice(0, 47) + '…' : text, { size: 0.22, bg: 'rgba(255,255,255,0.85)', color: '#222', font: '500 40px system-ui, sans-serif' });
    o.bubble.position.y = 2.5;
    o.g.add(o.bubble);
    o.bubbleUntil = performance.now() + 7000;
  }
  update(dt, t) {
    const a = 1 - Math.exp(-dt * 8);
    for (const o of this.map.values()) {
      const before = o.g.position.clone();
      o.g.position.lerp(o.target, a);
      let d = o.ry - o.g.rotation.y;
      d = Math.atan2(Math.sin(d), Math.cos(d));
      o.g.rotation.y += d * a;
      const moving = before.distanceTo(o.g.position) / Math.max(dt, 1e-3);
      o.g.userData.body.position.y = 0.66 + Math.abs(Math.sin(t * 9)) * Math.min(0.06, moving * 0.01);
      o.g.userData.halo.rotation.z = t;
      if (o.bubble && performance.now() > o.bubbleUntil) {
        o.g.remove(o.bubble);
        disposeTree(o.bubble);
        o.bubble = null;
      }
    }
  }
  count() {
    return this.map.size;
  }
  clear() {
    for (const id of [...this.map.keys()]) this.remove(id);
  }
}
