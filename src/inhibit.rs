//! Holding an OS sleep inhibitor.
//!
//! Linux: `systemd-inhibit ... cat` reading from a pipe we own, so the lock is
//! released even if eep is SIGKILLed (cat sees EOF and exits).
//! macOS: `caffeinate -w <our pid>`, which exits when we do.

use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

pub struct Inhibitor {
    child: Child,
    pub backend: String,
}

fn which(name: &str) -> Option<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|dir| dir.join(name))
        .find(|p| {
            p.metadata()
                .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        })
}

impl Inhibitor {
    pub fn start(why: &str, display: bool) -> Result<Self, String> {
        let (mut cmd, backend) = if cfg!(target_os = "macos") && which("caffeinate").is_some() {
            let flags = format!("-ims{}", if display { "d" } else { "" });
            let mut cmd = Command::new("caffeinate");
            cmd.args([flags.as_str(), "-w", &std::process::id().to_string()]);
            (cmd, format!("caffeinate {flags}"))
        } else if which("systemd-inhibit").is_some() {
            let what = if display { "sleep:idle" } else { "sleep" };
            let mut cmd = Command::new("systemd-inhibit");
            cmd.args([
                &format!("--what={what}"),
                "--who=eep",
                &format!("--why={why}"),
                "--mode=block",
                "cat",
            ]);
            (cmd, format!("systemd-inhibit --what={what}"))
        } else {
            return Err("no sleep inhibitor found (need systemd-inhibit or caffeinate)".into());
        };

        cmd.stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        // New session: keeps the terminal's ctrl-c / hangup away from the child.
        // SAFETY: setsid is async-signal-safe.
        unsafe {
            cmd.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }
        let child = cmd.spawn().map_err(|e| format!("{backend}: {e}"))?;
        let mut inh = Inhibitor { child, backend };

        // Give it a moment to fail loudly (no logind, polkit refusal, ...).
        let deadline = Instant::now() + Duration::from_millis(300);
        while Instant::now() < deadline {
            if let Ok(Some(status)) = inh.child.try_wait() {
                let mut err = String::new();
                if let Some(mut e) = inh.child.stderr.take() {
                    let _ = e.read_to_string(&mut err);
                }
                let err = err.trim();
                let why = if err.is_empty() {
                    status
                        .code()
                        .map_or_else(|| status.to_string(), |c| c.to_string())
                } else {
                    err.to_string()
                };
                return Err(format!("{} exited: {why}", inh.backend));
            }
            sleep(Duration::from_millis(20));
        }
        Ok(inh)
    }

    pub fn alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }
}

impl Drop for Inhibitor {
    fn drop(&mut self) {
        drop(self.child.stdin.take());
        if self.alive() {
            // SAFETY: plain kill(2) on our own child's pid.
            unsafe { libc::kill(self.child.id() as libc::pid_t, libc::SIGTERM) };
        }
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if !self.alive() {
                return;
            }
            sleep(Duration::from_millis(10));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
