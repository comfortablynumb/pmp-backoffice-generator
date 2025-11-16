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
    #[serde(flatten)]
    pub action_type: ActionType,
    pub data_source: String,
    pub query: Option<String>,
    pub endpoint: Option<String>,
    pub required_scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ActionType {
    List {
        fields: Vec<FieldConfig>,
        #[serde(default)]
        config: ListActionConfig,
    },
    Form {
        fields: Vec<FieldConfig>,
        #[serde(default)]
        config: FormActionConfig,
    },
    View {
        fields: Vec<FieldConfig>,
    },
    Custom {
        fields: Vec<FieldConfig>,
    },
}

/// Configuration specific to list actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListActionConfig {
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    #[serde(default)]
    pub enable_pagination: bool,
    #[serde(default)]
    pub filters: Vec<FilterConfig>,
    #[serde(default)]
    pub sortable_fields: Vec<String>,
    #[serde(default)]
    pub default_sort_field: Option<String>,
    #[serde(default)]
    pub default_sort_order: SortOrder,
}

fn default_page_size() -> usize {
    20
}

impl Default for ListActionConfig {
    fn default() -> Self {
        Self {
            page_size: default_page_size(),
            enable_pagination: false,
            filters: Vec::new(),
            sortable_fields: Vec::new(),
            default_sort_field: None,
            default_sort_order: SortOrder::Ascending,
        }
    }
}

/// Filter configuration for list actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    pub id: String,
    pub name: String,
    pub field: String,
    pub filter_type: FilterType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FilterType {
    Text,
    Number,
    Date,
    Select { options: Vec<String> },
    Boolean,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::Ascending
    }
}

/// Configuration specific to form actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormActionConfig {
    #[serde(default)]
    pub submit_button_text: Option<String>,
    #[serde(default)]
    pub cancel_button_text: Option<String>,
    #[serde(default)]
    pub form_mode: FormMode,
    #[serde(default)]
    pub redirect_on_success: Option<String>,
    #[serde(default)]
    pub show_success_message: bool,
}

impl Default for FormActionConfig {
    fn default() -> Self {
        Self {
            submit_button_text: None,
            cancel_button_text: None,
            form_mode: FormMode::Create,
            redirect_on_success: None,
            show_success_message: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FormMode {
    Create,
    Update,
    Delete,
}

impl Default for FormMode {
    fn default() -> Self {
        FormMode::Create
    }
}

/// Field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldConfig {
    pub id: String,
    pub name: String,
    #[serde(flatten)]
    pub field_type: FieldType,
    #[serde(default = "default_false")]
    pub required: bool,
    #[serde(default = "default_true")]
    pub editable: bool,
    #[serde(default = "default_true")]
    pub visible: bool,
    pub default_value: Option<serde_json::Value>,
    pub placeholder: Option<String>,
    pub help_text: Option<String>,
}

fn default_false() -> bool {
    false
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "field_type", rename_all = "lowercase")]
pub enum FieldType {
    Text {
        #[serde(default)]
        config: TextFieldConfig,
    },
    Number {
        #[serde(default)]
        config: NumberFieldConfig,
    },
    Email {
        #[serde(default)]
        config: TextFieldConfig,
    },
    Password {
        #[serde(default)]
        config: PasswordFieldConfig,
    },
    Date {
        #[serde(default)]
        config: DateFieldConfig,
    },
    DateTime {
        #[serde(default)]
        config: DateFieldConfig,
    },
    Boolean {
        #[serde(default)]
        config: BooleanFieldConfig,
    },
    Select {
        #[serde(default)]
        config: SelectFieldConfig,
    },
    TextArea {
        #[serde(default)]
        config: TextAreaFieldConfig,
    },
    File {
        #[serde(default)]
        config: FileFieldConfig,
    },
}

/// Text field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextFieldConfig {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
}

impl Default for TextFieldConfig {
    fn default() -> Self {
        Self {
            min_length: None,
            max_length: None,
            pattern: None,
        }
    }
}

/// Number field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberFieldConfig {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    #[serde(default)]
    pub allow_decimals: bool,
}

impl Default for NumberFieldConfig {
    fn default() -> Self {
        Self {
            min: None,
            max: None,
            step: None,
            allow_decimals: true,
        }
    }
}

/// Password field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordFieldConfig {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    #[serde(default)]
    pub require_uppercase: bool,
    #[serde(default)]
    pub require_lowercase: bool,
    #[serde(default)]
    pub require_number: bool,
    #[serde(default)]
    pub require_special: bool,
}

impl Default for PasswordFieldConfig {
    fn default() -> Self {
        Self {
            min_length: None,
            max_length: None,
            require_uppercase: false,
            require_lowercase: false,
            require_number: false,
            require_special: false,
        }
    }
}

/// Date field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateFieldConfig {
    pub min_date: Option<String>,
    pub max_date: Option<String>,
    pub format: Option<String>,
}

impl Default for DateFieldConfig {
    fn default() -> Self {
        Self {
            min_date: None,
            max_date: None,
            format: None,
        }
    }
}

/// Boolean field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BooleanFieldConfig {
    pub true_label: Option<String>,
    pub false_label: Option<String>,
}

impl Default for BooleanFieldConfig {
    fn default() -> Self {
        Self {
            true_label: None,
            false_label: None,
        }
    }
}

/// Select field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectFieldConfig {
    pub options: Vec<SelectOption>,
    #[serde(default)]
    pub multiple: bool,
    #[serde(default)]
    pub searchable: bool,
}

impl Default for SelectFieldConfig {
    fn default() -> Self {
        Self {
            options: Vec::new(),
            multiple: false,
            searchable: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

/// TextArea field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextAreaFieldConfig {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    #[serde(default = "default_rows")]
    pub rows: usize,
}

fn default_rows() -> usize {
    4
}

impl Default for TextAreaFieldConfig {
    fn default() -> Self {
        Self {
            min_length: None,
            max_length: None,
            rows: default_rows(),
        }
    }
}

/// File field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileFieldConfig {
    pub accepted_types: Option<Vec<String>>,
    pub max_size_mb: Option<f64>,
    #[serde(default)]
    pub multiple: bool,
}

impl Default for FileFieldConfig {
    fn default() -> Self {
        Self {
            accepted_types: None,
            max_size_mb: None,
            multiple: false,
        }
    }
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
