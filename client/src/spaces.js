// Spaces: beyond a house's entry hall, a chain of spaces Claude writes from
// scratch, each its own page. This moves you in, onward and back out, and
// keeps the next one coming: entering a house asks for space 1, entering
// space n asks for space n+1, so by the time you want something new it's
// ready. Spaces run in a sandboxed frame; /kit/bridge.js inside each one
// sends back "onward", "back" and the game's reserved keys.

let el = null;
let frame = null;
let here = null; // { world, x, z, n, at, idea }
let spaces = new Map(); // "world/x,z" -> [space]
let handlers = {};
let waiting = null; // { world, x, z, n } while the next one is still forming

const key = (w, x, z) => `${w}/${x},${z}`;
const esc = (s) => String(s ?? '').replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]);

async function post(path, body) {
  const r = await fetch(path, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) });
  const out = await r.json().catch(() => ({}));
  if (!r.ok) throw new Error(out.error || r.statusText);
  return out;
}

// handlers: { onKey(code, here), onBack(here), toast(title, text) }
export function initSpaces(h) {
  handlers = h;
  addEventListener('message', (e) => {
    if (!frame || e.source !== frame.contentWindow || !e.data?.branches) return;
    const m = e.data;
    if (m.branches === 'onward') onward();
    else if (m.branches === 'back') back();
    else if (m.branches === 'key') key_(m.code);
    else if (m.branches === 'lock') el?.classList.toggle('locked', !!m.locked);
    else if (m.branches === 'carry' && here) here.carry = m.items;
    else if (m.branches === 'error') console.warn('space error:', m.message);
  });
  addEventListener(
    'keydown',
    (e) => {
      if (!isOpen() || e.target?.tagName === 'TEXTAREA' || e.target?.tagName === 'INPUT') return;
      if (e.code === 'Escape' || e.code === 'Equal' || e.code === 'Minus' || e.code === 'KeyG' || e.code === 'Backquote' || e.key === '?') {
        e.preventDefault();
        e.stopImmediatePropagation();
        key_(e.key === '?' ? 'Help' : e.code);
      }
    },
    true,
  );
}

function key_(code) {
  if (code === 'Escape') return back();
  handlers.onKey?.(code, here);
}

export function isOpen() {
  return !!el;
}

export function current() {
  return here;
}

export function known(world, x, z) {
  return spaces.get(key(world, x, z)) || [];
}

export function setKnown(world, x, z, list) {
  spaces.set(key(world, x, z), list || []);
}

// You walked into a house (n = 0): start its first space if it isn't made.
export async function enteredHouse(world, x, z) {
  try {
    const r = await post(`/api/app/spaces/${encodeURIComponent(world)}/${x}/${z}/visit`, { n: 0 });
    setKnown(world, x, z, r.spaces);
    return r;
  } catch {
    return null;
  }
}

// A space finished (or started) being made, from the live connection.
export function onSpaceEvent(world, m) {
  const list = known(world, m.x, m.z).filter((s) => s.n !== m.n);
  list.push({ n: m.n, status: m.status, name: m.name || '' });
  list.sort((a, b) => a.n - b.n);
  setKnown(world, m.x, m.z, list);
  if (waiting && waiting.world === world && waiting.x === m.x && waiting.z === m.z && waiting.n === m.n) {
    if (m.status === 'ready') {
      const w = waiting;
      waiting = null;
      show(w.world, w.x, w.z, w.n, w.idea, w.carried);
    } else if (m.status === 'failed') {
      wait(waiting, 'That space came apart while it was being made. Another is being made now…');
    }
  }
}

export function ready(world, x, z, n) {
  return known(world, x, z).find((s) => s.n === n && s.status === 'ready');
}

// Go into space n of a house (from its entry hall, n = 1).
export async function enter(world, x, z, n, idea) {
  const prev = here;
  const left = prev ? { left: prev.n, seconds: (performance.now() - prev.at) / 1000 } : {};
  try {
    const r = await post(`/api/app/spaces/${encodeURIComponent(world)}/${x}/${z}/visit`, { n, ...left });
    setKnown(world, x, z, r.spaces);
    const carried = prev?.carry || prev?.carried || [];
    if (ready(world, x, z, n)) return show(world, x, z, n, idea, carried);
    wait({ world, x, z, n, idea, typical: r.typical_secs, carried });
  } catch (e) {
    handlers.toast?.('Hmm', e.message);
  }
}

function ensureEl() {
  if (el) return;
  el = document.createElement('div');
  el.id = 'space';
  el.innerHTML = `<div class="space-bar"><span class="space-where"></span><span class="space-keys">Esc back to the house · = warmer · - colder · G feedback · ? keys</span><button class="space-back">back to the house</button></div><div class="space-wait" hidden></div><div class="space-toasts"></div>`;
  el.querySelector('.space-back').onclick = back;
  document.body.appendChild(el);
  document.exitPointerLock?.();
}

function show(world, x, z, n, idea, carried = []) {
  ensureEl();
  here = { world, x, z, n, at: performance.now(), idea, carried };
  const s = ready(world, x, z, n);
  el.querySelector('.space-wait').hidden = true;
  el.querySelector('.space-where').innerHTML = `${idea ? `<i>${esc(idea)}</i> · ` : ''}space ${n}${s?.name ? `: <b>${esc(s.name)}</b>` : ''}`;
  frame?.remove();
  frame = document.createElement('iframe');
  frame.setAttribute('sandbox', 'allow-scripts allow-pointer-lock');
  frame.setAttribute('allow', 'autoplay');
  frame.src = `/space/${encodeURIComponent(world)}/${x},${z}/${n}/${carried.length ? `?carried=${encodeURIComponent(JSON.stringify(carried))}` : ''}`;
  el.prepend(frame);
  el.classList.add('fresh');
  setTimeout(() => el?.classList.remove('fresh'), 4000);
  setTimeout(() => frame?.focus(), 80);
}

function wait(w, text) {
  ensureEl();
  waiting = w;
  frame?.remove();
  frame = null;
  here = { world: w.world, x: w.x, z: w.z, n: w.n, at: performance.now(), idea: w.idea, carried: w.carried || [] };
  const box = el.querySelector('.space-wait');
  box.hidden = false;
  box.innerHTML = `<div><p>${esc(text || 'The next space is still taking shape.')}</p><p class="sub">${w.typical ? `Spaces take about ${Math.round(w.typical)}s to make. ` : ''}You'll go in the moment it's ready, or Esc to go back to the house.</p></div>`;
}

async function onward() {
  if (!here) return;
  const { world, x, z, n, idea } = here;
  enter(world, x, z, n + 1, idea);
}

export function back() {
  if (!here && !el) return;
  const h = here;
  if (h && h.n > 0) post(`/api/app/spaces/${encodeURIComponent(h.world)}/${h.x}/${h.z}/visit`, { n: 0, left: h.n, seconds: (performance.now() - h.at) / 1000 }).catch(() => {});
  frame?.remove();
  frame = null;
  el?.remove();
  el = null;
  here = null;
  waiting = null;
  handlers.onBack?.(h);
}
