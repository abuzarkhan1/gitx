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
    gitx_tui::run().await
}
