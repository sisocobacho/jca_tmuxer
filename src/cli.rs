use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser, Clone)]
#[command(name = "jca_tmuxer")]
#[command(version)]
#[command(about = "Project-aware tmux session launcher")]
pub struct Args {
    #[arg(required_unless_present = "list")]
    pub project: Option<String>,

    #[arg(value_name = "ADHOC_COMMAND")]
    pub adhoc_commands: Vec<String>,

    #[arg(short = 'c', long = "config")]
    pub config: Option<PathBuf>,

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
}
