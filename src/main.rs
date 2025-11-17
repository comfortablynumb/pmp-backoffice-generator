mod config;
mod data_source;
mod server;

use anyhow::Result;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing with environment filter support
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();

    info!("╔══════════════════════════════════════════════════════════╗");
    info!("║    PMP Backoffice Generator - Starting Application      ║");
    info!("╚══════════════════════════════════════════════════════════╝");

    // Load application configuration
    info!("Loading application configuration...");
    let app_config = match config::load_app_config("config/config.yaml").await {
        Ok(config) => {
            info!(
                host = %config.server.host,
                port = %config.server.port,
                security_enabled = %config.security.as_ref().map(|s| s.enabled).unwrap_or(false),
                "Application configuration loaded successfully"
            );
            config
        }
        Err(e) => {
            error!(error = %e, "Failed to load application configuration");
            return Err(e);
        }
    };

    // Load backoffice configurations
    info!("Loading backoffice configurations from config/backoffices...");
    let backoffices = match config::load_backoffices("config/backoffices").await {
        Ok(configs) => {
            info!(
                count = configs.len(),
                "Successfully loaded backoffice configurations"
            );

            // Log details about each backoffice
            for (idx, backoffice) in configs.iter().enumerate() {
                info!(
                    index = idx + 1,
                    id = %backoffice.id,
                    name = %backoffice.name,
                    sections = backoffice.sections.len(),
                    data_sources = backoffice.data_sources.len(),
                    relationships = backoffice.relationships.len(),
                    "Loaded backoffice"
                );

                // Log sections
                for section in &backoffice.sections {
                    info!(
                        backoffice = %backoffice.id,
                        section_id = %section.id,
                        section_name = %section.name,
                        actions = section.actions.len(),
                        has_audit = section.audit.is_some(),
                        "  └─ Section configured"
                    );
                }
            }

            configs
        }
        Err(e) => {
            error!(error = %e, "Failed to load backoffice configurations");
            return Err(e);
        }
    };

    if backoffices.is_empty() {
        warn!("No backoffice configurations found! The application will start but have no backends available.");
    }

    // Start web server
    info!("Starting web server...");
    let bind_addr = format!("{}:{}", app_config.server.host, app_config.server.port);
    info!(
        address = %bind_addr,
        backoffices = backoffices.len(),
        "Starting HTTP server"
    );

    match server::start_server(app_config, backoffices).await {
        Ok(_) => {
            info!("Server shutdown gracefully");
            Ok(())
        }
        Err(e) => {
            error!(error = %e, "Server encountered an error");
            Err(e)
        }
    }
}
