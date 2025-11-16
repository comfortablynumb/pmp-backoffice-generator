use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub security: Option<SecurityConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enabled: bool,
    pub jwt_secret: Option<String>,
}

/// Backoffice configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackofficeConfig {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub data_sources: HashMap<String, DataSourceConfig>,
    pub sections: Vec<SectionConfig>,
}

/// Data source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DataSourceConfig {
    #[serde(rename = "database")]
    Database {
        connection_string: String,
        db_type: DatabaseType,
    },
    #[serde(rename = "api")]
    Api {
        base_url: String,
        headers: Option<HashMap<String, String>>,
        auth: Option<ApiAuthConfig>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    Postgres,
    MySQL,
    Sqlite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiAuthConfig {
    pub auth_type: String,
    pub token: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

/// Section configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionConfig {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub actions: Vec<ActionConfig>,
}

/// Action configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionConfig {
    pub id: String,
    pub name: String,
    pub action_type: ActionType,
    pub data_source: String,
    pub query: Option<String>,
    pub endpoint: Option<String>,
    pub fields: Vec<FieldConfig>,
    pub required_scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionType {
    List,
    Create,
    Update,
    Delete,
    View,
    Custom,
}

/// Field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldConfig {
    pub id: String,
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
    pub editable: bool,
    pub visible: bool,
    pub default_value: Option<serde_json::Value>,
    pub validation: Option<ValidationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    Text,
    Number,
    Email,
    Password,
    Date,
    DateTime,
    Boolean,
    Select,
    TextArea,
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub options: Option<Vec<String>>,
}

/// Load application configuration
pub async fn load_app_config<P: AsRef<Path>>(path: P) -> Result<AppConfig> {
    let content = tokio::fs::read_to_string(path.as_ref())
        .await
        .context("Failed to read app config file")?;

    let config: AppConfig = serde_yaml::from_str(&content)
        .context("Failed to parse app config YAML")?;

    Ok(config)
}

/// Load all backoffice configurations from a directory
pub async fn load_backoffices<P: AsRef<Path>>(dir: P) -> Result<Vec<BackofficeConfig>> {
    let mut backoffices = Vec::new();

    for entry in WalkDir::new(dir.as_ref())
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("yaml") ||
                    e.path().extension().and_then(|s| s.to_str()) == Some("yml"))
    {
        let content = tokio::fs::read_to_string(entry.path())
            .await
            .context(format!("Failed to read backoffice config: {:?}", entry.path()))?;

        let config: BackofficeConfig = serde_yaml::from_str(&content)
            .context(format!("Failed to parse backoffice config: {:?}", entry.path()))?;

        backoffices.push(config);
    }

    Ok(backoffices)
}
