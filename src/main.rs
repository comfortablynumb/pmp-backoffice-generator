mod config;
mod server;
mod data_source;

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting PMP Backoffice Generator...");

    // Load configuration
    let app_config = config::load_app_config("config/config.yaml").await?;
    let backoffices = config::load_backoffices("config/backoffices").await?;

    info!("Loaded {} backoffice(s)", backoffices.len());

    // Start web server
    server::start_server(app_config, backoffices).await?;

    Ok(())
}
