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

/// Fork the main repository (GitHub returns the existing fork if there is one).
pub fn fork(token: &str) -> Result<String, String> {
    let f = call("POST", &format!("{API}/repos/{UPSTREAM}/forks"), token, Some(json!({ "default_branch_only": true })))?;
    f["full_name"].as_str().map(str::to_string).ok_or_else(|| "the fork did not come back".into())
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
    let mut opts = git2::PushOptions::new();
    opts.remote_callbacks(cb);
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

/// Open a pull request adding these worlds to the main universe.json.
pub fn share(a: &Account, worlds: &[(String, String)]) -> Result<String, String> {
    let fork = a.fork.as_deref().ok_or("not forked yet")?;
    let up = call("GET", &format!("{API}/repos/{UPSTREAM}"), &a.token, None)?;
    let base = up["default_branch"].as_str().unwrap_or("main").to_string();
    // Start from the main repository's latest universe.json.
    let cur = call("GET", &format!("{API}/repos/{UPSTREAM}/contents/universe.json?ref={base}"), &a.token, None)?;
    let text = cur["content"].as_str().map(|c| c.replace('\n', "")).and_then(|c| b64_decode(&c)).unwrap_or_default();
    let mut uni: Value = serde_json::from_slice(&text).unwrap_or_else(|_| json!({ "shared": [] }));
    let entry = json!({
        "owner": a.login,
        "repo": fork,
        "branch": "worlds",
        "worlds": worlds.iter().map(|(id, name)| json!({ "id": id, "name": name })).collect::<Vec<_>>(),
    });
    let shared = uni["shared"].as_array_mut().ok_or("universe.json has no shared list")?;
    shared.retain(|e| e["owner"] != a.login);
    shared.push(entry);
    let body = serde_json::to_vec_pretty(&uni).map_err(|e| e.to_string())?;
    // A branch in the fork, from the main repository's tip.
    let tip = call("GET", &format!("{API}/repos/{UPSTREAM}/git/ref/heads/{base}"), &a.token, None)?;
    let sha = tip["object"]["sha"].as_str().ok_or("no tip")?;
    let branch = format!("share-{}", crate::model::now_ms() / 1000);
    call("POST", &format!("{API}/repos/{fork}/git/refs"), &a.token, Some(json!({ "ref": format!("refs/heads/{branch}"), "sha": sha })))?;
    let mut put = json!({ "message": format!("Share {}'s worlds", a.login), "content": b64_encode(&body), "branch": branch });
    if let Some(s) = cur["sha"].as_str() {
        put["sha"] = json!(s);
    }
    call("PUT", &format!("{API}/repos/{fork}/contents/universe.json"), &a.token, Some(put))?;
    let names: Vec<&str> = worlds.iter().map(|(_, n)| n.as_str()).collect();
    let pr = call(
        "POST",
        &format!("{API}/repos/{UPSTREAM}/pulls"),
        &a.token,
        Some(json!({
            "title": format!("Add {}'s worlds to the Commons", a.login),
            "head": format!("{}:{branch}", a.login),
            "base": base,
            "body": format!("Adds a door in the Commons to {}'s worlds ({}), kept on the `worlds` branch of {fork}.\n\nOpened by Branches for Mac.", a.login, names.join(", ")),
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
