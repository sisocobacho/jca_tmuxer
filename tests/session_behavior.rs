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

fn write_multi_project_config(
    path: &std::path::Path,
    first_name: &str,
    first_root: &std::path::Path,
    second_name: &str,
    second_root: &std::path::Path,
) {
    std::fs::write(
        path,
        format!(
            "projects:\n  {first_name}:\n    root: {}\n    windows:\n      - name: editor\n        command: nvim\n  {second_name}:\n    root: {}\n    windows:\n      - name: editor\n        command: nvim\n",
            first_root.display(),
            second_root.display()
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

fn install_fake_terminal(bin_dir: &std::path::Path) {
    let script_path = bin_dir.join("fake-terminal");
    std::fs::write(
        &script_path,
        r#"#!/bin/sh
if [ -n "$TERMINAL_LOG" ]; then
  printf '%s\n' "$*" >> "$TERMINAL_LOG"
fi
exit 0
"#,
    )
    .expect("write fake terminal");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script_path)
            .expect("metadata")
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script_path, perms).expect("set perms");
    }
}

fn install_noisy_fake_terminal(bin_dir: &std::path::Path) {
    let script_path = bin_dir.join("fake-terminal-noisy");
    std::fs::write(
        &script_path,
        r#"#!/bin/sh
echo noisy-terminal-stdout
echo noisy-terminal-stderr 1>&2
exit 0
"#,
    )
    .expect("write noisy fake terminal");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script_path)
            .expect("metadata")
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script_path, perms).expect("set perms");
    }
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

#[test]
fn projects_mode_launches_only_selected_projects() {
    let temp = tempfile::tempdir().expect("tempdir");
    let app_root = temp.path().join("app");
    let api_root = temp.path().join("api");
    std::fs::create_dir_all(&app_root).expect("mkdir app");
    std::fs::create_dir_all(&api_root).expect("mkdir api");

    let cfg_path = temp.path().join("config.yaml");
    write_multi_project_config(&cfg_path, "app", &app_root, "api", &api_root);

    let bin_dir = temp.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    install_fake_tmux(&bin_dir);

    let log_path = temp.path().join("tmux.log");

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("--projects")
        .arg("api")
        .arg("--no-attach")
        .arg("--config")
        .arg(&cfg_path)
        .env("PATH", &bin_dir)
        .env("TMUX_LOG", &log_path)
        .env("FAKE_TMUX_HAS_SESSION", "0")
        .assert()
        .success();

    let lines = read_log(&log_path);
    assert!(
        lines.iter().any(|l| l.starts_with("has-session -t api")),
        "expected api has-session call in {lines:?}"
    );
    assert!(
        lines.iter().any(|l| l.starts_with("new-session -d -s api")),
        "expected api new-session call in {lines:?}"
    );
    assert!(
        lines.iter().all(|l| !l.contains(" app")),
        "did not expect app session commands in {lines:?}"
    );
}

#[test]
fn all_mode_with_open_terminals_spawns_one_terminal_per_project() {
    let temp = tempfile::tempdir().expect("tempdir");
    let app_root = temp.path().join("app");
    let api_root = temp.path().join("api");
    std::fs::create_dir_all(&app_root).expect("mkdir app");
    std::fs::create_dir_all(&api_root).expect("mkdir api");

    let cfg_path = temp.path().join("config.yaml");
    write_multi_project_config(&cfg_path, "app", &app_root, "api", &api_root);

    let bin_dir = temp.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    install_fake_tmux(&bin_dir);
    install_fake_terminal(&bin_dir);

    let tmux_log = temp.path().join("tmux.log");
    let terminal_log = temp.path().join("terminal.log");

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("--all")
        .arg("--open-terminals")
        .arg("--terminal-cmd")
        .arg("fake-terminal tmux attach-session -t {session}")
        .arg("--config")
        .arg(&cfg_path)
        .env("PATH", &bin_dir)
        .env("TMUX_LOG", &tmux_log)
        .env("TERMINAL_LOG", &terminal_log)
        .env("FAKE_TMUX_HAS_SESSION", "0")
        .assert()
        .success();

    let tmux_lines = read_log(&tmux_log);
    assert!(
        tmux_lines
            .iter()
            .any(|l| l.starts_with("new-session -d -s app")),
        "expected app session creation in {tmux_lines:?}"
    );
    assert!(
        tmux_lines
            .iter()
            .any(|l| l.starts_with("new-session -d -s api")),
        "expected api session creation in {tmux_lines:?}"
    );

    let mut terminal_lines = Vec::new();
    for _ in 0..25 {
        if terminal_log.exists() {
            terminal_lines = read_log(&terminal_log);
            if terminal_lines.len() >= 2 {
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert_eq!(
        terminal_lines.len(),
        2,
        "expected one terminal spawn per project"
    );
    assert!(
        terminal_lines
            .iter()
            .any(|l| l.contains("attach-session -t app")),
        "expected terminal for app in {terminal_lines:?}"
    );
    assert!(
        terminal_lines
            .iter()
            .any(|l| l.contains("attach-session -t api")),
        "expected terminal for api in {terminal_lines:?}"
    );
}

#[test]
fn projects_mode_best_effort_continues_after_failed_target() {
    let temp = tempfile::tempdir().expect("tempdir");
    let app_root = temp.path().join("app");
    std::fs::create_dir_all(&app_root).expect("mkdir app");

    let cfg_path = temp.path().join("config.yaml");
    std::fs::write(
        &cfg_path,
        format!(
            "projects:\n  app:\n    root: {}\n    windows:\n      - name: editor\n        command: nvim\n",
            app_root.display()
        ),
    )
    .expect("write config");

    let bin_dir = temp.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    install_fake_tmux(&bin_dir);

    let tmux_log = temp.path().join("tmux.log");

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("--projects")
        .arg("missing")
        .arg("app")
        .arg("--no-attach")
        .arg("--config")
        .arg(&cfg_path)
        .env("PATH", &bin_dir)
        .env("TMUX_LOG", &tmux_log)
        .env("FAKE_TMUX_HAS_SESSION", "0")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "batch launch completed with 1 error(s):",
        ));

    let tmux_lines = read_log(&tmux_log);
    assert!(
        tmux_lines
            .iter()
            .any(|l| l.starts_with("new-session -d -s app")),
        "expected app launch despite another failure in {tmux_lines:?}"
    );
}

#[test]
fn open_terminals_suppresses_launcher_stdio_output() {
    let temp = tempfile::tempdir().expect("tempdir");
    let app_root = temp.path().join("app");
    std::fs::create_dir_all(&app_root).expect("mkdir app");

    let cfg_path = temp.path().join("config.yaml");
    std::fs::write(
        &cfg_path,
        format!(
            "projects:\n  app:\n    root: {}\n    windows:\n      - name: editor\n        command: nvim\n",
            app_root.display()
        ),
    )
    .expect("write config");

    let bin_dir = temp.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    install_fake_tmux(&bin_dir);
    install_noisy_fake_terminal(&bin_dir);

    let tmux_log = temp.path().join("tmux.log");

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("--projects")
        .arg("app")
        .arg("--open-terminals")
        .arg("--terminal-cmd")
        .arg("fake-terminal-noisy tmux attach-session -t {session}")
        .arg("--config")
        .arg(&cfg_path)
        .env("PATH", &bin_dir)
        .env("TMUX_LOG", &tmux_log)
        .env("FAKE_TMUX_HAS_SESSION", "0")
        .assert()
        .success()
        .stdout(predicates::str::contains("noisy-terminal-stdout").not())
        .stderr(predicates::str::contains("noisy-terminal-stderr").not());
}
