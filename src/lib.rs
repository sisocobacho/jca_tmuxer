pub mod adhoc;
pub mod cli;
pub mod config;
pub mod plan;
pub mod resolver;
pub mod tmux;

use anyhow::Result;
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

    if !args.dry_run {
        tmux::ensure_tmux_installed()?;
    }

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

    let mut exists = tmux::has_session(&project.session_name)?;
    if exists && args.new {
        tmux::kill_session(&project.session_name)?;
        exists = false;
    }

    if !exists {
        tmux::create_session(&project.session_name, &windows)?;
    } else if !args.adhoc_commands.is_empty() {
        let adhoc_windows = plan::build_adhoc_windows(&project, &args.adhoc_commands)?;
        tmux::append_windows(&project.session_name, &adhoc_windows)?;
    }

    let attach_allowed = std::io::stdin().is_terminal();
    if !args.no_attach && attach_allowed {
        tmux::attach_or_switch(&project.session_name)?;
    } else if !args.no_attach {
        eprintln!("stdin is not a TTY, skipping attach; use --no-attach to silence this message");
    }

    Ok(0)
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
