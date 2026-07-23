pub mod adhoc;
pub mod cli;
pub mod config;
pub mod plan;
pub mod resolver;
pub mod tmux;

use anyhow::Result;
use cli::Args;
use std::io::IsTerminal;
use std::path::PathBuf;

pub fn run(args: Args) -> Result<i32> {
    if args.list {
        let cfg = config::load_from_args(&args)?;
        for name in cfg.projects.keys() {
            println!("{name}");
        }
        return Ok(0);
    }

    let mut cfg = if args.save {
        config::load_from_args_allow_missing(&args)?
    } else {
        config::load_from_args(&args)?
    };
    let project_input = args
        .project
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("project is required unless --list is used"))?;

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
