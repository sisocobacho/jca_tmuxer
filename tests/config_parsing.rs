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
