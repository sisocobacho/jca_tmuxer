use crate::resolver::expand_path;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdhocCommand {
    pub directory: PathBuf,
    pub command: String,
}

pub fn parse_adhoc(raw: &[String], project_root: &Path) -> Vec<AdhocCommand> {
    raw.iter()
        .map(|value| parse_one(value, project_root))
        .collect()
}

fn parse_one(raw: &str, project_root: &Path) -> AdhocCommand {
    let mut split_idx = None;
    let mut escaped = false;
    for (i, ch) in raw.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == ':' {
            split_idx = Some(i);
            break;
        }
    }

    let (dir, cmd) = if let Some(i) = split_idx {
        let dir = &raw[..i];
        let cmd = raw[i + 1..].replace("\\:", ":");
        let looks_like_path = dir.starts_with('/') || dir.starts_with("~") || dir.contains('/');
        if looks_like_path && !cmd.trim().is_empty() {
            (expand_path(dir), cmd)
        } else {
            (project_root.to_path_buf(), raw.replace("\\:", ":"))
        }
    } else {
        (project_root.to_path_buf(), raw.replace("\\:", ":"))
    };

    AdhocCommand {
        directory: dir,
        command: cmd,
    }
}

pub fn window_name(index: usize) -> String {
    format!("adhoc-{index}")
}
