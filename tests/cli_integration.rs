use assert_cmd::Command;

#[test]
fn dry_run_prints_tmux_commands() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cfg_path = temp.path().join("config.yaml");
    let root = temp.path().join("app");
    std::fs::create_dir_all(&root).expect("mkdir");
    std::fs::write(
        &cfg_path,
        format!(
            "projects:\n  app:\n    root: {}\n    windows:\n      - name: editor\n        command: nvim\n",
            root.display()
        ),
    )
    .expect("write config");

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("app")
        .arg("--dry-run")
        .arg("--config")
        .arg(&cfg_path)
        .assert()
        .success();
}

#[test]
fn save_creates_project_entry() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cfg_path = temp.path().join("config.yaml");
    let root = temp.path().join("app");
    std::fs::create_dir_all(&root).expect("mkdir");
    std::fs::write(&cfg_path, "projects: {}\n").expect("write config");

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("app")
        .arg("--save")
        .arg("--root")
        .arg(&root)
        .arg("--dry-run")
        .arg("--config")
        .arg(&cfg_path)
        .assert()
        .success();

    let cfg_text = std::fs::read_to_string(&cfg_path).expect("read config");
    assert!(cfg_text.contains("app:"));
    assert!(cfg_text.contains("root:"));
    assert!(cfg_text.contains("windows:"));
}

#[test]
fn save_without_existing_config_creates_defaults() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cfg_path = temp.path().join("config.yaml");
    let root = temp.path().join("app");
    std::fs::create_dir_all(&root).expect("mkdir");

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("app")
        .arg("--save")
        .arg("--root")
        .arg(&root)
        .arg("--dry-run")
        .arg("--config")
        .arg(&cfg_path)
        .assert()
        .success();

    let cfg_text = std::fs::read_to_string(&cfg_path).expect("read config");
    assert!(cfg_text.contains("search_paths:"));
    assert!(cfg_text.contains("defaults:"));
    assert!(cfg_text.contains("windows:"));
}

#[test]
fn config_path_prints_without_project() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cfg_path = temp.path().join("config.yaml");

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("--config-path")
        .arg("--config")
        .arg(&cfg_path)
        .assert()
        .success()
        .stdout(format!("{}\n", cfg_path.display()));
}

#[test]
fn edit_config_creates_defaults_and_uses_editor() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cfg_path = temp.path().join("config.yaml");
    let marker_path = temp.path().join("editor_called");
    let script_path = temp.path().join("fake_editor.sh");

    std::fs::write(
        &script_path,
        format!("#!/usr/bin/env sh\ntouch \"{}\"\n", marker_path.display()),
    )
    .expect("write script");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script_path)
            .expect("metadata")
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script_path, perms).expect("set perms");
    }

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("--edit-config")
        .arg("--config")
        .arg(&cfg_path)
        .env("VISUAL", &script_path)
        .assert()
        .success();

    let cfg_text = std::fs::read_to_string(&cfg_path).expect("read config");
    assert!(cfg_text.contains("search_paths:"));
    assert!(cfg_text.contains("defaults:"));
    assert!(marker_path.exists());
}
