use jca_tmuxer::cli::Args;
use jca_tmuxer::config;

#[test]
fn save_project_root_writes_missing_project() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cfg = temp.path().join("config.yaml");
    let root = temp.path().join("my_project");
    std::fs::create_dir_all(&root).expect("mkdir");
    std::fs::write(&cfg, "projects: {}\n").expect("write");

    let args = Args {
        project: Some("my_project".to_string()),
        adhoc_commands: vec![],
        config: Some(cfg.clone()),
        save: false,
        root: None,
        new: false,
        no_attach: true,
        dry_run: true,
        verbose: 0,
        list: false,
        print_config: false,
    };

    let changed = config::save_project_root(&args, "my_project", &root).expect("save");
    assert!(changed);

    let loaded = config::load_path(&cfg).expect("load");
    assert!(loaded.projects.contains_key("my_project"));
}

#[test]
fn save_project_root_is_noop_when_project_exists() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cfg = temp.path().join("config.yaml");
    let root = temp.path().join("my_project");
    std::fs::create_dir_all(&root).expect("mkdir");
    std::fs::write(
        &cfg,
        format!(
            "projects:\n  my_project:\n    root: {}\n",
            root.to_string_lossy()
        ),
    )
    .expect("write");

    let args = Args {
        project: Some("my_project".to_string()),
        adhoc_commands: vec![],
        config: Some(cfg.clone()),
        save: false,
        root: None,
        new: false,
        no_attach: true,
        dry_run: true,
        verbose: 0,
        list: false,
        print_config: false,
    };

    let changed = config::save_project_root(&args, "my_project", &root).expect("save");
    assert!(!changed);
}
