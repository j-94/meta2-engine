use super::types::Policy;
use anyhow::{anyhow, Context};
use tokio::process::Command;

#[derive(Debug)]
pub enum Action {
    Cli(String),
}

pub struct ExecResult {
    pub ok: bool,
    pub drift: bool,
    pub stdout: String,
}

pub async fn execute(action: Action, _policy: &Policy) -> anyhow::Result<ExecResult> {
    match action {
        Action::Cli(cmd) => {
            // Capability gate (simple heuristic). If STRICT_CAPS=1, block risky ops.
            if let Some(cap) = detect_capability(&cmd) {
                if std::env::var("STRICT_CAPS").ok().as_deref() == Some("1") {
                    return Err(anyhow!("capability gate blocked: {}", cap));
                }
            }
            let out = Command::new("bash")
                .arg("-lc")
                .arg(&cmd)
                .output()
                .await
                .with_context(|| format!("failed to spawn: {}", cmd))?;
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            Ok(ExecResult {
                ok: out.status.success(),
                drift: false,
                stdout,
            })
        }
    }
}

fn detect_capability(cmd: &str) -> Option<&'static str> {
    let s = cmd.to_lowercase();
    if s.contains("curl ") || s.contains("wget ") {
        return Some("network");
    }
    if s.contains(" rm ") || s.contains("rm -rf") || s.contains(" mv ") {
        return Some("file_write");
    }
    if s.contains("git push") || s.contains("gh release") {
        return Some("identity");
    }
    None
}
