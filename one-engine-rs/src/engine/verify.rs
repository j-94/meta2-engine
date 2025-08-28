use std::path::Path;
use super::executor::ExecResult;

pub fn deliverable_exists(res: &ExecResult) -> bool {
    res.ok && res.artifact.as_deref().map(Path::new).map(|p| p.exists()).unwrap_or(false)
}
