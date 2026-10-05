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
export async function openRoomPanel({ host, world, x, z, themes }) {
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
    <div class="form">
      <textarea id="log-text" rows="2" maxlength="400" placeholder="leave a note, or ask the architect for something (more water, a reading nook, stars…)"></textarea>
      <div class="btns">
        <button class="btn" data-kind="note">leave a note</button>
        <button class="btn" data-kind="request">ask the architect</button>
        <button class="btn" data-kind="praise">♥ this place</button>
      </div>
      <div id="log-msg"></div>
    </div>
    <div class="log" id="log-list" style="margin-top:10px">${room.log.map(entryHtml).join('') || '<div class="sub">No one has written here yet.</div>'}</div>
    ${room.claim ? '' : `
      <h3>Claim this address</h3>
      <div class="sub">Tend it: rename it, choose its theme, open doors to other worlds, and let your own agents build here.</div>
      <div class="row" style="margin-top:8px"><input id="claim-title" maxlength="60" placeholder="${esc(me.name)}'s house" /><button class="btn" id="claim">claim</button></div>
      <div id="claim-msg"></div>`}
    ${mine ? ownerTools(room, themes) : ''}
  `;
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
