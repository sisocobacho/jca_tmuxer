use assert_cmd::Command;
use jca_tmuxer::config;

fn install_fake_tmux(bin_dir: &std::path::Path) {
    let script_path = bin_dir.join("tmux");
    std::fs::write(
        &script_path,
        r#"#!/bin/sh
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
}

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
fn short_binary_alias_runs_same_cli() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cfg_path = temp.path().join("config.yaml");

    let mut cmd = Command::cargo_bin("jtmx").expect("bin");
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

#[test]
fn list_prints_configured_project_names() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cfg_path = temp.path().join("config.yaml");
    std::fs::write(
        &cfg_path,
        "projects:\n  alpha:\n    root: /tmp\n  beta:\n    root: /var/tmp\n",
    )
    .expect("write config");

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("--list")
        .arg("--config")
        .arg(&cfg_path)
        .assert()
        .success()
        .stdout(predicates::str::contains("alpha\n"))
        .stdout(predicates::str::contains("beta\n"));
}

#[test]
fn print_config_outputs_resolved_window_plan() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cfg_path = temp.path().join("config.yaml");
    let root = temp.path().join("app");
    std::fs::create_dir_all(&root).expect("mkdir");
    std::fs::write(
        &cfg_path,
        format!(
            "defaults:\n  layout: tiled\nprojects:\n  app:\n    root: {}\n    windows:\n      - name: editor\n        command: nvim\n",
            root.display()
        ),
    )
    .expect("write config");

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("app")
        .arg("--print-config")
        .arg("--config")
        .arg(&cfg_path)
        .assert()
        .success()
        .stdout(predicates::str::contains("- name: editor"))
        .stdout(predicates::str::contains("layout: tiled"))
        .stdout(predicates::str::contains("command: nvim"));
}

#[test]
fn non_dry_run_errors_when_tmux_missing_from_path() {
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
        .arg("--config")
        .arg(&cfg_path)
        .env("PATH", "")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "tmux is not installed or not in PATH",
        ));
}

#[test]
fn missing_project_argument_fails_without_mode_flags() {
    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.assert().failure();
}

#[test]
fn edit_config_falls_back_to_editor_when_visual_is_blank() {
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
        .env("VISUAL", "   ")
        .env("EDITOR", &script_path)
        .assert()
        .success();

    assert!(marker_path.exists());
}

#[test]
fn edit_config_falls_back_to_vi_when_visual_and_editor_are_unset() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cfg_path = temp.path().join("config.yaml");
    let marker_path = temp.path().join("vi_called");
    let bin_dir = temp.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    let vi_path = bin_dir.join("vi");

    std::fs::write(
        &vi_path,
        format!("#!/usr/bin/env sh\ntouch \"{}\"\n", marker_path.display()),
    )
    .expect("write vi script");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&vi_path).expect("metadata").permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&vi_path, perms).expect("set perms");
    }

    let path = format!("{}:/usr/bin:/bin", bin_dir.display());

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("--edit-config")
        .arg("--config")
        .arg(&cfg_path)
        .env("PATH", path)
        .env_remove("VISUAL")
        .env_remove("EDITOR")
        .assert()
        .success();

    assert!(marker_path.exists());
}

#[test]
fn save_with_adhoc_does_not_persist_default_windows_to_project() {
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
        .arg("npm run dev")
        .arg("--dry-run")
        .arg("--config")
        .arg(&cfg_path)
        .assert()
        .success();

    let loaded = config::load_path(&cfg_path).expect("load config");
    let project = loaded.projects.get("app").expect("project saved");
    assert!(
        project.windows.is_empty(),
        "expected no persisted default windows when ad-hoc commands are used"
    );
}

#[test]
fn falls_back_to_builtin_defaults_when_user_config_is_missing() {
    let temp = tempfile::tempdir().expect("tempdir");
    let xdg_home = temp.path().join("xdg");
    std::fs::create_dir_all(&xdg_home).expect("mkdir xdg");

    let root = temp.path().join("my_project");
    std::fs::create_dir_all(&root).expect("mkdir root");

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg(root.to_string_lossy().to_string())
        .arg("--print-config")
        .arg("--dry-run")
        .env("XDG_CONFIG_HOME", &xdg_home)
        .assert()
        .success()
        .stdout(predicates::str::contains("- name: editor"))
        .stdout(predicates::str::contains("command: nvim"));
}

#[test]
fn remove_yes_deletes_project_from_config() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cfg_path = temp.path().join("config.yaml");
    std::fs::write(
        &cfg_path,
        "projects:\n  app:\n    root: /tmp/app\n  keep:\n    root: /tmp/keep\n",
    )
    .expect("write config");

    let bin_dir = temp.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    install_fake_tmux(&bin_dir);

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("app")
        .arg("--remove")
        .arg("--yes")
        .arg("--config")
        .arg(&cfg_path)
        .env("PATH", &bin_dir)
        .env("FAKE_TMUX_HAS_SESSION", "0")
        .assert()
        .success()
        .stdout(predicates::str::contains("Removed: config project 'app'"));

    let cfg_text = std::fs::read_to_string(&cfg_path).expect("read config");
    assert!(!cfg_text.contains("\n  app:\n"));
    assert!(cfg_text.contains("\n  keep:\n"));
}

#[test]
fn remove_yes_reports_nothing_removed_when_targets_missing() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cfg_path = temp.path().join("config.yaml");
    std::fs::write(&cfg_path, "projects: {}\n").expect("write config");

    let bin_dir = temp.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    install_fake_tmux(&bin_dir);

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("app")
        .arg("--remove")
        .arg("--yes")
        .arg("--config")
        .arg(&cfg_path)
        .env("PATH", &bin_dir)
        .env("FAKE_TMUX_HAS_SESSION", "0")
        .assert()
        .success()
        .stdout(predicates::str::contains("Nothing was removed."));
}

#[test]
fn remove_without_yes_in_non_tty_fails_with_hint() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cfg_path = temp.path().join("config.yaml");
    std::fs::write(&cfg_path, "projects: {}\n").expect("write config");

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("app")
        .arg("--remove")
        .arg("--config")
        .arg(&cfg_path)
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "--remove requires confirmation from a TTY. Re-run with --yes",
        ));
}

#[test]
fn remove_path_input_uses_basename_for_config_key() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cfg_path = temp.path().join("config.yaml");
    let root = temp.path().join("monorepo");
    std::fs::create_dir_all(&root).expect("mkdir root");
    std::fs::write(
        &cfg_path,
        "projects:\n  monorepo:\n    root: /tmp/monorepo\n  keep:\n    root: /tmp/keep\n",
    )
    .expect("write config");

    let bin_dir = temp.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    install_fake_tmux(&bin_dir);

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg(root.to_string_lossy().to_string())
        .arg("--remove")
        .arg("--yes")
        .arg("--config")
        .arg(&cfg_path)
        .env("PATH", &bin_dir)
        .env("FAKE_TMUX_HAS_SESSION", "0")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "Removed: config project 'monorepo'",
        ));

    let cfg_text = std::fs::read_to_string(&cfg_path).expect("read config");
    assert!(!cfg_text.contains("\n  monorepo:\n"));
    assert!(cfg_text.contains("\n  keep:\n"));
}
