use one_engine::research;
use std::{fs::File, io::Write, path::PathBuf};

fn main() -> anyhow::Result<()> {
    let mut roots: Vec<PathBuf> = vec![PathBuf::from(".")];
    let mut out = PathBuf::from("research/index.jsonl");
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--root" => {
                if let Some(v) = args.next() {
                    roots.push(PathBuf::from(v));
                }
            }
            "--out" => {
                if let Some(v) = args.next() {
                    out = PathBuf::from(v);
                }
            }
            _ => {}
        }
    }
    let artifacts = research::build_index_multi(&roots)?;
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut f = File::create(&out)?;
    for a in artifacts {
        let line = serde_json::to_string(&a)?;
        f.write_all(line.as_bytes())?;
        f.write_all(b"\n")?;
    }
    eprintln!("wrote {}", out.display());
    Ok(())
}
