use clap::Parser;
use gitx_cli::cli::Cli;

fn main() {
    let cli = Cli::parse();
    init_tracing(&cli);
    std::process::exit(gitx_cli::run(cli));
}

/// Diagnostics via `tracing` (docs/03 §11): off by default; enabled with
/// `RUST_LOG` (e.g. `RUST_LOG=gitx=info,gitx_cli=debug gitx scan`) or
/// `--verbose`. stderr only — stdout stays machine-clean.
fn init_tracing(cli: &Cli) {
    let filter = match std::env::var("RUST_LOG") {
        Ok(f) if !f.is_empty() => f,
        _ if cli.verbose => "gitx=info,gitx_cli=debug".to_string(),
        _ => "warn".to_string(),
    };
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(filter))
        .with_writer(std::io::stderr)
        .try_init();
}
