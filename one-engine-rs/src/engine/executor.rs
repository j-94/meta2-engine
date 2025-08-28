use std::{fs, path::Path};
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use futures::future;

pub struct ExecResult { pub ok: bool, pub drift: bool, pub stdout: String, pub artifact: Option<String> }

pub enum Action {
    WriteFile { path: String, content: String },
    RunCli { cmd: String }
}

pub async fn execute(a: Action) -> anyhow::Result<ExecResult> {
    match a {
        Action::WriteFile { path, content } => {
            if let Some(dir) = Path::new(&path).parent() { fs::create_dir_all(dir).ok(); }
            fs::write(&path, content)?;
            Ok(ExecResult { ok: Path::new(&path).exists(), drift: false, stdout: format!("WROTE {}", &path), artifact: Some(path) })
        }
        Action::RunCli { cmd } => {
            let out = timeout(Duration::from_secs(30),
                Command::new("bash").arg("-lc").arg(&cmd).output()).await??;
            let ok = out.status.success();
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            Ok(ExecResult{ ok, drift:false, stdout, artifact:None })
        }
    }
}

/// run a batch of CLI actions concurrently
pub async fn execute_parallel(cmds: Vec<String>) -> Vec<ExecResult> {
    let futs = cmds.into_iter().map(|c| async {
        execute(Action::RunCli{cmd:c}).await.unwrap_or(ExecResult{ok:false,drift:false,stdout:"error".into(),artifact:None})
    });
    future::join_all(futs).await
}
