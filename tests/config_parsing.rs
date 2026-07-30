use jca_tmuxer::config;
use std::fs;

#[test]
fn parses_config_yaml() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("config.yaml");
    fs::write(
        &path,
        r#"
defaults:
  layout: stacked
  windows:
    - name: editor
      command: nvim

projects:
  app:
    root: ~/code/app
    windows:
      - name: api
        command: npm run dev
"#,
    )
    .expect("write");

    let cfg = config::load_path(&path).expect("load");
    assert_eq!(cfg.defaults.windows.len(), 1);
    assert!(cfg.projects.contains_key("app"));
}

#[test]
fn parses_all_supported_keys_and_nested_fields() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("config.yaml");
    fs::write(
        &path,
        r#"
search_paths:
  - ~/code
  - ~/workspace
defaults:
  layout: tiled
  windows:
    - name: editor
      command: nvim
      directory: <project_root>
      layout: main-vertical
      panes:
        - command: nvim .
          directory: <project_root>
          size: 70
        - command: cargo test
          size: 30

projects:
  app:
    root: ~/code/app
    extend: true
    windows:
      - name: api
        command: npm run dev
        directory: ~/code/app/packages/api
      - name: logs
        panes:
          - command: tail -f log.txt
            directory: ~/code/app
            size: 50
          - command: journalctl -f
            size: 50
"#,
    )
    .expect("write");

    let cfg = config::load_path(&path).expect("load");
    assert_eq!(cfg.search_paths.len(), 2);
    assert_eq!(cfg.defaults.layout.as_deref(), Some("tiled"));
    assert_eq!(cfg.defaults.windows.len(), 1);

    let default_window = &cfg.defaults.windows[0];
    assert_eq!(default_window.name, "editor");
    assert_eq!(default_window.command.as_deref(), Some("nvim"));
    assert_eq!(default_window.directory.as_deref(), Some("<project_root>"));
    assert_eq!(default_window.layout.as_deref(), Some("main-vertical"));
    assert_eq!(default_window.panes.len(), 2);
    assert_eq!(default_window.panes[0].size, Some(70));
    assert_eq!(default_window.panes[1].size, Some(30));

    let project = cfg.projects.get("app").expect("project app");
    assert_eq!(project.root.as_deref(), Some("~/code/app"));
    assert!(project.extend);
    assert_eq!(project.windows.len(), 2);
    assert_eq!(project.windows[0].name, "api");
    assert_eq!(
        project.windows[0].directory.as_deref(),
        Some("~/code/app/packages/api")
    );
    assert_eq!(project.windows[1].name, "logs");
    assert_eq!(project.windows[1].panes.len(), 2);
    assert_eq!(project.windows[1].panes[0].size, Some(50));
}

#[test]
fn invalid_yaml_returns_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("config.yaml");
    fs::write(
        &path,
        "defaults:\n  windows:\n    - name: editor\n      command: [\n",
    )
    .expect("write");

    let err = config::load_path(&path).expect_err("expected parse error");
    let msg = err.to_string();
    assert!(msg.contains("invalid YAML config"), "actual error: {msg}");
}

#[test]
fn missing_optional_sections_use_defaults() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("config.yaml");
    fs::write(
        &path,
        r#"
projects:
  app:
    root: ~/code/app
"#,
    )
    .expect("write");

    let cfg = config::load_path(&path).expect("load");
    assert!(cfg.search_paths.is_empty());
    assert!(cfg.defaults.windows.is_empty());
    assert!(cfg.defaults.layout.is_none());
    let project = cfg.projects.get("app").expect("project app");
    assert_eq!(project.root.as_deref(), Some("~/code/app"));
    assert!(!project.extend);
    assert!(project.windows.is_empty());
}
