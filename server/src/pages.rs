//! Your own page: a web address that opens your worlds, to share with
//! anyone. The app builds the static web version of Branches (the same
//! client it serves), adds `site.json` pointing at your fork's `worlds`
//! branch, commits it to a `gh-pages` branch, pushes it to your fork and
//! turns on GitHub Pages. No Actions needed, so it works on forks as is.
//!
//! Visitors walk your worlds read-only, straight from your repository, with
//! a door back to the Commons. To grow them, they get the app.

use crate::github::{self, Account};
use git2::{Oid, Repository, Signature};
use serde_json::json;
use std::path::Path;

/// Build a git tree from a folder, with some files replaced or added.
fn tree_from(repo: &Repository, dir: &Path, rel: &str, extra: &[(String, Vec<u8>)]) -> Result<Oid, String> {
    let e = |e: git2::Error| e.to_string();
    let mut tb = repo.treebuilder(None).map_err(e)?;
    let mut names: Vec<String> = vec![];
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            let path = entry.path();
            let sub = if rel.is_empty() { name.clone() } else { format!("{rel}/{name}") };
            if path.is_dir() {
                let t = tree_from(repo, &path, &sub, extra)?;
                tb.insert(&name, t, 0o040000).map_err(e)?;
            } else {
                let bytes = match extra.iter().find(|(p, _)| *p == sub) {
                    Some((_, b)) => b.clone(),
                    None => std::fs::read(&path).map_err(|x| x.to_string())?,
                };
                tb.insert(&name, repo.blob(&bytes).map_err(e)?, 0o100644).map_err(e)?;
            }
            names.push(sub);
        }
    }
    // Files that don't exist on disk at this level.
    for (p, b) in extra {
        let Some(name) = p.strip_prefix(&if rel.is_empty() { String::new() } else { format!("{rel}/") }) else { continue };
        if !name.contains('/') && !names.contains(p) {
            tb.insert(name, repo.blob(b).map_err(e)?, 0o100644).map_err(e)?;
        }
    }
    tb.write().map_err(e)
}

/// The served index.html, with paths made relative for a static host.
fn static_index(client: &Path) -> Result<Vec<u8>, String> {
    let s = std::fs::read_to_string(client.join("index.html")).map_err(|e| format!("no web client to publish: {e}"))?;
    Ok(s.replace("href=\"/style.css\"", "href=\"style.css\"").replace("\"/vendor/three.module.js\"", "\"./vendor/three.module.js\"").replace("src=\"/src/main.js\"", "src=\"src/main.js\"").into_bytes())
}

/// Where the page lives, from the Pages API (or the usual address).
fn page_url(a: &Account, fork: &str) -> String {
    github::call("GET", &format!("https://api.github.com/repos/{fork}/pages"), &a.token, None)
        .ok()
        .and_then(|v| v["html_url"].as_str().map(str::to_string))
        .unwrap_or_else(|| {
            let (owner, repo) = fork.split_once('/').unwrap_or((fork, "branches"));
            format!("https://{}.github.io/{repo}/", owner.to_lowercase())
        })
}

/// A link that opens your worlds on the main Branches page (works for
/// everyone, with or without your own page).
pub fn share_link(fork: &str) -> String {
    let (owner, repo) = fork.split_once('/').unwrap_or((fork, "branches"));
    let main = github::UPSTREAM.split_once('/').map(|(o, r)| format!("https://{}.github.io/{r}/", o.to_lowercase())).unwrap_or_default();
    format!("{main}?at=gh://{owner}/{repo}@worlds/w/the-lush/0,0")
}

/// Build, push and switch on the page. Returns (page url, share link).
pub fn publish(root: &Path, client: &Path, a: &Account) -> Result<(String, String), String> {
    let fork = a.fork.clone().ok_or("connect GitHub first")?;
    let share = share_link(&fork);
    if fork == github::UPSTREAM {
        // The main repository's page is the main world's front door; don't
        // replace it. Its link to your worlds works the same.
        return Ok((share.clone(), share));
    }
    let (owner, repo_name) = fork.split_once('/').unwrap_or((&fork, "branches"));
    let site = json!({
        "home": format!("gh://{owner}/{repo_name}@worlds"),
        "owner": owner,
        "about": format!("{owner}'s worlds in Branches"),
    });
    let extra = vec![
        ("index.html".to_string(), static_index(client)?),
        ("site.json".to_string(), serde_json::to_vec_pretty(&site).unwrap_or_default()),
        (".nojekyll".to_string(), vec![]),
    ];
    let repo = Repository::open(root).map_err(|e| e.to_string())?;
    let tree = repo.find_tree(tree_from(&repo, client, "", &extra)?).map_err(|e| e.to_string())?;
    let parent = repo.find_reference("refs/heads/gh-pages").ok().and_then(|r| r.peel_to_commit().ok());
    if parent.as_ref().is_some_and(|p| p.tree_id() == tree.id()) {
        // Nothing changed; still make sure Pages is on.
    } else {
        let sig = Signature::now("Branches", "app@branches.local").map_err(|e| e.to_string())?;
        let parents: Vec<&git2::Commit> = parent.iter().collect();
        repo.commit(Some("refs/heads/gh-pages"), &sig, &sig, &format!("{owner}'s page for Branches"), &tree, &parents).map_err(|e| e.to_string())?;
    }
    github::push_refs(root, a, &["+refs/heads/gh-pages:refs/heads/gh-pages".to_string()])?;
    let body = json!({ "source": { "branch": "gh-pages", "path": "/" } });
    let api = format!("https://api.github.com/repos/{fork}/pages");
    if github::call("POST", &api, &a.token, Some(body.clone())).is_err() {
        // Already on (409): point it at gh-pages.
        github::call("PUT", &api, &a.token, Some(json!({ "source": { "branch": "gh-pages", "path": "/" }, "build_type": "legacy" })))
            .or_else(|e| if e.contains("204") || e.contains("EOF") || e.contains("empty") { Ok(json!({})) } else { Err(e) })
            .map_err(|e| format!("could not switch on GitHub Pages for {fork}: {e}"))?;
    }
    Ok((page_url(a, &fork), share))
}

#[cfg(test)]
mod tests {
    #[test]
    fn builds_a_static_site_tree() {
        let dir = std::env::temp_dir().join(format!("branches-pages-{}", std::process::id()));
        let client = dir.join("client");
        std::fs::create_dir_all(client.join("src")).unwrap();
        std::fs::write(client.join("index.html"), "<link href=\"/style.css\"><script type=\"module\" src=\"/src/main.js\"></script>").unwrap();
        std::fs::write(client.join("src/main.js"), "boot()").unwrap();
        let repo = git2::Repository::init(dir.join("repo")).unwrap();
        let extra = vec![("index.html".to_string(), super::static_index(&client).unwrap()), ("site.json".to_string(), b"{}".to_vec())];
        let t = repo.find_tree(super::tree_from(&repo, &client, "", &extra).unwrap()).unwrap();
        let index = repo.find_blob(t.get_name("index.html").unwrap().id()).unwrap();
        assert!(std::str::from_utf8(index.content()).unwrap().contains("src=\"src/main.js\""));
        assert!(t.get_name("site.json").is_some());
        assert!(t.get_path(std::path::Path::new("src/main.js")).is_ok());
        assert_eq!(super::share_link("ada/branches"), "https://staplehorsey.github.io/branches/?at=gh://ada/branches@worlds/w/the-lush/0,0");
        let _ = std::fs::remove_dir_all(dir);
    }
}
