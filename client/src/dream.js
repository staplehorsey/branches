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
export async function feel(world, x, z, score) {
  if (performance.now() - lastFeel < 600) return;
  lastFeel = performance.now();
  try {
    const r = await call('POST', '/api/app/compass', { world, x, z, score });
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
