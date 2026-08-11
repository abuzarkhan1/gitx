use clap::Parser;
use gitx_cli::cli::Cli;

fn main() {
    let cli = Cli::parse();
    std::process::exit(gitx_cli::run(cli));
}
