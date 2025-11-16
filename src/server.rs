use crate::config::{AppConfig, BackofficeConfig};
use crate::data_source;
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
use tracing::info;

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub backoffices: Vec<BackofficeConfig>,
}

/// Start the web server
pub async fn start_server(config: AppConfig, backoffices: Vec<BackofficeConfig>) -> Result<()> {
    let state = Arc::new(AppState {
        config: config.clone(),
        backoffices,
    });

    // Build the router
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/config", get(config_handler))
        .route("/api/backoffices", get(backoffices_handler))
        .route("/api/backoffices/:id", get(backoffice_handler))
        .route("/api/backoffices/:backoffice_id/sections/:section_id/actions/:action_id",
               get(execute_action_handler).post(execute_mutation_handler))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

/// Serve the main HTML page
async fn index_handler() -> impl IntoResponse {
    Html(include_str!("../static/index.html"))
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
    #[serde(flatten)]
    params: HashMap<String, String>,
}

/// Execute a query action (GET)
async fn execute_action_handler(
    State(state): State<Arc<AppState>>,
    Path((backoffice_id, section_id, action_id)): Path<(String, String, String)>,
    Query(query): Query<ActionQuery>,
) -> impl IntoResponse {
    // Find the backoffice
    let backoffice = match state.backoffices.iter().find(|b| b.id == backoffice_id) {
        Some(b) => b,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Backoffice not found"}))).into_response(),
    };

    // Find the section
    let section = match backoffice.sections.iter().find(|s| s.id == section_id) {
        Some(s) => s,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Section not found"}))).into_response(),
    };

    // Find the action
    let action = match section.actions.iter().find(|a| a.id == action_id) {
        Some(a) => a,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Action not found"}))).into_response(),
    };

    // Get the data source
    let ds_config = match backoffice.data_sources.get(&action.data_source) {
        Some(ds) => ds,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Data source not found"}))).into_response(),
    };

    // Create data source instance
    let data_source = match data_source::create_data_source(ds_config) {
        Ok(ds) => ds,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    };

    // Execute the query
    let query_str = action.query.as_deref().or(action.endpoint.as_deref()).unwrap_or("");
    let params_converted: HashMap<String, Value> = query.params.iter()
        .map(|(k, v)| (k.clone(), Value::String(v.clone())))
        .collect();

    match data_source.execute_query(query_str, Some(&params_converted)).await {
        Ok(result) => (StatusCode::OK, Json(serde_json::json!({"data": result, "fields": action.fields}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
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
    // Find the backoffice
    let backoffice = match state.backoffices.iter().find(|b| b.id == backoffice_id) {
        Some(b) => b,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Backoffice not found"}))).into_response(),
    };

    // Find the section
    let section = match backoffice.sections.iter().find(|s| s.id == section_id) {
        Some(s) => s,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Section not found"}))).into_response(),
    };

    // Find the action
    let action = match section.actions.iter().find(|a| a.id == action_id) {
        Some(a) => a,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Action not found"}))).into_response(),
    };

    // Get the data source
    let ds_config = match backoffice.data_sources.get(&action.data_source) {
        Some(ds) => ds,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Data source not found"}))).into_response(),
    };

    // Create data source instance
    let data_source = match data_source::create_data_source(ds_config) {
        Ok(ds) => ds,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    };

    // Execute the mutation
    let query_str = action.query.as_deref().or(action.endpoint.as_deref()).unwrap_or("");

    match data_source.execute_mutation(query_str, &payload.data).await {
        Ok(result) => (StatusCode::OK, Json(serde_json::json!({"success": true, "data": result}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}
