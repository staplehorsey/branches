// WebXR: walk the worlds in a headset (built and tested against Quest 3).
//
// Your body is the same player primitive as on a screen. The headset sits on
// a "rig" placed so your real head is where the player stands: walking in
// your room moves the player (through walls' collisions too), the left stick
// walks, the right stick turns in comfortable 30° steps, and doors carry the
// whole rig with you.
//
// Controls (shown on your left wrist):
//   left stick   walk (click to go faster)      right stick  snap turn
//   A            visitor log / interact          B            more of this
//   Y            leave VR
import * as THREE from 'three';
import { canvasTexture } from './geo.js';

const SNAP = Math.PI / 6;
const DEAD = 0.15;

export class XR {
  constructor({ renderer, camera, rig, player, onAction, onEnd }) {
    this.renderer = renderer;
    this.camera = camera;
    this.rig = rig;
    this.player = player;
    this.onAction = onAction;
    this.onEnd = onEnd;
    this.rigYaw = 0;
    this.lastHead = null;
    this.snapLatch = false;
    this.pressed = new Map();
    this.wristText = '';
    renderer.xr.enabled = true;
    renderer.xr.setReferenceSpaceType('local-floor');
    renderer.xr.setFoveation(1);
    this.buildControllers();
    this.button = document.getElementById('vr-btn');
    this.button.onclick = () => (this.active ? this.exit() : this.enter());
    this.supported = false;
    navigator.xr
      ?.isSessionSupported('immersive-vr')
      .then((ok) => {
        this.supported = ok;
        this.button.hidden = !ok;
      })
      .catch(() => {});
  }

  get active() {
    return this.renderer.xr.isPresenting;
  }

  async enter() {
    if (this.active || !navigator.xr) return;
    try {
      const session = await navigator.xr.requestSession('immersive-vr', { optionalFeatures: ['local-floor', 'bounded-floor', 'hand-tracking'] });
      session.addEventListener('end', () => this.ended());
      await this.renderer.xr.setSession(session);
      this.rigYaw = this.player.yaw;
      this.lastHead = null;
      this.player.thirdPerson = false;
      this.button.textContent = 'Exit VR';
      document.body.classList.add('in-vr');
    } catch (e) {
      this.onAction('error', e.message || String(e));
    }
  }

  exit() {
    this.renderer.xr.getSession()?.end();
  }

  ended() {
    this.rig.position.set(0, 0, 0);
    this.rig.rotation.set(0, 0, 0);
    this.rig.updateMatrixWorld(true);
    this.button.textContent = 'Enter VR';
    document.body.classList.remove('in-vr');
    this.onEnd?.();
  }

  buildControllers() {
    const ray = new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(0, 0, 0), new THREE.Vector3(0, 0, -0.35)]);
    for (let i = 0; i < 2; i++) {
      const c = this.renderer.xr.getController(i);
      const orb = new THREE.Mesh(new THREE.SphereGeometry(0.018, 12, 8), new THREE.MeshBasicMaterial({ color: '#fff6e0' }));
      const line = new THREE.Line(ray, new THREE.LineBasicMaterial({ color: '#ffffff', transparent: true, opacity: 0.35 }));
      c.add(orb, line);
      c.addEventListener('connected', (e) => {
        c.userData.hand = e.data.handedness;
        if (e.data.handedness === 'left') c.add(this.wrist);
      });
      c.addEventListener('disconnected', () => c.remove(this.wrist));
      this.rig.add(c);
    }
    // A small card on the left wrist: where you are and what you can do.
    this.wristCanvas = document.createElement('canvas');
    this.wristCanvas.width = 512;
    this.wristCanvas.height = 256;
    this.wristTex = new THREE.CanvasTexture(this.wristCanvas);
    this.wristTex.colorSpace = THREE.SRGBColorSpace;
    this.wrist = new THREE.Mesh(new THREE.PlaneGeometry(0.16, 0.08), new THREE.MeshBasicMaterial({ map: this.wristTex, transparent: true, depthTest: false, fog: false }));
    this.wrist.renderOrder = 10;
    this.wrist.position.set(0.02, 0.05, 0.06);
    this.wrist.rotation.set(-1.1, 0.4, 0.2);
    this.setWrist('Branches', '');
  }

  setWrist(title, hint) {
    const text = `${title}|${hint}`;
    if (text === this.wristText) return;
    this.wristText = text;
    const ctx = this.wristCanvas.getContext('2d');
    ctx.clearRect(0, 0, 512, 256);
    ctx.fillStyle = 'rgba(255,252,247,0.88)';
    ctx.beginPath();
    ctx.roundRect(6, 6, 500, 244, 36);
    ctx.fill();
    ctx.fillStyle = '#23222b';
    ctx.font = 'italic 44px Georgia, serif';
    ctx.fillText(title.slice(0, 22), 32, 72);
    ctx.font = '28px system-ui, sans-serif';
    ctx.fillText(hint.slice(0, 34), 32, 120);
    ctx.globalAlpha = 0.6;
    ctx.font = '24px system-ui, sans-serif';
    ctx.fillText('stick walk · A interact · B more of this', 32, 180);
    ctx.fillText('right stick turn · Y leave VR', 32, 218);
    ctx.globalAlpha = 1;
    this.wristTex.needsUpdate = true;
  }

  edge(key, down) {
    const was = this.pressed.get(key) || false;
    this.pressed.set(key, down);
    return down && !was;
  }

  // Before physics: read sticks and buttons, follow the real head.
  beforeUpdate() {
    const session = this.renderer.xr.getSession();
    const p = this.player;
    let mx = 0, my = 0, run = false;
    for (const src of session?.inputSources || []) {
      const gp = src.gamepad;
      if (!gp) continue;
      const ax = gp.axes.length >= 4 ? gp.axes[2] : gp.axes[0] || 0;
      const ay = gp.axes.length >= 4 ? gp.axes[3] : gp.axes[1] || 0;
      const b = (i) => !!gp.buttons[i]?.pressed;
      if (src.handedness === 'left') {
        if (Math.hypot(ax, ay) > DEAD) {
          mx = ax;
          my = -ay;
        }
        run = b(3);
        if (this.edge('Y', b(5))) this.exit();
        if (this.edge('X', b(4))) this.onAction('admire');
      } else if (src.handedness === 'right') {
        if (Math.abs(ax) > 0.6 && !this.snapLatch) {
          this.rigYaw -= Math.sign(ax) * SNAP;
          this.snapLatch = true;
        } else if (Math.abs(ax) < 0.3) this.snapLatch = false;
        if (this.edge('A', b(4))) this.onAction('interact');
        if (this.edge('B', b(5))) this.onAction('admire');
      }
    }
    p.touch = { x: mx, y: my, active: mx !== 0 || my !== 0 };
    if (run) p.keys.add('ShiftLeft');
    else p.keys.delete('ShiftLeft');

    // Walk where the head looks.
    const q = new THREE.Quaternion();
    this.camera.getWorldQuaternion(q);
    const f = new THREE.Vector3(0, 0, -1).applyQuaternion(q);
    if (Math.hypot(f.x, f.z) > 1e-3) p.yaw = Math.atan2(-f.x, -f.z);

    // Physically walking in your room moves the player.
    const head = this.camera.position;
    if (this.lastHead) {
      const d = new THREE.Vector3(head.x - this.lastHead.x, 0, head.z - this.lastHead.z).applyAxisAngle(new THREE.Vector3(0, 1, 0), this.rigYaw);
      if (d.length() < 0.5) {
        p.pos.x += d.x;
        p.pos.z += d.z;
      }
    }
    this.lastHead = head.clone();
  }

  // A door turned the player; turn the room with them.
  carried(yawBefore, yawAfter) {
    this.rigYaw += yawAfter - yawBefore;
  }

  // After physics: put the rig under the player so the head is where they are.
  afterUpdate() {
    const head = this.camera.position;
    const off = new THREE.Vector3(head.x, 0, head.z).applyAxisAngle(new THREE.Vector3(0, 1, 0), this.rigYaw);
    this.rig.position.set(this.player.pos.x - off.x, this.player.pos.y, this.player.pos.z - off.z);
    this.rig.rotation.set(0, this.rigYaw, 0);
    this.rig.updateMatrixWorld(true);
    if (this.player.avatar) this.player.avatar.visible = false;
  }
}
