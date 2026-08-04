use jca_tmuxer::config::{Config, Defaults, PaneConfig, ProjectConfig, WindowConfig};
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
            layout: Some("main-vertical".to_string()),
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
    assert_eq!(windows[0].layout, "main-vertical");
    assert_eq!(windows[0].panes.len(), 1);
    assert_eq!(windows[0].panes[0].command, "npm run dev");
    assert_eq!(windows[0].panes[0].cwd, project.root);
}

#[test]
fn appends_multiple_adhoc_windows_with_incremented_names() {
    let dir = tempfile::tempdir().expect("tempdir");
    let project_root = dir.path().join("app");
    std::fs::create_dir_all(&project_root).expect("mkdir");

    let mut projects = BTreeMap::new();
    projects.insert("app".to_string(), ProjectConfig::default());
    let cfg = Config {
        defaults: Defaults {
            layout: Some("tiled".to_string()),
            windows: vec![WindowConfig {
                name: "editor".to_string(),
                command: Some("nvim".to_string()),
                ..WindowConfig::default()
            }],
        },
        projects,
        search_paths: Vec::new(),
    };
    let project = ResolvedProject {
        name: "app".to_string(),
        session_name: "app".to_string(),
        root: project_root,
    };
    let windows = build_windows(
        &cfg,
        &project,
        &["npm run dev".to_string(), "cargo test".to_string()],
    )
    .expect("plan");

    assert_eq!(windows.len(), 3);
    assert_eq!(windows[0].name, "editor");

    assert_eq!(windows[1].name, "adhoc-1");
    assert_eq!(windows[1].layout, "main-vertical");
    assert_eq!(windows[1].panes[0].command, "npm run dev");
    assert_eq!(windows[1].panes[0].cwd, project.root);

    assert_eq!(windows[2].name, "adhoc-2");
    assert_eq!(windows[2].layout, "main-vertical");
    assert_eq!(windows[2].panes[0].command, "cargo test");
    assert_eq!(windows[2].panes[0].cwd, project.root);
}

#[test]
fn project_windows_replace_defaults_when_extend_is_false() {
    let dir = tempfile::tempdir().expect("tempdir");
    let project_root = dir.path().join("app");
    std::fs::create_dir_all(&project_root).expect("mkdir");

    let mut projects = BTreeMap::new();
    projects.insert(
        "app".to_string(),
        ProjectConfig {
            extend: false,
            windows: vec![WindowConfig {
                name: "api".to_string(),
                command: Some("npm run dev".to_string()),
                ..WindowConfig::default()
            }],
            ..ProjectConfig::default()
        },
    );

    let cfg = Config {
        defaults: Defaults {
            layout: Some("main-vertical".to_string()),
            windows: vec![WindowConfig {
                name: "editor".to_string(),
                command: Some("nvim".to_string()),
                ..WindowConfig::default()
            }],
        },
        projects,
        search_paths: Vec::new(),
    };

    let project = ResolvedProject {
        name: "app".to_string(),
        session_name: "app".to_string(),
        root: project_root,
    };

    let windows = build_windows(&cfg, &project, &[]).expect("plan");
    assert_eq!(windows.len(), 1);
    assert_eq!(windows[0].name, "api");
    assert_eq!(windows[0].panes[0].command, "npm run dev");
}

#[test]
fn project_windows_merge_defaults_when_extend_is_true() {
    let dir = tempfile::tempdir().expect("tempdir");
    let project_root = dir.path().join("app");
    let api_dir = project_root.join("api");
    std::fs::create_dir_all(&api_dir).expect("mkdir");

    let mut projects = BTreeMap::new();
    projects.insert(
        "app".to_string(),
        ProjectConfig {
            extend: true,
            windows: vec![
                WindowConfig {
                    name: "editor".to_string(),
                    command: Some("hx".to_string()),
                    ..WindowConfig::default()
                },
                WindowConfig {
                    name: "api".to_string(),
                    command: Some("npm run dev".to_string()),
                    directory: Some(api_dir.to_string_lossy().to_string()),
                    ..WindowConfig::default()
                },
            ],
            ..ProjectConfig::default()
        },
    );

    let cfg = Config {
        defaults: Defaults {
            layout: Some("main-vertical".to_string()),
            windows: vec![
                WindowConfig {
                    name: "editor".to_string(),
                    command: Some("nvim".to_string()),
                    ..WindowConfig::default()
                },
                WindowConfig {
                    name: "git".to_string(),
                    command: None,
                    layout: Some("main-vertical".to_string()),
                    panes: vec![
                        PaneConfig {
                            command: "git status".to_string(),
                            directory: Some("<project_root>".to_string()),
                            size: None,
                        },
                        PaneConfig {
                            command: "git log --oneline".to_string(),
                            directory: Some("<project_root>".to_string()),
                            size: None,
                        },
                    ],
                    ..WindowConfig::default()
                },
            ],
        },
        projects,
        search_paths: Vec::new(),
    };

    let project = ResolvedProject {
        name: "app".to_string(),
        session_name: "app".to_string(),
        root: project_root,
    };

    let windows = build_windows(&cfg, &project, &[]).expect("plan");
    assert_eq!(windows.len(), 3);
    assert_eq!(windows[0].name, "editor");
    assert_eq!(windows[0].panes[0].command, "hx");
    assert_eq!(windows[1].name, "git");
    assert_eq!(windows[2].name, "api");
}

#[test]
fn errors_when_window_directory_does_not_exist() {
    let dir = tempfile::tempdir().expect("tempdir");
    let project_root = dir.path().join("app");
    std::fs::create_dir_all(&project_root).expect("mkdir");

    let cfg = Config {
        defaults: Defaults {
            layout: Some("main-vertical".to_string()),
            windows: vec![WindowConfig {
                name: "broken".to_string(),
                command: Some("bash".to_string()),
                directory: Some(project_root.join("missing").to_string_lossy().to_string()),
                ..WindowConfig::default()
            }],
        },
        projects: BTreeMap::new(),
        search_paths: Vec::new(),
    };

    let project = ResolvedProject {
        name: "app".to_string(),
        session_name: "app".to_string(),
        root: project_root
            .join(".")
            .canonicalize()
            .expect("canonical root"),
    };

    let err = build_windows(&cfg, &project, &[]).expect_err("expected missing directory error");
    assert!(
        err.to_string()
            .contains("window 'broken' directory does not exist"),
        "actual error: {err}"
    );
}

#[test]
fn duplicate_window_names_are_normalized() {
    let dir = tempfile::tempdir().expect("tempdir");
    let project_root = dir.path().join("app");
    std::fs::create_dir_all(&project_root).expect("mkdir");

    let cfg = Config {
        defaults: Defaults {
            layout: Some("main-vertical".to_string()),
            windows: vec![
                WindowConfig {
                    name: "editor".to_string(),
                    command: Some("nvim".to_string()),
                    ..WindowConfig::default()
                },
                WindowConfig {
                    name: "editor".to_string(),
                    command: Some("hx".to_string()),
                    ..WindowConfig::default()
                },
            ],
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
    assert_eq!(windows.len(), 2);
    assert_eq!(windows[0].name, "editor");
    assert_eq!(windows[1].name, "editor-2");
}
