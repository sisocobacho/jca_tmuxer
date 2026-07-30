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
        remove: false,
        yes: false,
        root: None,
        new: false,
        no_attach: true,
        dry_run: true,
        verbose: 0,
        list: false,
        print_config: false,
        config_path: false,
        edit_config: false,
    };

    let changed = config::save_project(&args, "my_project", &root, None).expect("save");
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
        remove: false,
        yes: false,
        root: None,
        new: false,
        no_attach: true,
        dry_run: true,
        verbose: 0,
        list: false,
        print_config: false,
        config_path: false,
        edit_config: false,
    };

    let changed = config::save_project(&args, "my_project", &root, None).expect("save");
    assert!(!changed);
}

#[test]
fn save_project_seeds_new_config_with_builtin_defaults() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cfg = temp.path().join("config.yaml");
    let root = temp.path().join("my_project");
    std::fs::create_dir_all(&root).expect("mkdir");

    let args = Args {
        project: Some("my_project".to_string()),
        adhoc_commands: vec![],
        config: Some(cfg.clone()),
        save: false,
        remove: false,
        yes: false,
        root: None,
        new: false,
        no_attach: true,
        dry_run: true,
        verbose: 0,
        list: false,
        print_config: false,
        config_path: false,
        edit_config: false,
    };

    let changed = config::save_project(&args, "my_project", &root, None).expect("save");
    assert!(changed);

    let loaded = config::load_path(&cfg).expect("load");
    assert!(!loaded.search_paths.is_empty());
    assert_eq!(loaded.defaults.layout.as_deref(), Some("stacked"));
    assert!(!loaded.defaults.windows.is_empty());
}

#[test]
fn save_project_persists_project_windows_when_provided() {
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
        remove: false,
        yes: false,
        root: None,
        new: false,
        no_attach: true,
        dry_run: true,
        verbose: 0,
        list: false,
        print_config: false,
        config_path: false,
        edit_config: false,
    };

    let defaults = config::builtin_defaults().defaults.windows;
    let changed = config::save_project(&args, "my_project", &root, Some(defaults)).expect("save");
    assert!(changed);

    let loaded = config::load_path(&cfg).expect("load");
    let project = loaded.projects.get("my_project").expect("project exists");
    assert!(!project.windows.is_empty());
}

#[test]
fn remove_project_deletes_existing_project() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cfg = temp.path().join("config.yaml");
    std::fs::write(
        &cfg,
        "projects:\n  my_project:\n    root: /tmp/my_project\n",
    )
    .expect("write");

    let args = Args {
        project: Some("my_project".to_string()),
        adhoc_commands: vec![],
        config: Some(cfg.clone()),
        save: false,
        remove: false,
        yes: false,
        root: None,
        new: false,
        no_attach: true,
        dry_run: true,
        verbose: 0,
        list: false,
        print_config: false,
        config_path: false,
        edit_config: false,
    };

    let removed = config::remove_project(&args, "my_project").expect("remove");
    assert!(removed);

    let loaded = config::load_path(&cfg).expect("load");
    assert!(!loaded.projects.contains_key("my_project"));
}

#[test]
fn remove_project_is_noop_when_project_missing() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cfg = temp.path().join("config.yaml");
    std::fs::write(&cfg, "projects: {}\n").expect("write");

    let args = Args {
        project: Some("my_project".to_string()),
        adhoc_commands: vec![],
        config: Some(cfg),
        save: false,
        remove: false,
        yes: false,
        root: None,
        new: false,
        no_attach: true,
        dry_run: true,
        verbose: 0,
        list: false,
        print_config: false,
        config_path: false,
        edit_config: false,
    };

    let removed = config::remove_project(&args, "my_project").expect("remove");
    assert!(!removed);
}

#[test]
fn remove_project_is_noop_when_config_missing() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cfg = temp.path().join("config.yaml");

    let args = Args {
        project: Some("my_project".to_string()),
        adhoc_commands: vec![],
        config: Some(cfg),
        save: false,
        remove: false,
        yes: false,
        root: None,
        new: false,
        no_attach: true,
        dry_run: true,
        verbose: 0,
        list: false,
        print_config: false,
        config_path: false,
        edit_config: false,
    };

    let removed = config::remove_project(&args, "my_project").expect("remove");
    assert!(!removed);
}
