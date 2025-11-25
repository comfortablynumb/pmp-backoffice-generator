use crate::config::{BackofficeConfig, RelationshipConfig, RelationshipType};
use crate::data_source::DataSource;
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Validate foreign key relationships before mutation
pub async fn validate_foreign_keys(
    data: &HashMap<String, Value>,
    section_id: &str,
    backoffice: &BackofficeConfig,
    data_sources: &HashMap<String, Box<dyn DataSource>>,
) -> Result<Vec<RelationshipError>> {
    let mut errors = Vec::new();

    // Find all relationships where this section is the source
    let outgoing_relationships: Vec<&RelationshipConfig> = backoffice
        .relationships
        .iter()
        .filter(|r| r.from_section == section_id)
        .collect();

    for relationship in outgoing_relationships {
        // Get the foreign key value from the data
        if let Some(fk_value) = data.get(&relationship.from_field) {
            // Skip null values (unless it's required, which would be caught by field validation)
            if fk_value.is_null() {
                continue;
            }

            // Get the data source for the target section
            let target_section = backoffice
                .sections
                .iter()
                .find(|s| s.id == relationship.to_section)
                .ok_or_else(|| anyhow!("Target section not found: {}", relationship.to_section))?;

            // Get the first action to determine the data source
            let target_action = target_section
                .actions
                .first()
                .ok_or_else(|| anyhow!("No actions found in target section"))?;

            let data_source = data_sources
                .get(&target_action.data_source)
                .ok_or_else(|| anyhow!("Data source not found: {}", target_action.data_source))?;

            // Build query to check if the referenced record exists
            let query = match &relationship.relationship_type {
                RelationshipType::OneToOne | RelationshipType::ManyToOne => {
                    // Check if a record exists with the given ID
                    format!(
                        "SELECT {} FROM {} WHERE {} = '{}'",
                        relationship.to_field,
                        relationship.to_section,
                        relationship.to_field,
                        fk_value.as_str().unwrap_or("")
                    )
                }
                RelationshipType::OneToMany => {
                    // OneToMany relationships don't need FK validation on the "one" side
                    continue;
                }
                RelationshipType::ManyToMany { .. } => {
                    // ManyToMany relationships are validated differently
                    continue;
                }
            };

            debug!(
                relationship = %relationship.id,
                query = %query,
                "Validating foreign key"
            );

            // Execute the query
            match data_source.execute_query(&query, None).await {
                Ok(results) => {
                    if results.is_empty() {
                        errors.push(RelationshipError {
                            relationship_id: relationship.id.clone(),
                            field: relationship.from_field.clone(),
                            message: format!(
                                "Referenced {} with {} = {} does not exist",
                                relationship.to_section,
                                relationship.to_field,
                                fk_value.as_str().unwrap_or("(non-string)")
                            ),
                        });
                    }
                }
                Err(e) => {
                    warn!(
                        error = %e,
                        relationship = %relationship.id,
                        "Failed to validate foreign key"
                    );
                    errors.push(RelationshipError {
                        relationship_id: relationship.id.clone(),
                        field: relationship.from_field.clone(),
                        message: format!("Failed to validate relationship: {}", e),
                    });
                }
            }
        }
    }

    Ok(errors)
}

/// Handle cascade delete operations
pub async fn handle_cascade_delete(
    record_id: &str,
    section_id: &str,
    backoffice: &BackofficeConfig,
    data_sources: &HashMap<String, Box<dyn DataSource>>,
) -> Result<Vec<CascadeOperation>> {
    let mut operations = Vec::new();

    // Find all relationships where this section is the target and cascade_delete is true
    let cascade_relationships: Vec<&RelationshipConfig> = backoffice
        .relationships
        .iter()
        .filter(|r| r.to_section == section_id && r.cascade_delete)
        .collect();

    for relationship in cascade_relationships {
        info!(
            relationship = %relationship.id,
            record_id = %record_id,
            "Processing cascade delete"
        );

        match &relationship.relationship_type {
            RelationshipType::OneToMany | RelationshipType::OneToOne => {
                // Find all records in the source section that reference this record
                let source_section = backoffice
                    .sections
                    .iter()
                    .find(|s| s.id == relationship.from_section)
                    .ok_or_else(|| {
                        anyhow!("Source section not found: {}", relationship.from_section)
                    })?;

                let source_action = source_section.actions.first().ok_or_else(|| {
                    anyhow!(
                        "No actions found in source section: {}",
                        relationship.from_section
                    )
                })?;

                let data_source =
                    data_sources
                        .get(&source_action.data_source)
                        .ok_or_else(|| {
                            anyhow!("Data source not found: {}", source_action.data_source)
                        })?;

                // Query for dependent records
                let query = format!(
                    "SELECT * FROM {} WHERE {} = '{}'",
                    relationship.from_section, relationship.from_field, record_id
                );

                debug!(query = %query, "Finding dependent records");

                match data_source.execute_query(&query, None).await {
                    Ok(dependent_records) => {
                        for record in dependent_records {
                            if let Some(dependent_id) = record.get("id") {
                                operations.push(CascadeOperation {
                                    operation_type: CascadeOperationType::Delete,
                                    section: relationship.from_section.clone(),
                                    record_id: dependent_id.to_string(),
                                    relationship_id: relationship.id.clone(),
                                });

                                // Recursively handle cascades for this record
                                let nested_ops = Box::pin(handle_cascade_delete(
                                    &dependent_id.to_string(),
                                    &relationship.from_section,
                                    backoffice,
                                    data_sources,
                                ))
                                .await?;
                                operations.extend(nested_ops);
                            }
                        }
                    }
                    Err(e) => {
                        warn!(
                            error = %e,
                            relationship = %relationship.id,
                            "Failed to find dependent records"
                        );
                    }
                }
            }
            RelationshipType::ManyToOne => {
                // ManyToOne cascade delete would delete the referenced record
                // This is usually not desired, so we skip it
                debug!(
                    relationship = %relationship.id,
                    "Skipping cascade delete for ManyToOne relationship"
                );
            }
            RelationshipType::ManyToMany { junction_table, .. } => {
                // For ManyToMany, delete entries in the junction table
                operations.push(CascadeOperation {
                    operation_type: CascadeOperationType::DeleteJunction,
                    section: junction_table.clone(),
                    record_id: record_id.to_string(),
                    relationship_id: relationship.id.clone(),
                });
            }
        }
    }

    Ok(operations)
}

/// Execute cascade operations
pub async fn execute_cascade_operations(
    operations: &[CascadeOperation],
    backoffice: &BackofficeConfig,
    data_sources: &HashMap<String, Box<dyn DataSource>>,
) -> Result<()> {
    for operation in operations {
        info!(
            operation_type = ?operation.operation_type,
            section = %operation.section,
            record_id = %operation.record_id,
            "Executing cascade operation"
        );

        let section = backoffice
            .sections
            .iter()
            .find(|s| s.id == operation.section)
            .ok_or_else(|| anyhow!("Section not found: {}", operation.section))?;

        let action = section
            .actions
            .first()
            .ok_or_else(|| anyhow!("No actions found in section: {}", operation.section))?;

        let data_source = data_sources
            .get(&action.data_source)
            .ok_or_else(|| anyhow!("Data source not found: {}", action.data_source))?;

        match operation.operation_type {
            CascadeOperationType::Delete => {
                let query = format!(
                    "DELETE FROM {} WHERE id = '{}'",
                    operation.section, operation.record_id
                );

                debug!(query = %query, "Executing cascade delete");

                let mut data = HashMap::new();
                data.insert("id".to_string(), Value::String(operation.record_id.clone()));

                data_source.execute_mutation(&query, &data).await?;
            }
            CascadeOperationType::DeleteJunction => {
                // Find the relationship to get junction table details
                let relationship = backoffice
                    .relationships
                    .iter()
                    .find(|r| r.id == operation.relationship_id)
                    .ok_or_else(|| {
                        anyhow!("Relationship not found: {}", operation.relationship_id)
                    })?;

                if let RelationshipType::ManyToMany {
                    junction_table,
                    from_junction_field,
                    ..
                } = &relationship.relationship_type
                {
                    let query = format!(
                        "DELETE FROM {} WHERE {} = '{}'",
                        junction_table, from_junction_field, operation.record_id
                    );

                    debug!(query = %query, "Executing junction table delete");

                    let mut data = HashMap::new();
                    data.insert(
                        from_junction_field.clone(),
                        Value::String(operation.record_id.clone()),
                    );

                    data_source.execute_mutation(&query, &data).await?;
                }
            }
            CascadeOperationType::SetNull => {
                // Set foreign key to NULL instead of deleting
                let query = format!(
                    "UPDATE {} SET {} = NULL WHERE id = '{}'",
                    operation.section, "foreign_key_field", operation.record_id
                );

                warn!(query = %query, "SetNull cascade not fully implemented");
            }
        }
    }

    Ok(())
}

/// Validate ManyToMany relationships
pub async fn validate_many_to_many(
    data: &HashMap<String, Value>,
    section_id: &str,
    backoffice: &BackofficeConfig,
    data_sources: &HashMap<String, Box<dyn DataSource>>,
) -> Result<Vec<RelationshipError>> {
    let mut errors = Vec::new();

    let m2m_relationships: Vec<&RelationshipConfig> = backoffice
        .relationships
        .iter()
        .filter(|r| {
            r.from_section == section_id
                && matches!(r.relationship_type, RelationshipType::ManyToMany { .. })
        })
        .collect();

    for relationship in m2m_relationships {
        // Check if the field contains an array of IDs
        if let Some(Value::Array(ids)) = data.get(&relationship.from_field) {
            for id_value in ids {
                if let Some(id) = id_value.as_str() {
                    // Validate that each referenced record exists
                    let target_section = backoffice
                        .sections
                        .iter()
                        .find(|s| s.id == relationship.to_section)
                        .ok_or_else(|| {
                            anyhow!("Target section not found: {}", relationship.to_section)
                        })?;

                    let target_action = target_section.actions.first().ok_or_else(|| {
                        anyhow!(
                            "No actions found in target section: {}",
                            relationship.to_section
                        )
                    })?;

                    let data_source =
                        data_sources
                            .get(&target_action.data_source)
                            .ok_or_else(|| {
                                anyhow!("Data source not found: {}", target_action.data_source)
                            })?;

                    let query = format!(
                        "SELECT {} FROM {} WHERE {} = '{}'",
                        relationship.to_field, relationship.to_section, relationship.to_field, id
                    );

                    debug!(
                        relationship = %relationship.id,
                        id = %id,
                        "Validating ManyToMany reference"
                    );

                    match data_source.execute_query(&query, None).await {
                        Ok(results) => {
                            if results.is_empty() {
                                errors.push(RelationshipError {
                                    relationship_id: relationship.id.clone(),
                                    field: relationship.from_field.clone(),
                                    message: format!(
                                        "Referenced {} with {} = {} does not exist",
                                        relationship.to_section, relationship.to_field, id
                                    ),
                                });
                            }
                        }
                        Err(e) => {
                            warn!(
                                error = %e,
                                relationship = %relationship.id,
                                "Failed to validate ManyToMany reference"
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(errors)
}

/// Relationship validation error
#[derive(Debug, Clone)]
pub struct RelationshipError {
    pub relationship_id: String,
    pub field: String,
    pub message: String,
}

/// Cascade operation to be executed
#[derive(Debug, Clone)]
pub struct CascadeOperation {
    pub operation_type: CascadeOperationType,
    pub section: String,
    pub record_id: String,
    pub relationship_id: String,
}

#[derive(Debug, Clone)]
pub enum CascadeOperationType {
    Delete,
    DeleteJunction,
    #[allow(dead_code)]
    SetNull,
}
