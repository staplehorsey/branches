//! Running your agent: the architect command, the night shift and merges
//! all go through here, so they behave the same.
//!
//! Claude Code gets special care so that "use my subscription" just works:
//! the app finds `claude` wherever the installer put it (apps opened from
//! Finder don't see your shell's PATH), and it removes `ANTHROPIC_API_KEY`
//! from the agent's environment so Claude Code signs in with your Claude
//! account instead of billing an API key.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
}

/// Folders where agents tend to be installed.
fn extra_dirs() -> Vec<PathBuf> {
    let h = home();
    let mut dirs = vec![
        h.join(".claude/local"),
        h.join(".local/bin"),
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
        h.join(".npm-global/bin"),
        h.join(".bun/bin"),
        h.join(".volta/bin"),
        h.join("bin"),
    ];
    // Node version managers keep one bin folder per version; newest last.
    if let Ok(rd) = std::fs::read_dir(h.join(".nvm/versions/node")) {
        let mut v: Vec<PathBuf> = rd.flatten().map(|e| e.path().join("bin")).collect();
        v.sort();
        dirs.extend(v.into_iter().rev());
    }
    dirs
}

/// PATH for agents: yours, plus the usual install folders.
pub fn path() -> String {
    let mut parts: Vec<String> = std::env::var("PATH").unwrap_or_default().split(':').filter(|s| !s.is_empty()).map(str::to_string).collect();
    for d in extra_dirs() {
        let s = d.to_string_lossy().to_string();
        if !parts.contains(&s) {
            parts.push(s);
        }
    }
    parts.join(":")
}

/// Where `claude` (Claude Code) is installed, if it is.
pub fn find_claude() -> Option<PathBuf> {
    path().split(':').map(|d| Path::new(d).join("claude")).find(|p| p.is_file())
}

fn is_claude(cmd: &str) -> bool {
    cmd.split_whitespace().next().is_some_and(|w| w == "claude" || w.ends_with("/claude"))
}

/// A shell command for an agent, ready to spawn.
pub fn command(cmd: &str) -> Command {
    let mut c = Command::new("/bin/sh");
    c.arg("-c").arg(cmd).env("PATH", path());
    if is_claude(cmd) {
        c.env_remove("ANTHROPIC_API_KEY");
    }
    c
}

/// Run `cmd` with `input` on stdin. Its stdout, or why not.
pub fn run(cmd: &str, input: &str, dir: Option<&Path>, max: Duration) -> Result<String, String> {
    use std::io::{Read, Write};
    use std::process::Stdio;
    let mut c = command(cmd);
    if let Some(d) = dir {
        c.current_dir(d);
    }
    let mut child = c.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e| format!("could not start `{cmd}`: {e}"))?;
    let mut stdin = child.stdin.take().ok_or("no stdin")?;
    let input = input.to_string();
    std::thread::spawn(move || {
        let _ = stdin.write_all(input.as_bytes());
    });
    let read = |mut r: Box<dyn Read + Send>| {
        std::thread::spawn(move || {
            let mut s = String::new();
            let _ = r.read_to_string(&mut s);
            s
        })
    };
    let out = read(Box::new(child.stdout.take().ok_or("no stdout")?));
    let err = read(Box::new(child.stderr.take().ok_or("no stderr")?));
    let started = Instant::now();
    loop {
        match child.try_wait().map_err(|e| e.to_string())? {
            Some(status) => {
                let (out, err) = (out.join().unwrap_or_default(), err.join().unwrap_or_default());
                if status.success() {
                    return Ok(out);
                }
                let why = if err.trim().is_empty() { out } else { err };
                let why: String = why.trim().chars().take(400).collect();
                return Err(if status.code() == Some(127) { format!("`{}` is not installed (or not on the PATH)", cmd.split_whitespace().next().unwrap_or(cmd)) } else if why.is_empty() { format!("exited with {status}") } else { why });
            }
            None if started.elapsed() > max => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("no answer within {}s", max.as_secs()));
            }
            None => std::thread::sleep(Duration::from_millis(150)),
        }
    }
}

/// A quiet folder for the architect to run in, so a coding agent doesn't
/// wander through whatever directory the app was started from.
pub fn workdir(root: &Path) -> PathBuf {
    let d = root.join(".branches/agent");
    let _ = std::fs::create_dir_all(&d);
    d
}

/// Ask the agent for a tiny JSON reply: is it installed, signed in, working?
pub fn check(cmd: &str, root: &Path) -> Result<Duration, String> {
    let started = Instant::now();
    let out = run(cmd, "Reply with exactly this JSON and nothing else: {\"ok\": true}", Some(&workdir(root)), Duration::from_secs(120))?;
    match crate::architect::extract_json(&out) {
        Some(v) if v["ok"] == true => Ok(started.elapsed()),
        _ => {
            let said: String = out.trim().chars().take(300).collect();
            if said.to_lowercase().contains("login") || said.to_lowercase().contains("api key") {
                Err(format!("Claude Code needs you to sign in: open Terminal, run `claude`, and type /login. It said: {said}"))
            } else {
                Err(format!("it answered, but not with JSON: {said}"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn runs_commands_and_reports_failures() {
        let d = std::time::Duration::from_secs(5);
        assert_eq!(super::run("cat", "hi", None, d).unwrap(), "hi");
        assert!(super::run("definitely-not-a-command-xyz", "", None, d).unwrap_err().contains("not installed"));
        assert!(super::run("sleep 3", "", None, std::time::Duration::from_millis(300)).unwrap_err().contains("no answer"));
        assert!(super::is_claude("claude -p --model sonnet") && !super::is_claude("python3 claude.py"));
    }
}
