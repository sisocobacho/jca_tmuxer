use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;

fn write_project_config(path: &std::path::Path, root: &std::path::Path, command: &str) {
    std::fs::write(
        path,
        format!(
            "projects:\n  app:\n    root: {}\n    windows:\n      - name: editor\n        command: {}\n",
            root.display(),
            command
        ),
    )
    .expect("write config");
}

#[test]
fn config_flag_takes_precedence_over_env() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("app");
    std::fs::create_dir_all(&root).expect("mkdir");

    let cfg_flag = temp.path().join("flag.yaml");
    let cfg_env = temp.path().join("env.yaml");
    write_project_config(&cfg_flag, &root, "from_flag");
    write_project_config(&cfg_env, &root, "from_env");

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("app")
        .arg("--print-config")
        .arg("--dry-run")
        .arg("--config")
        .arg(&cfg_flag)
        .env("JCA_TMUXER_CONFIG", &cfg_env)
        .assert()
        .success()
        .stdout(predicates::str::contains("command: from_flag"))
        .stdout(predicates::str::contains("command: from_env").not());
}

#[test]
fn env_config_is_used_when_flag_not_set() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("app");
    std::fs::create_dir_all(&root).expect("mkdir");

    let cfg_env = temp.path().join("env.yaml");
    write_project_config(&cfg_env, &root, "from_env");

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("app")
        .arg("--print-config")
        .arg("--dry-run")
        .env("JCA_TMUXER_CONFIG", &cfg_env)
        .assert()
        .success()
        .stdout(predicates::str::contains("command: from_env"));
}

#[test]
fn user_config_is_used_when_no_flag_or_env() {
    let temp = tempfile::tempdir().expect("tempdir");
    let xdg_home = temp.path().join("xdg");
    let user_cfg_dir = xdg_home.join("jca_tmuxer");
    std::fs::create_dir_all(&user_cfg_dir).expect("mkdir cfg dir");

    let root = temp.path().join("app");
    std::fs::create_dir_all(&root).expect("mkdir root");

    let user_cfg = user_cfg_dir.join("config.yaml");
    write_project_config(&user_cfg, &root, "from_user");

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("app")
        .arg("--print-config")
        .arg("--dry-run")
        .env("XDG_CONFIG_HOME", &xdg_home)
        .assert()
        .success()
        .stdout(predicates::str::contains("command: from_user"));
}

#[test]
fn local_config_overrides_user_config() {
    let temp = tempfile::tempdir().expect("tempdir");
    let xdg_home = temp.path().join("xdg");
    let user_cfg_dir = xdg_home.join("jca_tmuxer");
    std::fs::create_dir_all(&user_cfg_dir).expect("mkdir cfg dir");

    let root = temp.path().join("app");
    std::fs::create_dir_all(&root).expect("mkdir root");

    let user_cfg = user_cfg_dir.join("config.yaml");
    write_project_config(&user_cfg, &root, "from_user");

    let workspace = temp.path().join("repo");
    std::fs::create_dir_all(&workspace).expect("mkdir workspace");
    let local_cfg = workspace.join(".jca_tmuxer.yaml");
    write_project_config(&local_cfg, &root, "from_local");

    let mut cmd = Command::cargo_bin("jca_tmuxer").expect("bin");
    cmd.arg("app")
        .arg("--print-config")
        .arg("--dry-run")
        .env("XDG_CONFIG_HOME", &xdg_home)
        .current_dir(&workspace)
        .assert()
        .success()
        .stdout(predicates::str::contains("command: from_local"))
        .stdout(predicates::str::contains("command: from_user").not());
}
