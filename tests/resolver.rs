use jca_tmuxer::config::{Config, ProjectConfig};
use jca_tmuxer::resolver::{project_key_from_input, resolve_project};
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

#[test]
fn resolves_path_input_using_basename_for_session() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("monorepo");
    std::fs::create_dir_all(&root).expect("mkdir");

    let cfg = Config::default();
    let result = resolve_project(&cfg, &root.to_string_lossy()).expect("resolve");

    assert_eq!(result.name, "monorepo");
    assert_eq!(result.session_name, "monorepo");
    assert_eq!(
        result.root,
        std::fs::canonicalize(&root).expect("canonical root")
    );
}

#[test]
fn project_key_from_path_input_uses_resolved_root_basename() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("my_api");
    std::fs::create_dir_all(&root).expect("mkdir");
    let canonical = std::fs::canonicalize(&root).expect("canonical root");

    let key = project_key_from_input(&root.to_string_lossy(), &canonical);
    assert_eq!(key, "my_api");
}

#[test]
fn resolves_from_search_paths_by_exact_case_insensitive_and_unique_prefix() {
    let dir = tempfile::tempdir().expect("tempdir");
    let base = dir.path().join("work");
    std::fs::create_dir_all(&base).expect("mkdir base");

    let exact = base.join("api");
    let case_dir = base.join("CaseProj");
    let prefix_unique = base.join("alpha-service");
    std::fs::create_dir_all(&exact).expect("mkdir exact");
    std::fs::create_dir_all(&case_dir).expect("mkdir case");
    std::fs::create_dir_all(&prefix_unique).expect("mkdir prefix");

    let cfg = Config {
        search_paths: vec![base.to_string_lossy().to_string()],
        ..Config::default()
    };

    let exact_resolved = resolve_project(&cfg, "api").expect("resolve exact");
    assert_eq!(exact_resolved.root, exact);

    let case_resolved = resolve_project(&cfg, "caseproj").expect("resolve case-insensitive");
    assert_eq!(case_resolved.root, case_dir);

    let prefix_resolved = resolve_project(&cfg, "alpha").expect("resolve unique prefix");
    assert_eq!(prefix_resolved.root, prefix_unique);
}

#[test]
fn unknown_project_error_contains_suggestions() {
    let mut projects = BTreeMap::new();
    projects.insert("api".to_string(), ProjectConfig::default());
    projects.insert("web".to_string(), ProjectConfig::default());
    projects.insert("worker".to_string(), ProjectConfig::default());

    let cfg = Config {
        projects,
        ..Config::default()
    };

    let err = resolve_project(&cfg, "ap").expect_err("expected unknown project");
    let msg = err.to_string();
    assert!(msg.contains("unknown project 'ap'"));
    assert!(msg.contains("suggestions:"));
    assert!(msg.contains("api"));
}

#[test]
fn does_not_auto_resolve_when_search_path_prefix_is_ambiguous() {
    let dir = tempfile::tempdir().expect("tempdir");
    let base = dir.path().join("work");
    std::fs::create_dir_all(&base).expect("mkdir base");

    std::fs::create_dir_all(base.join("api-core")).expect("mkdir a");
    std::fs::create_dir_all(base.join("api-web")).expect("mkdir b");

    let cfg = Config {
        search_paths: vec![base.to_string_lossy().to_string()],
        ..Config::default()
    };

    let err = resolve_project(&cfg, "api").expect_err("expected ambiguous search miss");
    assert!(err.to_string().contains("unknown project 'api'"));
}

#[test]
fn does_not_auto_resolve_when_case_insensitive_match_is_ambiguous() {
    let dir = tempfile::tempdir().expect("tempdir");
    let base = dir.path().join("work");
    std::fs::create_dir_all(&base).expect("mkdir base");

    std::fs::create_dir_all(base.join("Service")).expect("mkdir service 1");
    std::fs::create_dir_all(base.join("sErvice")).expect("mkdir service 2");

    let cfg = Config {
        search_paths: vec![base.to_string_lossy().to_string()],
        ..Config::default()
    };

    let err = resolve_project(&cfg, "service").expect_err("expected ambiguous search miss");
    assert!(err.to_string().contains("unknown project 'service'"));
}
