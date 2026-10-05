// DOM: HUD, toasts, chat and the panels (visitor log, worlds, inbox).
import { identity, saveIdentity } from './net.js';

const $ = (id) => document.getElementById(id);
const esc = (s) => String(s ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]);

export function ago(ms) {
  const s = Math.max(0, (Date.now() - ms) / 1000);
  if (s < 60) return 'just now';
  if (s < 3600) return `${Math.floor(s / 60)}m ago`;
  if (s < 86400) return `${Math.floor(s / 3600)}h ago`;
  return `${Math.floor(s / 86400)}d ago`;
}

export function toast(title, body = '', onClick) {
  const el = document.createElement('div');
  el.className = 'toast';
  el.innerHTML = `<b>${esc(title)}</b>${esc(body)}`;
  if (onClick) {
    el.style.cursor = 'pointer';
    el.onclick = onClick;
  }
  $('toasts').prepend(el);
  setTimeout(() => el.remove(), 7200);
}

export function chatLine(name, color, text, sys = false) {
  const el = document.createElement('div');
  if (sys) {
    el.className = 'sys';
    el.textContent = text;
  } else el.innerHTML = `<span style="color:${esc(color)};font-weight:600;filter:brightness(0.75)">${esc(name)}</span> ${esc(text)}`;
  const log = $('chat-log');
  log.append(el);
  while (log.children.length > 12) log.firstChild.remove();
}

let bannerTimer;
export function banner(text) {
  const b = $('banner');
  b.textContent = text;
  b.classList.add('on');
  clearTimeout(bannerTimer);
  bannerTimer = setTimeout(() => b.classList.remove('on'), 3200);
}

export function hint(text) {
  $('hint').textContent = text || '';
}

export function setUnread(n) {
  $('unread').textContent = n;
  $('bell').classList.toggle('has', n > 0);
}

export function isPanelOpen() {
  return !$('panel').hidden;
}

export function closePanel() {
  $('panel').hidden = true;
  onClose?.();
  onClose = null;
}
let onClose = null;

function openPanel(html, close) {
  $('panel-body').innerHTML = html;
  $('panel').hidden = false;
  onClose = close;
  document.exitPointerLock?.();
  return $('panel-body');
}

$('panel-close').onclick = closePanel;
$('panel').addEventListener('mousedown', (e) => {
  if (e.target.id === 'panel') closePanel();
});

const KIND_ICON = { visit: '·', note: '✎', request: '✦', praise: '♥', growth: '❀', claim: '⚑', portal: '⟶' };

function entryHtml(e) {
  return `<div class="entry ${esc(e.kind)}"><span>${KIND_ICON[e.kind] || '·'}</span> <span class="who">${esc(e.who)}</span> ${esc(e.text)}<span class="when">${ago(e.at)}</span></div>`;
}

// The visitor log for one address, plus claim and owner tools.
export async function openRoomPanel({ host, world, x, z, themes, local = false, needsApp }) {
  const me = identity();
  const body = openPanel('<p class="sub">opening the visitor log…</p>');
  let room;
  try {
    room = await host.room(world, x, z);
  } catch (e) {
    body.innerHTML = `<p class="err">${esc(e.message)}</p>`;
    return;
  }
  const mine = room.claim?.owner === me.player;
  const pct = Math.min(100, (room.attention / Math.max(1, room.next_growth_at)) * 100);
  const title = room.claim?.title || room.theme_name;
  body.innerHTML = `
    <h2>${esc(title)}</h2>
    <div class="sub">${esc(room.theme_name)} · ${x},${z} · ${room.chambers.length} chamber${room.chambers.length === 1 ? '' : 's'}${room.claim ? ` · tended by ${esc(room.claim.owner_name)}` : ''}</div>
    <h3>The architect</h3>
    <div class="sub">${room.building ? `building now, ready in about ${Math.max(1, Math.round((room.building.ready_at - Date.now()) / 1000))}s` : `${Math.round(room.attention)}s of attention gathered, ${Math.round(room.next_growth_at)}s grows the next chamber`}</div>
    <div class="meter"><div style="width:${room.building ? 100 : pct}%"></div></div>
    <div class="sub">leaning toward</div>
    <div class="chips">${room.leaning.map((t) => `<span class="chip">${esc(t)}</span>`).join('')}</div>
    ${room.chambers.length > 1 ? `<h3>Chambers</h3><div class="chips">${room.chambers.map((c) => `<span class="chip">${esc(c.name)}</span>`).join('')}</div>` : ''}
    ${room.investors.length ? `<h3>Who lingered</h3><div class="chips">${room.investors.map((i) => `<span class="chip">${esc(i.name)} · ${Math.max(1, Math.round(i.seconds / 60))}m</span>`).join('')}</div>` : ''}
    ${room.portals.length ? `<h3>Doors from here</h3><div class="chips">${room.portals.map((p) => `<span class="chip">chamber ${p.slot + 1} → ${esc(p.label)}${mine && p.by !== 'generator' ? ` <a href="#" data-del="${esc(p.id)}">remove</a>` : ''}</span>`).join('')}</div>` : ''}
    <h3>Visitor log</h3>
    ${local ? `<div class="sub">Visitor books are kept by Branches for Mac, along with the architect that reads them.</div><div class="btns" style="margin-top:8px"><button class="btn" id="need-app">write in this book</button></div>` : ''}
    <div class="form" ${local ? 'hidden' : ''}>
      <textarea id="log-text" rows="2" maxlength="400" placeholder="leave a note, or ask the architect for something (more water, a reading nook, stars…)"></textarea>
      <div class="btns">
        <button class="btn" data-kind="note">leave a note</button>
        <button class="btn" data-kind="request">ask the architect</button>
        <button class="btn" data-kind="praise">♥ this place</button>
      </div>
      <div id="log-msg"></div>
    </div>
    <div class="log" id="log-list" style="margin-top:10px">${room.log.map(entryHtml).join('') || '<div class="sub">No one has written here yet.</div>'}</div>
    ${room.claim || local ? '' : `
      <h3>Claim this address</h3>
      <div class="sub">Tend it: rename it, choose its theme, open doors to other worlds, and let your own agents build here.</div>
      <div class="row" style="margin-top:8px"><input id="claim-title" maxlength="60" placeholder="${esc(me.name)}'s house" /><button class="btn" id="claim">claim</button></div>
      <div id="claim-msg"></div>`}
    ${mine ? ownerTools(room, themes) : ''}
  `;
  $('need-app') && ($('need-app').onclick = () => needsApp?.('book'));
  const msg = (id, text, ok) => ($(id).innerHTML = `<span class="${ok ? 'ok' : 'err'}">${esc(text)}</span>`);
  const reopen = () => openRoomPanel({ host, world, x, z, themes });

  body.querySelectorAll('[data-kind]').forEach((b) => {
    b.onclick = async () => {
      const kind = b.dataset.kind;
      let text = $('log-text').value.trim();
      if (kind === 'praise' && !text) text = 'loved it here';
      if (!text) return msg('log-msg', 'write something first');
      try {
        const out = await host.send('POST', host.roomPath(world, x, z, '/log'), { kind, text });
        $('log-text').value = '';
        $('log-list').insertAdjacentHTML('afterbegin', entryHtml(out.entry));
        if (kind === 'request') msg('log-msg', out.architect_heard?.length ? `the architect heard: ${out.architect_heard.join(', ')}` : 'the architect will think about it', true);
      } catch (e) {
        msg('log-msg', e.message);
      }
    };
  });
  $('claim') &&
    ($('claim').onclick = async () => {
      try {
        await host.send('POST', host.roomPath(world, x, z, '/claim'), { title: $('claim-title').value.trim() || undefined });
        reopen();
      } catch (e) {
        msg('claim-msg', e.message);
      }
    });
  body.querySelectorAll('[data-del]').forEach((a) => {
    a.onclick = async (e) => {
      e.preventDefault();
      await host.send('DELETE', host.roomPath(world, x, z, `/portals/${a.dataset.del}`), {}).catch(() => {});
      reopen();
    };
  });
  if (mine) {
    $('own-save').onclick = async () => {
      try {
        const max = $('own-max').value === '' ? null : Number($('own-max').value);
        await host.send('PATCH', host.roomPath(world, x, z), {
          title: $('own-title').value,
          theme: $('own-theme').value,
          outbound: { closed: $('own-closed').checked, max },
        });
        msg('own-msg', 'saved', true);
      } catch (e) {
        msg('own-msg', e.message);
      }
    };
    $('door-add').onclick = async () => {
      try {
        await host.send('POST', host.roomPath(world, x, z, '/portals'), {
          target: $('door-target').value.trim(),
          label: $('door-label').value.trim() || undefined,
          slot: Number($('door-slot').value) || 0,
        });
        reopen();
      } catch (e) {
        msg('door-msg', e.message);
      }
    };
    $('copy-token').onclick = () => {
      navigator.clipboard?.writeText(`${me.player}:${me.secret}`);
      msg('agent-msg', 'copied. treat it like a password.', true);
    };
  }
}

function ownerTools(room, themes) {
  const me = identity();
  const rule = room.claim.outbound || {};
  return `
    <h3>Tend this address</h3>
    <div class="form">
      <input id="own-title" maxlength="60" value="${esc(room.claim.title)}" />
      <select id="own-theme">${(themes || []).map((t) => `<option value="${esc(t.id)}" ${t.id === room.theme ? 'selected' : ''}>${esc(t.name)}</option>`).join('')}</select>
      <label class="sub"><input type="checkbox" id="own-closed" ${rule.closed ? 'checked' : ''}/> close this address to new doors out</label>
      <label class="sub">at most <input id="own-max" type="number" min="0" max="20" style="width:70px" value="${rule.max ?? ''}" /> doors out (blank = world limit)</label>
      <div class="btns"><button class="btn" id="own-save">save</button></div>
      <div id="own-msg"></div>
    </div>
    <h3>Open a door</h3>
    <div class="form">
      <input id="door-target" placeholder="/w/dusk-orchard/3,-2  or  https://another.host/w/their-world/0,0" />
      <div class="row"><input id="door-label" maxlength="60" placeholder="label" /><select id="door-slot">${room.chambers.map((c, i) => `<option value="${i}">in ${esc(c.name)}</option>`).join('')}</select></div>
      <div class="btns"><button class="btn" id="door-add">open door</button></div>
      <div id="door-msg"></div>
    </div>
    <h3>Let your agent build</h3>
    <div class="sub">Anything you can do here, an agent can do over HTTP with <code>Authorization: Bearer &lt;token&gt;</code>. For example:</div>
    <p><code>POST ${esc(location.origin)}/api/worlds/${esc(room.world)}/rooms/${room.x}/${room.z}/features {"tag":"water"}</code></p>
    <div class="btns"><button class="btn" id="copy-token">copy my token</button></div>
    <div id="agent-msg"></div>
    <div class="sub" style="margin-top:6px">signed in as ${esc(me.name)}</div>
  `;
}

export async function openWorldsPanel({ host, currentWorld }) {
  const body = openPanel('<p class="sub">listening for worlds…</p>');
  let worlds = [];
  try {
    worlds = await host.get('/api/worlds');
  } catch (e) {
    body.innerHTML = `<p class="err">${esc(e.message)}</p>`;
    return;
  }
  const me = identity();
  body.innerHTML = `
    <h2>Worlds nearby</h2>
    <div class="sub">There are no shortcuts: every world is somewhere you can walk to. Each world's first house (0,0) has a door to every other one, and doors hide in houses everywhere.</div>
    <div class="worlds" style="margin-top:14px">${worlds
      .map(
        (w) => `<div class="world"><div><div class="name">${esc(w.name)}${w.id === currentWorld ? ' · you are here' : ''}</div><div class="meta">${esc(w.tagline)}</div><div class="meta">${w.online} here now · ${w.rooms_grown} rooms grown · doors out: ${esc(w.portal_policy.mode)}</div></div></div>`,
      )
      .join('')}</div>
    <h3>You</h3>
    <div class="row"><input id="me-name" maxlength="24" value="${esc(me.name)}" /><input id="me-color" type="color" value="${esc(me.color)}" /><button class="btn" id="me-save">save</button></div>
    <div class="sub">Lost? Press H to close your eyes and wake up at home.</div>
    <div id="app-settings"></div>
  `;
  $('me-save').onclick = () => {
    saveIdentity({ ...me, name: $('me-name').value.trim() || me.name, color: $('me-color').value });
    closePanel();
  };
}

export async function openInbox({ host, onRead }) {
  const me = identity();
  const body = openPanel('<p class="sub">…</p>');
  let data;
  try {
    data = await host.get(`/api/players/${me.player}/inbox?secret=${me.secret}`);
  } catch (e) {
    body.innerHTML = `<p class="err">${esc(e.message)}</p>`;
    return;
  }
  const prefs = Object.entries(data.prefs || {}).sort((a, b) => b[1] - a[1]).slice(0, 6);
  body.innerHTML = `
    <h2>While you were wandering</h2>
    <div class="sub">places you spent time in, growing without you</div>
    <div class="log" style="margin-top:12px;max-height:340px">${data.inbox.map((n) => `<div class="entry ${n.read ? '' : 'growth'}">❀ ${esc(n.text)} <span class="when">${ago(n.at)}</span> <span class="when">${esc(n.world)} · ${n.x},${n.z}</span></div>`).join('') || '<div class="sub">Nothing yet. Linger somewhere.</div>'}</div>
    ${prefs.length ? `<h3>What the architect thinks you like</h3><div class="chips">${prefs.map(([t]) => `<span class="chip">${esc(t)}</span>`).join('')}</div>` : ''}
  `;
  host.send('POST', `/api/players/${me.player}/inbox/read`, {}).then(onRead).catch(() => {});
}

// ------------------------------------------------------------ reading

export function openReading(title, text, sub = '') {
  openPanel(`<h2>${esc(title)}</h2>${sub ? `<div class="sub">${esc(sub)}</div>` : ''}<p class="reading">${esc(text)}</p>`);
}

// ------------------------------------------------------------ the Mac app

export const APP_URL = 'http://localhost:7878';
export const DOWNLOAD = 'https://github.com/staplehorsey/branches/releases/download/mac-latest/Branches-mac.zip';

const WHY = {
  book: ['Write in visitor books', 'Visitor books are kept by Branches for Mac. It saves your worlds on your computer and runs the architect that reads what you write.'],
  claim: ['Make a house yours', 'Claiming houses, opening doors and asking the architect for things happen in Branches for Mac.'],
  time: ['Keep this world growing', 'Right now everything lives in this browser tab. Branches for Mac keeps your worlds on your computer, saved in a git repository, and the architect keeps building while you are away.'],
};

// One card for every moment that needs the app: why, how to get it, and
// one click to connect if it is already running.
export function needsApp(reason, { onConnect } = {}) {
  const [title, why] = WHY[reason] || WHY.time;
  const body = openPanel(`
    <h2>${esc(title)}</h2>
    <p class="sub" style="font-size:14px;line-height:1.5">${esc(why)}</p>
    <ol class="steps">
      <li><a class="btn primary-link" href="${DOWNLOAD}" target="_blank" rel="noopener">Download Branches for Mac</a></li>
      <li>Unzip it and drag <b>Branches</b> into Applications.</li>
      <li>Right-click it and choose <b>Open</b> the first time (it isn't notarized yet).</li>
    </ol>
    <div class="btns"><button class="btn" id="app-connect">Branches is running: connect</button><button class="btn" id="app-later">keep wandering</button></div>
    <div id="app-msg"></div>
  `);
  $('app-later').onclick = closePanel;
  $('app-connect').onclick = async () => {
    $('app-msg').innerHTML = '<span class="sub">looking for Branches on this Mac…</span>';
    const ok = await onConnect?.();
    if (!ok) $('app-msg').innerHTML = `<span class="err">Branches isn't answering at ${APP_URL}. Open the app, then try again.</span>`;
  };
  return body;
}

// A small, dismissable card in the corner (not a modal).
export function nudge(title, text, actions) {
  document.getElementById('nudge')?.remove();
  const el = document.createElement('div');
  el.id = 'nudge';
  el.className = 'nudge';
  el.innerHTML = `<b>${esc(title)}</b><p>${esc(text)}</p><div class="btns"></div>`;
  for (const [label, fn] of actions) {
    const b = document.createElement('button');
    b.className = 'btn';
    b.textContent = label;
    b.onclick = () => {
      el.remove();
      fn?.();
    };
    el.querySelector('.btns').append(b);
  }
  document.getElementById('hud').append(el);
  return el;
}

// Settings that only exist inside the app: who the architect is, quitting.
export async function appSettings(host, el) {
  let s;
  try {
    s = await host.get('/api/app/settings');
  } catch {
    return;
  }
  if (!s.app) return;
  el.innerHTML = `
    <h3>The architect</h3>
    <div class="form">
      <label class="sub"><input type="radio" name="arch" value="heuristic" ${s.settings.architect !== 'command' ? 'checked' : ''}/> built in: quick, always available</label>
      <label class="sub"><input type="radio" name="arch" value="command" ${s.settings.architect === 'command' ? 'checked' : ''}/> a program you choose, for example Claude Code</label>
      <input id="arch-cmd" value="${esc(s.settings.command)}" placeholder="claude -p" />
      <div class="sub">It gets a description of the house and what visitors want on stdin and prints a JSON plan. Quick sketches always use the built-in architect; your program gets the rooms people linger in.</div>
      <div class="btns"><button class="btn" id="arch-save">save</button></div>
      <div id="arch-msg"></div>
    </div>
    <h3>Your worlds</h3>
    <div class="sub">Saved as a git repository in <code>${esc(s.data_dir)}</code></div>
    <div class="btns" style="margin-top:8px"><button class="btn" id="app-quit">quit Branches</button></div>
  `;
  $('arch-save').onclick = async () => {
    const architect = el.querySelector('input[name=arch]:checked').value;
    try {
      await host.send('POST', '/api/app/settings', { architect, command: $('arch-cmd').value.trim() });
      $('arch-msg').innerHTML = '<span class="ok">saved</span>';
    } catch (e) {
      $('arch-msg').innerHTML = `<span class="err">${esc(e.message)}</span>`;
    }
  };
  $('app-quit').onclick = async () => {
    await host.send('POST', '/api/app/quit', {}).catch(() => {});
    document.body.innerHTML = '<div style="display:grid;place-items:center;height:100%;font-family:Georgia,serif;font-style:italic;font-size:22px;color:#556">Branches is resting. Open the app to come back.</div>';
  };
}

// ------------------------------------------------------------ the map

// Cells around you: where you have been, where everyone lingers (heat),
// grown houses, the version bands, spawn, and you.
export function openMap({ worldName, center, yaw, explored, heat, rooms, versions, ring, spawn, local }) {
  const body = openPanel(`
    <h2>${esc(worldName)}</h2>
    <div class="sub">${local ? 'Where you have walked.' : 'Where you have walked, and where everyone lingers.'} Each square is a house. Rings mark older versions.</div>
    <canvas id="map" width="560" height="560" style="width:100%;max-width:560px;aspect-ratio:1;margin-top:12px;border-radius:14px;background:#f3f1ea"></canvas>
    <div class="chips" style="margin-top:8px">
      <span class="chip"><span style="color:#5a8a6a">\u25a0</span> walked by you</span>
      ${local ? '' : '<span class="chip"><span style="color:#e9824a">\u25a0</span> lingered in</span>'}
      <span class="chip"><span style="color:#8a5ad9">\u25cf</span> grown house</span>
      <span class="chip">\u2605 spawn</span>
    </div>
  `);
  const cv = body.querySelector('#map');
  const ctx = cv.getContext('2d');
  const R = 16;
  const px = cv.width / (R * 2 + 1);
  const toX = (x) => (x - center.x + R) * px, toY = (z) => (z - center.z + R) * px;
  ctx.fillStyle = '#f3f1ea';
  ctx.fillRect(0, 0, cv.width, cv.height);
  const sp = spawn || [0, 0];
  // Version bands: band i starts where n*d/(d+ring) reaches i.
  const n = (versions || 0) + 1;
  for (let i = n - 1; i >= 1; i--) {
    const d = (ring * i) / (n - i);
    ctx.fillStyle = `rgba(120, 100, 160, ${0.04 + 0.03 * i})`;
    const x0 = toX(sp[0] - d), y0 = toY(sp[1] - d);
    ctx.fillRect(x0, y0, (2 * d + 1) * px, (2 * d + 1) * px);
  }
  // Every cell is a house on its lawn, with streets between.
  ctx.fillStyle = 'rgba(35, 34, 43, 0.07)';
  for (let x = center.x - R; x <= center.x + R; x++)
    for (let z = center.z - R; z <= center.z + R; z++) ctx.fillRect(toX(x) + px * 0.25, toY(z) + px * 0.25, px * 0.5, px * 0.42);
  const maxHeat = Math.max(1, ...(heat || []).map((h) => h[2] + h[3] * 0.3));
  for (const k of explored) {
    const [x, z] = k.split(',').map(Number);
    if (Math.abs(x - center.x) > R || Math.abs(z - center.z) > R) continue;
    ctx.fillStyle = 'rgba(90, 138, 106, 0.55)';
    ctx.fillRect(toX(x) + 1, toY(z) + 1, px - 2, px - 2);
  }
  for (const [x, z, inside, outside] of heat || []) {
    if (Math.abs(x - center.x) > R || Math.abs(z - center.z) > R) continue;
    const a = Math.min(0.85, Math.sqrt((inside + outside * 0.3) / maxHeat));
    ctx.fillStyle = `rgba(233, 130, 74, ${a})`;
    ctx.fillRect(toX(x) + 3, toY(z) + 3, px - 6, px - 6);
  }
  for (const r of rooms || []) {
    if (Math.abs(r.x - center.x) > R || Math.abs(r.z - center.z) > R) continue;
    ctx.fillStyle = '#8a5ad9';
    ctx.beginPath();
    ctx.arc(toX(r.x) + px / 2, toY(r.z) + px / 2, Math.min(px / 2.5, 3 + r.growth), 0, 7);
    ctx.fill();
  }
  ctx.fillStyle = '#23222b';
  ctx.font = `${Math.round(px * 0.8)}px Georgia`;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText('\u2605', toX(sp[0]) + px / 2, toY(sp[1]) + px / 2);
  // You: an arrow.
  const cx = toX(center.fx) + px / 2, cy = toY(center.fz) + px / 2;
  ctx.save();
  ctx.translate(cx, cy);
  ctx.rotate(-yaw);
  ctx.fillStyle = '#e94a6a';
  ctx.beginPath();
  ctx.moveTo(0, -px * 0.7);
  ctx.lineTo(px * 0.4, px * 0.4);
  ctx.lineTo(-px * 0.4, px * 0.4);
  ctx.closePath();
  ctx.fill();
  ctx.restore();
}
