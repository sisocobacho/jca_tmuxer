use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn render_terminal_command(template: &str, session_name: &str) -> String {
    template.replace("{session}", &shell_escape(session_name))
}

pub fn detect_terminal_template() -> Option<&'static str> {
    if std::env::var_os("KITTY_PID").is_some() {
        return Some("kitty tmux attach-session -t {session}");
    }
    if std::env::var_os("WEZTERM_PANE").is_some() {
        return Some("wezterm start -- tmux attach-session -t {session}");
    }
    if std::env::var_os("KONSOLE_VERSION").is_some() {
        return Some("konsole -e tmux attach-session -t {session}");
    }
    if let Ok(term_program) = std::env::var("TERM_PROGRAM")
        && term_program.eq_ignore_ascii_case("wezterm")
    {
        return Some("wezterm start -- tmux attach-session -t {session}");
    }

    detect_from_process_tree(std::process::id()).and_then(|name| template_for_terminal(&name))
}

pub fn spawn_in_new_terminal(command: &str) -> Result<()> {
    Command::new("/bin/sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("failed to launch terminal with command: {command}"))?;
    Ok(())
}

fn detect_from_process_tree(start_pid: u32) -> Option<String> {
    let mut pid = start_pid;
    for _ in 0..64 {
        if let Some(name) = process_name(pid)
            && template_for_terminal(&name).is_some()
        {
            return Some(name);
        }

        let Some(ppid) = parent_pid(pid) else {
            break;
        };
        if ppid == 0 || ppid == pid {
            break;
        }
        pid = ppid;
    }
    None
}

fn process_name(pid: u32) -> Option<String> {
    let exe_path = fs::read_link(format!("/proc/{pid}/exe")).ok()?;
    exe_path
        .file_name()
        .and_then(|v| v.to_str())
        .map(|v| v.to_string())
}

fn parent_pid(pid: u32) -> Option<u32> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let marker = stat.rfind(") ")?;
    let tail = &stat[marker + 2..];
    let mut fields = tail.split_whitespace();
    let _state = fields.next()?;
    fields.next()?.parse::<u32>().ok()
}

fn template_for_terminal(name: &str) -> Option<&'static str> {
    let base = Path::new(name)
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or(name);

    match base {
        "kitty" => Some("kitty tmux attach-session -t {session}"),
        "wezterm" | "wezterm-gui" => Some("wezterm start -- tmux attach-session -t {session}"),
        "gnome-terminal" | "gnome-terminal-server" => {
            Some("gnome-terminal -- tmux attach-session -t {session}")
        }
        "alacritty" => Some("alacritty -e tmux attach-session -t {session}"),
        "konsole" => Some("konsole -e tmux attach-session -t {session}"),
        "xterm" => Some("xterm -e tmux attach-session -t {session}"),
        "foot" => Some("foot tmux attach-session -t {session}"),
        "tilix" => Some("tilix -e tmux attach-session -t {session}"),
        _ => None,
    }
}

fn shell_escape(input: &str) -> String {
    if input
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "._-/".contains(c))
    {
        input.to_string()
    } else {
        format!("'{}'", input.replace('\'', "'\\''"))
    }
}
