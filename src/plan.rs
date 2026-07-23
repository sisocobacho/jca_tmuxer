use crate::adhoc::{parse_adhoc, window_name};
use crate::config::{Config, PaneConfig, ProjectConfig, WindowConfig};
use crate::resolver::{ResolvedProject, expand_path};
use anyhow::{Result, bail};
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct WindowPlan {
    pub name: String,
    pub layout: String,
    pub panes: Vec<PanePlan>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PanePlan {
    pub command: String,
    pub cwd: PathBuf,
    pub size: Option<u8>,
}

pub fn build_windows(
    cfg: &Config,
    project: &ResolvedProject,
    adhoc_raw: &[String],
) -> Result<Vec<WindowPlan>> {
    let project_cfg = cfg.projects.get(&project.name).cloned().unwrap_or_default();
    let mut windows_cfg = resolve_project_windows(cfg, project_cfg);

    windows_cfg.extend(build_adhoc_window_configs(adhoc_raw, project));

    normalize_window_names(&mut windows_cfg);

    windows_cfg
        .iter()
        .map(|w| {
            to_plan(
                w,
                &project.root,
                cfg.defaults.layout.as_deref().unwrap_or("stacked"),
            )
        })
        .collect()
}

pub fn build_adhoc_windows(
    project: &ResolvedProject,
    adhoc_raw: &[String],
) -> Result<Vec<WindowPlan>> {
    let mut windows_cfg = build_adhoc_window_configs(adhoc_raw, project);
    normalize_window_names(&mut windows_cfg);
    windows_cfg
        .iter()
        .map(|w| to_plan(w, &project.root, "stacked"))
        .collect()
}

fn build_adhoc_window_configs(
    adhoc_raw: &[String],
    project: &ResolvedProject,
) -> Vec<WindowConfig> {
    let adhoc = parse_adhoc(adhoc_raw, &project.root);
    let mut windows_cfg = Vec::with_capacity(adhoc.len());
    for (idx, cmd) in adhoc.iter().enumerate() {
        windows_cfg.push(WindowConfig {
            name: window_name(idx + 1),
            command: Some(cmd.command.clone()),
            directory: Some(cmd.directory.to_string_lossy().to_string()),
            layout: Some("stacked".to_string()),
            panes: Vec::new(),
        });
    }
    windows_cfg
}

fn resolve_project_windows(cfg: &Config, project_cfg: ProjectConfig) -> Vec<WindowConfig> {
    if project_cfg.windows.is_empty() {
        return cfg.defaults.windows.clone();
    }
    if !project_cfg.extend {
        return project_cfg.windows;
    }

    let mut out = cfg.defaults.windows.clone();
    for project_window in project_cfg.windows {
        if let Some(idx) = out.iter().position(|w| w.name == project_window.name) {
            out[idx] = merge_window(out[idx].clone(), project_window);
        } else {
            out.push(project_window);
        }
    }
    out
}

fn merge_window(base: WindowConfig, over: WindowConfig) -> WindowConfig {
    WindowConfig {
        name: over.name,
        command: over.command.or(base.command),
        directory: over.directory.or(base.directory),
        layout: over.layout.or(base.layout),
        panes: if over.panes.is_empty() {
            base.panes
        } else {
            over.panes
        },
    }
}

fn to_plan(w: &WindowConfig, project_root: &Path, default_layout: &str) -> Result<WindowPlan> {
    let layout = w
        .layout
        .clone()
        .unwrap_or_else(|| default_layout.to_string());
    let mut panes = if !w.panes.is_empty() {
        w.panes
            .iter()
            .map(|pane| pane_to_plan(pane, project_root, w.directory.as_deref()))
            .collect::<Result<Vec<_>>>()?
    } else if let Some(command) = w.command.as_ref() {
        let cwd = resolve_dir(w.directory.as_deref(), project_root);
        if !cwd.exists() {
            bail!(
                "window '{}' directory does not exist: {}",
                w.name,
                cwd.display()
            );
        }
        vec![PanePlan {
            command: command.clone(),
            cwd,
            size: None,
        }]
    } else {
        bail!("window '{}' has neither command nor panes", w.name)
    };

    if panes.is_empty() {
        bail!("window '{}' has no panes", w.name);
    }

    if panes.len() == 1 {
        panes[0].size = None;
    }

    Ok(WindowPlan {
        name: w.name.clone(),
        layout,
        panes,
    })
}

fn pane_to_plan(
    pane: &PaneConfig,
    project_root: &Path,
    window_dir: Option<&str>,
) -> Result<PanePlan> {
    let cwd = resolve_dir(pane.directory.as_deref().or(window_dir), project_root);
    if !cwd.exists() {
        bail!("pane directory does not exist: {}", cwd.display());
    }
    Ok(PanePlan {
        command: pane.command.clone(),
        cwd,
        size: pane.size,
    })
}

fn resolve_dir(dir: Option<&str>, project_root: &Path) -> PathBuf {
    match dir {
        None => project_root.to_path_buf(),
        Some("<project_root>") => project_root.to_path_buf(),
        Some(value) => expand_path(value),
    }
}

fn normalize_window_names(windows: &mut [WindowConfig]) {
    let mut seen: HashMap<String, usize> = HashMap::new();
    for window in windows {
        let count = seen.entry(window.name.clone()).or_insert(0);
        *count += 1;
        if *count > 1 {
            window.name = format!("{}-{}", window.name, count);
        }
    }
}
