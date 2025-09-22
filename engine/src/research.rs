use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, io::Read, path::Path, time::SystemTime};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResearchArtifact {
    pub id: String,
    pub kind: String,
    pub path: String,
    pub ts: String,
    pub ttl: u64,
    pub tags: Vec<String>,
    pub checksum: String,
    pub git_commit: Option<String>,
    pub git_branch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ExpiringArtifact {
    pub path: String,
    pub expires_at: String,
    pub seconds_remaining: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TimeSpan {
    pub earliest: String,
    pub latest: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ResearchSummary {
    pub total: usize,
    pub by_kind: BTreeMap<String, usize>,
    pub by_tag: BTreeMap<String, usize>,
    pub by_branch: BTreeMap<String, usize>,
    pub untagged: usize,
    pub missing_git_commit: usize,
    pub time_span: Option<TimeSpan>,
    pub expiring_soon: Vec<ExpiringArtifact>,
    pub expired: Vec<ExpiringArtifact>,
}

fn kind_for(path: &Path) -> String {
    let p = path.to_string_lossy().to_lowercase();
    if p.contains("/prompts/") {
        return "prompt".into();
    }
    if p.contains("/policies/") {
        return "policy".into();
    }
    if p.contains("/schemas/") {
        return "schema".into();
    }
    if p.contains("/trace/golden/") {
        return "trace".into();
    }
    if p.contains("/docs/") || p.ends_with("readme.md") {
        return "doc".into();
    }
    if p.ends_with(".json") || p.ends_with(".yaml") || p.ends_with(".yml") {
        return "dataset".into();
    }
    "other".into()
}

fn adler32(bytes: &[u8]) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    const MOD: u32 = 65521;
    for chunk in bytes.chunks(5552) {
        for &x in chunk {
            a = (a + x as u32) % MOD;
            b = (b + a) % MOD;
        }
    }
    (b << 16) | a
}

fn ts_from(path: &Path) -> String {
    match fs::metadata(path).and_then(|m| m.modified()) {
        Ok(st) => match st.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(d) => chrono::DateTime::<chrono::Utc>::from(std::time::UNIX_EPOCH + d).to_rfc3339(),
            Err(_) => chrono::Utc::now().to_rfc3339(),
        },
        Err(_) => chrono::Utc::now().to_rfc3339(),
    }
}

pub fn build_index(root: &Path) -> anyhow::Result<Vec<ResearchArtifact>> {
    let mut out = Vec::new();
    let branch = git_branch().ok();
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if !matches!(ext, "md" | "json" | "jsonl" | "yaml" | "yml") {
            continue;
        }
        // read file
        let mut f = fs::File::open(path)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        let checksum = format!("{:08x}", adler32(&buf));
        let ts = ts_from(path);
        let ttl = if path.to_string_lossy().contains("trace/golden/") {
            0
        } else {
            14 * 24 * 3600
        };
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        let kind = kind_for(path);
        // tags from simple front-matter if present
        let mut tags = front_matter_tags(&buf);
        if tags.is_empty() && kind == "policy" {
            tags.push("policy".into());
        }
        let id = format!("{}#{}", rel, checksum);
        let git_commit = git_last_commit(path).ok();
        out.push(ResearchArtifact {
            id,
            kind,
            path: rel,
            ts,
            ttl,
            tags,
            checksum,
            git_commit,
            git_branch: branch.clone(),
        });
    }
    Ok(out)
}

pub fn build_index_multi(roots: &[std::path::PathBuf]) -> anyhow::Result<Vec<ResearchArtifact>> {
    use std::collections::HashSet;
    let mut all = Vec::new();
    let mut seen: HashSet<String> = HashSet::new(); // dedup by checksum
    for r in roots {
        let items = build_index(r)?;
        for a in items.into_iter() {
            if seen.insert(a.checksum.clone()) {
                all.push(a);
            }
        }
    }
    Ok(all)
}

fn parse_ts(ts: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(ts)
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
}

pub fn summarize(artifacts: &[ResearchArtifact], now: DateTime<Utc>) -> ResearchSummary {
    let mut by_kind: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_tag: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_branch: BTreeMap<String, usize> = BTreeMap::new();
    let mut untagged = 0usize;
    let mut missing_git_commit = 0usize;
    let mut earliest: Option<DateTime<Utc>> = None;
    let mut latest: Option<DateTime<Utc>> = None;
    let mut expiring_soon: Vec<ExpiringArtifact> = Vec::new();
    let mut expired: Vec<ExpiringArtifact> = Vec::new();
    let soon_window = Duration::hours(72);

    for art in artifacts {
        *by_kind.entry(art.kind.clone()).or_insert(0) += 1;
        if art.tags.is_empty() {
            untagged += 1;
        }
        for tag in &art.tags {
            *by_tag.entry(tag.clone()).or_insert(0) += 1;
        }
        match art.git_branch.as_ref() {
            Some(branch) if !branch.is_empty() => {
                *by_branch.entry(branch.clone()).or_insert(0) += 1;
            }
            _ => {
                *by_branch.entry("<unknown>".to_string()).or_insert(0) += 1;
            }
        }
        if art
            .git_commit
            .as_ref()
            .map(|s| s.is_empty())
            .unwrap_or(true)
        {
            missing_git_commit += 1;
        }

        if let Some(ts) = parse_ts(&art.ts) {
            earliest = Some(match earliest {
                Some(cur) => cur.min(ts),
                None => ts,
            });
            latest = Some(match latest {
                Some(cur) => cur.max(ts),
                None => ts,
            });

            if art.ttl > 0 {
                let expires = ts + Duration::seconds(art.ttl as i64);
                let remaining = expires - now;
                let record = ExpiringArtifact {
                    path: art.path.clone(),
                    expires_at: expires.to_rfc3339(),
                    seconds_remaining: remaining.num_seconds(),
                };
                if remaining <= Duration::zero() {
                    expired.push(record);
                } else if remaining <= soon_window {
                    expiring_soon.push(record);
                }
            }
        }
    }

    expiring_soon.sort_by_key(|item| item.seconds_remaining);
    expired.sort_by_key(|item| item.seconds_remaining);

    let time_span = match (earliest, latest) {
        (Some(a), Some(b)) => Some(TimeSpan {
            earliest: a.to_rfc3339(),
            latest: b.to_rfc3339(),
        }),
        _ => None,
    };

    ResearchSummary {
        total: artifacts.len(),
        by_kind,
        by_tag,
        by_branch,
        untagged,
        missing_git_commit,
        time_span,
        expiring_soon,
        expired,
    }
}

fn front_matter_tags(buf: &[u8]) -> Vec<String> {
    // Minimal YAML front-matter parser: --- ... --- at top
    let s = String::from_utf8_lossy(buf);
    let mut lines = s.lines();
    if !matches!(lines.next(), Some(l) if l.trim()=="---") {
        return Vec::new();
    }
    let mut tags: Vec<String> = Vec::new();
    let mut in_tags = false;
    for line in lines {
        let t = line.trim();
        if t == "---" {
            break;
        }
        if t.starts_with("tags:") {
            in_tags = true;
            continue;
        }
        if in_tags {
            if t.starts_with('-') {
                let val = t.trim_start_matches('-').trim();
                if !val.is_empty() {
                    tags.push(val.to_string());
                }
            } else if t.is_empty() {
                break;
            } else {
                in_tags = false;
            }
        }
    }
    tags
}

fn git_branch() -> anyhow::Result<String> {
    let out = std::process::Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        anyhow::bail!("git branch failed")
    }
}

fn git_last_commit(path: &Path) -> anyhow::Result<String> {
    let out = std::process::Command::new("git")
        .args([
            "log",
            "-n",
            "1",
            "--pretty=%h",
            "--",
            path.to_string_lossy().as_ref(),
        ])
        .output()?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        anyhow::bail!("git log failed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn artifact(
        kind: &str,
        path: &str,
        ts: &str,
        ttl: u64,
        tags: &[&str],
        branch: Option<&str>,
        commit: Option<&str>,
    ) -> ResearchArtifact {
        ResearchArtifact {
            id: format!("{}#{}", path, ttl),
            kind: kind.to_string(),
            path: path.to_string(),
            ts: ts.to_string(),
            ttl,
            tags: tags.iter().map(|t| t.to_string()).collect(),
            checksum: "deadbeef".into(),
            git_commit: commit.map(|c| c.to_string()),
            git_branch: branch.map(|b| b.to_string()),
        }
    }

    #[test]
    fn summary_counts_and_expiry() {
        let now = DateTime::parse_from_rfc3339("2024-01-02T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let arts = vec![
            artifact(
                "doc",
                "docs/a.md",
                "2024-01-01T00:00:00Z",
                86_400,
                &["alpha"],
                Some("main"),
                Some("abc123"),
            ),
            artifact(
                "policy",
                "policies/b.yaml",
                "2024-01-01T12:00:00Z",
                172_800,
                &["beta", "alpha"],
                Some("dev"),
                Some("def456"),
            ),
            artifact(
                "prompt",
                "prompts/c.md",
                "2024-01-05T00:00:00Z",
                0,
                &[],
                None,
                None,
            ),
        ];

        let summary = summarize(&arts, now);

        assert_eq!(summary.total, 3);
        assert_eq!(summary.by_kind.get("doc"), Some(&1));
        assert_eq!(summary.by_kind.get("policy"), Some(&1));
        assert_eq!(summary.by_kind.get("prompt"), Some(&1));
        assert_eq!(summary.by_tag.get("alpha"), Some(&2));
        assert_eq!(summary.by_tag.get("beta"), Some(&1));
        assert_eq!(summary.untagged, 1);
        assert_eq!(summary.missing_git_commit, 1);
        assert_eq!(summary.by_branch.get("main"), Some(&1));
        assert_eq!(summary.by_branch.get("dev"), Some(&1));
        assert_eq!(summary.by_branch.get("<unknown>"), Some(&1));

        let span = summary.time_span.expect("time span");
        assert_eq!(span.earliest, "2024-01-01T00:00:00+00:00");
        assert_eq!(span.latest, "2024-01-05T00:00:00+00:00");

        assert_eq!(summary.expiring_soon.len(), 1);
        let soon = &summary.expiring_soon[0];
        assert_eq!(soon.path, "policies/b.yaml");
        assert_eq!(soon.expires_at, "2024-01-03T12:00:00+00:00");
        assert_eq!(soon.seconds_remaining, 129_600);

        assert_eq!(summary.expired.len(), 1);
        let expired = &summary.expired[0];
        assert_eq!(expired.path, "docs/a.md");
        assert_eq!(expired.expires_at, "2024-01-02T00:00:00+00:00");
        assert_eq!(expired.seconds_remaining, 0);
    }
}
