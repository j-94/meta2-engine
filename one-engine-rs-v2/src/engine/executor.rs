use anyhow::Context;
use tokio::process::Command;
use super::types::Policy;

#[derive(Debug)]
pub enum Action {
    Cli(String)
}

pub struct ExecResult {
    pub ok: bool,
    pub drift: bool,
    pub stdout: String
}

pub async fn execute(action: Action, _policy: &Policy) -> anyhow::Result<ExecResult> {
    match action {
        Action::Cli(cmd) => {
            let out = Command::new("bash").arg("-lc").arg(&cmd)
                .output().await
                .with_context(|| format!("failed to spawn: {}", cmd))?;
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            Ok(ExecResult{ ok: out.status.success(), drift: false, stdout })
        }
    }
}
