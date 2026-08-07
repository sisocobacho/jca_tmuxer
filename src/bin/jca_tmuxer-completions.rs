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

fn generate_for(shell: CompletionShell, bin_name: &str) {
    let mut command = jca_tmuxer::cli::Args::command();
    generate(
        Shell::from(shell),
        &mut command,
        bin_name,
        &mut io::stdout(),
    );

    match shell {
        CompletionShell::Bash => print!("{}", bash_dynamic_snippet(bin_name)),
        CompletionShell::Zsh => print!("{}", zsh_dynamic_snippet(bin_name)),
        CompletionShell::Fish | CompletionShell::Elvish | CompletionShell::Powershell => {}
    }
}

fn bash_dynamic_snippet(bin_name: &str) -> String {
    let completion_fn = format!("_{bin_name}");
    format!(
        r#"

# Dynamic project completion (bash): <PROJECT> and --projects values
__jca_tmuxer_dynamic_projects() {{
    {bin_name} --list 2>/dev/null
}}

__jca_tmuxer_dynamic_completion() {{
    local cur prev
    COMPREPLY=()
    cur="${{COMP_WORDS[COMP_CWORD]}}"
    prev="${{COMP_WORDS[COMP_CWORD-1]}}"

    if [[ "$prev" == "--projects" || $COMP_CWORD -eq 1 ]]; then
        local projects
        projects="$(__jca_tmuxer_dynamic_projects)"
        COMPREPLY=( $(compgen -W "$projects" -- "$cur") )
        if [[ ${{#COMPREPLY[@]}} -gt 0 ]]; then
            return 0
        fi
    fi

    if declare -F {completion_fn} >/dev/null 2>&1; then
        {completion_fn} "$1" "$cur" "$prev"
    fi
}}

complete -o nosort -o bashdefault -o default -F __jca_tmuxer_dynamic_completion {bin_name}
"#
    )
}

fn zsh_dynamic_snippet(bin_name: &str) -> String {
    let completion_fn = format!("_{bin_name}");
    let static_fn = format!("{completion_fn}_static");
    format!(
        r#"

# Dynamic project completion (zsh): <PROJECT> and --projects values
{static_fn}() {{
    {completion_fn} "$@"
}}

_jca_tmuxer_dynamic_projects() {{
    {bin_name} --list 2>/dev/null
}}

{completion_fn}() {{
    local -a projects
    local prev
    prev="${{words[CURRENT-1]}}"

    if [[ "$prev" == "--projects" || $CURRENT -eq 2 ]]; then
        projects=(${{(f)$(_jca_tmuxer_dynamic_projects)}})
        if (( ${{#projects[@]}} > 0 )); then
            compadd -- $projects
            return 0
        fi
    fi

    {static_fn} "$@"
}}

compdef {completion_fn} {bin_name}
"#
    )
}

fn main() {
    let args = Args::parse();

    generate_for(args.shell, &args.bin_name);
}
