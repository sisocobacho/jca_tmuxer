use crate::cli::Args;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    pub search_paths: Vec<String>,
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default)]
    pub projects: BTreeMap<String, ProjectConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Defaults {
    #[serde(default)]
    pub layout: Option<String>,
    #[serde(default)]
    pub windows: Vec<WindowConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ProjectConfig {
    pub root: Option<String>,
    #[serde(default)]
    pub extend: bool,
    #[serde(default)]
    pub windows: Vec<WindowConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct WindowConfig {
    pub name: String,
    pub command: Option<String>,
    pub directory: Option<String>,
    pub layout: Option<String>,
    #[serde(default)]
    pub panes: Vec<PaneConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PaneConfig {
    pub command: String,
    pub directory: Option<String>,
    pub size: Option<u8>,
}

pub fn load_from_args(args: &Args) -> Result<Config> {
    load_from_args_with_missing_policy(args, false)
}

pub fn load_from_args_allow_missing(args: &Args) -> Result<Config> {
    load_from_args_with_missing_policy(args, true)
}

fn load_from_args_with_missing_policy(args: &Args, allow_missing: bool) -> Result<Config> {
    if let Some(path) = args.config.as_ref() {
        if allow_missing && !path.exists() {
            return Ok(builtin_defaults());
        }
        return Ok(merge(builtin_defaults(), load_path(path)?));
    }

    if let Ok(path) = std::env::var("JCA_TMUXER_CONFIG") {
        let config_path = Path::new(&path);
        if allow_missing && !config_path.exists() {
            return Ok(builtin_defaults());
        }
        return Ok(merge(builtin_defaults(), load_path(config_path)?));
    }

    let mut base = load_default_user_config()?;
    if let Some(local_path) = discover_local_config(std::env::current_dir()?) {
        let local_cfg = load_path(&local_path)?;
        base = merge(base, local_cfg);
    }
    Ok(base)
}

fn load_default_user_config() -> Result<Config> {
    let Some(dirs) = ProjectDirs::from("", "", "jca_tmuxer") else {
        return Ok(Config::default());
    };
    let path = dirs.config_dir().join("config.yaml");
    if !path.exists() {
        return Ok(builtin_defaults());
    }
    let cfg = load_path(&path)?;
    Ok(merge(builtin_defaults(), cfg))
}

pub fn load_path(path: &Path) -> Result<Config> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read config at {}", path.display()))?;
    let cfg: Config = serde_yaml::from_str(&raw)
        .with_context(|| format!("invalid YAML config at {}", path.display()))?;
    Ok(cfg)
}

pub fn discover_local_config(start: PathBuf) -> Option<PathBuf> {
    let mut current = start;
    loop {
        let candidate = current.join(".jca_tmuxer.yaml");
        if candidate.exists() {
            return Some(candidate);
        }
        if !current.pop() {
            return None;
        }
    }
}

pub fn merge(mut base: Config, overlay: Config) -> Config {
    if !overlay.search_paths.is_empty() {
        base.search_paths = overlay.search_paths;
    }

    if overlay.defaults.layout.is_some() {
        base.defaults.layout = overlay.defaults.layout;
    }
    if !overlay.defaults.windows.is_empty() {
        base.defaults.windows = overlay.defaults.windows;
    }

    for (name, project) in overlay.projects {
        base.projects.insert(name, project);
    }

    base
}

pub fn builtin_defaults() -> Config {
    Config {
        search_paths: vec![
            "~/code".to_string(),
            "~/projects".to_string(),
            "~/workspace".to_string(),
        ],
        defaults: Defaults {
            layout: Some("main-vertical".to_string()),
            windows: vec![
                WindowConfig {
                    name: "editor".to_string(),
                    command: Some("nvim".to_string()),
                    directory: Some("<project_root>".to_string()),
                    layout: None,
                    panes: Vec::new(),
                },
                WindowConfig {
                    name: "git".to_string(),
                    command: Some("lazygit".to_string()),
                    directory: Some("<project_root>".to_string()),
                    layout: None,
                    panes: Vec::new(),
                },
                WindowConfig {
                    name: "terminal".to_string(),
                    command: Some("bash".to_string()),
                    directory: Some("<project_root>".to_string()),
                    layout: None,
                    panes: Vec::new(),
                },
                WindowConfig {
                    name: "opencode".to_string(),
                    command: Some("opencode".to_string()),
                    directory: Some("<project_root>".to_string()),
                    layout: None,
                    panes: Vec::new(),
                },
            ],
        },
        projects: BTreeMap::new(),
    }
}

pub fn resolve_write_path(args: &Args) -> Result<PathBuf> {
    if let Some(path) = args.config.as_ref() {
        return Ok(path.clone());
    }

    if let Ok(path) = std::env::var("JCA_TMUXER_CONFIG") {
        return Ok(PathBuf::from(path));
    }

    let Some(dirs) = ProjectDirs::from("", "", "jca_tmuxer") else {
        anyhow::bail!("cannot determine user config directory")
    };
    Ok(dirs.config_dir().join("config.yaml"))
}

pub fn save_project(
    args: &Args,
    project_name: &str,
    root: &Path,
    windows: Option<Vec<WindowConfig>>,
) -> Result<bool> {
    let config_path = resolve_write_path(args)?;
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config directory {}", parent.display()))?;
    }

    let mut cfg = if config_path.exists() {
        load_path(&config_path)?
    } else {
        builtin_defaults()
    };

    if cfg.projects.contains_key(project_name) {
        return Ok(false);
    }

    let mut project = ProjectConfig {
        root: Some(root.to_string_lossy().to_string()),
        ..ProjectConfig::default()
    };
    if let Some(default_windows) = windows {
        project.windows = default_windows;
    }

    cfg.projects.insert(project_name.to_string(), project);

    let serialized = serde_yaml::to_string(&cfg)?;
    fs::write(&config_path, serialized)
        .with_context(|| format!("failed to write config at {}", config_path.display()))?;
    Ok(true)
}

pub fn remove_project(args: &Args, project_name: &str) -> Result<bool> {
    let config_path = resolve_write_path(args)?;
    if !config_path.exists() {
        return Ok(false);
    }

    let mut cfg = load_path(&config_path)?;
    let removed = cfg.projects.remove(project_name).is_some();
    if !removed {
        return Ok(false);
    }

    let serialized = serde_yaml::to_string(&cfg)?;
    fs::write(&config_path, serialized)
        .with_context(|| format!("failed to write config at {}", config_path.display()))?;

    Ok(true)
}

pub fn ensure_config_exists(args: &Args) -> Result<PathBuf> {
    let config_path = resolve_write_path(args)?;
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config directory {}", parent.display()))?;
    }

    if !config_path.exists() {
        let serialized = serde_yaml::to_string(&builtin_defaults())?;
        fs::write(&config_path, serialized)
            .with_context(|| format!("failed to write config at {}", config_path.display()))?;
    }

    Ok(config_path)
}

pub fn open_in_editor(path: &Path) -> Result<()> {
    let editor = std::env::var("VISUAL")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| {
            std::env::var("EDITOR")
                .ok()
                .filter(|v| !v.trim().is_empty())
        })
        .unwrap_or_else(|| "vi".to_string());

    let status = Command::new("sh")
        .arg("-c")
        .arg("editor_cmd=\"$1\"; target=\"$2\"; exec $editor_cmd \"$target\"")
        .arg("sh")
        .arg(editor.as_str())
        .arg(path.as_os_str())
        .status()
        .with_context(|| format!("failed to launch editor '{}'", editor))?;

    if !status.success() {
        anyhow::bail!("editor '{}' exited with status {}", editor, status);
    }

    Ok(())
}
