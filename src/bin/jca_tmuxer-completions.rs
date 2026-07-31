use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::{Shell, generate};
use std::io;

#[derive(Debug, Parser)]
#[command(name = "jca_tmuxer-completions")]
#[command(version)]
#[command(about = "Generate shell completion scripts for jca_tmuxer")]
struct Args {
    #[arg(value_enum)]
    shell: CompletionShell,

    #[arg(long = "bin-name", default_value = "jca_tmuxer")]
    bin_name: String,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    Elvish,
    Powershell,
}

impl From<CompletionShell> for Shell {
    fn from(shell: CompletionShell) -> Self {
        match shell {
            CompletionShell::Bash => Shell::Bash,
            CompletionShell::Zsh => Shell::Zsh,
            CompletionShell::Fish => Shell::Fish,
            CompletionShell::Elvish => Shell::Elvish,
            CompletionShell::Powershell => Shell::PowerShell,
        }
    }
}

fn generate_for(shell: Shell, bin_name: &str) {
    let mut command = jca_tmuxer::cli::Args::command();
    generate(shell, &mut command, bin_name, &mut io::stdout());
}

fn main() {
    let args = Args::parse();

    let shell = Shell::from(args.shell);
    generate_for(shell, &args.bin_name);
}
