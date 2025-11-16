use crate::config::{DataSourceConfig, DatabaseType};
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;

/// Data source trait for executing queries
#[async_trait::async_trait]
pub trait DataSource: Send + Sync {
    async fn execute_query(&self, query: &str, params: Option<&HashMap<String, Value>>) -> Result<Vec<HashMap<String, Value>>>;
    async fn execute_mutation(&self, query: &str, data: &HashMap<String, Value>) -> Result<Value>;
}

/// Database data source
pub struct DatabaseDataSource {
    #[allow(dead_code)]
    config: DataSourceConfig,
    #[allow(dead_code)]
    db_type: DatabaseType,
}

impl DatabaseDataSource {
    pub fn new(config: DataSourceConfig, db_type: DatabaseType) -> Self {
        Self { config, db_type }
    }
}

#[async_trait::async_trait]
impl DataSource for DatabaseDataSource {
    async fn execute_query(&self, query: &str, _params: Option<&HashMap<String, Value>>) -> Result<Vec<HashMap<String, Value>>> {
        // This is a placeholder implementation
        // In a real implementation, you would use sqlx to execute the query
        tracing::warn!("Database query execution not yet fully implemented: {}", query);

        // Return mock data for demonstration
        Ok(vec![])
    }

    async fn execute_mutation(&self, query: &str, _data: &HashMap<String, Value>) -> Result<Value> {
        tracing::warn!("Database mutation execution not yet fully implemented: {}", query);
        Ok(Value::Bool(true))
    }
}

/// API data source
pub struct ApiDataSource {
    base_url: String,
    client: reqwest::Client,
    headers: HashMap<String, String>,
}

impl ApiDataSource {
    pub fn new(base_url: String, headers: Option<HashMap<String, String>>) -> Self {
        let client = reqwest::Client::new();
        Self {
            base_url,
            client,
            headers: headers.unwrap_or_default(),
        }
    }
}

#[async_trait::async_trait]
impl DataSource for ApiDataSource {
    async fn execute_query(&self, endpoint: &str, params: Option<&HashMap<String, Value>>) -> Result<Vec<HashMap<String, Value>>> {
        let url = format!("{}/{}", self.base_url, endpoint);

        let mut request = self.client.get(&url);

        // Add headers
        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        // Add query parameters
        if let Some(params) = params {
            for (key, value) in params {
                if let Some(s) = value.as_str() {
                    request = request.query(&[(key, s)]);
                }
            }
        }

        let response = request.send().await?;
        let data: Value = response.json().await?;

        // Try to convert the response to a list of objects
        match data {
            Value::Array(arr) => {
                let mut result = Vec::new();
                for item in arr {
                    if let Value::Object(obj) = item {
                        let map: HashMap<String, Value> = obj.into_iter().collect();
                        result.push(map);
                    }
                }
                Ok(result)
            }
            Value::Object(obj) => {
                let map: HashMap<String, Value> = obj.into_iter().collect();
                Ok(vec![map])
            }
            _ => Err(anyhow!("Unexpected API response format")),
        }
    }

    async fn execute_mutation(&self, endpoint: &str, data: &HashMap<String, Value>) -> Result<Value> {
        let url = format!("{}/{}", self.base_url, endpoint);

        let mut request = self.client.post(&url);

        // Add headers
        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        let response = request.json(&data).send().await?;
        let result: Value = response.json().await?;

        Ok(result)
    }
}

/// Factory to create data sources
pub fn create_data_source(config: &DataSourceConfig) -> Result<Box<dyn DataSource>> {
    match config {
        DataSourceConfig::Database { db_type, .. } => {
            Ok(Box::new(DatabaseDataSource::new(config.clone(), db_type.clone())))
        }
        DataSourceConfig::Api { base_url, headers, .. } => {
            Ok(Box::new(ApiDataSource::new(base_url.clone(), headers.clone())))
        }
    }
}
