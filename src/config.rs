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
    #[serde(rename = "graphql")]
    GraphQL {
        endpoint: String,
        headers: Option<HashMap<String, String>>,
        auth: Option<ApiAuthConfig>,
    },
    #[serde(rename = "mongodb")]
    MongoDB {
        connection_string: String,
        database: String,
        collection: String,
    },
    #[serde(rename = "redis")]
    Redis {
        connection_string: String,
        key_prefix: Option<String>,
    },
    #[serde(rename = "elasticsearch")]
    Elasticsearch {
        nodes: Vec<String>,
        index: String,
        auth: Option<ApiAuthConfig>,
    },
    #[serde(rename = "grpc")]
    Grpc {
        endpoint: String,
        proto_file: String,
        service_name: String,
        tls_enabled: bool,
    },
    #[serde(rename = "kafka")]
    Kafka {
        brokers: Vec<String>,
        topic: String,
        group_id: String,
    },
    #[serde(rename = "s3")]
    S3 {
        bucket: String,
        region: String,
        access_key: Option<String>,
        secret_key: Option<String>,
        prefix: Option<String>,
    },
    #[serde(rename = "firebase")]
    Firebase {
        project_id: String,
        collection: String,
        credentials_path: Option<String>,
    },
    #[serde(rename = "supabase")]
    Supabase {
        url: String,
        api_key: String,
        table: String,
    },
    #[serde(rename = "websocket")]
    WebSocket {
        url: String,
        reconnect: bool,
        heartbeat_interval: Option<u32>,
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
#[derive(Default)]
pub enum SortOrder {
    #[default]
    Ascending,
    Descending,
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
#[derive(Default)]
pub enum FormMode {
    #[default]
    Create,
    Update,
    Delete,
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
    #[serde(default)]
    pub validations: Vec<ValidationRule>,
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
        config: EmailFieldConfig,
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
    Time {
        #[serde(default)]
        config: TimeFieldConfig,
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
    Url {
        #[serde(default)]
        config: UrlFieldConfig,
    },
    Phone {
        #[serde(default)]
        config: PhoneFieldConfig,
    },
    Currency {
        #[serde(default)]
        config: CurrencyFieldConfig,
    },
    Color {
        #[serde(default)]
        config: ColorFieldConfig,
    },
    Range {
        #[serde(default)]
        config: RangeFieldConfig,
    },
    Rating {
        #[serde(default)]
        config: RatingFieldConfig,
    },
    Tags {
        #[serde(default)]
        config: TagsFieldConfig,
    },
    Image {
        #[serde(default)]
        config: ImageFieldConfig,
    },
    Json {
        #[serde(default)]
        config: JsonFieldConfig,
    },
    Slug {
        #[serde(default)]
        config: SlugFieldConfig,
    },
    Weekday {
        #[serde(default)]
        config: WeekdayFieldConfig,
    },
    Month {
        #[serde(default)]
        config: MonthFieldConfig,
    },
    Geolocation {
        #[serde(default)]
        config: GeolocationFieldConfig,
    },
    Duration {
        #[serde(default)]
        config: DurationFieldConfig,
    },
    Percentage {
        #[serde(default)]
        config: PercentageFieldConfig,
    },
    Code {
        #[serde(default)]
        config: CodeFieldConfig,
    },
    Markdown {
        #[serde(default)]
        config: MarkdownFieldConfig,
    },
    RichText {
        #[serde(default)]
        config: RichTextFieldConfig,
    },
    IpAddress {
        #[serde(default)]
        config: IpAddressFieldConfig,
    },
    MultiCheckbox {
        #[serde(default)]
        config: MultiCheckboxFieldConfig,
    },
    Radio {
        #[serde(default)]
        config: RadioFieldConfig,
    },
    Autocomplete {
        #[serde(default)]
        config: AutocompleteFieldConfig,
    },
    Signature {
        #[serde(default)]
        config: SignatureFieldConfig,
    },
    Video {
        #[serde(default)]
        config: VideoFieldConfig,
    },
    Audio {
        #[serde(default)]
        config: AudioFieldConfig,
    },
    Barcode {
        #[serde(default)]
        config: BarcodeFieldConfig,
    },
    DateTimeRange {
        #[serde(default)]
        config: DateTimeRangeFieldConfig,
    },
    Slider {
        #[serde(default)]
        config: SliderFieldConfig,
    },
    ColorPalette {
        #[serde(default)]
        config: ColorPaletteFieldConfig,
    },
}

/// Text field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct TextFieldConfig {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
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
#[derive(Default)]
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


/// Date field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct DateFieldConfig {
    pub min_date: Option<String>,
    pub max_date: Option<String>,
    pub format: Option<String>,
}


/// Boolean field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct BooleanFieldConfig {
    pub true_label: Option<String>,
    pub false_label: Option<String>,
}


/// Select field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct SelectFieldConfig {
    pub options: Vec<SelectOption>,
    #[serde(default)]
    pub multiple: bool,
    #[serde(default)]
    pub searchable: bool,
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

/// Email field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailFieldConfig {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    #[serde(default)]
    pub allow_multiple: bool,
    pub domain_whitelist: Option<Vec<String>>,
    pub domain_blacklist: Option<Vec<String>>,
}

impl Default for EmailFieldConfig {
    fn default() -> Self {
        Self {
            min_length: None,
            max_length: None,
            pattern: Some(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$".to_string()),
            allow_multiple: false,
            domain_whitelist: None,
            domain_blacklist: None,
        }
    }
}

/// File field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct FileFieldConfig {
    pub accepted_types: Option<Vec<String>>,
    pub max_size_mb: Option<f64>,
    #[serde(default)]
    pub multiple: bool,
}


/// URL field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlFieldConfig {
    pub allowed_protocols: Option<Vec<String>>,
    pub require_protocol: bool,
    pub allow_localhost: bool,
}

impl Default for UrlFieldConfig {
    fn default() -> Self {
        Self {
            allowed_protocols: Some(vec!["http".to_string(), "https".to_string()]),
            require_protocol: true,
            allow_localhost: false,
        }
    }
}

/// Phone field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct PhoneFieldConfig {
    pub format: Option<String>,
    pub country_code: Option<String>,
    pub allow_extensions: bool,
    pub validation_pattern: Option<String>,
}


/// Currency field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyFieldConfig {
    pub currency_code: String,
    pub symbol: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub allow_negative: bool,
    pub decimal_places: usize,
}

impl Default for CurrencyFieldConfig {
    fn default() -> Self {
        Self {
            currency_code: "USD".to_string(),
            symbol: Some("$".to_string()),
            min: None,
            max: None,
            allow_negative: false,
            decimal_places: 2,
        }
    }
}

/// Color field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorFieldConfig {
    pub format: ColorFormat,
    pub allow_alpha: bool,
    pub presets: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorFormat {
    Hex,
    Rgb,
    Rgba,
    Hsl,
}

impl Default for ColorFieldConfig {
    fn default() -> Self {
        Self {
            format: ColorFormat::Hex,
            allow_alpha: false,
            presets: None,
        }
    }
}

/// Range field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeFieldConfig {
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub show_value: bool,
    pub show_ticks: bool,
    pub labels: Option<HashMap<String, String>>,
}

impl Default for RangeFieldConfig {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 100.0,
            step: 1.0,
            show_value: true,
            show_ticks: false,
            labels: None,
        }
    }
}

/// Time field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeFieldConfig {
    pub format: Option<String>,
    pub min_time: Option<String>,
    pub max_time: Option<String>,
    pub step_minutes: Option<u32>,
}

impl Default for TimeFieldConfig {
    fn default() -> Self {
        Self {
            format: Some("HH:MM".to_string()),
            min_time: None,
            max_time: None,
            step_minutes: None,
        }
    }
}

/// Rating field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingFieldConfig {
    pub max_rating: u8,
    pub icon: RatingIcon,
    pub allow_half: bool,
    pub allow_clear: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RatingIcon {
    Star,
    Heart,
    Circle,
    Thumb,
}

impl Default for RatingFieldConfig {
    fn default() -> Self {
        Self {
            max_rating: 5,
            icon: RatingIcon::Star,
            allow_half: false,
            allow_clear: true,
        }
    }
}

/// Tags field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagsFieldConfig {
    pub min_tags: Option<usize>,
    pub max_tags: Option<usize>,
    pub min_tag_length: Option<usize>,
    pub max_tag_length: Option<usize>,
    pub predefined_tags: Option<Vec<String>>,
    pub allow_custom: bool,
    pub case_sensitive: bool,
}

impl Default for TagsFieldConfig {
    fn default() -> Self {
        Self {
            min_tags: None,
            max_tags: None,
            min_tag_length: None,
            max_tag_length: Some(50),
            predefined_tags: None,
            allow_custom: true,
            case_sensitive: false,
        }
    }
}

/// Image field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageFieldConfig {
    pub max_size_mb: Option<f64>,
    pub accepted_formats: Option<Vec<String>>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub min_width: Option<u32>,
    pub min_height: Option<u32>,
    pub aspect_ratio: Option<String>,
    pub allow_crop: bool,
    pub allow_resize: bool,
    #[serde(default)]
    pub multiple: bool,
}

impl Default for ImageFieldConfig {
    fn default() -> Self {
        Self {
            max_size_mb: Some(5.0),
            accepted_formats: Some(vec![
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
                "gif".to_string(),
                "webp".to_string(),
            ]),
            max_width: None,
            max_height: None,
            min_width: None,
            min_height: None,
            aspect_ratio: None,
            allow_crop: false,
            allow_resize: false,
            multiple: false,
        }
    }
}

/// JSON field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonFieldConfig {
    pub schema: Option<String>,
    pub pretty_print: bool,
    pub validate_on_change: bool,
    pub min_depth: Option<usize>,
    pub max_depth: Option<usize>,
}

impl Default for JsonFieldConfig {
    fn default() -> Self {
        Self {
            schema: None,
            pretty_print: true,
            validate_on_change: true,
            min_depth: None,
            max_depth: None,
        }
    }
}

/// Slug field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlugFieldConfig {
    pub source_field: Option<String>,
    pub separator: String,
    pub lowercase: bool,
    pub max_length: Option<usize>,
    pub allow_unicode: bool,
}

impl Default for SlugFieldConfig {
    fn default() -> Self {
        Self {
            source_field: None,
            separator: "-".to_string(),
            lowercase: true,
            max_length: Some(100),
            allow_unicode: false,
        }
    }
}

/// Weekday field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeekdayFieldConfig {
    pub format: WeekdayFormat,
    pub start_day: Option<String>,
    pub multiple: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WeekdayFormat {
    Short,  // Mon, Tue, Wed
    Long,   // Monday, Tuesday, Wednesday
    Number, // 1-7
}

impl Default for WeekdayFieldConfig {
    fn default() -> Self {
        Self {
            format: WeekdayFormat::Long,
            start_day: Some("monday".to_string()),
            multiple: false,
        }
    }
}

/// Month field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthFieldConfig {
    pub format: MonthFormat,
    pub multiple: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MonthFormat {
    Short,  // Jan, Feb, Mar
    Long,   // January, February, March
    Number, // 1-12
}

impl Default for MonthFieldConfig {
    fn default() -> Self {
        Self {
            format: MonthFormat::Long,
            multiple: false,
        }
    }
}

/// Geolocation field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeolocationFieldConfig {
    pub enable_map: bool,
    pub default_zoom: u8,
    pub min_lat: Option<f64>,
    pub max_lat: Option<f64>,
    pub min_lng: Option<f64>,
    pub max_lng: Option<f64>,
    pub enable_geocoding: bool,
}

impl Default for GeolocationFieldConfig {
    fn default() -> Self {
        Self {
            enable_map: true,
            default_zoom: 13,
            min_lat: Some(-90.0),
            max_lat: Some(90.0),
            min_lng: Some(-180.0),
            max_lng: Some(180.0),
            enable_geocoding: false,
        }
    }
}

/// Duration field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurationFieldConfig {
    pub format: DurationFormat,
    pub min_duration: Option<u32>,
    pub max_duration: Option<u32>,
    pub step_minutes: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DurationFormat {
    HoursMinutes, // 2h 30m
    Minutes,      // 150 minutes
    Seconds,      // 9000 seconds
}

impl Default for DurationFieldConfig {
    fn default() -> Self {
        Self {
            format: DurationFormat::HoursMinutes,
            min_duration: None,
            max_duration: None,
            step_minutes: Some(15),
        }
    }
}

/// Percentage field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PercentageFieldConfig {
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub decimal_places: usize,
    pub show_slider: bool,
}

impl Default for PercentageFieldConfig {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 100.0,
            step: 1.0,
            decimal_places: 0,
            show_slider: false,
        }
    }
}

/// Code field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeFieldConfig {
    pub language: String,
    pub theme: CodeTheme,
    pub line_numbers: bool,
    pub min_lines: Option<usize>,
    pub max_lines: Option<usize>,
    pub read_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CodeTheme {
    Light,
    Dark,
    Monokai,
    Github,
}

impl Default for CodeFieldConfig {
    fn default() -> Self {
        Self {
            language: "javascript".to_string(),
            theme: CodeTheme::Github,
            line_numbers: true,
            min_lines: None,
            max_lines: None,
            read_only: false,
        }
    }
}

/// Markdown field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownFieldConfig {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub enable_preview: bool,
    pub enable_toolbar: bool,
    pub allowed_elements: Option<Vec<String>>,
}

impl Default for MarkdownFieldConfig {
    fn default() -> Self {
        Self {
            min_length: None,
            max_length: None,
            enable_preview: true,
            enable_toolbar: true,
            allowed_elements: None,
        }
    }
}

/// Rich text field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichTextFieldConfig {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub toolbar_items: Vec<String>,
    pub allow_images: bool,
    pub allow_links: bool,
    pub allow_tables: bool,
}

impl Default for RichTextFieldConfig {
    fn default() -> Self {
        Self {
            min_length: None,
            max_length: None,
            toolbar_items: vec![
                "bold".to_string(),
                "italic".to_string(),
                "underline".to_string(),
                "list".to_string(),
            ],
            allow_images: true,
            allow_links: true,
            allow_tables: false,
        }
    }
}

/// IP Address field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpAddressFieldConfig {
    pub version: IpVersion,
    pub allow_private: bool,
    pub allow_loopback: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IpVersion {
    V4,
    V6,
    Both,
}

impl Default for IpAddressFieldConfig {
    fn default() -> Self {
        Self {
            version: IpVersion::V4,
            allow_private: true,
            allow_loopback: false,
        }
    }
}

/// Multi-checkbox field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiCheckboxFieldConfig {
    pub options: Vec<CheckboxOption>,
    pub min_selections: Option<usize>,
    pub max_selections: Option<usize>,
    pub layout: CheckboxLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckboxOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckboxLayout {
    Vertical,
    Horizontal,
    Grid,
}

impl Default for MultiCheckboxFieldConfig {
    fn default() -> Self {
        Self {
            options: Vec::new(),
            min_selections: None,
            max_selections: None,
            layout: CheckboxLayout::Vertical,
        }
    }
}

/// Radio field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadioFieldConfig {
    pub options: Vec<RadioOption>,
    pub layout: RadioLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadioOption {
    pub value: String,
    pub label: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RadioLayout {
    Vertical,
    Horizontal,
    Cards,
}

impl Default for RadioFieldConfig {
    fn default() -> Self {
        Self {
            options: Vec::new(),
            layout: RadioLayout::Vertical,
        }
    }
}

/// Autocomplete field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocompleteFieldConfig {
    pub options: Vec<String>,
    pub min_chars: usize,
    pub max_results: usize,
    pub allow_custom: bool,
    pub case_sensitive: bool,
}

impl Default for AutocompleteFieldConfig {
    fn default() -> Self {
        Self {
            options: Vec::new(),
            min_chars: 1,
            max_results: 10,
            allow_custom: false,
            case_sensitive: false,
        }
    }
}

/// Signature field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureFieldConfig {
    pub width: u32,
    pub height: u32,
    pub pen_color: String,
    pub background_color: String,
    pub line_width: u8,
}

impl Default for SignatureFieldConfig {
    fn default() -> Self {
        Self {
            width: 400,
            height: 200,
            pen_color: "#000000".to_string(),
            background_color: "#FFFFFF".to_string(),
            line_width: 2,
        }
    }
}

/// Video field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoFieldConfig {
    pub max_size_mb: Option<f64>,
    pub accepted_formats: Option<Vec<String>>,
    pub max_duration_seconds: Option<u32>,
    pub enable_preview: bool,
    pub multiple: bool,
}

impl Default for VideoFieldConfig {
    fn default() -> Self {
        Self {
            max_size_mb: Some(100.0),
            accepted_formats: Some(vec![
                "mp4".to_string(),
                "webm".to_string(),
                "ogg".to_string(),
            ]),
            max_duration_seconds: None,
            enable_preview: true,
            multiple: false,
        }
    }
}

/// Audio field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFieldConfig {
    pub max_size_mb: Option<f64>,
    pub accepted_formats: Option<Vec<String>>,
    pub max_duration_seconds: Option<u32>,
    pub enable_preview: bool,
    pub multiple: bool,
}

impl Default for AudioFieldConfig {
    fn default() -> Self {
        Self {
            max_size_mb: Some(50.0),
            accepted_formats: Some(vec![
                "mp3".to_string(),
                "wav".to_string(),
                "ogg".to_string(),
            ]),
            max_duration_seconds: None,
            enable_preview: true,
            multiple: false,
        }
    }
}

/// Barcode field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarcodeFieldConfig {
    pub format: BarcodeFormat,
    pub enable_scanner: bool,
    pub validation_pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BarcodeFormat {
    Qr,
    Ean13,
    Ean8,
    Upca,
    Code128,
    Code39,
}

impl Default for BarcodeFieldConfig {
    fn default() -> Self {
        Self {
            format: BarcodeFormat::Qr,
            enable_scanner: false,
            validation_pattern: None,
        }
    }
}

/// Date/Time range field configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DateTimeRangeFieldConfig {
    pub include_time: bool,
    pub min_date: Option<String>,
    pub max_date: Option<String>,
    pub min_range_days: Option<u32>,
    pub max_range_days: Option<u32>,
}

/// Slider field configuration (multi-value)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliderFieldConfig {
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub handles: usize,
    pub show_values: bool,
    pub show_ticks: bool,
    pub range_mode: bool,
}

impl Default for SliderFieldConfig {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 100.0,
            step: 1.0,
            handles: 2,
            show_values: true,
            show_ticks: false,
            range_mode: true,
        }
    }
}

/// Color palette field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPaletteFieldConfig {
    pub max_colors: usize,
    pub default_colors: Option<Vec<String>>,
    pub allow_custom: bool,
}

impl Default for ColorPaletteFieldConfig {
    fn default() -> Self {
        Self {
            max_colors: 5,
            default_colors: None,
            allow_custom: true,
        }
    }
}

/// Custom validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_type: ValidationType,
    pub message: Option<String>,
    pub condition: Option<ValidationCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ValidationType {
    Required {
        value: bool,
    },
    MinLength {
        value: usize,
    },
    MaxLength {
        value: usize,
    },
    Pattern {
        regex: String,
    },
    Min {
        value: f64,
    },
    Max {
        value: f64,
    },
    Email,
    Url,
    Phone,
    CustomFunction {
        function_name: String,
    },
    DependsOn {
        field: String,
        expected_value: serde_json::Value,
    },
    UniqueIn {
        field_list: Vec<String>,
    },
    MatchField {
        field: String,
    },
    CreditCard,
    Ipv4,
    Ipv6,
    Uuid,
    DateRange {
        start_field: String,
        end_field: String,
    },
    FileSize {
        max_size_mb: f64,
    },
    FileType {
        allowed_types: Vec<String>,
    },
    StrongPassword {
        min_length: usize,
        require_uppercase: bool,
        require_lowercase: bool,
        require_number: bool,
        require_special: bool,
    },
    AlphaNumeric,
    Luhn,
    MacAddress,
    Isbn,
    Iban,
    Ssn,
    PostalCode {
        country_code: String,
    },
    Base64,
    Json,
    Hex,
    Ascii,
    NotEmpty,
    Future,
    Past,
    MinAge {
        years: u8,
    },
    MaxAge {
        years: u8,
    },
    Between {
        min: f64,
        max: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCondition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Contains,
    NotContains,
    In,
    NotIn,
}

/// Load application configuration
pub async fn load_app_config<P: AsRef<Path>>(path: P) -> Result<AppConfig> {
    let content = tokio::fs::read_to_string(path.as_ref())
        .await
        .context("Failed to read app config file")?;

    let config: AppConfig =
        serde_yaml::from_str(&content).context("Failed to parse app config YAML")?;

    Ok(config)
}

/// Load all backoffice configurations from a directory
pub async fn load_backoffices<P: AsRef<Path>>(dir: P) -> Result<Vec<BackofficeConfig>> {
    let mut backoffices = Vec::new();

    for entry in WalkDir::new(dir.as_ref())
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().and_then(|s| s.to_str()) == Some("yaml")
                || e.path().extension().and_then(|s| s.to_str()) == Some("yml")
        })
    {
        let content = tokio::fs::read_to_string(entry.path())
            .await
            .context(format!(
                "Failed to read backoffice config: {:?}",
                entry.path()
            ))?;

        let config: BackofficeConfig = serde_yaml::from_str(&content).context(format!(
            "Failed to parse backoffice config: {:?}",
            entry.path()
        ))?;

        backoffices.push(config);
    }

    Ok(backoffices)
}
