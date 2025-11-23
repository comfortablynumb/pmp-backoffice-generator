use crate::config::{DataSourceConfig, DatabaseType};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use sqlx::{
    any::{AnyPoolOptions, AnyRow},
    AnyPool, Column, Row, TypeInfo,
};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

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

/// Database data source with connection pooling
pub struct DatabaseDataSource {
    pool: Arc<AnyPool>,
    db_type: DatabaseType,
}

impl DatabaseDataSource {
    pub async fn new(connection_string: String, db_type: DatabaseType) -> Result<Self> {
        tracing::info!(
            db_type = ?db_type,
            "Connecting to database"
        );

        let pool = AnyPoolOptions::new()
            .max_connections(5)
            .connect(&connection_string)
            .await
            .map_err(|e| anyhow!("Failed to connect to database: {}", e))?;

        tracing::info!("Database connection established");

        Ok(Self {
            pool: Arc::new(pool),
            db_type,
        })
    }

    /// Convert a database row to a HashMap
    fn row_to_map(row: &AnyRow) -> Result<HashMap<String, Value>> {
        let mut map = HashMap::new();

        for (i, column) in row.columns().iter().enumerate() {
            let column_name = column.name().to_string();
            let type_info = column.type_info();

            // Try to extract the value based on the type
            let value = if type_info.is_null() {
                Value::Null
            } else {
                let type_name = type_info.name();
                match type_name {
                    "TEXT" | "VARCHAR" | "CHAR" | "STRING" | "BPCHAR" => {
                        if let Ok(val) = row.try_get::<String, _>(i) {
                            Value::String(val)
                        } else {
                            Value::Null
                        }
                    }
                    "INTEGER" | "INT" | "SMALLINT" | "BIGINT" | "INT4" | "INT2" | "INT8" => {
                        if let Ok(val) = row.try_get::<i64, _>(i) {
                            Value::Number(serde_json::Number::from(val))
                        } else if let Ok(val) = row.try_get::<i32, _>(i) {
                            Value::Number(serde_json::Number::from(val))
                        } else {
                            Value::Null
                        }
                    }
                    "REAL" | "FLOAT" | "DOUBLE" | "NUMERIC" | "DECIMAL" | "FLOAT4" | "FLOAT8" => {
                        if let Ok(val) = row.try_get::<f64, _>(i) {
                            if let Some(num) = serde_json::Number::from_f64(val) {
                                Value::Number(num)
                            } else {
                                Value::Null
                            }
                        } else {
                            Value::Null
                        }
                    }
                    "BOOLEAN" | "BOOL" => {
                        if let Ok(val) = row.try_get::<bool, _>(i) {
                            Value::Bool(val)
                        } else {
                            Value::Null
                        }
                    }
                    "JSON" | "JSONB" => {
                        if let Ok(val) = row.try_get::<String, _>(i) {
                            serde_json::from_str(&val).unwrap_or(Value::Null)
                        } else {
                            Value::Null
                        }
                    }
                    "TIMESTAMP" | "TIMESTAMPTZ" | "DATETIME" | "DATE" | "TIME" => {
                        if let Ok(val) = row.try_get::<String, _>(i) {
                            Value::String(val)
                        } else {
                            Value::Null
                        }
                    }
                    _ => {
                        // Try string as fallback
                        if let Ok(val) = row.try_get::<String, _>(i) {
                            Value::String(val)
                        } else {
                            tracing::warn!(
                                column = %column_name,
                                type_name = %type_name,
                                "Unknown column type, using null"
                            );
                            Value::Null
                        }
                    }
                }
            };

            map.insert(column_name, value);
        }

        Ok(map)
    }
}

#[async_trait::async_trait]
impl DataSource for DatabaseDataSource {
    async fn execute_query(
        &self,
        query: &str,
        params: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<HashMap<String, Value>>> {
        tracing::info!(
            query = %query,
            params = ?params,
            db_type = ?self.db_type,
            "Executing database query"
        );

        // For now, we execute queries without parameters
        // A full implementation would need to bind parameters properly
        let rows = sqlx::query(query)
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| anyhow!("Query execution failed: {}", e))?;

        let mut results = Vec::new();
        for row in &rows {
            results.push(Self::row_to_map(row)?);
        }

        tracing::info!(row_count = results.len(), "Query executed successfully");

        Ok(results)
    }

    async fn execute_mutation(&self, query: &str, data: &HashMap<String, Value>) -> Result<Value> {
        tracing::info!(
            query = %query,
            data = ?data,
            db_type = ?self.db_type,
            "Executing database mutation"
        );

        // Execute the mutation
        let result = sqlx::query(query)
            .execute(&*self.pool)
            .await
            .map_err(|e| anyhow!("Mutation execution failed: {}", e))?;

        let rows_affected = result.rows_affected();

        tracing::info!(
            rows_affected = rows_affected,
            "Mutation executed successfully"
        );

        Ok(Value::Number(serde_json::Number::from(rows_affected)))
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
#[cfg(feature = "mongodb-datasource")]
pub struct MongoDBDataSource {
    client: mongodb::Client,
    database_name: String,
    collection_name: String,
}

#[cfg(feature = "mongodb-datasource")]
impl MongoDBDataSource {
    pub async fn new(
        connection_string: String,
        database: String,
        collection: String,
    ) -> Result<Self> {
        tracing::info!(
            database = %database,
            collection = %collection,
            "Connecting to MongoDB"
        );

        let client = mongodb::Client::with_uri_str(&connection_string)
            .await
            .map_err(|e| anyhow!("Failed to connect to MongoDB: {}", e))?;

        tracing::info!("MongoDB connection established");

        Ok(Self {
            client,
            database_name: database,
            collection_name: collection,
        })
    }
}

#[cfg(feature = "mongodb-datasource")]
#[async_trait::async_trait]
impl DataSource for MongoDBDataSource {
    async fn execute_query(
        &self,
        query: &str,
        _params: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<HashMap<String, Value>>> {
        use futures_util::TryStreamExt;
        use mongodb::bson::{doc, Document};

        tracing::info!(
            query = %query,
            database = %self.database_name,
            collection = %self.collection_name,
            "Executing MongoDB query"
        );

        let db = self.client.database(&self.database_name);
        let collection = db.collection::<Document>(&self.collection_name);

        // Parse query as BSON document (expected to be JSON)
        let filter: Document = if query.is_empty() || query == "{}" {
            doc! {}
        } else {
            serde_json::from_str(query)
                .map_err(|e| anyhow!("Invalid MongoDB filter JSON: {}", e))?
        };

        let mut cursor = collection
            .find(filter, None)
            .await
            .map_err(|e| anyhow!("MongoDB find failed: {}", e))?;

        let mut results = Vec::new();
        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| anyhow!("Failed to read MongoDB cursor: {}", e))?
        {
            // Convert BSON document to JSON
            let json_str = serde_json::to_string(&doc)
                .map_err(|e| anyhow!("Failed to serialize BSON to JSON: {}", e))?;
            let json_value: Value = serde_json::from_str(&json_str)?;

            if let Value::Object(obj) = json_value {
                results.push(obj.into_iter().collect());
            }
        }

        tracing::info!(count = results.len(), "MongoDB query completed");
        Ok(results)
    }

    async fn execute_mutation(&self, _query: &str, data: &HashMap<String, Value>) -> Result<Value> {
        use mongodb::bson::Document;

        tracing::info!(
            database = %self.database_name,
            collection = %self.collection_name,
            "Executing MongoDB mutation"
        );

        let db = self.client.database(&self.database_name);
        let collection = db.collection::<Document>(&self.collection_name);

        // Convert HashMap to BSON document
        let json_str = serde_json::to_string(data)?;
        let document: Document = serde_json::from_str(&json_str)
            .map_err(|e| anyhow!("Failed to convert data to BSON: {}", e))?;

        let result = collection
            .insert_one(document, None)
            .await
            .map_err(|e| anyhow!("MongoDB insert failed: {}", e))?;

        tracing::info!(inserted_id = ?result.inserted_id, "MongoDB mutation completed");

        Ok(Value::String(result.inserted_id.to_string()))
    }
}

// Stub implementation when feature is disabled
#[cfg(not(feature = "mongodb-datasource"))]
pub struct MongoDBDataSource {
    _phantom: std::marker::PhantomData<()>,
}

#[cfg(not(feature = "mongodb-datasource"))]
impl MongoDBDataSource {
    pub async fn new(
        _connection_string: String,
        _database: String,
        _collection: String,
    ) -> Result<Self> {
        Err(anyhow!(
            "MongoDB support not enabled. Enable the 'mongodb-datasource' feature in Cargo.toml"
        ))
    }
}

#[cfg(not(feature = "mongodb-datasource"))]
#[async_trait::async_trait]
impl DataSource for MongoDBDataSource {
    async fn execute_query(
        &self,
        _query: &str,
        _params: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<HashMap<String, Value>>> {
        Err(anyhow!("MongoDB support not enabled"))
    }

    async fn execute_mutation(
        &self,
        _query: &str,
        _data: &HashMap<String, Value>,
    ) -> Result<Value> {
        Err(anyhow!("MongoDB support not enabled"))
    }
}

/// Redis data source
#[cfg(feature = "redis-datasource")]
pub struct RedisDataSource {
    client: redis::Client,
    key_prefix: Option<String>,
}

#[cfg(feature = "redis-datasource")]
impl RedisDataSource {
    pub async fn new(connection_string: String, key_prefix: Option<String>) -> Result<Self> {
        tracing::info!(
            connection_string = %connection_string,
            "Connecting to Redis"
        );

        let client = redis::Client::open(connection_string)
            .map_err(|e| anyhow!("Failed to create Redis client: {}", e))?;

        // Test connection
        let mut con = client
            .get_async_connection()
            .await
            .map_err(|e| anyhow!("Failed to connect to Redis: {}", e))?;

        // Ping to verify connection
        redis::cmd("PING")
            .query_async::<_, String>(&mut con)
            .await
            .map_err(|e| anyhow!("Redis ping failed: {}", e))?;

        tracing::info!("Redis connection established");

        Ok(Self { client, key_prefix })
    }

    fn prefixed_key(&self, key: &str) -> String {
        if let Some(prefix) = &self.key_prefix {
            format!("{}:{}", prefix, key)
        } else {
            key.to_string()
        }
    }
}

#[cfg(feature = "redis-datasource")]
#[async_trait::async_trait]
impl DataSource for RedisDataSource {
    async fn execute_query(
        &self,
        key: &str,
        _params: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<HashMap<String, Value>>> {
        use redis::AsyncCommands;

        tracing::info!(key = %key, "Executing Redis query");

        let mut con = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| anyhow!("Failed to get Redis connection: {}", e))?;

        let prefixed_key = self.prefixed_key(key);

        // Try to get as string first
        let value: Option<String> = con
            .get(&prefixed_key)
            .await
            .map_err(|e| anyhow!("Redis GET failed: {}", e))?;

        if let Some(val) = value {
            // Try to parse as JSON
            if let Ok(json_value) = serde_json::from_str::<Value>(&val) {
                if let Value::Object(obj) = json_value {
                    return Ok(vec![obj.into_iter().collect()]);
                } else if let Value::Array(arr) = json_value {
                    let mut results = Vec::new();
                    for item in arr {
                        if let Value::Object(obj) = item {
                            results.push(obj.into_iter().collect());
                        }
                    }
                    return Ok(results);
                }
            }

            // If not JSON, return as string value
            let mut map = HashMap::new();
            map.insert("value".to_string(), Value::String(val));
            Ok(vec![map])
        } else {
            Ok(vec![])
        }
    }

    async fn execute_mutation(&self, key: &str, data: &HashMap<String, Value>) -> Result<Value> {
        use redis::AsyncCommands;

        tracing::info!(key = %key, "Executing Redis mutation");

        let mut con = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| anyhow!("Failed to get Redis connection: {}", e))?;

        let prefixed_key = self.prefixed_key(key);

        // Convert data to JSON string
        let json_str = serde_json::to_string(data)?;

        // Set the value
        let result: String = con
            .set(&prefixed_key, json_str)
            .await
            .map_err(|e| anyhow!("Redis SET failed: {}", e))?;

        tracing::info!(result = %result, "Redis mutation completed");

        Ok(Value::String(result))
    }
}

// Stub implementation when feature is disabled
#[cfg(not(feature = "redis-datasource"))]
pub struct RedisDataSource {
    _phantom: std::marker::PhantomData<()>,
}

#[cfg(not(feature = "redis-datasource"))]
impl RedisDataSource {
    pub async fn new(_connection_string: String, _key_prefix: Option<String>) -> Result<Self> {
        Err(anyhow!(
            "Redis support not enabled. Enable the 'redis-datasource' feature in Cargo.toml"
        ))
    }
}

#[cfg(not(feature = "redis-datasource"))]
#[async_trait::async_trait]
impl DataSource for RedisDataSource {
    async fn execute_query(
        &self,
        _key: &str,
        _params: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<HashMap<String, Value>>> {
        Err(anyhow!("Redis support not enabled"))
    }

    async fn execute_mutation(&self, _key: &str, _data: &HashMap<String, Value>) -> Result<Value> {
        Err(anyhow!("Redis support not enabled"))
    }
}

/// Elasticsearch data source
pub struct ElasticsearchDataSource {
    #[allow(dead_code)]
    nodes: Vec<String>,
    #[allow(dead_code)]
    index: String,
    #[allow(dead_code)]
    client: reqwest::Client,
}

impl ElasticsearchDataSource {
    pub fn new(nodes: Vec<String>, index: String) -> Self {
        let client = reqwest::Client::new();
        Self {
            nodes,
            index,
            client,
        }
    }
}

#[async_trait::async_trait]
impl DataSource for ElasticsearchDataSource {
    async fn execute_query(
        &self,
        query: &str,
        _params: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<HashMap<String, Value>>> {
        tracing::warn!(
            "Elasticsearch query execution not yet fully implemented: {}",
            query
        );
        Ok(vec![])
    }

    async fn execute_mutation(
        &self,
        _doc_id: &str,
        _data: &HashMap<String, Value>,
    ) -> Result<Value> {
        tracing::warn!("Elasticsearch mutation execution not yet fully implemented");
        Ok(Value::Bool(true))
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

/// S3 data source for object storage
#[cfg(feature = "s3-datasource")]
pub struct S3DataSource {
    client: aws_sdk_s3::Client,
    bucket: String,
    prefix: Option<String>,
}

#[cfg(feature = "s3-datasource")]
impl S3DataSource {
    pub async fn new(bucket: String, region: String, prefix: Option<String>) -> Result<Self> {
        use aws_config::BehaviorVersion;

        info!(
            bucket = %bucket,
            region = %region,
            prefix = ?prefix,
            "Initializing S3 data source"
        );

        // Load AWS configuration
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(aws_config::Region::new(region))
            .load()
            .await;

        let client = aws_sdk_s3::Client::new(&config);

        // Verify bucket access by attempting to list objects
        match client
            .list_objects_v2()
            .bucket(&bucket)
            .max_keys(1)
            .send()
            .await
        {
            Ok(_) => {
                info!(bucket = %bucket, "Successfully connected to S3 bucket");
            }
            Err(e) => {
                error!(
                    bucket = %bucket,
                    error = %e,
                    "Failed to access S3 bucket"
                );
                return Err(anyhow!("Failed to access S3 bucket: {}", e));
            }
        }

        Ok(Self {
            client,
            bucket,
            prefix,
        })
    }

    fn full_key(&self, key: &str) -> String {
        if let Some(prefix) = &self.prefix {
            format!("{}/{}", prefix, key)
        } else {
            key.to_string()
        }
    }
}

#[cfg(feature = "s3-datasource")]
#[async_trait::async_trait]
impl DataSource for S3DataSource {
    async fn execute_query(
        &self,
        key: &str,
        _params: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<HashMap<String, Value>>> {
        let full_key = self.full_key(key);

        debug!(
            bucket = %self.bucket,
            key = %full_key,
            "Fetching object from S3"
        );

        // Get object from S3
        let result = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&full_key)
            .send()
            .await;

        match result {
            Ok(output) => {
                let bytes = output.body.collect().await?.into_bytes();
                let content = String::from_utf8(bytes.to_vec())?;

                debug!(
                    key = %full_key,
                    size = bytes.len(),
                    "Successfully fetched object from S3"
                );

                // Try to parse as JSON
                match serde_json::from_str::<Value>(&content) {
                    Ok(Value::Array(arr)) => {
                        // If it's a JSON array, parse each element
                        let mut result = Vec::new();
                        for item in arr {
                            if let Value::Object(obj) = item {
                                result.push(obj.into_iter().collect());
                            }
                        }
                        Ok(result)
                    }
                    Ok(Value::Object(obj)) => {
                        // If it's a JSON object, return it as a single-item array
                        Ok(vec![obj.into_iter().collect()])
                    }
                    _ => {
                        // If it's not JSON or not an array/object, return as a single field
                        let mut map = HashMap::new();
                        map.insert("content".to_string(), Value::String(content));
                        map.insert("key".to_string(), Value::String(full_key));
                        Ok(vec![map])
                    }
                }
            }
            Err(e) => {
                warn!(
                    key = %full_key,
                    error = %e,
                    "Failed to fetch object from S3"
                );
                Err(anyhow!("Failed to fetch object from S3: {}", e))
            }
        }
    }

    async fn execute_mutation(&self, key: &str, data: &HashMap<String, Value>) -> Result<Value> {
        use aws_sdk_s3::primitives::ByteStream;

        let full_key = self.full_key(key);

        debug!(
            bucket = %self.bucket,
            key = %full_key,
            "Storing object in S3"
        );

        // Serialize data to JSON
        let json = serde_json::to_string(data)?;
        let bytes = ByteStream::from(json.into_bytes());

        // Put object in S3
        let result = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(&full_key)
            .body(bytes)
            .content_type("application/json")
            .send()
            .await;

        match result {
            Ok(output) => {
                info!(
                    key = %full_key,
                    etag = ?output.e_tag(),
                    "Successfully stored object in S3"
                );
                Ok(json!({"success": true, "key": full_key}))
            }
            Err(e) => {
                error!(
                    key = %full_key,
                    error = %e,
                    "Failed to store object in S3"
                );
                Err(anyhow!("Failed to store object in S3: {}", e))
            }
        }
    }
}

// Stub when feature is disabled
#[cfg(not(feature = "s3-datasource"))]
pub struct S3DataSource {
    #[allow(dead_code)]
    bucket: String,
}

#[cfg(not(feature = "s3-datasource"))]
impl S3DataSource {
    pub async fn new(bucket: String, _region: String, _prefix: Option<String>) -> Result<Self> {
        Err(anyhow!(
            "S3 support not enabled. Enable the 's3-datasource' feature in Cargo.toml"
        ))
    }
}

#[cfg(not(feature = "s3-datasource"))]
#[async_trait::async_trait]
impl DataSource for S3DataSource {
    async fn execute_query(
        &self,
        _key: &str,
        _params: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<HashMap<String, Value>>> {
        Err(anyhow!("S3 support not enabled"))
    }

    async fn execute_mutation(&self, _key: &str, _data: &HashMap<String, Value>) -> Result<Value> {
        Err(anyhow!("S3 support not enabled"))
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

/// WebSocket data source for real-time communication
#[cfg(feature = "websocket-datasource")]
pub struct WebSocketDataSource {
    url: String,
    #[allow(dead_code)]
    reconnect: bool,
    #[allow(dead_code)]
    heartbeat_interval: Option<u32>,
}

#[cfg(feature = "websocket-datasource")]
impl WebSocketDataSource {
    pub async fn new(
        url: String,
        reconnect: bool,
        heartbeat_interval: Option<u32>,
    ) -> Result<Self> {
        use tokio_tungstenite::connect_async;

        info!(
            url = %url,
            reconnect = reconnect,
            heartbeat_interval = ?heartbeat_interval,
            "Initializing WebSocket data source"
        );

        // Test connection
        match connect_async(&url).await {
            Ok((ws_stream, _)) => {
                info!(url = %url, "Successfully connected to WebSocket");
                drop(ws_stream); // Close test connection
            }
            Err(e) => {
                error!(
                    url = %url,
                    error = %e,
                    "Failed to connect to WebSocket"
                );
                return Err(anyhow!("Failed to connect to WebSocket: {}", e));
            }
        }

        Ok(Self {
            url,
            reconnect,
            heartbeat_interval,
        })
    }

    async fn connect(
        &self,
    ) -> Result<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    > {
        use tokio_tungstenite::connect_async;

        debug!(url = %self.url, "Connecting to WebSocket");

        let (ws_stream, _) = connect_async(&self.url)
            .await
            .map_err(|e| anyhow!("WebSocket connection failed: {}", e))?;

        Ok(ws_stream)
    }
}

#[cfg(feature = "websocket-datasource")]
#[async_trait::async_trait]
impl DataSource for WebSocketDataSource {
    async fn execute_query(
        &self,
        message: &str,
        params: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<HashMap<String, Value>>> {
        use futures_util::{SinkExt, StreamExt};
        use tokio_tungstenite::tungstenite::Message;

        let mut ws_stream = self.connect().await?;

        // Prepare the query message
        let query_data = if let Some(params) = params {
            let mut data = params.clone();
            data.insert("query".to_string(), Value::String(message.to_string()));
            serde_json::to_string(&data)?
        } else {
            message.to_string()
        };

        debug!(message_length = query_data.len(), "Sending WebSocket query");

        // Send the query
        ws_stream
            .send(Message::Text(query_data))
            .await
            .map_err(|e| anyhow!("Failed to send WebSocket message: {}", e))?;

        // Wait for response
        match ws_stream.next().await {
            Some(Ok(Message::Text(text))) => {
                debug!(response_length = text.len(), "Received WebSocket response");

                // Try to parse as JSON
                match serde_json::from_str::<Value>(&text) {
                    Ok(Value::Array(arr)) => {
                        let mut result = Vec::new();
                        for item in arr {
                            if let Value::Object(obj) = item {
                                result.push(obj.into_iter().collect());
                            }
                        }
                        Ok(result)
                    }
                    Ok(Value::Object(obj)) => Ok(vec![obj.into_iter().collect()]),
                    _ => {
                        // If not JSON, return as text
                        let mut map = HashMap::new();
                        map.insert("response".to_string(), Value::String(text));
                        Ok(vec![map])
                    }
                }
            }
            Some(Ok(Message::Binary(bytes))) => {
                // Handle binary response
                let text = String::from_utf8_lossy(&bytes).to_string();
                let mut map = HashMap::new();
                map.insert("response".to_string(), Value::String(text));
                Ok(vec![map])
            }
            Some(Ok(Message::Close(_))) => {
                warn!("WebSocket connection closed by server");
                Err(anyhow!("WebSocket connection closed by server"))
            }
            Some(Err(e)) => {
                error!(error = %e, "WebSocket error");
                Err(anyhow!("WebSocket error: {}", e))
            }
            None => {
                warn!("WebSocket stream ended unexpectedly");
                Err(anyhow!("WebSocket stream ended unexpectedly"))
            }
            _ => {
                warn!("Unexpected WebSocket message type");
                Ok(vec![])
            }
        }
    }

    async fn execute_mutation(
        &self,
        message: &str,
        data: &HashMap<String, Value>,
    ) -> Result<Value> {
        use futures_util::{SinkExt, StreamExt};
        use tokio_tungstenite::tungstenite::Message;

        let mut ws_stream = self.connect().await?;

        // Prepare the mutation message
        let mut mutation_data = data.clone();
        mutation_data.insert("mutation".to_string(), Value::String(message.to_string()));
        let json = serde_json::to_string(&mutation_data)?;

        debug!(message_length = json.len(), "Sending WebSocket mutation");

        // Send the mutation
        ws_stream
            .send(Message::Text(json))
            .await
            .map_err(|e| anyhow!("Failed to send WebSocket message: {}", e))?;

        // Wait for response
        match ws_stream.next().await {
            Some(Ok(Message::Text(text))) => {
                info!(
                    response_length = text.len(),
                    "Received WebSocket mutation response"
                );

                // Try to parse as JSON
                match serde_json::from_str::<Value>(&text) {
                    Ok(value) => Ok(value),
                    Err(_) => Ok(Value::String(text)),
                }
            }
            Some(Ok(Message::Binary(bytes))) => {
                let text = String::from_utf8_lossy(&bytes).to_string();
                Ok(Value::String(text))
            }
            Some(Ok(Message::Close(_))) => {
                warn!("WebSocket connection closed by server");
                Err(anyhow!("WebSocket connection closed by server"))
            }
            Some(Err(e)) => {
                error!(error = %e, "WebSocket error");
                Err(anyhow!("WebSocket error: {}", e))
            }
            None => {
                warn!("WebSocket stream ended unexpectedly");
                Err(anyhow!("WebSocket stream ended unexpectedly"))
            }
            _ => Ok(json!({"success": true})),
        }
    }
}

// Stub when feature is disabled
#[cfg(not(feature = "websocket-datasource"))]
pub struct WebSocketDataSource {
    #[allow(dead_code)]
    url: String,
}

#[cfg(not(feature = "websocket-datasource"))]
impl WebSocketDataSource {
    pub async fn new(
        url: String,
        _reconnect: bool,
        _heartbeat_interval: Option<u32>,
    ) -> Result<Self> {
        Err(anyhow!(
            "WebSocket support not enabled. Enable the 'websocket-datasource' feature in Cargo.toml"
        ))
    }
}

#[cfg(not(feature = "websocket-datasource"))]
#[async_trait::async_trait]
impl DataSource for WebSocketDataSource {
    async fn execute_query(
        &self,
        _message: &str,
        _params: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<HashMap<String, Value>>> {
        Err(anyhow!("WebSocket support not enabled"))
    }

    async fn execute_mutation(
        &self,
        _message: &str,
        _data: &HashMap<String, Value>,
    ) -> Result<Value> {
        Err(anyhow!("WebSocket support not enabled"))
    }
}

/// Factory to create data sources
pub async fn create_data_source(config: &DataSourceConfig) -> Result<Box<dyn DataSource>> {
    match config {
        DataSourceConfig::Database {
            connection_string,
            db_type,
        } => Ok(Box::new(
            DatabaseDataSource::new(connection_string.clone(), db_type.clone()).await?,
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
            ElasticsearchDataSource::new(nodes.clone(), index.clone()),
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
        } => Ok(Box::new(
            WebSocketDataSource::new(url.clone(), *reconnect, *heartbeat_interval).await?,
        )),
    }
}
