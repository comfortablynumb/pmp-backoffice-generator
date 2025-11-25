use crate::audit::{AuditLogger, AuditOperation};
use crate::config::{ActionType, AppConfig, BackofficeConfig};
use crate::data_source;
use crate::relationships;
use crate::validation;
use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tower_http::services::ServeDir;
use tracing::{debug, error, info, warn};

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub backoffices: Vec<BackofficeConfig>,
    pub audit_logger: Arc<AuditLogger>,
}

/// Start the web server
pub async fn start_server(config: AppConfig, backoffices: Vec<BackofficeConfig>) -> Result<()> {
    let backoffice_count = backoffices.len();

    debug!(backoffices = backoffice_count, "Creating application state");

    let audit_logger = Arc::new(AuditLogger::new("logs/audit"));

    let state = Arc::new(AppState {
        config: config.clone(),
        backoffices,
        audit_logger,
    });

    // Build the router
    debug!("Setting up API routes");
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/config", get(config_handler))
        .route("/api/backoffices", get(backoffices_handler))
        .route("/api/backoffices/:id", get(backoffice_handler))
        .route(
            "/api/backoffices/:backoffice_id/sections/:section_id/actions/:action_id",
            get(execute_action_handler)
                .post(execute_mutation_handler)
                .delete(execute_delete_handler),
        )
        .route("/api/docs", get(api_docs_handler))
        .route("/openapi.yaml", get(openapi_spec_handler))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    info!("Routes configured:");
    info!("  GET  /                     - Main application page");
    info!("  GET  /api/config           - Application configuration");
    info!("  GET  /api/backoffices      - List all backoffices");
    info!("  GET  /api/backoffices/:id  - Get backoffice by ID");
    info!("  GET  /api/docs             - API documentation (Swagger UI)");
    info!("  GET  /openapi.yaml         - OpenAPI specification");
    info!("  *    /static/*             - Static files");

    let addr = format!("{}:{}", config.server.host, config.server.port);
    debug!(address = %addr, "Binding TCP listener");

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => {
            let local_addr = listener.local_addr()?;
            info!("╔══════════════════════════════════════════════════════════╗");
            info!("║              Server Ready and Listening                  ║");
            info!("╚══════════════════════════════════════════════════════════╝");
            info!("🚀 Server started successfully!");
            info!("📍 Address: http://{}", local_addr);
            info!("📊 API Docs: http://{}/api/docs", local_addr);
            info!("📚 Backoffices loaded: {}", backoffice_count);
            info!("");
            info!("Press Ctrl+C to stop the server");
            info!("");
            listener
        }
        Err(e) => {
            error!(address = %addr, error = %e, "Failed to bind to address");
            return Err(e.into());
        }
    };

    info!("Server is now accepting connections...");

    match axum::serve(listener, app).await {
        Ok(_) => {
            info!("Server stopped");
            Ok(())
        }
        Err(e) => {
            error!(error = %e, "Server error");
            Err(e.into())
        }
    }
}

/// Serve the main HTML page
async fn index_handler() -> impl IntoResponse {
    Html(include_str!("../static/index.html"))
}

/// Serve OpenAPI specification
async fn openapi_spec_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("content-type", "application/x-yaml")],
        include_str!("../openapi.yaml"),
    )
}

/// Serve Swagger UI for API documentation
async fn api_docs_handler() -> impl IntoResponse {
    Html(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>API Documentation - PMP Backoffice Generator</title>
    <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css" />
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-standalone-preset.js"></script>
    <script>
        window.onload = function() {
            window.ui = SwaggerUIBundle({
                url: '/openapi.yaml',
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIStandalonePreset
                ],
                plugins: [
                    SwaggerUIBundle.plugins.DownloadUrl
                ],
                layout: "StandaloneLayout"
            });
        };
    </script>
</body>
</html>
    "#,
    )
}

/// Get application configuration
async fn config_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(state.config.clone())
}

/// Get all backoffices
async fn backoffices_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(state.backoffices.clone())
}

/// Get a specific backoffice
async fn backoffice_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.backoffices.iter().find(|b| b.id == id) {
        Some(backoffice) => (StatusCode::OK, Json(backoffice)).into_response(),
        None => (StatusCode::NOT_FOUND, "Backoffice not found").into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct ActionQuery {
    page: Option<usize>,
    page_size: Option<usize>,
    #[allow(dead_code)]
    sort_by: Option<String>,
    #[allow(dead_code)]
    sort_order: Option<String>,
    #[serde(flatten)]
    params: HashMap<String, String>,
}

/// Execute a query action (GET)
async fn execute_action_handler(
    State(state): State<Arc<AppState>>,
    Path((backoffice_id, section_id, action_id)): Path<(String, String, String)>,
    Query(query): Query<ActionQuery>,
) -> impl IntoResponse {
    use crate::config::ActionType;

    // Find the backoffice
    let backoffice = match state.backoffices.iter().find(|b| b.id == backoffice_id) {
        Some(b) => b,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Backoffice not found"})),
            )
                .into_response()
        }
    };

    // Find the section
    let section = match backoffice.sections.iter().find(|s| s.id == section_id) {
        Some(s) => s,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Section not found"})),
            )
                .into_response()
        }
    };

    // Find the action
    let action = match section.actions.iter().find(|a| a.id == action_id) {
        Some(a) => a,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Action not found"})),
            )
                .into_response()
        }
    };

    // Get the data source
    let ds_config = match backoffice.data_sources.get(&action.data_source) {
        Some(ds) => ds,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Data source not found"})),
            )
                .into_response()
        }
    };

    // Create data source instance
    let data_source = match data_source::create_data_source(ds_config).await {
        Ok(ds) => ds,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    };

    // Execute the query
    let query_str = action
        .query
        .as_deref()
        .or(action.endpoint.as_deref())
        .unwrap_or("");
    let params_converted: HashMap<String, Value> = query
        .params
        .iter()
        .map(|(k, v)| (k.clone(), Value::String(v.clone())))
        .collect();

    match &action.action_type {
        ActionType::List { fields, config } => {
            match data_source
                .execute_query(query_str, Some(&params_converted))
                .await
            {
                Ok(mut result) => {
                    let total_items = result.len();

                    // Handle pagination if enabled
                    if config.enable_pagination {
                        let page = query.page.unwrap_or(1);
                        let page_size = query.page_size.unwrap_or(config.page_size);
                        let start = (page - 1) * page_size;

                        result = result.into_iter().skip(start).take(page_size).collect();

                        (
                            StatusCode::OK,
                            Json(serde_json::json!({
                                "data": result,
                                "fields": fields,
                                "config": config,
                                "pagination": {
                                    "page": page,
                                    "page_size": page_size,
                                    "total_items": total_items,
                                    "total_pages": total_items.div_ceil(page_size),
                                }
                            })),
                        )
                            .into_response()
                    } else {
                        (
                            StatusCode::OK,
                            Json(serde_json::json!({
                                "data": result,
                                "fields": fields,
                                "config": config,
                            })),
                        )
                            .into_response()
                    }
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
                    .into_response(),
            }
        }
        ActionType::View { fields } | ActionType::Custom { fields } => {
            match data_source
                .execute_query(query_str, Some(&params_converted))
                .await
            {
                Ok(result) => (
                    StatusCode::OK,
                    Json(serde_json::json!({"data": result, "fields": fields})),
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
                    .into_response(),
            }
        }
        ActionType::Form { fields, config } => {
            // For form actions in GET, return the form configuration
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "fields": fields,
                    "config": config,
                })),
            )
                .into_response()
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct MutationData {
    #[serde(flatten)]
    data: HashMap<String, Value>,
}

/// Execute a mutation action (POST)
async fn execute_mutation_handler(
    State(state): State<Arc<AppState>>,
    Path((backoffice_id, section_id, action_id)): Path<(String, String, String)>,
    Json(payload): Json<MutationData>,
) -> impl IntoResponse {
    info!(
        backoffice_id = %backoffice_id,
        section_id = %section_id,
        action_id = %action_id,
        "Processing mutation request"
    );

    // Find the backoffice
    let backoffice = match state.backoffices.iter().find(|b| b.id == backoffice_id) {
        Some(b) => b,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Backoffice not found"})),
            )
                .into_response()
        }
    };

    // Find the section
    let section = match backoffice.sections.iter().find(|s| s.id == section_id) {
        Some(s) => s,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Section not found"})),
            )
                .into_response()
        }
    };

    // Find the action
    let action = match section.actions.iter().find(|a| a.id == action_id) {
        Some(a) => a,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Action not found"})),
            )
                .into_response()
        }
    };

    // Get fields from the action for validation
    let fields = match &action.action_type {
        ActionType::Form { fields, .. } => fields,
        ActionType::Custom { fields } => fields,
        _ => {
            warn!("Mutation attempted on non-form action");
            &vec![]
        }
    };

    // Step 1: Validate data against field configurations
    info!("Validating request data");
    match validation::validate_data(&payload.data, fields) {
        Ok(validation_errors) => {
            if !validation_errors.is_empty() {
                let error_messages: Vec<serde_json::Value> = validation_errors
                    .iter()
                    .map(|e| {
                        serde_json::json!({
                            "field": e.field,
                            "message": e.message
                        })
                    })
                    .collect();

                warn!(error_count = validation_errors.len(), "Validation failed");

                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "Validation failed",
                        "validation_errors": error_messages
                    })),
                )
                    .into_response();
            }
        }
        Err(e) => {
            error!(error = %e, "Validation error");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Validation error: {}", e)})),
            )
                .into_response();
        }
    }

    // Step 2: Create data sources map for relationship validation
    let mut data_sources_map: HashMap<String, Box<dyn data_source::DataSource>> = HashMap::new();
    for (name, ds_config) in &backoffice.data_sources {
        match data_source::create_data_source(ds_config).await {
            Ok(ds) => {
                data_sources_map.insert(name.clone(), ds);
            }
            Err(e) => {
                error!(
                    data_source = %name,
                    error = %e,
                    "Failed to create data source"
                );
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("Failed to create data source: {}", e)})),
                )
                    .into_response();
            }
        }
    }

    // Step 3: Validate foreign key relationships
    info!("Validating foreign key relationships");
    match relationships::validate_foreign_keys(
        &payload.data,
        &section_id,
        backoffice,
        &data_sources_map,
    )
    .await
    {
        Ok(relationship_errors) => {
            if !relationship_errors.is_empty() {
                let error_messages: Vec<serde_json::Value> = relationship_errors
                    .iter()
                    .map(|e| {
                        serde_json::json!({
                            "relationship_id": e.relationship_id,
                            "field": e.field,
                            "message": e.message
                        })
                    })
                    .collect();

                warn!(
                    error_count = relationship_errors.len(),
                    "Relationship validation failed"
                );

                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "Relationship validation failed",
                        "relationship_errors": error_messages
                    })),
                )
                    .into_response();
            }
        }
        Err(e) => {
            error!(error = %e, "Relationship validation error");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Relationship validation error: {}", e)})),
            )
                .into_response();
        }
    }

    // Step 4: Validate many-to-many relationships
    match relationships::validate_many_to_many(
        &payload.data,
        &section_id,
        backoffice,
        &data_sources_map,
    )
    .await
    {
        Ok(m2m_errors) => {
            if !m2m_errors.is_empty() {
                let error_messages: Vec<serde_json::Value> = m2m_errors
                    .iter()
                    .map(|e| {
                        serde_json::json!({
                            "relationship_id": e.relationship_id,
                            "field": e.field,
                            "message": e.message
                        })
                    })
                    .collect();

                warn!(
                    error_count = m2m_errors.len(),
                    "Many-to-many relationship validation failed"
                );

                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "Many-to-many relationship validation failed",
                        "relationship_errors": error_messages
                    })),
                )
                    .into_response();
            }
        }
        Err(e) => {
            error!(error = %e, "Many-to-many validation error");
        }
    }

    // Get the data source for execution
    let data_source = match data_sources_map.get(&action.data_source) {
        Some(ds) => ds,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Data source not found"})),
            )
                .into_response()
        }
    };

    // Step 5: Execute the mutation
    let query_str = action
        .query
        .as_deref()
        .or(action.endpoint.as_deref())
        .unwrap_or("");

    info!(query = %query_str, "Executing mutation");

    match data_source.execute_mutation(query_str, &payload.data).await {
        Ok(result) => {
            info!("Mutation executed successfully");

            // Log audit trail if enabled
            if AuditLogger::should_audit(&section.audit, &AuditOperation::Create) {
                let record_id = result.as_str().map(|s| s.to_string());
                let audit_entry = AuditLogger::create_entry(
                    section_id.clone(),
                    record_id.clone(),
                    &payload.data,
                    None, // TODO: Extract user ID from auth header
                );

                if let Err(e) = state.audit_logger.log(audit_entry) {
                    warn!(error = %e, "Failed to log audit entry");
                }
            }

            (
                StatusCode::OK,
                Json(serde_json::json!({"success": true, "data": result})),
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "Mutation execution failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

/// Execute a delete action (DELETE)
async fn execute_delete_handler(
    State(state): State<Arc<AppState>>,
    Path((backoffice_id, section_id, action_id)): Path<(String, String, String)>,
    Query(query): Query<ActionQuery>,
) -> impl IntoResponse {
    info!(
        backoffice_id = %backoffice_id,
        section_id = %section_id,
        action_id = %action_id,
        "Processing delete request"
    );

    // Find the backoffice
    let backoffice = match state.backoffices.iter().find(|b| b.id == backoffice_id) {
        Some(b) => b,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Backoffice not found"})),
            )
                .into_response()
        }
    };

    // Find the section
    let section = match backoffice.sections.iter().find(|s| s.id == section_id) {
        Some(s) => s,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Section not found"})),
            )
                .into_response()
        }
    };

    // Extract record ID from query params
    let record_id = match query.params.get("id") {
        Some(id) => id,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Record ID is required"})),
            )
                .into_response()
        }
    };

    // Create data sources map
    let mut data_sources_map: HashMap<String, Box<dyn data_source::DataSource>> = HashMap::new();
    for (name, ds_config) in &backoffice.data_sources {
        match data_source::create_data_source(ds_config).await {
            Ok(ds) => {
                data_sources_map.insert(name.clone(), ds);
            }
            Err(e) => {
                error!(
                    data_source = %name,
                    error = %e,
                    "Failed to create data source"
                );
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("Failed to create data source: {}", e)})),
                )
                    .into_response();
            }
        }
    }

    // Step 1: Handle cascade delete operations
    info!(record_id = %record_id, "Processing cascade delete");
    match relationships::handle_cascade_delete(
        record_id,
        &section_id,
        backoffice,
        &data_sources_map,
    )
    .await
    {
        Ok(cascade_ops) => {
            if !cascade_ops.is_empty() {
                info!(
                    operation_count = cascade_ops.len(),
                    "Executing cascade delete operations"
                );

                // Execute cascade operations
                if let Err(e) = relationships::execute_cascade_operations(
                    &cascade_ops,
                    backoffice,
                    &data_sources_map,
                )
                .await
                {
                    error!(error = %e, "Failed to execute cascade operations");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": format!("Failed to execute cascade operations: {}", e)
                        })),
                    )
                        .into_response();
                }
            }
        }
        Err(e) => {
            error!(error = %e, "Failed to process cascade delete");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Failed to process cascade delete: {}", e)})),
            )
                .into_response();
        }
    }

    // Step 2: Delete the record itself
    let action = match section.actions.iter().find(|a| a.id == action_id) {
        Some(a) => a,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Action not found"})),
            )
                .into_response()
        }
    };

    let data_source = match data_sources_map.get(&action.data_source) {
        Some(ds) => ds,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Data source not found"})),
            )
                .into_response()
        }
    };

    // Build delete query
    let delete_query = format!("DELETE FROM {} WHERE id = '{}'", section_id, record_id);

    info!(query = %delete_query, "Executing delete");

    let mut delete_data = HashMap::new();
    delete_data.insert("id".to_string(), Value::String(record_id.clone()));

    match data_source
        .execute_mutation(&delete_query, &delete_data)
        .await
    {
        Ok(result) => {
            info!("Delete executed successfully");

            // Log audit trail if enabled
            if AuditLogger::should_audit(&section.audit, &AuditOperation::Delete) {
                let audit_entry = AuditLogger::delete_entry(
                    section_id.clone(),
                    record_id.clone(),
                    None, // TODO: Fetch old data before delete if needed
                    None, // TODO: Extract user ID from auth header
                );

                if let Err(e) = state.audit_logger.log(audit_entry) {
                    warn!(error = %e, "Failed to log audit entry");
                }
            }

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "data": result,
                    "message": format!("Record {} deleted successfully", record_id)
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "Delete execution failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        ActionConfig, ActionType, BackofficeConfig, DataSourceConfig, FieldConfig, SectionConfig,
        ServerConfig,
    };

    fn create_test_state() -> Arc<AppState> {
        let config = AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
            },
            security: Some(crate::config::SecurityConfig {
                enabled: false,
                jwt_secret: None,
            }),
        };

        let audit_logger = Arc::new(AuditLogger::new("logs/audit/test"));

        let backoffice = BackofficeConfig {
            id: "test".to_string(),
            name: "Test Backoffice".to_string(),
            description: Some("Test description".to_string()),
            data_sources: HashMap::from([(
                "test_api".to_string(),
                DataSourceConfig::Api {
                    base_url: "https://api.example.com".to_string(),
                    headers: Some(HashMap::new()),
                    auth: None,
                },
            )]),
            relationships: vec![],
            sections: vec![SectionConfig {
                id: "test_section".to_string(),
                name: "Test Section".to_string(),
                icon: Some("fa-test".to_string()),
                actions: vec![ActionConfig {
                    id: "test_action".to_string(),
                    name: "Test Action".to_string(),
                    action_type: ActionType::List {
                        fields: vec![FieldConfig {
                            id: "id".to_string(),
                            name: "ID".to_string(),
                            field_type: crate::config::FieldType::Text {
                                config: Default::default(),
                            },
                            required: false,
                            editable: false,
                            visible: true,
                            default_value: None,
                            placeholder: None,
                            help_text: None,
                            validations: vec![],
                            relationship_id: None,
                        }],
                        config: Default::default(),
                    },
                    data_source: "test_api".to_string(),
                    required_scopes: vec![],
                    query: Some("SELECT * FROM users".to_string()),
                    endpoint: None,
                }],
                audit: None,
            }],
        };

        Arc::new(AppState {
            config,
            backoffices: vec![backoffice],
            audit_logger,
        })
    }

    #[tokio::test]
    async fn test_config_handler() {
        let state = create_test_state();
        let response = config_handler(State(state.clone())).await;
        let json = response.into_response();
        assert_eq!(json.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_backoffices_handler() {
        let state = create_test_state();
        let response = backoffices_handler(State(state.clone())).await;
        let json = response.into_response();
        assert_eq!(json.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_backoffice_handler_found() {
        let state = create_test_state();
        let response = backoffice_handler(State(state.clone()), Path("test".to_string())).await;
        let json = response.into_response();
        assert_eq!(json.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_backoffice_handler_not_found() {
        let state = create_test_state();
        let response =
            backoffice_handler(State(state.clone()), Path("nonexistent".to_string())).await;
        let json = response.into_response();
        assert_eq!(json.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_app_state_clone() {
        let state = create_test_state();
        let cloned = state.clone();
        assert_eq!(state.backoffices.len(), cloned.backoffices.len());
        assert_eq!(state.config.server.port, cloned.config.server.port);
    }
}
