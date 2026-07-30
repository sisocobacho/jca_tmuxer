use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser, Clone)]
#[command(name = "jca_tmuxer")]
#[command(version)]
#[command(about = "Project-aware tmux session launcher")]
pub struct Args {
    #[arg(required_unless_present_any = ["list", "config_path", "edit_config"])]
    pub project: Option<String>,

    #[arg(value_name = "ADHOC_COMMAND")]
    pub adhoc_commands: Vec<String>,

    #[arg(short = 'c', long = "config")]
    pub config: Option<PathBuf>,

    #[arg(long = "save")]
    pub save: bool,

    #[arg(long = "remove", conflicts_with_all = ["save", "root", "new", "no_attach", "dry_run", "print_config", "list", "config_path", "edit_config"])]
    pub remove: bool,

    #[arg(long = "yes", requires = "remove")]
    pub yes: bool,

    #[arg(long = "root")]
    pub root: Option<PathBuf>,

    #[arg(short = 'n', long = "new")]
    pub new: bool,

    #[arg(long = "no-attach")]
    pub no_attach: bool,

    #[arg(long = "dry-run")]
    pub dry_run: bool,

    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[arg(long = "list")]
    pub list: bool,

    #[arg(long = "print-config")]
    pub print_config: bool,

    #[arg(long = "config-path")]
    pub config_path: bool,

    #[arg(long = "edit-config")]
    pub edit_config: bool,
}
