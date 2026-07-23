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
}
