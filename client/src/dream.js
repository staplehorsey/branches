// The dream: muses (links and words you love, distilled into aesthetics)
// and the compass (warmer, colder) that steer what the architect imagines.
// Press I for muses; = and - inside a house for warmer and colder.
import * as ui from './ui.js';

const esc = (s) => String(s ?? '').replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]);

async function call(method, path, body) {
  const r = await fetch(path, { method, headers: { 'content-type': 'application/json' }, body: body === undefined ? undefined : JSON.stringify(body) });
  const out = await r.json().catch(() => ({}));
  if (!r.ok) throw new Error(out.error || r.statusText);
  return out;
}

let lastFeel = 0;
export async function feel(world, x, z, score, space = null) {
  if (performance.now() - lastFeel < 600) return;
  lastFeel = performance.now();
  try {
    const r = await call('POST', '/api/app/compass', { world, x, z, score, space });
    ui.toast(score > 0 ? 'warmer' : 'colder', score > 0 ? `More like ${r.idea}. This house will deepen sooner, and new ones will lean this way.` : `Less like ${r.idea}. New houses will drift away from it.`);
  } catch (e) {
    ui.toast('Hmm', e.message);
  }
}

export async function openMuses() {
  const body = ui.openPanel('<h2>Muses</h2><p class="sub">Loading…</p>');
  let d;
  try {
    d = await call('GET', '/api/app/dream');
  } catch (e) {
    body.innerHTML = `<h2>Muses</h2><p>${esc(e.message)}</p>`;
    return;
  }
  const warm = d.compass.filter((m) => m.score > 0).length;
  const cold = d.compass.filter((m) => m.score < 0).length;
  body.innerHTML = `
    <h2>Muses</h2>
    <p class="sub">Things you love: a tweet, a story, a song, a picture's link, or just words. Claude distills each into an aesthetic, and new houses grow ideas from them. Inside any house, <b>=</b> says warmer and <b>-</b> says colder; the dream steps toward what's warm.</p>
    <div class="form">
      <input id="muse-url" placeholder="a link (https://…)" />
      <textarea id="muse-text" rows="3" placeholder="or words: a line you love, a feeling, a place (optional with a link)"></textarea>
      <div class="btns"><button class="btn" id="muse-add">add muse</button></div>
      <div id="muse-msg" class="sub"></div>
    </div>
    <div id="muse-list">${d.muses.length ? '' : '<p class="sub">No muses yet. The dream drifts on its own until you add some.</p>'}</div>
    <p class="sub">Compass: ${warm} warmer, ${cold} colder so far.</p>`;
  const list = body.querySelector('#muse-list');
  for (const m of [...d.muses].reverse()) {
    const el = document.createElement('div');
    el.className = 'muse';
    el.innerHTML = `<b>${esc(m.name)}</b> ${m.url ? `<a href="${esc(m.url)}" target="_blank" rel="noopener">link</a>` : ''} <button class="btn small">remove</button><div class="sub">${esc(m.essence)}</div>`;
    el.querySelector('button').onclick = async () => {
      await call('DELETE', `/api/app/muses/${encodeURIComponent(m.id)}`).catch(() => {});
      el.remove();
    };
    list.appendChild(el);
  }
  const add = body.querySelector('#muse-add');
  const msg = body.querySelector('#muse-msg');
  for (const el of body.querySelectorAll('input, textarea')) el.addEventListener('keydown', (e) => e.key !== 'Escape' && e.stopPropagation());
  add.onclick = async () => {
    const url = body.querySelector('#muse-url').value.trim();
    const text = body.querySelector('#muse-text').value.trim();
    if (!url && !text) return;
    add.disabled = true;
    msg.textContent = 'Reading it and distilling its aesthetic… (up to a minute)';
    try {
      await call('POST', '/api/app/muses', { url: url || null, text });
      openMuses();
    } catch (e) {
      msg.textContent = e.message;
      add.disabled = false;
    }
  };
}

// A small box over everything (the game or a space): warmer, colder, and
// a few words about why. G opens it.
export function openFeedback({ world, x, z, space = null, about = '' }) {
  if (document.getElementById('feedback')) return;
  document.exitPointerLock?.();
  const el = document.createElement('div');
  el.id = 'feedback';
  el.innerHTML = `<div class="card">
    <b>How is this?</b> <span class="sub">${esc(about)}</span>
    <textarea rows="3" placeholder="optional: what pulls you in, what you want more or less of, where it should go next"></textarea>
    <div class="btns"><button class="btn" data-s="1">warmer</button><button class="btn" data-s="-1">colder</button><button class="btn" data-s="0">just the words</button><button class="btn" data-s="x">cancel</button></div>
    <div class="sub">Esc to cancel · it steers the next spaces and houses</div>
  </div>`;
  document.body.appendChild(el);
  const ta = el.querySelector('textarea');
  const close = () => el.remove();
  ta.addEventListener('keydown', (e) => {
    e.stopPropagation();
    if (e.key === 'Escape') close();
  });
  el.addEventListener('mousedown', (e) => e.target === el && close());
  for (const b of el.querySelectorAll('button')) {
    b.onclick = async () => {
      if (b.dataset.s === 'x') return close();
      const score = +b.dataset.s;
      const note = ta.value.trim();
      if (!score && !note) return ta.focus();
      try {
        await call('POST', '/api/app/compass', { world, x, z, score, space, note: note || null });
        ui.toast(score > 0 ? 'warmer' : score < 0 ? 'colder' : 'noted', note || 'The dream will lean that way.');
        close();
      } catch (e) {
        ui.toast('Hmm', e.message);
      }
    };
  }
  setTimeout(() => ta.focus(), 30);
}

// Every key, in one place. ? opens it.
export function openHelp({ app = false } = {}) {
  if (document.getElementById('keyhelp')) return document.getElementById('keyhelp').remove();
  document.exitPointerLock?.();
  const row = (k, what) => `<tr><td><kbd>${k}</kbd></td><td>${what}</td></tr>`;
  const el = document.createElement('div');
  el.id = 'keyhelp';
  el.innerHTML = `<div class="card"><h3>Keys</h3><table>
    ${row('W A S D', 'walk')}${row('shift', 'faster')}${row('space', 'hop')}${row('mouse', 'look (click to capture)')}
    ${row('E', 'talk, read, play, visitor book, ride')}${row('F', 'more of this')}${row('Q', 'scooter')}${row('V', 'first / third person')}
    ${row('enter', 'chat')}${row('M', 'map')}${row('L', 'worlds and settings')}${row('N', 'notifications')}${row('R', 'residents')}${row('H', 'wake up at home')}
    ${app ? `<tr><td colspan="2" class="sub">the dream</td></tr>
    ${row('walk into the mist', 'at the back of a house: into its spaces')}
    ${row('=', 'warmer: more like this')}${row('-', 'colder: less like this')}${row('G', 'feedback: warmer or colder, in your words')}
    ${row('I', 'muses: links and words you love')}${row('Esc', 'out of a space, back to the house')}${row('`', 'talk to Claude about the game itself')}` : ''}
    ${row('?', 'this list')}
  </table><div class="sub">Esc or ? to close</div></div>`;
  document.body.appendChild(el);
  const close = (e) => {
    if (e.type === 'mousedown' || e.key === 'Escape' || e.key === '?') {
      e.stopImmediatePropagation();
      el.remove();
      removeEventListener('keydown', close, true);
    }
  };
  addEventListener('keydown', close, true);
  el.addEventListener('mousedown', close);
}
