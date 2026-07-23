use jca_tmuxer::config::{Config, Defaults, ProjectConfig, WindowConfig};
use jca_tmuxer::plan::build_windows;
use jca_tmuxer::resolver::ResolvedProject;
use std::collections::BTreeMap;

#[test]
fn builds_from_defaults_when_project_windows_missing() {
    let dir = tempfile::tempdir().expect("tempdir");
    let project_root = dir.path().join("app");
    std::fs::create_dir_all(&project_root).expect("mkdir");

    let cfg = Config {
        defaults: Defaults {
            layout: Some("stacked".to_string()),
            windows: vec![WindowConfig {
                name: "editor".to_string(),
                command: Some("nvim".to_string()),
                ..WindowConfig::default()
            }],
        },
        projects: BTreeMap::new(),
        search_paths: Vec::new(),
    };

    let project = ResolvedProject {
        name: "app".to_string(),
        session_name: "app".to_string(),
        root: project_root,
    };

    let windows = build_windows(&cfg, &project, &[]).expect("plan");
    assert_eq!(windows.len(), 1);
    assert_eq!(windows[0].name, "editor");
}

#[test]
fn appends_adhoc_windows() {
    let dir = tempfile::tempdir().expect("tempdir");
    let project_root = dir.path().join("app");
    std::fs::create_dir_all(&project_root).expect("mkdir");

    let mut projects = BTreeMap::new();
    projects.insert("app".to_string(), ProjectConfig::default());
    let cfg = Config {
        defaults: Defaults::default(),
        projects,
        search_paths: Vec::new(),
    };
    let project = ResolvedProject {
        name: "app".to_string(),
        session_name: "app".to_string(),
        root: project_root,
    };
    let windows = build_windows(&cfg, &project, &["npm run dev".to_string()]).expect("plan");
    assert_eq!(windows.len(), 1);
    assert_eq!(windows[0].name, "adhoc-1");
}
