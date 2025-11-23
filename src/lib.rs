// Library exports for testing and potential reuse

pub mod config;
pub mod data_source;
pub mod relationships;
pub mod server;
pub mod validation;

// Re-export commonly used types
pub use config::{AppConfig, BackofficeConfig};
pub use server::AppState;
