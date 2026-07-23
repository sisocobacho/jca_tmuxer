use crate::config::Config;
use anyhow::{Result, bail};
use std::path::{Path, PathBuf};
use strsim::levenshtein;

#[derive(Debug, Clone)]
pub struct ResolvedProject {
    pub name: String,
    pub session_name: String,
    pub root: PathBuf,
}

pub fn resolve_project(config: &Config, input: &str) -> Result<ResolvedProject> {
    let input_path = Path::new(input);
    if input_path.exists() {
        let root = std::fs::canonicalize(input_path)?;
        let session_name = root
            .file_name()
            .map(|v| v.to_string_lossy().to_string())
            .unwrap_or_else(|| input.to_string());
        return Ok(ResolvedProject {
            name: session_name.clone(),
            session_name,
            root,
        });
    }

    if let Some(project) = config.projects.get(input) {
        let root_raw = project
            .root
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("project '{input}' has no root configured"))?;
        let root = expand_path(root_raw);
        if !root.exists() {
            bail!(
                "configured root does not exist for project '{input}': {}",
                root.display()
            );
        }
        return Ok(ResolvedProject {
            name: input.to_string(),
            session_name: input.to_string(),
            root,
        });
    }

    if let Some(root) = resolve_from_search_paths(config, input) {
        return Ok(ResolvedProject {
            name: input.to_string(),
            session_name: input.to_string(),
            root,
        });
    }

    let mut candidates: Vec<_> = config.projects.keys().cloned().collect();
    candidates.sort_by_key(|name| levenshtein(name, input));
    if candidates.is_empty() {
        bail!("unknown project '{input}'")
    }
    let suggestions = candidates
        .into_iter()
        .take(3)
        .collect::<Vec<_>>()
        .join(", ");
    bail!("unknown project '{input}'. suggestions: {suggestions}")
}

fn resolve_from_search_paths(config: &Config, input: &str) -> Option<PathBuf> {
    let mut case_insensitive = Vec::new();
    let mut prefixed = Vec::new();

    for raw in &config.search_paths {
        let path = expand_path(raw);
        let Ok(entries) = std::fs::read_dir(path) else {
            continue;
        };

        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let Ok(ft) = entry.file_type() else {
                continue;
            };
            if !ft.is_dir() {
                continue;
            }

            if name == input {
                return Some(entry.path());
            }
            if name.eq_ignore_ascii_case(input) {
                case_insensitive.push(entry.path());
            }
            if name.starts_with(input) {
                prefixed.push(entry.path());
            }
        }
    }

    if case_insensitive.len() == 1 {
        return case_insensitive.pop();
    }
    if prefixed.len() == 1 {
        return prefixed.pop();
    }
    None
}

pub fn expand_path(raw: &str) -> PathBuf {
    let expanded = shellexpand::full(raw)
        .map(|v| v.to_string())
        .unwrap_or_else(|_| raw.to_string());
    PathBuf::from(expanded)
}
