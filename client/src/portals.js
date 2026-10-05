// Seamless doors. A door shows a live view of the door it is linked to,
// rendered from a virtual camera, and carries you through when you cross.
//
// Each door has a frame: origin at the centre of the opening, +z pointing to
// its front (the side you walk in from). Linking doors A and B gives
// A.M = B.frame · rotY(π) · A.frame⁻¹: stepping out of the back of A puts you
// on the front of B, facing into B's room. The two doors can be anywhere in
// the scene, at any heading: a house's front door and its pocket below, or a
// side-wall door in one world and an entry hall in another.
import * as THREE from 'three';
import { G, aabb } from './geo.js';

const MAX_LIVE = 2;
const UP = new THREE.Vector3(0, 1, 0);
const FLIP = new THREE.Matrix4().makeRotationY(Math.PI);

export class DoorPortal {
  constructor({ pos, normal, w, h, color, space, depth = 0.5, sealed = false }) {
    this.pos = pos.clone();
    this.normal = normal.clone().normalize();
    this.xAxis = new THREE.Vector3().crossVectors(UP, this.normal);
    this.w = w;
    this.h = h;
    this.space = space;
    this.sealed = sealed;
    this.partner = null;
    this.M = null;
    this.frame = new THREE.Matrix4().makeBasis(this.xAxis, UP, this.normal).setPosition(this.pos);
    this.frameInv = this.frame.clone().invert();
    this.fallback = new THREE.MeshBasicMaterial({ color });
    this.mesh = new THREE.Mesh(G.box, this.fallback);
    this.mesh.matrixAutoUpdate = false;
    this.mesh.matrix.copy(this.frame).multiply(new THREE.Matrix4().makeTranslation(0, 0, -depth / 2)).multiply(new THREE.Matrix4().makeScale(w, h, depth));
    this.mesh.matrixWorldNeedsUpdate = true;
    this.mesh.updateMatrixWorld(true);
    this.mesh.visible = !sealed;
    // Until linked, a door is solid.
    const ex = Math.abs(this.normal.x) > 0.5;
    this.block = ex
      ? aabb(pos.x - 0.2, pos.z - w / 2, pos.x + 0.2, pos.z + w / 2, pos.y - h / 2 - 1, pos.y + h / 2)
      : aabb(pos.x - w / 2, pos.z - 0.2, pos.x + w / 2, pos.z + 0.2, pos.y - h / 2 - 1, pos.y + h / 2);
  }
  get open() {
    return !!this.partner && !this.sealed;
  }
  setColor(c) {
    this.fallback.color.set(c);
  }
  side(p) {
    return _t.copy(p).sub(this.pos).dot(this.normal);
  }
  // Did a move from a to b pass through the opening, front to back?
  crossed(a, b) {
    if (!this.open) return false;
    const sa = this.side(a), sb = this.side(b);
    if (!(sa >= 0 && sb < 0)) return false;
    _t.copy(a).lerp(b, sa / (sa - sb)).sub(this.pos);
    return Math.abs(_t.dot(this.xAxis)) <= this.w / 2 + 0.05 && Math.abs(_t.y) <= this.h / 2 + 1.2;
  }
  dispose() {
    unlink(this);
    this.mesh.removeFromParent();
    this.fallback.dispose();
  }
}

export function link(a, b) {
  unlink(a);
  unlink(b);
  a.partner = b;
  b.partner = a;
  a.M = new THREE.Matrix4().multiplyMatrices(b.frame, FLIP).multiply(a.frameInv);
  b.M = a.M.clone().invert();
}

export function unlink(a) {
  const b = a.partner;
  a.partner = null;
  a.M = null;
  if (b && b.partner === a) {
    b.partner = null;
    b.M = null;
  }
}

const _t = new THREE.Vector3();
const _n = new THREE.Vector3();

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
      .filter((p) => p.open && p.space === camSpace && p.side(camera.position) > -0.05 && p.pos.distanceTo(camera.position) < 45 && this.frustum.intersectsObject(p.mesh))
      .sort((a, b) => a.pos.distanceToSquared(camera.position) - b.pos.distanceToSquared(camera.position))
      .slice(0, MAX_LIVE);

    for (const p of portals) p.mesh.material = p.fallback;

    live.forEach((p, i) => {
      const slot = this.slots[i];
      const v = this.vcam;
      const dest = p.partner;
      v.copy(camera);
      v.matrixWorld.multiplyMatrices(p.M, camera.matrixWorld);
      v.matrixWorld.decompose(v.position, v.quaternion, v.scale);
      v.updateMatrixWorld(true);
      prepare(dest.space, v.position);
      p.mesh.visible = false;
      dest.mesh.visible = false;
      this.plane.setFromNormalAndCoplanarPoint(dest.normal, _n.copy(dest.pos).addScaledVector(dest.normal, -0.03));
      r.clippingPlanes = [this.plane];
      r.setRenderTarget(slot.rt);
      r.clear();
      r.render(scene, v);
      r.clippingPlanes = [];
      p.mesh.visible = true;
      dest.mesh.visible = !dest.sealed;
      p.mesh.material = slot.mat;
    });

    r.setRenderTarget(null);
    prepare(camSpace, camera.position);
    r.render(scene, camera);
    return live.length;
  }
}
