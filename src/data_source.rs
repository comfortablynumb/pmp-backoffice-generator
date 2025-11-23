use crate::config::{DataSourceConfig, DatabaseType};
use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::collections::HashMap;

/// Data source trait for executing queries
#[async_trait::async_trait]
pub trait DataSource: Send + Sync {
    async fn execute_query(
        &self,
        query: &str,
        params: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<HashMap<String, Value>>>;
    async fn execute_mutation(&self, query: &str, data: &HashMap<String, Value>) -> Result<Value>;
}

/// Database data source
pub struct DatabaseDataSource {
    #[cfg(feature = "database")]
    pool: sqlx::AnyPool,
    #[cfg(not(feature = "database"))]
    _config: DataSourceConfig,
    #[cfg(not(feature = "database"))]
    _db_type: DatabaseType,
}

impl DatabaseDataSource {
    pub async fn new(config: DataSourceConfig, db_type: DatabaseType) -> Result<Self> {
        #[cfg(feature = "database")]
        {
            let connection_string = match &config {
                DataSourceConfig::Database {
                    connection_string, ..
                } => connection_string,
                _ => return Err(anyhow!("Invalid config for DatabaseDataSource")),
            };

            tracing::info!(
                db_type = ?db_type,
                "Connecting to database"
            );

            let pool = sqlx::AnyPool::connect(connection_string)
                .await
                .context("Failed to connect to database")?;

            tracing::info!("Database connection established successfully");

            Ok(Self { pool })
        }

        #[cfg(not(feature = "database"))]
        {
            tracing::warn!("Database feature not enabled, returning stub implementation");
            Ok(Self {
                _config: config,
                _db_type: db_type,
            })
        }
    }
}

#[async_trait::async_trait]
impl DataSource for DatabaseDataSource {
    async fn execute_query(
        &self,
        query: &str,
        params: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<HashMap<String, Value>>> {
        #[cfg(feature = "database")]
        {
            use sqlx::{Column, Row, TypeInfo};

            tracing::debug!(query = %query, "Executing database query");

            let mut query_builder = sqlx::query(query);

            // Bind parameters if provided
            if let Some(params) = params {
                for (key, value) in params.iter() {
                    tracing::trace!(param = %key, value = ?value, "Binding parameter");
                    match value {
                        Value::String(s) => query_builder = query_builder.bind(s.clone()),
                        Value::Number(n) => {
                            if let Some(i) = n.as_i64() {
                                query_builder = query_builder.bind(i);
                            } else if let Some(f) = n.as_f64() {
                                query_builder = query_builder.bind(f);
                            }
                        }
                        Value::Bool(b) => query_builder = query_builder.bind(*b),
                        Value::Null => query_builder = query_builder.bind(None::<String>),
                        _ => {
                            query_builder = query_builder.bind(value.to_string());
                        }
                    }
                }
            }

            let rows = query_builder
                .fetch_all(&self.pool)
                .await
                .context("Failed to execute query")?;

            let mut results = Vec::new();
            for row in rows {
                let mut map = HashMap::new();
                for (i, column) in row.columns().iter().enumerate() {
                    let column_name = column.name().to_string();
                    let value = match column.type_info().name() {
                        "TEXT" | "VARCHAR" | "CHAR" => {
                            row.try_get::<String, _>(i)
                                .ok()
                                .map(Value::String)
                                .unwrap_or(Value::Null)
                        }
                        "INTEGER" | "INT" | "BIGINT" | "SMALLINT" => {
                            row.try_get::<i64, _>(i)
                                .ok()
                                .map(|v| Value::Number(v.into()))
                                .unwrap_or(Value::Null)
                        }
                        "REAL" | "FLOAT" | "DOUBLE" | "NUMERIC" | "DECIMAL" => row
                            .try_get::<f64, _>(i)
                            .ok()
                            .and_then(|v| serde_json::Number::from_f64(v).map(Value::Number))
                            .unwrap_or(Value::Null),
                        "BOOLEAN" | "BOOL" => {
                            row.try_get::<bool, _>(i)
                                .ok()
                                .map(Value::Bool)
                                .unwrap_or(Value::Null)
                        }
                        _ => {
                            // Try to get as string for other types
                            row.try_get::<String, _>(i)
                                .ok()
                                .map(Value::String)
                                .unwrap_or(Value::Null)
                        }
                    };
                    map.insert(column_name, value);
                }
                results.push(map);
            }

            tracing::debug!(rows_returned = results.len(), "Query executed successfully");
            Ok(results)
        }

        #[cfg(not(feature = "database"))]
        {
            tracing::warn!(
                "Database query execution not available (feature not enabled): {}",
                query
            );
            Ok(vec![])
        }
    }

    async fn execute_mutation(&self, query: &str, data: &HashMap<String, Value>) -> Result<Value> {
        #[cfg(feature = "database")]
        {
            tracing::debug!(query = %query, "Executing database mutation");

            let mut query_builder = sqlx::query(query);

            // Bind data parameters
            for (key, value) in data.iter() {
                tracing::trace!(param = %key, value = ?value, "Binding mutation parameter");
                match value {
                    Value::String(s) => query_builder = query_builder.bind(s.clone()),
                    Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            query_builder = query_builder.bind(i);
                        } else if let Some(f) = n.as_f64() {
                            query_builder = query_builder.bind(f);
                        }
                    }
                    Value::Bool(b) => query_builder = query_builder.bind(*b),
                    Value::Null => query_builder = query_builder.bind(None::<String>),
                    _ => {
                        query_builder = query_builder.bind(value.to_string());
                    }
                }
            }

            let result = query_builder
                .execute(&self.pool)
                .await
                .context("Failed to execute mutation")?;

            let rows_affected = result.rows_affected();
            tracing::info!(rows_affected = rows_affected, "Mutation executed successfully");

            Ok(serde_json::json!({
                "rows_affected": rows_affected,
                "success": true
            }))
        }

        #[cfg(not(feature = "database"))]
        {
            tracing::warn!(
                "Database mutation execution not available (feature not enabled): {}",
                query
            );
            Ok(Value::Bool(true))
        }
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
    async fn execute_query(
        &self,
        endpoint: &str,
        params: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<HashMap<String, Value>>> {
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

    async fn execute_mutation(
        &self,
        endpoint: &str,
        data: &HashMap<String, Value>,
    ) -> Result<Value> {
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

/// GraphQL data source
pub struct GraphQLDataSource {
    endpoint: String,
    client: reqwest::Client,
    headers: HashMap<String, String>,
}

impl GraphQLDataSource {
    pub fn new(endpoint: String, headers: Option<HashMap<String, String>>) -> Self {
        let client = reqwest::Client::new();
        Self {
            endpoint,
            client,
            headers: headers.unwrap_or_default(),
        }
    }
}

#[async_trait::async_trait]
impl DataSource for GraphQLDataSource {
    async fn execute_query(
        &self,
        query: &str,
        params: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<HashMap<String, Value>>> {
        let mut request_body = HashMap::new();
        request_body.insert("query", Value::String(query.to_string()));

        if let Some(params) = params {
            request_body.insert(
                "variables",
                Value::Object(params.clone().into_iter().collect()),
            );
        }

        let mut request = self.client.post(&self.endpoint);

        // Add headers
        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        let response = request.json(&request_body).send().await?;
        let data: Value = response.json().await?;

        // Extract data from GraphQL response
        if let Some(obj) = data.as_object() {
            if let Some(Value::Object(data_obj)) = obj.get("data") {
                let map: HashMap<String, Value> = data_obj.clone().into_iter().collect();
                return Ok(vec![map]);
            }
        }

        Ok(vec![])
    }

    async fn execute_mutation(
        &self,
        mutation: &str,
        variables: &HashMap<String, Value>,
    ) -> Result<Value> {
        let mut request_body = HashMap::new();
        request_body.insert("query", Value::String(mutation.to_string()));
        request_body.insert(
            "variables",
            Value::Object(variables.clone().into_iter().collect()),
        );

        let mut request = self.client.post(&self.endpoint);

        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        let response = request.json(&request_body).send().await?;
        let result: Value = response.json().await?;

        Ok(result)
    }
}

/// MongoDB data source
pub struct MongoDBDataSource {
    #[cfg(feature = "mongodb")]
    client: mongodb::Client,
    #[cfg(feature = "mongodb")]
    database: String,
    #[cfg(feature = "mongodb")]
    collection: String,
    #[cfg(not(feature = "mongodb"))]
    _connection_string: String,
    #[cfg(not(feature = "mongodb"))]
    _database: String,
    #[cfg(not(feature = "mongodb"))]
    _collection: String,
}

impl MongoDBDataSource {
    pub async fn new(
        connection_string: String,
        database: String,
        collection: String,
    ) -> Result<Self> {
        #[cfg(feature = "mongodb")]
        {
            tracing::info!(
                database = %database,
                collection = %collection,
                "Connecting to MongoDB"
            );

            let client = mongodb::Client::with_uri_str(&connection_string)
                .await
                .context("Failed to connect to MongoDB")?;

            // Test the connection
            client
                .list_database_names(None, None)
                .await
                .context("Failed to list databases (connection test)")?;

            tracing::info!("MongoDB connection established successfully");

            Ok(Self {
                client,
                database,
                collection,
            })
        }

        #[cfg(not(feature = "mongodb"))]
        {
            tracing::warn!("MongoDB feature not enabled, returning stub implementation");
            Ok(Self {
                _connection_string: connection_string,
                _database: database,
                _collection: collection,
            })
        }
    }
}

#[async_trait::async_trait]
impl DataSource for MongoDBDataSource {
    async fn execute_query(
        &self,
        query: &str,
        params: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<HashMap<String, Value>>> {
        #[cfg(feature = "mongodb")]
        {
            use mongodb::bson::{doc, Document};

            tracing::debug!(query = %query, "Executing MongoDB query");

            let db = self.client.database(&self.database);
            let collection = db.collection::<Document>(&self.collection);

            // Parse query as BSON document (expected to be JSON string)
            let filter: Document = if query.is_empty() {
                doc! {}
            } else {
                serde_json::from_str(query)
                    .context("Failed to parse MongoDB query as JSON")?
            };

            // Add parameters to filter if provided
            let mut final_filter = filter;
            if let Some(params) = params {
                for (key, value) in params {
                    // Convert serde_json::Value to bson::Bson
                    let bson_value = mongodb::bson::to_bson(value)
                        .context("Failed to convert parameter to BSON")?;
                    final_filter.insert(key, bson_value);
                }
            }

            let mut cursor = collection
                .find(final_filter, None)
                .await
                .context("Failed to execute MongoDB find")?;

            let mut results = Vec::new();
            use futures::stream::StreamExt;
            while let Some(doc) = cursor.next().await {
                let doc = doc.context("Failed to read document from cursor")?;
                // Convert BSON document to serde_json::Value
                let json_value: Value = mongodb::bson::from_document(doc)
                    .context("Failed to convert BSON to JSON")?;
                if let Value::Object(obj) = json_value {
                    let map: HashMap<String, Value> = obj.into_iter().collect();
                    results.push(map);
                }
            }

            tracing::debug!(
                documents_returned = results.len(),
                "MongoDB query executed successfully"
            );
            Ok(results)
        }

        #[cfg(not(feature = "mongodb"))]
        {
            tracing::warn!(
                "MongoDB query execution not available (feature not enabled): {}",
                query
            );
            Ok(vec![])
        }
    }

    async fn execute_mutation(
        &self,
        query: &str,
        data: &HashMap<String, Value>,
    ) -> Result<Value> {
        #[cfg(feature = "mongodb")]
        {
            use mongodb::bson::{doc, Document};

            tracing::debug!(query = %query, "Executing MongoDB mutation");

            let db = self.client.database(&self.database);
            let collection = db.collection::<Document>(&self.collection);

            // Convert data to BSON document
            let bson_data = mongodb::bson::to_bson(data)
                .context("Failed to convert data to BSON")?;
            let document = bson_data
                .as_document()
                .ok_or_else(|| anyhow!("Data must be a document"))?
                .clone();

            // Determine operation type from query
            let result = if query.contains("insert") || query.is_empty() {
                // Insert operation
                let insert_result = collection
                    .insert_one(document, None)
                    .await
                    .context("Failed to insert document")?;
                tracing::info!(
                    inserted_id = ?insert_result.inserted_id,
                    "Document inserted successfully"
                );
                serde_json::json!({
                    "inserted_id": insert_result.inserted_id.to_string(),
                    "success": true
                })
            } else if query.contains("update") {
                // Update operation - parse filter from query
                let filter: Document = serde_json::from_str(query)
                    .unwrap_or_else(|_| doc! {});
                let update = doc! { "$set": document };
                let update_result = collection
                    .update_many(filter, update, None)
                    .await
                    .context("Failed to update documents")?;
                tracing::info!(
                    matched = update_result.matched_count,
                    modified = update_result.modified_count,
                    "Documents updated successfully"
                );
                serde_json::json!({
                    "matched_count": update_result.matched_count,
                    "modified_count": update_result.modified_count,
                    "success": true
                })
            } else if query.contains("delete") {
                // Delete operation
                let filter: Document = serde_json::from_str(query)
                    .unwrap_or_else(|_| doc! {});
                let delete_result = collection
                    .delete_many(filter, None)
                    .await
                    .context("Failed to delete documents")?;
                tracing::info!(
                    deleted = delete_result.deleted_count,
                    "Documents deleted successfully"
                );
                serde_json::json!({
                    "deleted_count": delete_result.deleted_count,
                    "success": true
                })
            } else {
                return Err(anyhow!("Unknown MongoDB operation type"));
            };

            Ok(result)
        }

        #[cfg(not(feature = "mongodb"))]
        {
            tracing::warn!(
                "MongoDB mutation execution not available (feature not enabled): {}",
                query
            );
            Ok(Value::Bool(true))
        }
    }
}

/// Redis data source
pub struct RedisDataSource {
    #[cfg(feature = "redis")]
    connection: redis::aio::ConnectionManager,
    #[cfg(feature = "redis")]
    key_prefix: Option<String>,
    #[cfg(not(feature = "redis"))]
    _connection_string: String,
    #[cfg(not(feature = "redis"))]
    _key_prefix: Option<String>,
}

impl RedisDataSource {
    pub async fn new(connection_string: String, key_prefix: Option<String>) -> Result<Self> {
        #[cfg(feature = "redis")]
        {
            tracing::info!(
                prefix = ?key_prefix,
                "Connecting to Redis"
            );

            let client = redis::Client::open(connection_string.clone())
                .context("Failed to create Redis client")?;

            let connection = redis::aio::ConnectionManager::new(client)
                .await
                .context("Failed to connect to Redis")?;

            tracing::info!("Redis connection established successfully");

            Ok(Self {
                connection,
                key_prefix,
            })
        }

        #[cfg(not(feature = "redis"))]
        {
            tracing::warn!("Redis feature not enabled, returning stub implementation");
            Ok(Self {
                _connection_string: connection_string,
                _key_prefix: key_prefix,
            })
        }
    }

    #[cfg(feature = "redis")]
    fn apply_prefix(&self, key: &str) -> String {
        if let Some(prefix) = &self.key_prefix {
            format!("{}:{}", prefix, key)
        } else {
            key.to_string()
        }
    }
}

#[async_trait::async_trait]
impl DataSource for RedisDataSource {
    async fn execute_query(
        &self,
        key: &str,
        params: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<HashMap<String, Value>>> {
        #[cfg(feature = "redis")]
        {
            use redis::AsyncCommands;

            tracing::debug!(key = %key, "Executing Redis query");

            let mut conn = self.connection.clone();
            let prefixed_key = self.apply_prefix(key);

            // Support pattern matching with KEYS command or direct GET
            let results = if key.contains('*') || key.contains('?') {
                // Pattern matching - get multiple keys
                let keys: Vec<String> = conn
                    .keys(&prefixed_key)
                    .await
                    .context("Failed to execute Redis KEYS command")?;

                let mut results = Vec::new();
                for k in keys {
                    let value: Option<String> = conn
                        .get(&k)
                        .await
                        .context(format!("Failed to get value for key: {}", k))?;

                    if let Some(v) = value {
                        let mut map = HashMap::new();
                        map.insert("key".to_string(), Value::String(k.clone()));
                        // Try to parse as JSON, otherwise store as string
                        let parsed_value: Value = serde_json::from_str(&v).unwrap_or(Value::String(v));
                        map.insert("value".to_string(), parsed_value);
                        results.push(map);
                    }
                }
                results
            } else {
                // Single key get
                let value: Option<String> = conn
                    .get(&prefixed_key)
                    .await
                    .context("Failed to get value from Redis")?;

                if let Some(v) = value {
                    let mut map = HashMap::new();
                    map.insert("key".to_string(), Value::String(prefixed_key.clone()));
                    let parsed_value: Value = serde_json::from_str(&v).unwrap_or(Value::String(v));
                    map.insert("value".to_string(), parsed_value);
                    vec![map]
                } else {
                    vec![]
                }
            };

            // Apply additional filtering from params if provided
            let filtered_results = if let Some(params) = params {
                results
                    .into_iter()
                    .filter(|item| {
                        params.iter().all(|(k, v)| {
                            item.get(k).map(|item_v| item_v == v).unwrap_or(false)
                        })
                    })
                    .collect()
            } else {
                results
            };

            tracing::debug!(
                keys_returned = filtered_results.len(),
                "Redis query executed successfully"
            );
            Ok(filtered_results)
        }

        #[cfg(not(feature = "redis"))]
        {
            tracing::warn!(
                "Redis query execution not available (feature not enabled): {}",
                key
            );
            Ok(vec![])
        }
    }

    async fn execute_mutation(&self, key: &str, data: &HashMap<String, Value>) -> Result<Value> {
        #[cfg(feature = "redis")]
        {
            use redis::AsyncCommands;

            tracing::debug!(key = %key, "Executing Redis mutation");

            let mut conn = self.connection.clone();
            let prefixed_key = self.apply_prefix(key);

            // Get value from data HashMap
            if let Some(value) = data.get("value") {
                // Serialize value to JSON string
                let value_str = serde_json::to_string(value)
                    .context("Failed to serialize value to JSON")?;

                // Check for TTL
                let ttl = data
                    .get("ttl")
                    .and_then(|v| v.as_i64())
                    .map(|v| v as u64);

                if let Some(ttl_seconds) = ttl {
                    // SET with expiration
                    let _: () = conn
                        .set_ex(&prefixed_key, value_str, ttl_seconds)
                        .await
                        .context("Failed to set key with TTL in Redis")?;
                    tracing::info!(
                        key = %prefixed_key,
                        ttl = ttl_seconds,
                        "Key set with TTL successfully"
                    );
                } else {
                    // SET without expiration
                    let _: () = conn
                        .set(&prefixed_key, value_str)
                        .await
                        .context("Failed to set key in Redis")?;
                    tracing::info!(key = %prefixed_key, "Key set successfully");
                }

                Ok(serde_json::json!({
                    "key": prefixed_key,
                    "success": true
                }))
            } else if data.get("delete").and_then(|v| v.as_bool()).unwrap_or(false) {
                // DELETE operation
                let deleted: u32 = conn
                    .del(&prefixed_key)
                    .await
                    .context("Failed to delete key from Redis")?;
                tracing::info!(
                    key = %prefixed_key,
                    deleted = deleted,
                    "Key deleted successfully"
                );
                Ok(serde_json::json!({
                    "deleted": deleted,
                    "success": true
                }))
            } else {
                Err(anyhow!(
                    "Redis mutation requires 'value' field or 'delete: true' flag"
                ))
            }
        }

        #[cfg(not(feature = "redis"))]
        {
            tracing::warn!(
                "Redis mutation execution not available (feature not enabled): {}",
                key
            );
            Ok(Value::Bool(true))
        }
    }
}

/// Elasticsearch data source
pub struct ElasticsearchDataSource {
    #[cfg(feature = "elasticsearch")]
    client: elasticsearch::Elasticsearch,
    #[cfg(feature = "elasticsearch")]
    index: String,
    #[cfg(not(feature = "elasticsearch"))]
    _nodes: Vec<String>,
    #[cfg(not(feature = "elasticsearch"))]
    _index: String,
}

impl ElasticsearchDataSource {
    pub async fn new(nodes: Vec<String>, index: String) -> Result<Self> {
        #[cfg(feature = "elasticsearch")]
        {
            use elasticsearch::{
                auth::Credentials,
                http::transport::{SingleNodeConnectionPool, TransportBuilder},
                Elasticsearch,
            };

            tracing::info!(
                nodes = ?nodes,
                index = %index,
                "Connecting to Elasticsearch"
            );

            // Use the first node as the connection URL
            let url = nodes
                .first()
                .ok_or_else(|| anyhow!("No Elasticsearch nodes provided"))?;

            let url = url::Url::parse(url).context("Failed to parse Elasticsearch URL")?;

            // Check if URL contains credentials
            let transport = if !url.username().is_empty() {
                let username = url.username().to_string();
                let password = url.password().unwrap_or("").to_string();
                let credentials = Credentials::Basic(username, password);

                // Build URL without credentials
                let mut clean_url = url.clone();
                let _ = clean_url.set_username("");
                let _ = clean_url.set_password(None);

                let conn_pool = SingleNodeConnectionPool::new(clean_url);
                TransportBuilder::new(conn_pool)
                    .auth(credentials)
                    .build()
                    .context("Failed to build Elasticsearch transport with auth")?
            } else {
                let conn_pool = SingleNodeConnectionPool::new(url);
                TransportBuilder::new(conn_pool)
                    .build()
                    .context("Failed to build Elasticsearch transport")?
            };

            let client = Elasticsearch::new(transport);

            // Test the connection
            client
                .ping()
                .send()
                .await
                .context("Failed to ping Elasticsearch")?;

            tracing::info!("Elasticsearch connection established successfully");

            Ok(Self { client, index })
        }

        #[cfg(not(feature = "elasticsearch"))]
        {
            tracing::warn!("Elasticsearch feature not enabled, returning stub implementation");
            Ok(Self {
                _nodes: nodes,
                _index: index,
            })
        }
    }
}

#[async_trait::async_trait]
impl DataSource for ElasticsearchDataSource {
    async fn execute_query(
        &self,
        query: &str,
        params: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<HashMap<String, Value>>> {
        #[cfg(feature = "elasticsearch")]
        {
            use elasticsearch::SearchParts;

            tracing::debug!(query = %query, index = %self.index, "Executing Elasticsearch query");

            // Parse query as JSON
            let mut query_body: Value = if query.is_empty() {
                serde_json::json!({
                    "query": {
                        "match_all": {}
                    }
                })
            } else {
                serde_json::from_str(query).context("Failed to parse query as JSON")?
            };

            // Add params to query if provided
            if let Some(params) = params {
                if let Some(query_obj) = query_body.get_mut("query") {
                    if let Some(query_map) = query_obj.as_object_mut() {
                        for (key, value) in params {
                            query_map.insert(key.clone(), value.clone());
                        }
                    }
                }
            }

            let response = self
                .client
                .search(SearchParts::Index(&[&self.index]))
                .body(query_body)
                .send()
                .await
                .context("Failed to execute Elasticsearch search")?;

            let response_body = response
                .json::<Value>()
                .await
                .context("Failed to parse Elasticsearch response")?;

            // Extract hits from response
            let mut results = Vec::new();
            if let Some(hits) = response_body
                .get("hits")
                .and_then(|h| h.get("hits"))
                .and_then(|h| h.as_array())
            {
                for hit in hits {
                    if let Some(source) = hit.get("_source") {
                        if let Value::Object(obj) = source {
                            let mut map: HashMap<String, Value> = obj.clone().into_iter().collect();

                            // Add metadata fields
                            if let Some(id) = hit.get("_id").and_then(|v| v.as_str()) {
                                map.insert("_id".to_string(), Value::String(id.to_string()));
                            }
                            if let Some(score) = hit.get("_score") {
                                map.insert("_score".to_string(), score.clone());
                            }

                            results.push(map);
                        }
                    }
                }
            }

            tracing::debug!(
                documents_returned = results.len(),
                "Elasticsearch query executed successfully"
            );
            Ok(results)
        }

        #[cfg(not(feature = "elasticsearch"))]
        {
            tracing::warn!(
                "Elasticsearch query execution not available (feature not enabled): {}",
                query
            );
            Ok(vec![])
        }
    }

    async fn execute_mutation(
        &self,
        doc_id: &str,
        data: &HashMap<String, Value>,
    ) -> Result<Value> {
        #[cfg(feature = "elasticsearch")]
        {
            use elasticsearch::IndexParts;

            tracing::debug!(
                doc_id = %doc_id,
                index = %self.index,
                "Executing Elasticsearch mutation"
            );

            // Index or update document
            let response = self
                .client
                .index(IndexParts::IndexId(&self.index, doc_id))
                .body(data)
                .send()
                .await
                .context("Failed to index document in Elasticsearch")?;

            let response_body = response
                .json::<Value>()
                .await
                .context("Failed to parse Elasticsearch response")?;

            tracing::info!(
                doc_id = %doc_id,
                result = ?response_body.get("result"),
                "Document indexed successfully"
            );

            Ok(serde_json::json!({
                "id": doc_id,
                "result": response_body.get("result"),
                "success": true
            }))
        }

        #[cfg(not(feature = "elasticsearch"))]
        {
            tracing::warn!(
                "Elasticsearch mutation execution not available (feature not enabled): {}",
                doc_id
            );
            Ok(Value::Bool(true))
        }
    }
}

/// gRPC data source
pub struct GrpcDataSource {
    #[allow(dead_code)]
    endpoint: String,
    #[allow(dead_code)]
    proto_file: String,
    #[allow(dead_code)]
    service_name: String,
    #[allow(dead_code)]
    tls_enabled: bool,
}

impl GrpcDataSource {
    pub fn new(
        endpoint: String,
        proto_file: String,
        service_name: String,
        tls_enabled: bool,
    ) -> Self {
        Self {
            endpoint,
            proto_file,
            service_name,
            tls_enabled,
        }
    }
}

#[async_trait::async_trait]
impl DataSource for GrpcDataSource {
    async fn execute_query(
        &self,
        method: &str,
        _params: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<HashMap<String, Value>>> {
        tracing::warn!("gRPC query execution not yet fully implemented: {}", method);
        Ok(vec![])
    }

    async fn execute_mutation(
        &self,
        method: &str,
        _data: &HashMap<String, Value>,
    ) -> Result<Value> {
        tracing::warn!(
            "gRPC mutation execution not yet fully implemented: {}",
            method
        );
        Ok(Value::Bool(true))
    }
}

/// Kafka data source
pub struct KafkaDataSource {
    #[allow(dead_code)]
    brokers: Vec<String>,
    #[allow(dead_code)]
    topic: String,
    #[allow(dead_code)]
    group_id: String,
}

impl KafkaDataSource {
    pub fn new(brokers: Vec<String>, topic: String, group_id: String) -> Self {
        Self {
            brokers,
            topic,
            group_id,
        }
    }
}

#[async_trait::async_trait]
impl DataSource for KafkaDataSource {
    async fn execute_query(
        &self,
        query: &str,
        _params: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<HashMap<String, Value>>> {
        tracing::warn!("Kafka query execution not yet fully implemented: {}", query);
        Ok(vec![])
    }

    async fn execute_mutation(
        &self,
        message: &str,
        _data: &HashMap<String, Value>,
    ) -> Result<Value> {
        tracing::warn!(
            "Kafka mutation execution not yet fully implemented: {}",
            message
        );
        Ok(Value::Bool(true))
    }
}

/// S3 data source
pub struct S3DataSource {
    #[cfg(feature = "s3")]
    client: aws_sdk_s3::Client,
    #[cfg(feature = "s3")]
    bucket: String,
    #[cfg(feature = "s3")]
    prefix: Option<String>,
    #[cfg(not(feature = "s3"))]
    _bucket: String,
    #[cfg(not(feature = "s3"))]
    _region: String,
    #[cfg(not(feature = "s3"))]
    _prefix: Option<String>,
}

impl S3DataSource {
    pub async fn new(bucket: String, region: String, prefix: Option<String>) -> Result<Self> {
        #[cfg(feature = "s3")]
        {
            tracing::info!(
                bucket = %bucket,
                region = %region,
                prefix = ?prefix,
                "Initializing S3 client"
            );

            let region_provider = aws_sdk_s3::config::Region::new(region.clone());
            let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
                .region(region_provider)
                .load()
                .await;

            let client = aws_sdk_s3::Client::new(&config);

            // Test connection by checking if bucket exists
            client
                .head_bucket()
                .bucket(&bucket)
                .send()
                .await
                .context("Failed to access S3 bucket (bucket may not exist or no permissions)")?;

            tracing::info!("S3 client initialized successfully");

            Ok(Self {
                client,
                bucket,
                prefix,
            })
        }

        #[cfg(not(feature = "s3"))]
        {
            tracing::warn!("S3 feature not enabled, returning stub implementation");
            Ok(Self {
                _bucket: bucket,
                _region: region,
                _prefix: prefix,
            })
        }
    }

    #[cfg(feature = "s3")]
    fn apply_prefix(&self, key: &str) -> String {
        if let Some(prefix) = &self.prefix {
            format!("{}/{}", prefix.trim_end_matches('/'), key)
        } else {
            key.to_string()
        }
    }
}

#[async_trait::async_trait]
impl DataSource for S3DataSource {
    async fn execute_query(
        &self,
        key: &str,
        params: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<HashMap<String, Value>>> {
        #[cfg(feature = "s3")]
        {
            tracing::debug!(key = %key, bucket = %self.bucket, "Executing S3 query");

            // If key contains wildcard, list objects with prefix
            if key.contains('*') || key.is_empty() {
                let prefix = if key.is_empty() {
                    self.prefix.clone().unwrap_or_default()
                } else {
                    self.apply_prefix(&key.replace('*', ""))
                };

                let mut list_request = self
                    .client
                    .list_objects_v2()
                    .bucket(&self.bucket)
                    .prefix(&prefix);

                // Add max_keys from params if provided
                if let Some(params) = params {
                    if let Some(max_keys) = params.get("max_keys").and_then(|v| v.as_i64()) {
                        list_request = list_request.max_keys(max_keys as i32);
                    }
                }

                let response = list_request
                    .send()
                    .await
                    .context("Failed to list S3 objects")?;

                let mut results = Vec::new();
                for object in response.contents() {
                    let mut map = HashMap::new();
                    if let Some(key) = object.key() {
                        map.insert("key".to_string(), Value::String(key.to_string()));
                    }
                    if let Some(size) = object.size() {
                        map.insert("size".to_string(), Value::Number(size.into()));
                    }
                    if let Some(last_modified) = object.last_modified() {
                        map.insert(
                            "last_modified".to_string(),
                            Value::String(last_modified.to_string()),
                        );
                    }
                    if let Some(etag) = object.e_tag() {
                        map.insert("etag".to_string(), Value::String(etag.to_string()));
                    }
                    results.push(map);
                }

                tracing::debug!(
                    objects_returned = results.len(),
                    "S3 list query executed successfully"
                );
                Ok(results)
            } else {
                // Get single object
                let full_key = self.apply_prefix(key);

                let response = self
                    .client
                    .get_object()
                    .bucket(&self.bucket)
                    .key(&full_key)
                    .send()
                    .await
                    .context("Failed to get S3 object")?;

                // Get content type before consuming the response body
                let mime_type = response.content_type().map(|s| s.to_string());

                // Read the object body
                let body_bytes = response
                    .body
                    .collect()
                    .await
                    .context("Failed to read S3 object body")?
                    .into_bytes();

                let mut map = HashMap::new();
                map.insert("key".to_string(), Value::String(full_key.clone()));
                map.insert("size".to_string(), Value::Number(body_bytes.len().into()));

                // Try to parse as JSON, otherwise base64 encode
                if let Ok(json_value) = serde_json::from_slice::<Value>(&body_bytes) {
                    map.insert("content".to_string(), json_value);
                    map.insert("content_type".to_string(), Value::String("json".to_string()));
                } else if let Ok(text) = String::from_utf8(body_bytes.to_vec()) {
                    map.insert("content".to_string(), Value::String(text));
                    map.insert("content_type".to_string(), Value::String("text".to_string()));
                } else {
                    use base64::{engine::general_purpose, Engine as _};
                    let encoded = general_purpose::STANDARD.encode(&body_bytes);
                    map.insert("content".to_string(), Value::String(encoded));
                    map.insert(
                        "content_type".to_string(),
                        Value::String("binary".to_string()),
                    );
                }

                if let Some(content_type) = mime_type {
                    map.insert("mime_type".to_string(), Value::String(content_type));
                }

                tracing::debug!(key = %full_key, "S3 get query executed successfully");
                Ok(vec![map])
            }
        }

        #[cfg(not(feature = "s3"))]
        {
            tracing::warn!(
                "S3 query execution not available (feature not enabled): {}",
                key
            );
            Ok(vec![])
        }
    }

    async fn execute_mutation(&self, key: &str, data: &HashMap<String, Value>) -> Result<Value> {
        #[cfg(feature = "s3")]
        {
            use aws_sdk_s3::primitives::ByteStream;

            tracing::debug!(key = %key, bucket = %self.bucket, "Executing S3 mutation");

            let full_key = self.apply_prefix(key);

            // Check for delete operation
            if data.get("delete").and_then(|v| v.as_bool()).unwrap_or(false) {
                self.client
                    .delete_object()
                    .bucket(&self.bucket)
                    .key(&full_key)
                    .send()
                    .await
                    .context("Failed to delete S3 object")?;

                tracing::info!(key = %full_key, "S3 object deleted successfully");
                return Ok(serde_json::json!({
                    "key": full_key,
                    "deleted": true,
                    "success": true
                }));
            }

            // Put object operation
            let content = data
                .get("content")
                .ok_or_else(|| anyhow!("Missing 'content' field for S3 put operation"))?;

            // Convert content to bytes
            let body_bytes = if let Some(text) = content.as_str() {
                // Check if it's base64 encoded
                if data
                    .get("encoding")
                    .and_then(|v| v.as_str())
                    .map(|e| e == "base64")
                    .unwrap_or(false)
                {
                    use base64::{engine::general_purpose, Engine as _};
                    general_purpose::STANDARD
                        .decode(text)
                        .context("Failed to decode base64 content")?
                } else {
                    text.as_bytes().to_vec()
                }
            } else {
                // Serialize as JSON
                serde_json::to_vec(content).context("Failed to serialize content as JSON")?
            };

            let body = ByteStream::from(body_bytes);

            let mut put_request = self
                .client
                .put_object()
                .bucket(&self.bucket)
                .key(&full_key)
                .body(body);

            // Add content type if provided
            if let Some(content_type) = data.get("content_type").and_then(|v| v.as_str()) {
                put_request = put_request.content_type(content_type);
            }

            // Add metadata if provided
            if let Some(metadata) = data.get("metadata").and_then(|v| v.as_object()) {
                for (k, v) in metadata {
                    if let Some(s) = v.as_str() {
                        put_request = put_request.metadata(k, s);
                    }
                }
            }

            let response = put_request
                .send()
                .await
                .context("Failed to put S3 object")?;

            tracing::info!(
                key = %full_key,
                etag = ?response.e_tag(),
                "S3 object uploaded successfully"
            );

            Ok(serde_json::json!({
                "key": full_key,
                "etag": response.e_tag(),
                "success": true
            }))
        }

        #[cfg(not(feature = "s3"))]
        {
            tracing::warn!(
                "S3 mutation execution not available (feature not enabled): {}",
                key
            );
            Ok(Value::Bool(true))
        }
    }
}

/// Firebase data source
pub struct FirebaseDataSource {
    #[allow(dead_code)]
    project_id: String,
    #[allow(dead_code)]
    collection: String,
}

impl FirebaseDataSource {
    pub fn new(project_id: String, collection: String) -> Self {
        Self {
            project_id,
            collection,
        }
    }
}

#[async_trait::async_trait]
impl DataSource for FirebaseDataSource {
    async fn execute_query(
        &self,
        query: &str,
        _params: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<HashMap<String, Value>>> {
        tracing::warn!(
            "Firebase query execution not yet fully implemented: {}",
            query
        );
        Ok(vec![])
    }

    async fn execute_mutation(
        &self,
        doc_id: &str,
        _data: &HashMap<String, Value>,
    ) -> Result<Value> {
        tracing::warn!(
            "Firebase mutation execution not yet fully implemented: {}",
            doc_id
        );
        Ok(Value::Bool(true))
    }
}

/// Supabase data source
pub struct SupabaseDataSource {
    url: String,
    api_key: String,
    table: String,
    client: reqwest::Client,
}

impl SupabaseDataSource {
    pub fn new(url: String, api_key: String, table: String) -> Self {
        let client = reqwest::Client::new();
        Self {
            url,
            api_key,
            table,
            client,
        }
    }
}

#[async_trait::async_trait]
impl DataSource for SupabaseDataSource {
    async fn execute_query(
        &self,
        _query: &str,
        _params: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<HashMap<String, Value>>> {
        let url = format!("{}/rest/v1/{}", self.url, self.table);

        let request = self
            .client
            .get(&url)
            .header("apikey", &self.api_key)
            .header("Authorization", format!("Bearer {}", self.api_key));

        let response = request.send().await?;
        let data: Value = response.json().await?;

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
            _ => Ok(vec![]),
        }
    }

    async fn execute_mutation(
        &self,
        _doc_id: &str,
        data: &HashMap<String, Value>,
    ) -> Result<Value> {
        let url = format!("{}/rest/v1/{}", self.url, self.table);

        let request = self
            .client
            .post(&url)
            .header("apikey", &self.api_key)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&data);

        let response = request.send().await?;
        let result: Value = response.json().await?;

        Ok(result)
    }
}

/// WebSocket data source
pub struct WebSocketDataSource {
    #[allow(dead_code)]
    url: String,
    #[allow(dead_code)]
    reconnect: bool,
    #[allow(dead_code)]
    heartbeat_interval: Option<u32>,
}

impl WebSocketDataSource {
    pub fn new(url: String, reconnect: bool, heartbeat_interval: Option<u32>) -> Self {
        Self {
            url,
            reconnect,
            heartbeat_interval,
        }
    }
}

#[async_trait::async_trait]
impl DataSource for WebSocketDataSource {
    async fn execute_query(
        &self,
        message: &str,
        _params: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<HashMap<String, Value>>> {
        tracing::warn!(
            "WebSocket query execution not yet fully implemented: {}",
            message
        );
        Ok(vec![])
    }

    async fn execute_mutation(
        &self,
        message: &str,
        _data: &HashMap<String, Value>,
    ) -> Result<Value> {
        tracing::warn!(
            "WebSocket mutation execution not yet fully implemented: {}",
            message
        );
        Ok(Value::Bool(true))
    }
}

/// Factory to create data sources
pub async fn create_data_source(config: &DataSourceConfig) -> Result<Box<dyn DataSource>> {
    match config {
        DataSourceConfig::Database { db_type, .. } => Ok(Box::new(
            DatabaseDataSource::new(config.clone(), db_type.clone()).await?,
        )),
        DataSourceConfig::Api {
            base_url, headers, ..
        } => Ok(Box::new(ApiDataSource::new(
            base_url.clone(),
            headers.clone(),
        ))),
        DataSourceConfig::GraphQL {
            endpoint, headers, ..
        } => Ok(Box::new(GraphQLDataSource::new(
            endpoint.clone(),
            headers.clone(),
        ))),
        DataSourceConfig::MongoDB {
            connection_string,
            database,
            collection,
        } => Ok(Box::new(
            MongoDBDataSource::new(
                connection_string.clone(),
                database.clone(),
                collection.clone(),
            )
            .await?,
        )),
        DataSourceConfig::Redis {
            connection_string,
            key_prefix,
        } => Ok(Box::new(
            RedisDataSource::new(connection_string.clone(), key_prefix.clone()).await?,
        )),
        DataSourceConfig::Elasticsearch { nodes, index, .. } => Ok(Box::new(
            ElasticsearchDataSource::new(nodes.clone(), index.clone()).await?,
        )),
        DataSourceConfig::Grpc {
            endpoint,
            proto_file,
            service_name,
            tls_enabled,
        } => Ok(Box::new(GrpcDataSource::new(
            endpoint.clone(),
            proto_file.clone(),
            service_name.clone(),
            *tls_enabled,
        ))),
        DataSourceConfig::Kafka {
            brokers,
            topic,
            group_id,
        } => Ok(Box::new(KafkaDataSource::new(
            brokers.clone(),
            topic.clone(),
            group_id.clone(),
        ))),
        DataSourceConfig::S3 {
            bucket,
            region,
            prefix,
            ..
        } => Ok(Box::new(
            S3DataSource::new(bucket.clone(), region.clone(), prefix.clone()).await?,
        )),
        DataSourceConfig::Firebase {
            project_id,
            collection,
            ..
        } => Ok(Box::new(FirebaseDataSource::new(
            project_id.clone(),
            collection.clone(),
        ))),
        DataSourceConfig::Supabase {
            url,
            api_key,
            table,
        } => Ok(Box::new(SupabaseDataSource::new(
            url.clone(),
            api_key.clone(),
            table.clone(),
        ))),
        DataSourceConfig::WebSocket {
            url,
            reconnect,
            heartbeat_interval,
        } => Ok(Box::new(WebSocketDataSource::new(
            url.clone(),
            *reconnect,
            *heartbeat_interval,
        ))),
    }
}
