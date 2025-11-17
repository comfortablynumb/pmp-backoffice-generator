// Library exports for testing and potential reuse

pub mod config;
pub mod data_source;
pub mod server;

// Re-export commonly used types
pub use config::{AppConfig, BackofficeConfig};
pub use server::AppState;
