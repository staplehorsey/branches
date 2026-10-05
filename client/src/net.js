// Talking to world hosts. A host is just an origin that speaks the Branches
// HTTP + websocket protocol; the client can hop between hosts freely.

const KEY = 'branches.identity.v1';

function randomHex(n) {
  const bytes = new Uint8Array(n);
  crypto.getRandomValues(bytes);
  return [...bytes].map((b) => b.toString(16).padStart(2, '0')).join('');
}

const NAMES = ['moth', 'fern', 'tide', 'lumen', 'pollen', 'drift', 'wren', 'clover', 'haze', 'sorrel', 'quill', 'juniper'];
const COLORS = ['#ff9ec7', '#9ed8ff', '#ffd59e', '#b8ff9e', '#d3a6ff', '#ffffff', '#ff8f7a', '#7affd7'];

export function identity() {
  let id = null;
  try {
    id = JSON.parse(localStorage.getItem(KEY));
  } catch {}
  if (!id || !id.player || !id.secret) {
    const pick = (l) => l[Math.floor(Math.random() * l.length)];
    id = {
      player: randomHex(8),
      secret: randomHex(16),
      name: `${pick(NAMES)}-${Math.floor(Math.random() * 90 + 10)}`,
      color: pick(COLORS),
    };
    saveIdentity(id);
  }
  return id;
}

export function saveIdentity(id) {
  try {
    localStorage.setItem(KEY, JSON.stringify(id));
  } catch {}
}

export class Host {
  constructor(origin) {
    this.origin = origin.replace(/\/$/, '');
  }
  url(path) {
    return this.origin + path;
  }
  async get(path) {
    const r = await fetch(this.url(path));
    const body = await r.json().catch(() => ({}));
    if (!r.ok) throw new Error(body.error || r.statusText);
    return body;
  }
  async send(method, path, body) {
    const me = identity();
    const r = await fetch(this.url(path), {
      method,
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ player: me.player, secret: me.secret, ...body }),
    });
    const out = await r.json().catch(() => ({}));
    if (!r.ok) throw new Error(out.error || r.statusText);
    return out;
  }
  world(id) {
    return this.get(`/api/worlds/${encodeURIComponent(id)}`);
  }
  chunk(world, x0, z0, x1, z1) {
    return this.get(`/api/worlds/${encodeURIComponent(world)}/chunk?x0=${x0}&z0=${z0}&x1=${x1}&z1=${z1}`);
  }
  room(world, x, z) {
    return this.get(`/api/worlds/${encodeURIComponent(world)}/rooms/${x}/${z}`);
  }
  roomPath(world, x, z, rest = '') {
    return `/api/worlds/${encodeURIComponent(world)}/rooms/${x}/${z}${rest}`;
  }
}

// Live connection to one world on one host.
export class Live {
  constructor(host, world, handlers) {
    this.host = host;
    this.world = world;
    this.handlers = handlers;
    this.closed = false;
    this.ws = null;
    this.lastSent = 0;
    this.connect();
  }
  connect() {
    if (this.closed) return;
    const wsUrl = this.host.origin.replace(/^http/, 'ws') + `/ws/${encodeURIComponent(this.world)}`;
    const ws = new WebSocket(wsUrl);
    this.ws = ws;
    ws.onopen = () => {
      const me = identity();
      ws.send(JSON.stringify({ t: 'hello', player: me.player, secret: me.secret, name: me.name, color: me.color }));
    };
    ws.onmessage = (e) => {
      let m;
      try {
        m = JSON.parse(e.data);
      } catch {
        return;
      }
      this.handlers[m.t]?.(m);
    };
    ws.onclose = () => {
      this.handlers.disconnected?.();
      if (!this.closed) setTimeout(() => this.connect(), 2000);
    };
  }
  send(msg) {
    if (this.ws?.readyState === 1) this.ws.send(JSON.stringify(msg));
  }
  close() {
    this.closed = true;
    this.ws?.close();
  }
}
