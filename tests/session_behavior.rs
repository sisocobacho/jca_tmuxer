use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;

fn write_project_config(path: &std::path::Path, root: &std::path::Path) {
    std::fs::write(
        path,
        format!(
            "projects:\n  app:\n    root: {}\n    windows:\n      - name: editor\n        command: nvim\n",
            root.display()
        ),
    )
    .expect("write config");
}

fn install_fake_tmux(bin_dir: &std::path::Path) -> std::path::PathBuf {
    let script_path = bin_dir.join("tmux");
    std::fs::write(
        &script_path,
        r#"#!/bin/sh
if [ -n "$TMUX_LOG" ]; then
  printf '%s\n' "$*" >> "$TMUX_LOG"
fi

if [ "$1" = "has-session" ]; then
  if [ "$FAKE_TMUX_HAS_SESSION" = "1" ]; then
    exit 0
  fi
  exit 1
fi

exit 0
"#,
    )
    .expect("write fake tmux");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script_path)
            .expect("metadata")
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script_path, perms).expect("set perms");
    }

    script_path
}

fn read_log(log_path: &std::path::Path) -> Vec<String> {
    std::fs::read_to_string(log_path)
        .expect("read log")
        .lines()
        .map(|s| s.to_string())
        .collect()
}

#[test]
fn existing_session_with_new_kills_then_recreates() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("app");
    std::fs::create_dir_all(&root).expect("mkdir");

    let cfg_path = temp.path().join("config.yaml");
    write_project_config(&cfg_path, &root);

    let bin_dir = temp.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    install_fake_tmux(&bin_dir);

    let log_path = temp.path().join("tmux.log");

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("app")
        .arg("--new")
        .arg("--no-attach")
        .arg("--config")
        .arg(&cfg_path)
        .env("PATH", &bin_dir)
        .env("TMUX_LOG", &log_path)
        .env("FAKE_TMUX_HAS_SESSION", "1")
        .assert()
        .success();

    let lines = read_log(&log_path);
    let has_idx = lines
        .iter()
        .position(|l| l.starts_with("has-session -t app"))
        .expect("has-session called");
    let kill_idx = lines
        .iter()
        .position(|l| l.starts_with("kill-session -t app"))
        .expect("kill-session called");
    let new_idx = lines
        .iter()
        .position(|l| l.starts_with("new-session -d -s app"))
        .expect("new-session called");

    assert!(
        has_idx < kill_idx,
        "expected has-session before kill-session"
    );
    assert!(
        kill_idx < new_idx,
        "expected kill-session before new-session"
    );
}

#[test]
fn existing_session_with_adhoc_appends_windows_only() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("app");
    std::fs::create_dir_all(&root).expect("mkdir");

    let cfg_path = temp.path().join("config.yaml");
    write_project_config(&cfg_path, &root);

    let bin_dir = temp.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    install_fake_tmux(&bin_dir);

    let log_path = temp.path().join("tmux.log");

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("app")
        .arg("npm test")
        .arg("--no-attach")
        .arg("--config")
        .arg(&cfg_path)
        .env("PATH", &bin_dir)
        .env("TMUX_LOG", &log_path)
        .env("FAKE_TMUX_HAS_SESSION", "1")
        .assert()
        .success();

    let lines = read_log(&log_path);
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("new-window -t app -n adhoc-1")),
        "expected adhoc window append command in {lines:?}"
    );
    assert!(
        lines
            .iter()
            .all(|l| !l.starts_with("new-session -d -s app")),
        "did not expect new-session for existing session with adhoc"
    );
}

#[test]
fn existing_session_without_new_or_adhoc_does_not_create_or_append() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("app");
    std::fs::create_dir_all(&root).expect("mkdir");

    let cfg_path = temp.path().join("config.yaml");
    write_project_config(&cfg_path, &root);

    let bin_dir = temp.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    install_fake_tmux(&bin_dir);

    let log_path = temp.path().join("tmux.log");

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("app")
        .arg("--no-attach")
        .arg("--config")
        .arg(&cfg_path)
        .env("PATH", &bin_dir)
        .env("TMUX_LOG", &log_path)
        .env("FAKE_TMUX_HAS_SESSION", "1")
        .assert()
        .success();

    let lines = read_log(&log_path);
    assert_eq!(lines.len(), 1, "expected only has-session call");
    assert!(lines[0].starts_with("has-session -t app"));
}

#[test]
fn non_tty_without_no_attach_emits_skip_warning() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("app");
    std::fs::create_dir_all(&root).expect("mkdir");

    let cfg_path = temp.path().join("config.yaml");
    write_project_config(&cfg_path, &root);

    let bin_dir = temp.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    install_fake_tmux(&bin_dir);

    let log_path = temp.path().join("tmux.log");

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("app")
        .arg("--config")
        .arg(&cfg_path)
        .env("PATH", &bin_dir)
        .env("TMUX_LOG", &log_path)
        .env("FAKE_TMUX_HAS_SESSION", "1")
        .assert()
        .success()
        .stderr(predicates::str::contains(
            "stdin is not a TTY, skipping attach",
        ));

    let lines = read_log(&log_path);
    assert_eq!(lines.len(), 1, "expected only has-session call");
    assert!(lines[0].starts_with("has-session -t app"));
}

#[test]
fn no_attach_suppresses_non_tty_skip_warning() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("app");
    std::fs::create_dir_all(&root).expect("mkdir");

    let cfg_path = temp.path().join("config.yaml");
    write_project_config(&cfg_path, &root);

    let bin_dir = temp.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    install_fake_tmux(&bin_dir);

    let log_path = temp.path().join("tmux.log");

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("app")
        .arg("--no-attach")
        .arg("--config")
        .arg(&cfg_path)
        .env("PATH", &bin_dir)
        .env("TMUX_LOG", &log_path)
        .env("FAKE_TMUX_HAS_SESSION", "1")
        .assert()
        .success()
        .stderr(predicates::str::contains("stdin is not a TTY, skipping attach").not());

    let lines = read_log(&log_path);
    assert_eq!(lines.len(), 1, "expected only has-session call");
    assert!(lines[0].starts_with("has-session -t app"));
}

#[test]
fn remove_yes_kills_existing_session() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("app");
    std::fs::create_dir_all(&root).expect("mkdir");

    let cfg_path = temp.path().join("config.yaml");
    write_project_config(&cfg_path, &root);

    let bin_dir = temp.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    install_fake_tmux(&bin_dir);

    let log_path = temp.path().join("tmux.log");

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("app")
        .arg("--remove")
        .arg("--yes")
        .arg("--config")
        .arg(&cfg_path)
        .env("PATH", &bin_dir)
        .env("TMUX_LOG", &log_path)
        .env("FAKE_TMUX_HAS_SESSION", "1")
        .assert()
        .success();

    let lines = read_log(&log_path);
    assert!(
        lines.iter().any(|l| l.starts_with("has-session -t app")),
        "expected has-session call in {lines:?}"
    );
    assert!(
        lines.iter().any(|l| l.starts_with("kill-session -t app")),
        "expected kill-session call in {lines:?}"
    );
}

#[test]
fn remove_yes_does_not_kill_missing_session() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("app");
    std::fs::create_dir_all(&root).expect("mkdir");

    let cfg_path = temp.path().join("config.yaml");
    write_project_config(&cfg_path, &root);

    let bin_dir = temp.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    install_fake_tmux(&bin_dir);

    let log_path = temp.path().join("tmux.log");

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("app")
        .arg("--remove")
        .arg("--yes")
        .arg("--config")
        .arg(&cfg_path)
        .env("PATH", &bin_dir)
        .env("TMUX_LOG", &log_path)
        .env("FAKE_TMUX_HAS_SESSION", "0")
        .assert()
        .success();

    let lines = read_log(&log_path);
    assert!(
        lines.iter().any(|l| l.starts_with("has-session -t app")),
        "expected has-session call in {lines:?}"
    );
    assert!(
        lines.iter().all(|l| !l.starts_with("kill-session -t app")),
        "did not expect kill-session call in {lines:?}"
    );
}
