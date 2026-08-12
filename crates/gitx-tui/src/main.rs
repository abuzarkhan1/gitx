use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Diagnostics via `tracing` (docs/03 §11), controlled by `RUST_LOG`
    // (e.g. `RUST_LOG=gitx=debug gitx-tui`). stderr only.
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "warn".to_string());
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(filter))
        .with_writer(std::io::stderr)
        .try_init();
    // `[ui] vim_keys` from the effective config (defaults when unset): the
    // CLI passes the same flag when it launches the dashboard in-process.
    let vim_keys = gitx_core::config::Config::load(
        gitx_core::config::Config::default_path()
            .as_deref()
            .unwrap_or(std::path::Path::new("")),
    )
    .map(|c| c.ui.vim_keys)
    .unwrap_or(true);
    gitx_tui::run(vim_keys).await
}
