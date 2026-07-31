use jca_tmuxer::terminal::{detect_terminal_template, render_terminal_command};
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

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
fn detects_kitty_from_environment_hint() {
    let _guard = ENV_LOCK.lock().expect("lock env");
    let original = std::env::var_os("KITTY_PID");

    set_env_var("KITTY_PID", "123");
    let detected = detect_terminal_template();

    if let Some(v) = original {
        set_env_var("KITTY_PID", v);
    } else {
        remove_env_var("KITTY_PID");
    }

    assert_eq!(detected, Some("kitty tmux attach-session -t {session}"));
}

#[test]
fn renders_terminal_template_with_session_placeholder() {
    let rendered = render_terminal_command(
        "wezterm start -- tmux attach-session -t {session}",
        "my-session",
    );
    assert_eq!(
        rendered,
        "wezterm start -- tmux attach-session -t my-session"
    );
}
