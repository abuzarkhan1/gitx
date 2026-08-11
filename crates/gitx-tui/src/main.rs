use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    gitx_tui::run().await
}
