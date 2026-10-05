//! GitHub, for backing up and sharing worlds. Everything is optional and
//! one click at a time:
//!
//! 1. Connect: reuse the GitHub CLI's login if there is one (`gh auth
//!    token`), else GitHub's device sign-in when this build has an OAuth app
//!    id, else a pasted token.
//! 2. Fork the main repository to your account, and keep your worlds pushed
//!    to a `worlds` branch of the fork, tags included.
//! 3. Share: open a pull request adding your worlds to `universe.json` in
//!    the main repository, so a door to them appears in the Commons.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub const UPSTREAM: &str = "staplehorsey/branches";
const API: &str = "https://api.github.com";
const UA: &str = "branches-app";

/// A GitHub OAuth app id with device flow enabled, set at build time.
pub fn client_id() -> Option<&'static str> {
    option_env!("BRANCHES_GITHUB_CLIENT_ID").filter(|s| !s.is_empty())
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Account {
    pub token: String,
    pub login: String,
    #[serde(default)]
    pub fork: Option<String>,
    #[serde(default)]
    pub shared: Option<String>,
    #[serde(default)]
    pub last_push: u64,
    /// Why the last backup failed, shown in the app until one succeeds.
    #[serde(default)]
    pub push_error: Option<String>,
}

fn file(root: &Path) -> PathBuf {
    root.join(".branches/github.json")
}

pub fn load(root: &Path) -> Option<Account> {
    serde_json::from_slice(&std::fs::read(file(root)).ok()?).ok()
}

pub fn save(root: &Path, a: &Account) {
    let _ = std::fs::write(file(root), serde_json::to_vec_pretty(a).unwrap_or_default());
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder().timeout_global(Some(std::time::Duration::from_secs(30))).build().into()
}

fn call(method: &str, url: &str, token: &str, body: Option<Value>) -> Result<Value, String> {
    let a = agent();
    let auth = format!("Bearer {token}");
    let res = match (method, body) {
        ("GET", _) => a.get(url).header("authorization", &auth).header("user-agent", UA).header("accept", "application/vnd.github+json").call(),
        (m, b) => {
            let req = match m {
                "POST" => a.post(url),
                "PUT" => a.put(url),
                _ => a.patch(url),
            };
            req.header("authorization", &auth).header("user-agent", UA).header("accept", "application/vnd.github+json").send_json(b.unwrap_or(json!({})))
        }
    };
    match res {
        Ok(mut r) => r.body_mut().read_json::<Value>().map_err(|e| e.to_string()),
        Err(ureq::Error::StatusCode(code)) => Err(format!("GitHub said {code}")),
        Err(e) => Err(e.to_string()),
    }
}

/// The GitHub CLI's token, if the CLI is installed and signed in.
pub fn gh_cli_token() -> Option<String> {
    for gh in ["gh", "/opt/homebrew/bin/gh", "/usr/local/bin/gh"] {
        if let Ok(out) = std::process::Command::new(gh).args(["auth", "token"]).output() {
            let t = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if out.status.success() && !t.is_empty() {
                return Some(t);
            }
        }
    }
    None
}

pub fn whoami(token: &str) -> Result<String, String> {
    let u = call("GET", &format!("{API}/user"), token, None)?;
    u["login"].as_str().map(str::to_string).ok_or_else(|| "GitHub did not say who you are".into())
}

/// Device sign-in, step one: a code to type at github.com/login/device.
pub fn device_start() -> Result<Value, String> {
    let id = client_id().ok_or("this build has no GitHub sign-in app")?;
    let mut r = agent()
        .post("https://github.com/login/device/code")
        .header("accept", "application/json")
        .header("user-agent", UA)
        .send_json(json!({ "client_id": id, "scope": "public_repo" }))
        .map_err(|e| e.to_string())?;
    r.body_mut().read_json::<Value>().map_err(|e| e.to_string())
}

/// Device sign-in, step two: the token once the person has approved.
pub fn device_poll(device_code: &str) -> Result<Option<String>, String> {
    let id = client_id().ok_or("this build has no GitHub sign-in app")?;
    let mut r = agent()
        .post("https://github.com/login/oauth/access_token")
        .header("accept", "application/json")
        .header("user-agent", UA)
        .send_json(json!({ "client_id": id, "device_code": device_code, "grant_type": "urn:ietf:params:oauth:grant-type:device_code" }))
        .map_err(|e| e.to_string())?;
    let v: Value = r.body_mut().read_json().map_err(|e| e.to_string())?;
    Ok(v["access_token"].as_str().map(str::to_string))
}

/// Fork the main repository for this account and wait until GitHub has
/// made it (forking is asynchronous; a brand-new fork can take a little
/// while to accept pushes). The owner of the main repository gets the main
/// repository itself back, which is fine: their worlds go on its `worlds`
/// branch.
pub fn fork(token: &str) -> Result<String, String> {
    let f = call("POST", &format!("{API}/repos/{UPSTREAM}/forks"), token, Some(json!({ "default_branch_only": true })))
        .map_err(|e| if e.contains("403") || e.contains("404") { format!("{e}: this token can't fork repositories (it needs the public_repo scope)") } else { e })?;
    let name = f["full_name"].as_str().map(str::to_string).ok_or("the fork did not come back")?;
    for _ in 0..20 {
        if call("GET", &format!("{API}/repos/{name}"), token, None).is_ok() {
            return Ok(name);
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
    Err(format!("GitHub is still making {name}; try again in a minute"))
}

/// The account a repository belongs to.
pub fn owner(full_name: &str) -> &str {
    full_name.split('/').next().unwrap_or(full_name)
}

/// Push the worlds repository (branch and version tags) to the fork.
pub fn push(root: &Path, a: &Account) -> Result<(), String> {
    let fork = a.fork.as_deref().ok_or("not forked yet")?;
    let repo = git2::Repository::open(root).map_err(|e| e.to_string())?;
    let head = repo.head().map_err(|e| e.to_string())?;
    let branch = head.shorthand().unwrap_or("master").to_string();
    let mut remote = repo.remote_anonymous(&format!("https://github.com/{fork}.git")).map_err(|e| e.to_string())?;
    let mut cb = git2::RemoteCallbacks::new();
    let token = a.token.clone();
    cb.credentials(move |_, _, _| git2::Cred::userpass_plaintext("x-access-token", &token));
    let mut po = git2::ProxyOptions::new();
    po.auto();
    let mut opts = git2::PushOptions::new();
    opts.remote_callbacks(cb);
    opts.proxy_options(po);
    let mut specs = vec![format!("+refs/heads/{branch}:refs/heads/worlds")];
    repo.tag_foreach(|_, name| {
        if let Ok(n) = std::str::from_utf8(name) {
            specs.push(format!("+{n}:{n}"));
        }
        true
    })
    .map_err(|e| e.to_string())?;
    remote.push(&specs, Some(&mut opts)).map_err(|e| e.to_string())
}

/// The same push with the git command line, when it is installed (Xcode's
/// command line tools or Homebrew). The token goes in an environment
/// variable, never on the command line.
fn push_with_git(root: &Path, a: &Account) -> Result<(), String> {
    let fork = a.fork.as_deref().ok_or("not forked yet")?;
    let git = ["/opt/homebrew/bin/git", "/usr/local/bin/git", "/Library/Developer/CommandLineTools/usr/bin/git", "/Applications/Xcode.app/Contents/Developer/usr/bin/git", "/usr/bin/git"]
        .into_iter()
        .find(|g| Path::new(g).exists() && (!g.starts_with("/usr/bin") || !cfg!(target_os = "macos") || Path::new("/Library/Developer/CommandLineTools").exists()))
        .ok_or("git is not installed")?;
    let auth = format!("Authorization: Basic {}", b64_encode(format!("x-access-token:{}", a.token).as_bytes()));
    let out = std::process::Command::new(git)
        .current_dir(root)
        .args(["push", "--force", &format!("https://github.com/{fork}.git"), "HEAD:refs/heads/worlds", "refs/tags/*:refs/tags/*"])
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "http.extraHeader")
        .env("GIT_CONFIG_VALUE_0", auth)
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() { Ok(()) } else { Err(String::from_utf8_lossy(&out.stderr).trim().replace(&a.token, "***")) }
}

/// Push, retrying while a new fork settles, then with the git command line.
pub fn push_patiently(root: &Path, a: &Account) -> Result<(), String> {
    let mut last = String::new();
    for i in 0..4 {
        match push(root, a) {
            Ok(()) => return Ok(()),
            Err(e) => last = e,
        }
        if i == 1 {
            match push_with_git(root, a) {
                Ok(()) => return Ok(()),
                Err(e) => tracing::warn!("git push failed too: {e}"),
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(2 + 2 * i));
    }
    Err(last)
}

/// Bring the fork's copy of the code up to date with the main repository
/// (GitHub's "sync fork"). Your worlds live on another branch and are not
/// touched.
pub fn sync_fork_code(a: &Account) -> Result<String, String> {
    let fork = a.fork.as_deref().ok_or("not forked yet")?;
    if fork == UPSTREAM {
        return Ok("this is the main repository".into());
    }
    let up = call("GET", &format!("{API}/repos/{UPSTREAM}"), &a.token, None)?;
    let base = up["default_branch"].as_str().unwrap_or("main");
    let r = call("POST", &format!("{API}/repos/{fork}/merge-upstream"), &a.token, Some(json!({ "branch": base })))?;
    Ok(r["message"].as_str().unwrap_or("up to date").to_string())
}

/// A house to put a door to in the Commons.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SharedRoom {
    pub world: String,
    pub x: i32,
    pub z: i32,
    pub title: String,
}

/// Open a pull request adding these worlds and houses to the main
/// universe.json. Sharing again adds to what you shared before.
pub fn share(a: &Account, worlds: &[(String, String)], rooms: &[SharedRoom]) -> Result<String, String> {
    let fork = a.fork.as_deref().ok_or("not forked yet")?;
    let fork_owner = owner(fork).to_string();
    let up = call("GET", &format!("{API}/repos/{UPSTREAM}"), &a.token, None)?;
    let base = up["default_branch"].as_str().unwrap_or("main").to_string();
    // Start from the main repository's latest universe.json.
    let cur = call("GET", &format!("{API}/repos/{UPSTREAM}/contents/universe.json?ref={base}"), &a.token, None)?;
    let text = cur["content"].as_str().map(|c| c.replace('\n', "")).and_then(|c| b64_decode(&c)).unwrap_or_default();
    let mut uni: Value = serde_json::from_slice(&text).unwrap_or_else(|_| json!({ "shared": [] }));
    if !uni["shared"].is_array() {
        uni["shared"] = json!([]);
    }
    let shared = uni["shared"].as_array_mut().unwrap();
    let before = shared.iter().position(|e| e["owner"] == a.login.as_str()).map(|i| shared.remove(i));
    let mut all_worlds: Vec<Value> = before.as_ref().and_then(|e| e["worlds"].as_array().cloned()).unwrap_or_default();
    for (id, name) in worlds {
        if !all_worlds.iter().any(|w| w["id"] == id.as_str()) {
            all_worlds.push(json!({ "id": id, "name": name }));
        }
    }
    let mut all_rooms: Vec<SharedRoom> = before.as_ref().and_then(|e| serde_json::from_value(e["rooms"].clone()).ok()).unwrap_or_default();
    for r in rooms {
        all_rooms.retain(|o| !(o.world == r.world && o.x == r.x && o.z == r.z));
        all_rooms.push(r.clone());
    }
    shared.push(json!({ "owner": a.login, "repo": fork, "branch": "worlds", "worlds": all_worlds, "rooms": all_rooms }));
    let body = serde_json::to_vec_pretty(&uni).map_err(|e| e.to_string())?;
    // A branch in the fork, from the main repository's tip.
    let tip = call("GET", &format!("{API}/repos/{UPSTREAM}/git/ref/heads/{base}"), &a.token, None)?;
    let sha = tip["object"]["sha"].as_str().ok_or("no tip")?;
    let branch = format!("share-{}", crate::model::now_ms() / 1000);
    call("POST", &format!("{API}/repos/{fork}/git/refs"), &a.token, Some(json!({ "ref": format!("refs/heads/{branch}"), "sha": sha })))
        .map_err(|e| format!("{e}: could not make a branch in {fork} (is the fork up to date? try sync)"))?;
    let what = if rooms.is_empty() { format!("{}'s worlds", a.login) } else { format!("{} ({})", rooms.iter().map(|r| r.title.as_str()).collect::<Vec<_>>().join(", "), a.login) };
    let mut put = json!({ "message": format!("Share {what}"), "content": b64_encode(&body), "branch": branch });
    if let Some(s) = cur["sha"].as_str() {
        put["sha"] = json!(s);
    }
    call("PUT", &format!("{API}/repos/{fork}/contents/universe.json"), &a.token, Some(put))?;
    let mut lines: Vec<String> = all_rooms.iter().map(|r| format!("- the house \u{201c}{}\u{201d} at {} {},{}", r.title, r.world, r.x, r.z)).collect();
    lines.extend(worlds.iter().map(|(id, n)| format!("- the world {n} (`{id}`)")));
    let pr = call(
        "POST",
        &format!("{API}/repos/{UPSTREAM}/pulls"),
        &a.token,
        Some(json!({
            "title": format!("Add {what} to the Commons"),
            "head": format!("{fork_owner}:{branch}"),
            "base": base,
            "body": format!("Adds doors in the Commons to:\n\n{}\n\nThey are read from the `worlds` branch of {fork}.\n\nOpened by Branches for Mac.", lines.join("\n")),
        })),
    )?;
    pr["html_url"].as_str().map(str::to_string).ok_or_else(|| "the pull request did not open".into())
}

const B64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn b64_encode(data: &[u8]) -> String {
    let mut out = String::new();
    for c in data.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        for i in 0..4 {
            if i <= c.len() {
                out.push(B64[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

fn b64_decode(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0;
    for ch in s.bytes() {
        if ch == b'=' {
            break;
        }
        let v = B64.iter().position(|b| *b == ch)? as u32;
        buf = buf << 6 | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits & 0xff) as u8);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    #[test]
    fn base64_round_trips() {
        for s in ["", "a", "ab", "abc", "{\"shared\": []}\n"] {
            assert_eq!(super::b64_decode(&super::b64_encode(s.as_bytes())).unwrap(), s.as_bytes());
        }
        assert_eq!(super::b64_encode(b"hello"), "aGVsbG8=");
    }
}
