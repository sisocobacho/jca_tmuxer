use crate::plan::WindowPlan;
use anyhow::{Context, Result, bail};
use std::process::{Command, ExitStatus};

pub fn dry_run_commands(
    session_name: &str,
    windows: &[WindowPlan],
    force_new: bool,
) -> Vec<String> {
    let mut cmds = Vec::new();
    cmds.push(format!("tmux has-session -t {session_name}"));
    if force_new {
        cmds.push(format!("tmux kill-session -t {session_name}"));
    }

    if windows.is_empty() {
        return cmds;
    }

    let first = &windows[0];
    let first_dir = shell_escape_path(&first.panes[0].cwd.to_string_lossy());
    cmds.push(format!(
        "tmux new-session -d -s {} -n {} -c {}",
        shell_escape(session_name),
        shell_escape(&first.name),
        first_dir
    ));

    for (widx, window) in windows.iter().enumerate() {
        if widx > 0 {
            let dir = shell_escape_path(&window.panes[0].cwd.to_string_lossy());
            cmds.push(format!(
                "tmux new-window -t {} -n {} -c {}",
                shell_escape(session_name),
                shell_escape(&window.name),
                dir
            ));
        }

        for (pidx, pane) in window.panes.iter().enumerate() {
            if pidx > 0 {
                cmds.push(format!(
                    "tmux split-window -t {}:{} -c {}",
                    shell_escape(session_name),
                    shell_escape(&window.name),
                    shell_escape_path(&pane.cwd.to_string_lossy())
                ));
            }

            cmds.push(format!(
                "tmux send-keys -t {}:{}.{pidx} {} C-m",
                shell_escape(session_name),
                shell_escape(&window.name),
                shell_escape(&pane.command)
            ));

            if let Some(size) = pane.size {
                cmds.push(format!(
                    "tmux resize-pane -t {}:{}.{pidx} -p {}",
                    shell_escape(session_name),
                    shell_escape(&window.name),
                    size
                ));
            }
        }

        cmds.push(format!(
            "tmux select-layout -t {}:{} {}",
            shell_escape(session_name),
            shell_escape(&window.name),
            shell_escape(&window.layout)
        ));
    }

    cmds.push(format!(
        "tmux attach-session -t {}",
        shell_escape(session_name)
    ));
    cmds
}

pub fn ensure_tmux_installed() -> anyhow::Result<()> {
    which::which("tmux")
        .map(|_| ())
        .map_err(|_| anyhow::anyhow!("tmux is not installed or not in PATH"))
}

pub fn has_session(session_name: &str) -> Result<bool> {
    let status = Command::new("tmux")
        .args(["has-session", "-t", session_name])
        .status()
        .context("failed to run tmux has-session")?;
    Ok(status.success())
}

pub fn kill_session(session_name: &str) -> Result<()> {
    run_tmux(["kill-session", "-t", session_name]).map(|_| ())
}

pub fn kill_session_if_exists(session_name: &str) -> Result<bool> {
    if has_session(session_name)? {
        kill_session(session_name)?;
        return Ok(true);
    }
    Ok(false)
}

pub fn create_session(session_name: &str, windows: &[WindowPlan]) -> Result<()> {
    if windows.is_empty() {
        bail!("no windows to create")
    }

    let first = &windows[0];
    let first_dir = first.panes[0].cwd.to_string_lossy().to_string();
    run_tmux([
        "new-session",
        "-d",
        "-s",
        session_name,
        "-n",
        &first.name,
        "-c",
        &first_dir,
    ])?;

    seed_window(session_name, first)?;

    for window in windows.iter().skip(1) {
        create_window(session_name, window)?;
        seed_window(session_name, window)?;
    }

    Ok(())
}

pub fn append_windows(session_name: &str, windows: &[WindowPlan]) -> Result<()> {
    for window in windows {
        create_window(session_name, window)?;
        seed_window(session_name, window)?;
    }
    Ok(())
}

pub fn attach_or_switch(session_name: &str) -> Result<()> {
    if std::env::var("TMUX")
        .ok()
        .filter(|v| !v.is_empty())
        .is_some()
    {
        run_tmux(["switch-client", "-t", session_name]).map(|_| ())
    } else {
        run_tmux(["attach-session", "-t", session_name]).map(|_| ())
    }
}

fn create_window(session_name: &str, window: &WindowPlan) -> Result<()> {
    let dir = window.panes[0].cwd.to_string_lossy().to_string();
    run_tmux([
        "new-window",
        "-t",
        session_name,
        "-n",
        &window.name,
        "-c",
        &dir,
    ])
    .map(|_| ())
}

fn seed_window(session_name: &str, window: &WindowPlan) -> Result<()> {
    let window_target = format!("{session_name}:{}", window.name);

    for (pane_idx, pane) in window.panes.iter().enumerate() {
        if pane_idx > 0 {
            let dir = pane.cwd.to_string_lossy().to_string();
            run_tmux(["split-window", "-t", &window_target, "-c", &dir]).map(|_| ())?;
        }

        let pane_target = format!("{window_target}.{pane_idx}");
        run_tmux(["send-keys", "-t", &pane_target, &pane.command, "C-m"]).map(|_| ())?;

        if let Some(size) = pane.size {
            let size_arg = size.to_string();
            run_tmux(["resize-pane", "-t", &pane_target, "-p", &size_arg]).map(|_| ())?;
        }
    }

    run_tmux(["select-layout", "-t", &window_target, &window.layout]).map(|_| ())
}

fn run_tmux<const N: usize>(args: [&str; N]) -> Result<ExitStatus> {
    Command::new("tmux")
        .args(args)
        .status()
        .with_context(|| format!("failed to run tmux with args: {}", args.join(" ")))
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

fn shell_escape_path(input: &str) -> String {
    shell_escape(input)
}
