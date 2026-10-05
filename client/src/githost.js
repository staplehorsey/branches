// Worlds read straight from a GitHub repository: someone's shared worlds,
// walked into through a door in the Commons. Read-only; to change a world,
// fork it and run Branches for Mac. Untouched houses come from the same
// generator the app uses, so doors line up.
//
// Origin format: gh://<owner>/<repo>@<branch>
import { defaultRoom, THEMES } from './procgen.js';

export const GH = 'gh://';

export function isGit(origin) {
  return origin.startsWith(GH);
}

export class GitHost {
  constructor(origin) {
    const m = /^gh:\/\/([^/]+)\/([^@]+)@(.+)$/.exec(origin);
    if (!m) throw new Error('not a git world address');
    [, this.owner, this.repo, this.branch] = m;
    this.cache = new Map();
  }

  raw(path) {
    return `https://raw.githubusercontent.com/${this.owner}/${this.repo}/refs/heads/${this.branch}/${path}`;
  }

  async json(path, fallback) {
    if (!this.cache.has(path)) {
      this.cache.set(
        path,
        fetch(this.raw(path))
          .then((r) => (r.ok ? r.json() : fallback))
          .catch(() => fallback),
      );
    }
    return this.cache.get(path);
  }

  async worlds() {
    const index = await this.json('worlds/index.json', []);
    return Promise.all(index.map((w) => this.json(`worlds/${w.id}/world.json`, null))).then((l) => l.filter(Boolean));
  }

  async room(w, x, z) {
    const stored = await this.json(`worlds/${w}/rooms/${x},${z}.json`, null);
    if (stored) return stored;
    const all = await this.worlds();
    const me = all.find((m) => m.id === w);
    if (!me) throw new Error('no such world');
    return defaultRoom(me, all.filter((o) => o.id !== w), x, z);
  }

  async get(path) {
    const u = new URL(path, 'http://x');
    let m;
    if (u.pathname === '/api/themes') return { themes: THEMES };
    if (u.pathname === '/api/worlds') return (await this.worlds()).map((w) => ({ id: w.id, name: w.name, tagline: w.tagline, online: 0, rooms_grown: 0, portal_policy: w.portal_policy }));
    if ((m = u.pathname.match(/^\/api\/worlds\/([^/]+)$/))) {
      const manifest = await this.json(`worlds/${m[1]}/world.json`, null);
      if (!manifest) throw new Error('no such world');
      return { manifest, versions: [], ring: 6 };
    }
    if ((m = u.pathname.match(/^\/api\/worlds\/([^/]+)\/chunk$/))) {
      const index = await this.json(`worlds/${m[1]}/index.json`, []);
      const cells = index.map(([x, z, growth, theme, claimed]) => ({ x, z, growth, theme, claimed, portals: 0, building: false }));
      return { cells };
    }
    if ((m = u.pathname.match(/^\/api\/worlds\/([^/]+)\/map$/))) {
      const heat = await this.json(`worlds/${m[1]}/heat.json`, {});
      const manifest = await this.json(`worlds/${m[1]}/world.json`, {});
      return { spawn: manifest.spawn, heat: Object.entries(heat).map(([k, h]) => [...k.split(',').map(Number), h.inside || 0, h.outside || 0, h.visits || 0]), rooms: [], versions: 0, ring: 6 };
    }
    if ((m = u.pathname.match(/^\/api\/worlds\/([^/]+)\/rooms\/(-?\d+)\/(-?\d+)$/))) {
      const r = await this.room(m[1], Number(m[2]), Number(m[3]));
      const theme = THEMES.find((t) => t.id === r.theme);
      return {
        ...r,
        world: m[1],
        theme_name: theme?.name || r.theme,
        things: r.things || [],
        log: (r.log || []).slice(-60).reverse(),
        growth: r.chambers.length - 1,
        next_growth_at: 0,
        leaning: [],
        investors: [],
        version: null,
        band: 0,
        owner: this.owner,
      };
    }
    if (u.pathname.startsWith('/api/players/')) return { inbox: [], prefs: {} };
    throw new Error('not found');
  }

  async send() {
    throw new Error(`This world lives in ${this.owner}'s repository. Fork ${this.owner}/${this.repo} and open it in Branches for Mac to change it.`);
  }

  // A file in the repository, as text (attractions, for example).
  async text(path) {
    const r = await fetch(this.raw(path));
    if (!r.ok) throw new Error(`could not load ${path}`);
    return r.text();
  }
}
