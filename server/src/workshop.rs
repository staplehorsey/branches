//! The workshop: talk to Claude Code about the game itself, from inside it.
//!
//! Only when the app runs from its source code (scripts/dev.sh): messages go
//! to `claude -p` in the repository, one conversation that carries on until
//! you start a new one. What it says and does streams to everyone in the
//! game. When it has changed the client the page reloads; when it has
//! changed the server, the app rebuilds and restarts itself on the new code.

use crate::state::App;
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};

struct Bench {
    session: Option<String>,
    busy: Option<u32>,
    history: Vec<Value>,
}

static BENCH: Mutex<Bench> = Mutex::new(Bench { session: None, busy: None, history: Vec::new() });

/// When this server started: the page compares it to know a restart is done.
pub fn boot() -> u64 {
    static BOOT: OnceLock<u64> = OnceLock::new();
    *BOOT.get_or_init(crate::model::now_ms)
}

/// The repository the app is running from, if it is running from one.
pub fn source_root(app: &App) -> Option<PathBuf> {
    let root = app.cfg.client_dir.canonicalize().ok()?.parent()?.to_path_buf();
    (root.join("server/Cargo.toml").is_file() && root.join(".git").exists() && root.join("scripts/dev.sh").is_file()).then_some(root)
}

pub fn status(app: &App) -> Value {
    let why = if !app.cfg.app {
        Some("only in the app")
    } else if source_root(app).is_none() {
        Some("run Branches from its source code (scripts/dev.sh) to work on it from inside")
    } else if crate::agent::find_claude().is_none() {
        Some("Claude Code isn't installed")
    } else {
        None
    };
    let b = BENCH.lock().unwrap();
    json!({ "available": why.is_none(), "why": why, "busy": b.busy.is_some(), "boot": boot(), "history": b.history })
}

fn emit(app: &App, mut m: Value) {
    m["t"] = json!("workshop");
    m["at"] = json!(crate::model::now_ms());
    {
        let mut b = BENCH.lock().unwrap();
        b.history.push(m.clone());
        let n = b.history.len();
        if n > 300 {
            b.history.drain(..n - 300);
        }
    }
    let worlds: Vec<String> = app.uni.lock().unwrap().worlds.keys().cloned().collect();
    for w in worlds {
        app.broadcast(&w, &m);
    }
}

/// Start a new conversation next time.
pub fn reset(app: &App) {
    let mut b = BENCH.lock().unwrap();
    b.session = None;
    b.history.clear();
    drop(b);
    emit(app, json!({ "kind": "status", "text": "New conversation." }));
}

/// Stop what Claude is doing.
pub fn stop() {
    if let Some(pid) = BENCH.lock().unwrap().busy {
        let _ = Command::new("kill").arg(pid.to_string()).status();
    }
}

/// Hand a message to Claude Code. `place` says where the speaker is.
pub fn ask(app: &Arc<App>, text: &str, who: &str, place: &str) -> Result<(), String> {
    let root = source_root(app).ok_or("Branches isn't running from its source code")?;
    let claude = crate::agent::find_claude().ok_or("Claude Code isn't installed")?;
    let session = {
        let mut b = BENCH.lock().unwrap();
        if b.busy.is_some() {
            return Err("Claude is still working on the last message".into());
        }
        b.busy = Some(0);
        b.session.clone()
    };
    emit(app, json!({ "kind": "you", "who": who, "text": text }));
    let input = format!("[{who}, in the game: {place}]\n\n{text}");
    let app = app.clone();
    std::thread::spawn(move || {
        if let Err(e) = run(&app, &root, &claude, session, &input) {
            emit(&app, json!({ "kind": "error", "text": e }));
        }
        BENCH.lock().unwrap().busy = None;
        emit(&app, json!({ "kind": "idle" }));
    });
    Ok(())
}

const SYSTEM: &str = "You are being talked to from inside Branches, the game whose source code this repository is, by someone playing it right now. \
They want to change the game itself. Make the change they ask for, end to end. \
The client (client/, plain ES modules with three.js, no build step) and the server (server/, Rust with axum) are described in docs/ARCHITECTURE.md; read it when you need the lay of the land. \
When you finish, the game picks up your changes on its own: the page reloads for client changes, and the app rebuilds and restarts for server changes. So never start, stop or restart the server and never run scripts/dev.sh. \
If you change the server, make sure `cargo build --release` passes in server/ (and `cargo test --release` when you touched logic) before you finish. \
Commit finished work with git and a clear message; don't push. Don't touch inbox/. The player's worlds are not in this repository. \
Your replies show in a small panel in the game: keep them short and plain, say what you changed and how to see it in the game.";

fn git(root: &Path, args: &[&str]) -> String {
    Command::new("git").current_dir(root).args(args).output().map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default()
}

fn untracked(root: &Path) -> Vec<String> {
    git(root, &["ls-files", "--others", "--exclude-standard"]).lines().map(str::to_string).collect()
}

/// Files that differ from `base` (edited, committed since, or new), with
/// the git hash of what is in each now ("" when deleted).
fn touched(root: &Path, base: &str) -> std::collections::BTreeMap<String, String> {
    let mut files: Vec<String> = git(root, &["diff", "--name-only", base]).lines().map(str::to_string).collect();
    files.extend(untracked(root));
    files
        .into_iter()
        .map(|f| {
            let hash = if root.join(&f).is_file() { git(root, &["hash-object", "--", &f]).trim().to_string() } else { String::new() };
            (f, hash)
        })
        .collect()
}

fn run(app: &Arc<App>, root: &Path, claude: &Path, session: Option<String>, input: &str) -> Result<(), String> {
    let head = git(root, &["rev-parse", "HEAD"]).trim().to_string();
    // Uncommitted work from before doesn't count as Claude's change.
    let before = touched(root, &head);
    let mut cmd = Command::new(claude);
    cmd.current_dir(root)
        .env("PATH", crate::agent::path())
        .args(["-p", "--output-format", "stream-json", "--verbose", "--permission-mode", "acceptEdits", "--append-system-prompt", SYSTEM])
        .args(["--allowedTools", "Bash(cargo:*)", "Bash(git:*)", "Bash(node:*)", "Bash(ls:*)", "Bash(grep:*)", "Bash(find:*)"]);
    if let Some(s) = &session {
        cmd.args(["--resume", s]);
    }
    crate::agent::use_subscription(&mut cmd);
    let mut child = cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e| format!("could not start Claude Code: {e}"))?;
    BENCH.lock().unwrap().busy = Some(child.id());
    emit(app, json!({ "kind": "status", "text": if session.is_some() { "Claude is working…" } else { "Claude is starting a new conversation…" } }));
    let mut stdin = child.stdin.take().ok_or("no stdin")?;
    let input = input.to_string();
    std::thread::spawn(move || {
        let _ = stdin.write_all(input.as_bytes());
    });
    let mut stderr = child.stderr.take().ok_or("no stderr")?;
    let err = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = stderr.read_to_string(&mut s);
        s
    });
    let mut said = false;
    for line in BufReader::new(child.stdout.take().ok_or("no stdout")?).lines() {
        let Ok(line) = line else { break };
        let Ok(v) = serde_json::from_str::<Value>(&line) else { continue };
        if let Some(id) = v["session_id"].as_str() {
            BENCH.lock().unwrap().session = Some(id.to_string());
        }
        match v["type"].as_str() {
            Some("assistant") => {
                for block in v["message"]["content"].as_array().into_iter().flatten() {
                    match block["type"].as_str() {
                        Some("text") if !block["text"].as_str().unwrap_or("").trim().is_empty() => {
                            said = true;
                            emit(app, json!({ "kind": "claude", "text": block["text"] }));
                        }
                        Some("tool_use") => emit(app, json!({ "kind": "tool", "text": describe(root, block) })),
                        _ => {}
                    }
                }
            }
            Some("result") if v["is_error"] == true => emit(app, json!({ "kind": "error", "text": v["result"].as_str().unwrap_or("Claude stopped with an error") })),
            _ => {}
        }
    }
    let status = child.wait().map_err(|e| e.to_string())?;
    let err = err.join().unwrap_or_default();
    if !status.success() && !said {
        let why: String = err.trim().chars().take(400).collect();
        return Err(if why.is_empty() { format!("Claude Code stopped ({status})") } else { why });
    }

    // What Claude changed, committed or not.
    let changed: Vec<String> = touched(root, &head).into_iter().filter(|(f, h)| !f.starts_with("inbox/") && before.get(f) != Some(h)).map(|(f, _)| f).collect();
    if changed.iter().any(|f| f.starts_with("server/")) {
        emit(app, json!({ "kind": "status", "text": "Building the server…" }));
        let out = Command::new("cargo").current_dir(root.join("server")).env("PATH", crate::agent::path()).args(["build", "--release"]).output().map_err(|e| format!("could not run cargo: {e}"))?;
        if !out.status.success() {
            let log = String::from_utf8_lossy(&out.stderr);
            let tail: Vec<&str> = log.lines().rev().take(25).collect::<Vec<_>>().into_iter().rev().collect();
            return Err(format!("The server didn't build, so the game is still running the old version. Ask Claude to fix it.\n{}", tail.join("\n")));
        }
        emit(app, json!({ "kind": "restart", "changed": changed }));
        restart(root);
    } else if changed.iter().any(|f| f.starts_with("client/")) {
        emit(app, json!({ "kind": "reload", "changed": changed }));
    } else if !changed.is_empty() {
        emit(app, json!({ "kind": "status", "text": format!("Changed {}.", changed.join(", ")) }));
    }
    Ok(())
}

/// Restart on the new build: scripts/dev.sh stops this server and starts
/// the new one. It runs in its own process group so it outlives us.
fn restart(root: &Path) {
    let mut c = Command::new("bash");
    c.current_dir(root).arg("scripts/dev.sh").env("BRANCHES_NO_OPEN", "1").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        c.process_group(0);
    }
    if let Err(e) = c.spawn() {
        tracing::warn!("couldn't restart: {e}");
    }
}

/// One line for a tool call: "Edit client/src/ui.js", "Run cargo build".
fn describe(root: &Path, block: &Value) -> String {
    let name = block["name"].as_str().unwrap_or("tool");
    let i = &block["input"];
    let rel = |p: &str| p.strip_prefix(&format!("{}/", root.display())).unwrap_or(p).to_string();
    let detail = if let Some(p) = i["file_path"].as_str() {
        rel(p)
    } else if let Some(c) = i["command"].as_str() {
        c.lines().next().unwrap_or("").chars().take(80).collect()
    } else if let Some(p) = i["pattern"].as_str() {
        p.chars().take(60).collect()
    } else if let Some(d) = i["description"].as_str() {
        d.chars().take(80).collect()
    } else {
        String::new()
    };
    let verb = match name {
        "Bash" => "Run",
        "TodoWrite" => return "Update the plan".into(),
        n => n,
    };
    if detail.is_empty() { verb.to_string() } else { format!("{verb} {detail}") }
}
