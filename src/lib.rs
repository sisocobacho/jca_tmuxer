pub mod adhoc;
pub mod cli;
pub mod config;
pub mod plan;
pub mod resolver;
pub mod terminal;
pub mod tmux;

use anyhow::{Context, Result};
use cli::Args;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

pub fn run(args: Args) -> Result<i32> {
    if args.config_path {
        let config_path = config::resolve_write_path(&args)?;
        println!("{}", config_path.display());
        return Ok(0);
    }

    if args.edit_config {
        let config_path = config::ensure_config_exists(&args)?;
        config::open_in_editor(&config_path)?;
        return Ok(0);
    }

    if args.list {
        let cfg = config::load_from_args(&args)?;
        for name in cfg.projects.keys() {
            println!("{name}");
        }
        return Ok(0);
    }

    if !args.all && args.projects.is_empty() && (args.open_terminals || args.terminal_cmd.is_some())
    {
        anyhow::bail!("--open-terminals and --terminal-cmd require --all or --projects")
    }

    if args.all || !args.projects.is_empty() {
        return run_batch(&args);
    }

    let project_input = args.project.as_deref().ok_or_else(|| {
        anyhow::anyhow!(
            "project is required unless one of --list, --config-path, or --edit-config is used"
        )
    })?;

    if args.remove {
        remove_project_and_session(&args, project_input)?;
        return Ok(0);
    }

    let mut cfg = if args.save {
        config::load_from_args_allow_missing(&args)?
    } else {
        config::load_from_args(&args)?
    };

    if args.save && !cfg.projects.contains_key(project_input) {
        let root = resolve_root_for_save(&args, &cfg, project_input)?;
        let key = resolver::project_key_from_input(project_input, &root);
        let windows = if args.adhoc_commands.is_empty() {
            Some(cfg.defaults.windows.clone())
        } else {
            None
        };
        let saved = config::save_project(&args, &key, &root, windows)?;
        if saved {
            eprintln!("saved project '{key}' -> {}", root.display());
        }
        cfg = config::load_from_args(&args)?;
    }

    let project = resolver::resolve_project(&cfg, project_input)?;
    let windows = plan::build_windows(&cfg, &project, &args.adhoc_commands)?;

    if args.print_config {
        let out = serde_yaml::to_string(&windows)?;
        println!("{}", out.trim_end());
        return Ok(0);
    }

    if args.dry_run {
        for cmd in tmux::dry_run_commands(&project.session_name, &windows, args.new) {
            println!("{cmd}");
        }
        return Ok(0);
    }

    tmux::ensure_tmux_installed()?;
    apply_tmux_plan(&project, &windows, args.new, &args.adhoc_commands)?;

    let attach_allowed = std::io::stdin().is_terminal();
    if !args.no_attach && attach_allowed {
        tmux::attach_or_switch(&project.session_name)?;
    } else if !args.no_attach {
        eprintln!("stdin is not a TTY, skipping attach; use --no-attach to silence this message");
    }

    Ok(0)
}

fn run_batch(args: &Args) -> Result<i32> {
    validate_batch_args(args)?;

    let cfg = config::load_from_args(args)?;
    let targets = if args.all {
        cfg.projects.keys().cloned().collect::<Vec<_>>()
    } else {
        args.projects.clone()
    };

    if targets.is_empty() {
        anyhow::bail!("no projects selected for batch launch")
    }

    if !args.dry_run {
        tmux::ensure_tmux_installed()?;
    }

    let terminal_template = if args.open_terminals {
        Some(resolve_terminal_template(args)?)
    } else {
        None
    };

    let mut failures = Vec::new();

    for project_name in targets {
        let launch_result =
            launch_one_in_batch(args, &cfg, &project_name, terminal_template.as_deref());
        if let Err(err) = launch_result {
            failures.push(format!("{project_name}: {err}"));
        }
    }

    if !failures.is_empty() {
        eprintln!("batch launch completed with {} error(s):", failures.len());
        for failure in failures {
            eprintln!("- {failure}");
        }
        return Ok(1);
    }

    Ok(0)
}

fn validate_batch_args(args: &Args) -> Result<()> {
    if args.project.is_some() {
        anyhow::bail!("batch mode (--all/--projects) does not accept positional <PROJECT>")
    }
    if !args.adhoc_commands.is_empty() {
        anyhow::bail!("batch mode (--all/--projects) does not accept ad-hoc commands")
    }
    if args.save {
        anyhow::bail!("batch mode (--all/--projects) does not support --save")
    }
    if args.remove {
        anyhow::bail!("batch mode (--all/--projects) does not support --remove")
    }
    if args.root.is_some() {
        anyhow::bail!("batch mode (--all/--projects) does not support --root")
    }
    if args.print_config {
        anyhow::bail!("batch mode (--all/--projects) does not support --print-config")
    }
    Ok(())
}

fn launch_one_in_batch(
    args: &Args,
    cfg: &config::Config,
    project_name: &str,
    terminal_template: Option<&str>,
) -> Result<()> {
    let project = resolver::resolve_project(cfg, project_name)
        .with_context(|| format!("failed to resolve project '{project_name}'"))?;
    let windows = plan::build_windows(cfg, &project, &[])
        .with_context(|| format!("failed to build windows for project '{project_name}'"))?;

    if args.dry_run {
        for cmd in tmux::dry_run_commands(&project.session_name, &windows, args.new) {
            println!("[{project_name}] {cmd}");
        }
        if let Some(template) = terminal_template {
            let command = terminal::render_terminal_command(template, &project.session_name);
            println!("[{project_name}] {command}");
        }
        return Ok(());
    }

    apply_tmux_plan(&project, &windows, args.new, &[])
        .with_context(|| format!("failed to launch tmux session for project '{project_name}'"))?;

    if let Some(template) = terminal_template {
        let command = terminal::render_terminal_command(template, &project.session_name);
        terminal::spawn_in_new_terminal(&command).with_context(|| {
            format!(
                "failed to open terminal for session '{}'",
                project.session_name
            )
        })?;
    }

    Ok(())
}

fn apply_tmux_plan(
    project: &resolver::ResolvedProject,
    windows: &[plan::WindowPlan],
    force_new: bool,
    adhoc_commands: &[String],
) -> Result<()> {
    let mut exists = tmux::has_session(&project.session_name)?;
    if exists && force_new {
        tmux::kill_session(&project.session_name)?;
        exists = false;
    }

    if !exists {
        tmux::create_session(&project.session_name, windows)?;
    } else if !adhoc_commands.is_empty() {
        let adhoc_windows = plan::build_adhoc_windows(project, adhoc_commands)?;
        tmux::append_windows(&project.session_name, &adhoc_windows)?;
    }

    Ok(())
}

fn resolve_terminal_template(args: &Args) -> Result<String> {
    if let Some(template) = args.terminal_cmd.as_ref() {
        return Ok(template.clone());
    }
    if let Some(detected) = terminal::detect_terminal_template() {
        return Ok(detected.to_string());
    }

    anyhow::bail!(
        "unable to detect a supported terminal emulator; pass --terminal-cmd '<cmd with {{session}}>'"
    )
}

fn remove_project_and_session(args: &Args, project_input: &str) -> Result<()> {
    if !args.adhoc_commands.is_empty() {
        anyhow::bail!("--remove does not accept ad-hoc commands")
    }

    let (project_key, session_name) = remove_targets(project_input);

    if !args.yes {
        if !io::stdin().is_terminal() {
            anyhow::bail!("--remove requires confirmation from a TTY. Re-run with --yes")
        }
        print!(
            "Remove project '{}' from config and tmux session '{}'? [y/N] ",
            project_key, session_name
        );
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;
        if !is_yes_response(&response) {
            println!("Removal cancelled.");
            return Ok(());
        }
    }

    let removed_project = config::remove_project(args, &project_key)?;
    let removed_session = if tmux::ensure_tmux_installed().is_ok() {
        tmux::kill_session_if_exists(&session_name)?
    } else {
        false
    };

    if !removed_project && !removed_session {
        println!("Nothing was removed.");
        return Ok(());
    }

    if removed_project {
        println!("Removed: config project '{}'", project_key);
    }
    if removed_session {
        println!("Removed: tmux session '{}'", session_name);
    }

    Ok(())
}

fn remove_targets(project_input: &str) -> (String, String) {
    let input_path = std::path::Path::new(project_input);
    if input_path.exists()
        && let Ok(canonical) = std::fs::canonicalize(input_path)
    {
        let key = canonical
            .file_name()
            .map(|v| v.to_string_lossy().to_string())
            .unwrap_or_else(|| project_input.to_string());
        return (key.clone(), key);
    }
    (project_input.to_string(), project_input.to_string())
}

fn is_yes_response(value: &str) -> bool {
    matches!(value.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

fn resolve_root_for_save(
    args: &Args,
    cfg: &config::Config,
    project_input: &str,
) -> Result<PathBuf> {
    let root = if let Some(override_root) = args.root.as_ref() {
        resolver::expand_path(&override_root.to_string_lossy())
    } else {
        resolver::resolve_candidate_root_for_save(cfg, project_input).ok_or_else(|| {
            anyhow::anyhow!(
                "cannot resolve root for '{project_input}'. pass --root <path> or add search_paths"
            )
        })?
    };

    if !root.exists() {
        anyhow::bail!("save root does not exist: {}", root.display());
    }
    if !root.is_dir() {
        anyhow::bail!("save root is not a directory: {}", root.display());
    }

    let canonical = std::fs::canonicalize(&root)?;
    Ok(canonical)
}
