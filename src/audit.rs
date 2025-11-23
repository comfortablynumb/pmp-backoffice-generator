use crate::config::AuditConfig;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use tracing::{debug, error, info};

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub operation: AuditOperation,
    pub section_id: String,
    pub record_id: Option<String>,
    pub user_id: Option<String>,
    pub old_values: Option<HashMap<String, Value>>,
    pub new_values: Option<HashMap<String, Value>>,
    pub changes: Vec<FieldChange>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditOperation {
    Create,
    Update,
    Delete,
    Read,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldChange {
    pub field: String,
    pub old_value: Option<Value>,
    pub new_value: Option<Value>,
}

/// Audit logger that writes to JSON files
pub struct AuditLogger {
    log_dir: PathBuf,
    enabled: bool,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(log_dir: impl Into<PathBuf>) -> Self {
        let log_dir = log_dir.into();

        // Create log directory if it doesn't exist
        if let Err(e) = create_dir_all(&log_dir) {
            error!(
                path = ?log_dir,
                error = %e,
                "Failed to create audit log directory"
            );
        }

        Self {
            log_dir,
            enabled: true,
        }
    }

    /// Log an audit entry
    pub fn log(&self, entry: AuditLogEntry) -> anyhow::Result<()> {
        if !self.enabled {
            debug!("Audit logging disabled, skipping");
            return Ok(());
        }

        // Create a log file for today
        let date = Utc::now().format("%Y-%m-%d").to_string();
        let log_file = self.log_dir.join(format!("audit-{}.jsonl", date));

        debug!(
            file = ?log_file,
            operation = ?entry.operation,
            section = %entry.section_id,
            "Writing audit log entry"
        );

        // Open file in append mode
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .map_err(|e| anyhow::anyhow!("Failed to open audit log file: {}", e))?;

        // Write as JSON line
        let json = serde_json::to_string(&entry)
            .map_err(|e| anyhow::anyhow!("Failed to serialize audit entry: {}", e))?;

        writeln!(file, "{}", json)
            .map_err(|e| anyhow::anyhow!("Failed to write audit entry: {}", e))?;

        file.flush()
            .map_err(|e| anyhow::anyhow!("Failed to flush audit log: {}", e))?;

        info!(
            id = %entry.id,
            operation = ?entry.operation,
            "Audit entry logged successfully"
        );

        Ok(())
    }

    /// Create an audit entry for a create operation
    pub fn create_entry(
        section_id: String,
        record_id: Option<String>,
        data: &HashMap<String, Value>,
        user_id: Option<String>,
    ) -> AuditLogEntry {
        AuditLogEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            operation: AuditOperation::Create,
            section_id,
            record_id,
            user_id,
            old_values: None,
            new_values: Some(data.clone()),
            changes: data
                .iter()
                .map(|(field, value)| FieldChange {
                    field: field.clone(),
                    old_value: None,
                    new_value: Some(value.clone()),
                })
                .collect(),
            metadata: HashMap::new(),
        }
    }

    /// Create an audit entry for an update operation
    #[allow(dead_code)]
    pub fn update_entry(
        section_id: String,
        record_id: String,
        old_data: &HashMap<String, Value>,
        new_data: &HashMap<String, Value>,
        user_id: Option<String>,
    ) -> AuditLogEntry {
        let changes = compute_changes(old_data, new_data);

        AuditLogEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            operation: AuditOperation::Update,
            section_id,
            record_id: Some(record_id),
            user_id,
            old_values: Some(old_data.clone()),
            new_values: Some(new_data.clone()),
            changes,
            metadata: HashMap::new(),
        }
    }

    /// Create an audit entry for a delete operation
    pub fn delete_entry(
        section_id: String,
        record_id: String,
        old_data: Option<&HashMap<String, Value>>,
        user_id: Option<String>,
    ) -> AuditLogEntry {
        AuditLogEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            operation: AuditOperation::Delete,
            section_id,
            record_id: Some(record_id),
            user_id,
            old_values: old_data.cloned(),
            new_values: None,
            changes: old_data
                .map(|data| {
                    data.iter()
                        .map(|(field, value)| FieldChange {
                            field: field.clone(),
                            old_value: Some(value.clone()),
                            new_value: None,
                        })
                        .collect()
                })
                .unwrap_or_default(),
            metadata: HashMap::new(),
        }
    }

    /// Check if audit logging should be done for this section
    pub fn should_audit(audit_config: &Option<AuditConfig>, operation: &AuditOperation) -> bool {
        if let Some(config) = audit_config {
            match operation {
                AuditOperation::Create => config.track_created,
                AuditOperation::Update => config.track_updated,
                AuditOperation::Delete => config.track_deleted,
                AuditOperation::Read => false, // Generally don't track reads
            }
        } else {
            false
        }
    }

    /// Clean up old audit logs based on retention policy
    #[allow(dead_code)]
    pub fn cleanup_old_logs(&self, retention_days: u32) -> anyhow::Result<()> {
        info!(
            retention_days = retention_days,
            "Starting audit log cleanup"
        );

        let cutoff_date = Utc::now() - chrono::Duration::days(retention_days as i64);
        let cutoff_str = cutoff_date.format("%Y-%m-%d").to_string();

        let entries = std::fs::read_dir(&self.log_dir)
            .map_err(|e| anyhow::anyhow!("Failed to read audit log directory: {}", e))?;

        let mut deleted_count = 0;
        for entry in entries {
            let entry =
                entry.map_err(|e| anyhow::anyhow!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with("audit-") && file_name.ends_with(".jsonl") {
                    // Extract date from filename
                    if let Some(date_str) = file_name
                        .strip_prefix("audit-")
                        .and_then(|s| s.strip_suffix(".jsonl"))
                    {
                        if date_str < cutoff_str.as_str() {
                            debug!(file = %file_name, "Deleting old audit log");
                            std::fs::remove_file(&path).map_err(|e| {
                                anyhow::anyhow!("Failed to delete audit log: {}", e)
                            })?;
                            deleted_count += 1;
                        }
                    }
                }
            }
        }

        info!(deleted_count = deleted_count, "Audit log cleanup completed");

        Ok(())
    }
}

/// Compute changes between old and new data
#[allow(dead_code)]
fn compute_changes(
    old_data: &HashMap<String, Value>,
    new_data: &HashMap<String, Value>,
) -> Vec<FieldChange> {
    let mut changes = Vec::new();

    // Find all fields that exist in either old or new
    let mut all_fields: Vec<String> = old_data.keys().cloned().collect();
    for field in new_data.keys() {
        if !all_fields.contains(field) {
            all_fields.push(field.clone());
        }
    }

    // Compare each field
    for field in all_fields {
        let old_value = old_data.get(&field).cloned();
        let new_value = new_data.get(&field).cloned();

        // Only record if there's a change
        if old_value != new_value {
            changes.push(FieldChange {
                field,
                old_value,
                new_value,
            });
        }
    }

    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_changes() {
        let mut old_data = HashMap::new();
        old_data.insert("name".to_string(), Value::String("John".to_string()));
        old_data.insert("age".to_string(), Value::Number(30.into()));

        let mut new_data = HashMap::new();
        new_data.insert("name".to_string(), Value::String("Jane".to_string()));
        new_data.insert("age".to_string(), Value::Number(30.into()));

        let changes = compute_changes(&old_data, &new_data);

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "name");
    }

    #[test]
    fn test_audit_entry_creation() {
        let mut data = HashMap::new();
        data.insert("name".to_string(), Value::String("Test".to_string()));

        let entry = AuditLogger::create_entry(
            "users".to_string(),
            Some("123".to_string()),
            &data,
            Some("admin".to_string()),
        );

        assert_eq!(entry.section_id, "users");
        assert_eq!(entry.record_id, Some("123".to_string()));
        assert!(matches!(entry.operation, AuditOperation::Create));
        assert_eq!(entry.changes.len(), 1);
    }
}
