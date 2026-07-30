use jca_tmuxer::tmux::attach_or_switch;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn install_fake_tmux(bin_dir: &std::path::Path) {
    let script_path = bin_dir.join("tmux");
    std::fs::write(
        &script_path,
        r#"#!/bin/sh
if [ -n "$TMUX_LOG" ]; then
  printf '%s\n' "$*" >> "$TMUX_LOG"
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

fn read_log(path: &std::path::Path) -> Vec<String> {
    std::fs::read_to_string(path)
        .expect("read log")
        .lines()
        .map(|s| s.to_string())
        .collect()
}

fn set_env_var<K: AsRef<std::ffi::OsStr>, V: AsRef<std::ffi::OsStr>>(key: K, value: V) {
    unsafe {
        std::env::set_var(key, value);
    }
}

fn remove_env_var<K: AsRef<std::ffi::OsStr>>(key: K) {
    unsafe {
        std::env::remove_var(key);
    }
}

#[test]
fn attach_or_switch_attaches_when_not_inside_tmux() {
    let _guard = ENV_LOCK.lock().expect("lock env");

    let temp = tempfile::tempdir().expect("tempdir");
    let bin_dir = temp.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    install_fake_tmux(&bin_dir);

    let log_path = temp.path().join("tmux.log");
    let original_path = std::env::var_os("PATH");
    let original_tmux = std::env::var_os("TMUX");

    set_env_var("PATH", format!("{}:/usr/bin:/bin", bin_dir.display()));
    set_env_var("TMUX_LOG", &log_path);
    remove_env_var("TMUX");

    let result = attach_or_switch("app");

    if let Some(v) = original_path {
        set_env_var("PATH", v);
    } else {
        remove_env_var("PATH");
    }
    if let Some(v) = original_tmux {
        set_env_var("TMUX", v);
    } else {
        remove_env_var("TMUX");
    }
    remove_env_var("TMUX_LOG");

    result.expect("attach_or_switch");
    let lines = read_log(&log_path);
    assert_eq!(lines.len(), 1);
    assert!(lines[0].starts_with("attach-session -t app"));
}

#[test]
fn attach_or_switch_switches_when_inside_tmux() {
    let _guard = ENV_LOCK.lock().expect("lock env");

    let temp = tempfile::tempdir().expect("tempdir");
    let bin_dir = temp.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    install_fake_tmux(&bin_dir);

    let log_path = temp.path().join("tmux.log");
    let original_path = std::env::var_os("PATH");
    let original_tmux = std::env::var_os("TMUX");

    set_env_var("PATH", format!("{}:/usr/bin:/bin", bin_dir.display()));
    set_env_var("TMUX_LOG", &log_path);
    set_env_var("TMUX", "inside");

    let result = attach_or_switch("app");

    if let Some(v) = original_path {
        set_env_var("PATH", v);
    } else {
        remove_env_var("PATH");
    }
    if let Some(v) = original_tmux {
        set_env_var("TMUX", v);
    } else {
        remove_env_var("TMUX");
    }
    remove_env_var("TMUX_LOG");

    result.expect("attach_or_switch");
    let lines = read_log(&log_path);
    assert_eq!(lines.len(), 1);
    assert!(lines[0].starts_with("switch-client -t app"));
}
