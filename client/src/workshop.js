// The workshop: a side panel for talking to Claude Code about the game
// itself while you play. Only in the app, running from its source code.
// Toggle with ` (backquote). Claude's work streams in; when it changes the
// client the page reloads, and when it changes the server the app rebuilds
// and restarts, and the page reloads once the new server answers.

const OPEN_KEY = 'branches.workshop.open';
let el = null;
let log = null;
let input = null;
let state = { available: false, busy: false, boot: 0 };
let whereAmI = () => '';
let restarting = false;

const esc = (s) => String(s ?? '').replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]);
const md = (s) =>
  esc(s)
    .replace(/```[a-z]*\n?([\s\S]*?)```/g, (_, c) => `<pre>${c}</pre>`)
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    .replace(/\*\*([^*]+)\*\*/g, '<b>$1</b>')
    .replace(/\n/g, '<br>');

async function call(path, body) {
  const r = await fetch(path, body === undefined ? {} : { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) });
  const out = await r.json().catch(() => ({}));
  if (!r.ok) throw new Error(out.error || r.statusText);
  return out;
}

export async function initWorkshop({ where }) {
  whereAmI = where;
  try {
    state = await call('/api/app/workshop');
  } catch {
    return;
  }
  if (!state.available) return;
  build();
  for (const m of state.history || []) add(m, true);
  setBusy(state.busy);
  let open = false;
  try {
    open = sessionStorage.getItem(OPEN_KEY) === '1';
  } catch {}
  if (open) toggle(true, false);
}

export function workshopAvailable() {
  return !!el;
}

export function isWorkshopOpen() {
  return !!el && !el.hidden;
}

export function toggle(show = el?.hidden, focus = true) {
  if (!el) return;
  el.hidden = !show;
  try {
    sessionStorage.setItem(OPEN_KEY, show ? '1' : '0');
  } catch {}
  if (show) {
    log.scrollTop = log.scrollHeight;
    if (focus) {
      document.exitPointerLock?.();
      setTimeout(() => input.focus(), 0);
    }
  } else input.blur();
}

function build() {
  el = document.createElement('div');
  el.id = 'workshop';
  el.hidden = true;
  el.innerHTML = `
    <div class="ws-head">
      <b>Claude</b><span class="ws-sub">changes the game itself</span>
      <span class="ws-busy" hidden>working…</span>
      <button class="ws-stop" title="stop Claude" hidden>stop</button>
      <button class="ws-new" title="start a new conversation">new</button>
      <button class="ws-close" title="close (\`)">×</button>
    </div>
    <div class="ws-log"></div>
    <textarea class="ws-input" rows="3" placeholder="Ask Claude to change something… (enter to send, shift+enter for a new line, esc to play)"></textarea>`;
  document.body.appendChild(el);
  log = el.querySelector('.ws-log');
  input = el.querySelector('.ws-input');
  el.querySelector('.ws-close').onclick = () => toggle(false);
  el.querySelector('.ws-new').onclick = () => call('/api/app/workshop/new', {}).catch((e) => add({ kind: 'error', text: e.message }));
  el.querySelector('.ws-stop').onclick = () => call('/api/app/workshop/stop', {}).catch(() => {});
  input.addEventListener('keydown', (e) => {
    e.stopPropagation();
    if (e.key === 'Escape') {
      input.blur();
      return;
    }
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      send();
    }
  });
  input.addEventListener('keyup', (e) => e.stopPropagation());
}

async function send() {
  const text = input.value.trim();
  if (!text || state.busy) return;
  input.value = '';
  try {
    await call('/api/app/workshop', { text, who: whereAmI().who, place: whereAmI().place });
  } catch (e) {
    input.value = text;
    add({ kind: 'error', text: e.message });
  }
}

function setBusy(b) {
  state.busy = b;
  if (!el) return;
  el.querySelector('.ws-busy').hidden = !b;
  el.querySelector('.ws-stop').hidden = !b;
  input.disabled = b;
}

function add(m, quiet = false) {
  if (!log) return;
  if (m.kind === 'idle') return;
  const d = document.createElement('div');
  d.className = `ws-${m.kind}`;
  if (m.kind === 'you') d.innerHTML = `<span class="ws-who">${esc(m.who)}</span> ${esc(m.text)}`;
  else if (m.kind === 'claude') d.innerHTML = md(m.text);
  else if (m.kind === 'reload') d.textContent = quiet ? 'Reloaded with the changes.' : 'Reloading with the changes…';
  else if (m.kind === 'restart') d.textContent = quiet ? 'Restarted on the new build.' : 'Restarting on the new build…';
  else d.textContent = m.text || '';
  log.appendChild(d);
  log.scrollTop = log.scrollHeight;
}

// A message from the server (through the world's live connection).
export function onWorkshop(m) {
  if (!el) return;
  add(m);
  if (m.kind === 'you') setBusy(true);
  if (m.kind === 'idle') setBusy(false);
  if (!isWorkshopOpen() && (m.kind === 'claude' || m.kind === 'error')) {
    toggle(true, false);
  }
  if (m.kind === 'reload') setTimeout(reload, 1200);
  if (m.kind === 'restart') waitForRestart();
}

function reload() {
  try {
    sessionStorage.setItem('branches.reenter', '1');
  } catch {}
  location.reload();
}

async function waitForRestart() {
  if (restarting) return;
  restarting = true;
  for (let i = 0; i < 180; i++) {
    await new Promise((r) => setTimeout(r, 1000));
    try {
      const s = await call('/api/app/workshop');
      if (s.boot !== state.boot) return reload();
    } catch {}
  }
  restarting = false;
  add({ kind: 'error', text: "The app didn't come back after the restart. Check the terminal or run scripts/dev.sh log." });
}
