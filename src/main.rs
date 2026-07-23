use clap::Parser;

fn main() {
    let args = jca_tmuxer::cli::Args::parse();
    let code = match jca_tmuxer::run(args) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("error: {err}");
            1
        }
    };
    std::process::exit(code);
}
