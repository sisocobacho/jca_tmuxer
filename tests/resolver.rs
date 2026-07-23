use jca_tmuxer::config::{Config, ProjectConfig};
use jca_tmuxer::resolver::resolve_project;
use std::collections::BTreeMap;

#[test]
fn resolves_config_project() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("app");
    std::fs::create_dir_all(&root).expect("mkdir");

    let mut projects = BTreeMap::new();
    projects.insert(
        "app".to_string(),
        ProjectConfig {
            root: Some(root.to_string_lossy().to_string()),
            ..ProjectConfig::default()
        },
    );

    let cfg = Config {
        projects,
        ..Config::default()
    };

    let result = resolve_project(&cfg, "app").expect("resolve");
    assert_eq!(result.session_name, "app");
}
