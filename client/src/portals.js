// Seamless doors. A door portal shows a live view of where it leads,
// rendered from a virtual camera, and moves you there when you cross it.
//
// v1 portals are pure translations (a house's front door and the same door
// in its pocket below), which keeps the maths simple: the virtual camera is
// the real camera shifted by `offset`, and a clipping plane at the far door
// hides everything between that camera and the doorway.
import * as THREE from 'three';
import { G } from './geo.js';

const MAX_LIVE = 2;

export class DoorPortal {
  constructor({ pos, normal, w, h, offset, color, space, depth = 0.5 }) {
    this.pos = pos.clone();
    this.normal = normal.clone().normalize();
    this.w = w;
    this.h = h;
    this.offset = offset.clone();
    this.space = space;
    this.partner = null;
    this.fallback = new THREE.MeshBasicMaterial({ color });
    this.mesh = new THREE.Mesh(G.box, this.fallback);
    const across = Math.abs(this.normal.z) > 0.5;
    this.mesh.scale.set(across ? w : depth, h, across ? depth : w);
    this.mesh.position.copy(pos).addScaledVector(this.normal, -depth / 2);
    this.mesh.updateMatrixWorld();
  }
  setColor(c) {
    this.fallback.color.set(c);
  }
  // Signed distance of a point from the door plane (positive = in front).
  side(p) {
    return _t.copy(p).sub(this.pos).dot(this.normal);
  }
  // Did a move from a to b pass through the doorway from the front?
  crossed(a, b) {
    const sa = this.side(a), sb = this.side(b);
    if (!(sa >= 0 && sb < 0)) return false;
    const k = sa / (sa - sb);
    _t.copy(a).lerp(b, k).sub(this.pos);
    const lateral = Math.abs(this.normal.z) > 0.5 ? _t.x : _t.z;
    return Math.abs(lateral) <= this.w / 2 + 0.05 && Math.abs(_t.y) <= this.h / 2 + 1.2;
  }
  dispose() {
    this.fallback.dispose();
  }
}

const _t = new THREE.Vector3();

export class PortalRenderer {
  constructor(renderer) {
    this.renderer = renderer;
    this.size = new THREE.Vector2();
    this.vcam = new THREE.PerspectiveCamera();
    this.frustum = new THREE.Frustum();
    this.projScreen = new THREE.Matrix4();
    this.plane = new THREE.Plane();
    this.slots = [];
    for (let i = 0; i < MAX_LIVE; i++) {
      const rt = new THREE.WebGLRenderTarget(4, 4, { type: THREE.HalfFloatType, samples: 2 });
      const mat = new THREE.ShaderMaterial({
        uniforms: { map: { value: rt.texture }, res: { value: new THREE.Vector2(1, 1) } },
        side: THREE.DoubleSide,
        vertexShader: 'void main(){ gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0); }',
        fragmentShader: `uniform sampler2D map; uniform vec2 res;
          void main(){
            gl_FragColor = texture2D(map, gl_FragCoord.xy / res);
            #include <tonemapping_fragment>
            #include <colorspace_fragment>
          }`,
      });
      this.slots.push({ rt, mat });
    }
  }

  resize() {
    this.renderer.getDrawingBufferSize(this.size);
    for (const s of this.slots) {
      s.rt.setSize(this.size.x, this.size.y);
      s.mat.uniforms.res.value.copy(this.size);
    }
  }

  // `prepare(space, eye)` configures the scene for a space (visibility,
  // fog, lights). `spaceOf(point)` says which space a point is in.
  render(scene, camera, portals, prepare, spaceOf) {
    const r = this.renderer;
    camera.updateMatrixWorld();
    this.projScreen.multiplyMatrices(camera.projectionMatrix, camera.matrixWorldInverse);
    this.frustum.setFromProjectionMatrix(this.projScreen);
    const camSpace = spaceOf(camera.position);

    const live = portals
      .filter((p) => p.space === camSpace && p.side(camera.position) > -0.05 && p.pos.distanceTo(camera.position) < 45 && this.frustum.intersectsObject(p.mesh))
      .sort((a, b) => a.pos.distanceToSquared(camera.position) - b.pos.distanceToSquared(camera.position))
      .slice(0, MAX_LIVE);

    for (const p of portals) p.mesh.material = p.fallback;

    live.forEach((p, i) => {
      const slot = this.slots[i];
      const v = this.vcam;
      v.copy(camera);
      v.position.copy(camera.position).add(p.offset);
      v.updateMatrixWorld();
      const dest = _t.copy(p.pos).add(p.offset);
      prepare(spaceOf(dest), v.position);
      p.mesh.visible = false;
      if (p.partner) p.partner.mesh.visible = false;
      this.plane.setFromNormalAndCoplanarPoint(_n.copy(p.normal).negate(), dest.addScaledVector(p.normal, 0.03));
      r.clippingPlanes = [this.plane];
      r.setRenderTarget(slot.rt);
      r.clear();
      r.render(scene, v);
      r.clippingPlanes = [];
      p.mesh.visible = true;
      if (p.partner) p.partner.mesh.visible = true;
      p.mesh.material = slot.mat;
    });

    r.setRenderTarget(null);
    prepare(camSpace, camera.position);
    r.render(scene, camera);
    return live.length;
  }
}

const _n = new THREE.Vector3();
