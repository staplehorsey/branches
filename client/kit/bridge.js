// Injected into every space Claude makes. It is the whole contract between
// a space and the game: branches.onward(), branches.back(), branches.say(),
// and the game's reserved keys, which always reach the game.
(() => {
  const post = (m) => parent.postMessage({ branches: m.t, ...m }, '*');
  const RESERVED = new Set(['Escape', 'Equal', 'Minus', 'NumpadAdd', 'NumpadSubtract', 'Backquote', 'KeyG']);
  let sayEl = null;
  let sayTimer = 0;
  let carried = [];
  try {
    carried = JSON.parse(new URLSearchParams(location.search).get('carried') || '[]');
  } catch {}
  window.branches = {
    carried: Array.isArray(carried) ? carried.map(String).slice(0, 20) : [],
    carry(items) {
      post({ t: 'carry', items: (Array.isArray(items) ? items : [items]).map(String).slice(0, 20) });
    },
    onward() {
      post({ t: 'onward' });
    },
    back() {
      post({ t: 'back' });
    },
    say(text, seconds = 5) {
      if (!sayEl) {
        sayEl = document.createElement('div');
        sayEl.style.cssText = 'position:fixed;left:50%;bottom:12%;transform:translateX(-50%);max-width:70vw;padding:10px 16px;border-radius:12px;background:rgba(0,0,0,.45);color:#fff;font:16px/1.45 Georgia,serif;text-align:center;pointer-events:none;z-index:2147483647;transition:opacity .8s;opacity:0';
        (document.body || document.documentElement).appendChild(sayEl);
      }
      sayEl.textContent = String(text);
      sayEl.style.opacity = '1';
      clearTimeout(sayTimer);
      sayTimer = setTimeout(() => (sayEl.style.opacity = '0'), seconds * 1000);
    },
  };
  addEventListener(
    'keydown',
    (e) => {
      if (RESERVED.has(e.code) || e.key === '?') {
        e.preventDefault();
        e.stopImmediatePropagation();
        post({ t: 'key', code: e.key === '?' ? 'Help' : e.code });
      }
    },
    true,
  );
  // When pointer lock ends (Esc), tell the game so it can show its bar.
  document.addEventListener('pointerlockchange', () => post({ t: 'lock', locked: !!document.pointerLockElement }));
  addEventListener('error', (e) => post({ t: 'error', message: String(e.message || e.error || 'error') }));
})();
