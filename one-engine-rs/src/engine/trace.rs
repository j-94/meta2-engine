use anyhow::Context;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;

pub async fn append_jsonl(obj: &serde_json::Value) -> anyhow::Result<()> {
    fs::create_dir_all("trace").await.ok();
    let mut f = OpenOptions::new().create(true).append(true)
        .open("trace/ledger.jsonl").await.context("open ledger")?;
    f.write_all((obj.to_string()+"\n").as_bytes()).await.context("write ledger")?;
    Ok(())
}
