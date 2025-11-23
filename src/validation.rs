use crate::config::{ConditionOperator, FieldConfig, ValidationCondition, ValidationType};
use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDate, Utc};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, warn};

/// Validate data against field configurations
pub fn validate_data(
    data: &HashMap<String, Value>,
    fields: &[FieldConfig],
) -> Result<Vec<ValidationError>> {
    let mut errors = Vec::new();

    for field in fields {
        // Check required fields
        if field.required
            && (!data.contains_key(&field.id) || data[&field.id].is_null())
        {
            errors.push(ValidationError {
                field: field.id.clone(),
                message: format!("{} is required", field.name),
            });
            continue;
        }

        // If field is not present and not required, skip validation
        if !data.contains_key(&field.id) {
            continue;
        }

        let value = &data[&field.id];

        // Skip null values for non-required fields
        if value.is_null() && !field.required {
            continue;
        }

        // Validate each validation rule
        for validation in &field.validations {
            // Check if validation condition is met
            if let Some(condition) = &validation.condition {
                if !evaluate_condition(data, condition) {
                    debug!(
                        field = %field.id,
                        "Skipping validation - condition not met"
                    );
                    continue;
                }
            }

            if let Err(e) = validate_rule(value, &validation.rule_type, field, data) {
                errors.push(ValidationError {
                    field: field.id.clone(),
                    message: validation.message.clone().unwrap_or_else(|| e.to_string()),
                });
            }
        }
    }

    Ok(errors)
}

/// Validation error structure
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

/// Evaluate a validation condition
fn evaluate_condition(data: &HashMap<String, Value>, condition: &ValidationCondition) -> bool {
    let field_value = data.get(&condition.field);

    match &condition.operator {
        ConditionOperator::Equals => field_value == Some(&condition.value),
        ConditionOperator::NotEquals => field_value != Some(&condition.value),
        ConditionOperator::GreaterThan => {
            if let (Some(Value::Number(a)), Value::Number(b)) = (field_value, &condition.value) {
                a.as_f64().unwrap_or(0.0) > b.as_f64().unwrap_or(0.0)
            } else {
                false
            }
        }
        ConditionOperator::LessThan => {
            if let (Some(Value::Number(a)), Value::Number(b)) = (field_value, &condition.value) {
                a.as_f64().unwrap_or(0.0) < b.as_f64().unwrap_or(0.0)
            } else {
                false
            }
        }
        ConditionOperator::GreaterThanOrEqual => {
            if let (Some(Value::Number(a)), Value::Number(b)) = (field_value, &condition.value) {
                a.as_f64().unwrap_or(0.0) >= b.as_f64().unwrap_or(0.0)
            } else {
                false
            }
        }
        ConditionOperator::LessThanOrEqual => {
            if let (Some(Value::Number(a)), Value::Number(b)) = (field_value, &condition.value) {
                a.as_f64().unwrap_or(0.0) <= b.as_f64().unwrap_or(0.0)
            } else {
                false
            }
        }
        ConditionOperator::Contains => {
            if let (Some(Value::String(a)), Value::String(b)) = (field_value, &condition.value) {
                a.contains(b)
            } else {
                false
            }
        }
        ConditionOperator::NotContains => {
            if let (Some(Value::String(a)), Value::String(b)) = (field_value, &condition.value) {
                !a.contains(b)
            } else {
                false
            }
        }
        ConditionOperator::In => {
            if let (Some(val), Value::Array(arr)) = (field_value, &condition.value) {
                arr.contains(val)
            } else {
                false
            }
        }
        ConditionOperator::NotIn => {
            if let (Some(val), Value::Array(arr)) = (field_value, &condition.value) {
                !arr.contains(val)
            } else {
                false
            }
        }
    }
}

/// Validate a single value against a validation rule
fn validate_rule(
    value: &Value,
    rule: &ValidationType,
    field: &FieldConfig,
    all_data: &HashMap<String, Value>,
) -> Result<()> {
    match rule {
        ValidationType::Required { value: required } => {
            if *required && value.is_null() {
                return Err(anyhow!("{} is required", field.name));
            }
            Ok(())
        }
        ValidationType::MinLength { value: min } => {
            if let Some(s) = value.as_str() {
                if s.len() < *min {
                    return Err(anyhow!(
                        "{} must be at least {} characters",
                        field.name,
                        min
                    ));
                }
            }
            Ok(())
        }
        ValidationType::MaxLength { value: max } => {
            if let Some(s) = value.as_str() {
                if s.len() > *max {
                    return Err(anyhow!("{} must be at most {} characters", field.name, max));
                }
            }
            Ok(())
        }
        ValidationType::Pattern { regex } => {
            if let Some(s) = value.as_str() {
                let re = Regex::new(regex).map_err(|e| anyhow!("Invalid regex pattern: {}", e))?;
                if !re.is_match(s) {
                    return Err(anyhow!(
                        "{} does not match the required pattern",
                        field.name
                    ));
                }
            }
            Ok(())
        }
        ValidationType::Min { value: min } => {
            if let Some(n) = value.as_f64() {
                if n < *min {
                    return Err(anyhow!("{} must be at least {}", field.name, min));
                }
            }
            Ok(())
        }
        ValidationType::Max { value: max } => {
            if let Some(n) = value.as_f64() {
                if n > *max {
                    return Err(anyhow!("{} must be at most {}", field.name, max));
                }
            }
            Ok(())
        }
        ValidationType::Email => {
            if let Some(s) = value.as_str() {
                let email_regex = Regex::new(
                    r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$",
                )
                .unwrap();
                if !email_regex.is_match(s) {
                    return Err(anyhow!("{} must be a valid email address", field.name));
                }
            }
            Ok(())
        }
        ValidationType::Url => {
            if let Some(s) = value.as_str() {
                let url_regex =
                    Regex::new(r"^https?://[a-zA-Z0-9-._~:/?#\[\]@!$&'()*+,;=%]+$").unwrap();
                if !url_regex.is_match(s) {
                    return Err(anyhow!("{} must be a valid URL", field.name));
                }
            }
            Ok(())
        }
        ValidationType::Phone => {
            if let Some(s) = value.as_str() {
                let phone_regex = Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap();
                if !phone_regex.is_match(s) {
                    return Err(anyhow!("{} must be a valid phone number", field.name));
                }
            }
            Ok(())
        }
        ValidationType::CustomFunction { function_name } => {
            warn!(
                function = %function_name,
                "Custom validation functions not yet supported"
            );
            Ok(())
        }
        ValidationType::DependsOn {
            field: dep_field,
            expected_value,
        } => {
            if let Some(dep_value) = all_data.get(dep_field) {
                if dep_value != expected_value {
                    return Err(anyhow!(
                        "{} depends on {} having a specific value",
                        field.name,
                        dep_field
                    ));
                }
            }
            Ok(())
        }
        ValidationType::UniqueIn { field_list } => {
            // This would require database access to check uniqueness
            // For now, we'll log a warning
            warn!(
                fields = ?field_list,
                "UniqueIn validation requires database access - not validated"
            );
            Ok(())
        }
        ValidationType::MatchField { field: match_field } => {
            if let Some(match_value) = all_data.get(match_field) {
                if value != match_value {
                    return Err(anyhow!("{} must match {}", field.name, match_field));
                }
            }
            Ok(())
        }
        ValidationType::CreditCard => {
            if let Some(s) = value.as_str() {
                if !validate_luhn(s) {
                    return Err(anyhow!("{} must be a valid credit card number", field.name));
                }
            }
            Ok(())
        }
        ValidationType::Ipv4 => {
            if let Some(s) = value.as_str() {
                let ipv4_regex =
                    Regex::new(r"^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4}$").unwrap();
                if !ipv4_regex.is_match(s) {
                    return Err(anyhow!("{} must be a valid IPv4 address", field.name));
                }
            }
            Ok(())
        }
        ValidationType::Ipv6 => {
            if let Some(s) = value.as_str() {
                let ipv6_regex = Regex::new(
                    r"^(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))$"
                ).unwrap();
                if !ipv6_regex.is_match(s) {
                    return Err(anyhow!("{} must be a valid IPv6 address", field.name));
                }
            }
            Ok(())
        }
        ValidationType::Uuid => {
            if let Some(s) = value.as_str() {
                if uuid::Uuid::parse_str(s).is_err() {
                    return Err(anyhow!("{} must be a valid UUID", field.name));
                }
            }
            Ok(())
        }
        ValidationType::DateRange {
            start_field,
            end_field,
        } => {
            let start_value = all_data.get(start_field);
            let end_value = all_data.get(end_field);

            if let (Some(Value::String(start)), Some(Value::String(end))) = (start_value, end_value)
            {
                if let (Ok(start_date), Ok(end_date)) = (
                    DateTime::parse_from_rfc3339(start),
                    DateTime::parse_from_rfc3339(end),
                ) {
                    if start_date >= end_date {
                        return Err(anyhow!("Start date must be before end date"));
                    }
                }
            }
            Ok(())
        }
        ValidationType::FileSize { max_size_mb } => {
            // File size validation would typically be done on upload
            // This is a placeholder for future implementation
            warn!(
                max_size = max_size_mb,
                "FileSize validation requires file upload context"
            );
            Ok(())
        }
        ValidationType::FileType { allowed_types } => {
            // File type validation would typically be done on upload
            warn!(
                types = ?allowed_types,
                "FileType validation requires file upload context"
            );
            Ok(())
        }
        ValidationType::StrongPassword {
            min_length,
            require_uppercase,
            require_lowercase,
            require_number,
            require_special,
        } => {
            if let Some(s) = value.as_str() {
                if s.len() < *min_length {
                    return Err(anyhow!(
                        "Password must be at least {} characters",
                        min_length
                    ));
                }
                if *require_uppercase && !s.chars().any(|c| c.is_uppercase()) {
                    return Err(anyhow!(
                        "Password must contain at least one uppercase letter"
                    ));
                }
                if *require_lowercase && !s.chars().any(|c| c.is_lowercase()) {
                    return Err(anyhow!(
                        "Password must contain at least one lowercase letter"
                    ));
                }
                if *require_number && !s.chars().any(|c| c.is_numeric()) {
                    return Err(anyhow!("Password must contain at least one number"));
                }
                if *require_special && !s.chars().any(|c| !c.is_alphanumeric()) {
                    return Err(anyhow!(
                        "Password must contain at least one special character"
                    ));
                }
            }
            Ok(())
        }
        ValidationType::AlphaNumeric => {
            if let Some(s) = value.as_str() {
                if !s.chars().all(|c| c.is_alphanumeric()) {
                    return Err(anyhow!(
                        "{} must contain only alphanumeric characters",
                        field.name
                    ));
                }
            }
            Ok(())
        }
        ValidationType::Luhn => {
            if let Some(s) = value.as_str() {
                if !validate_luhn(s) {
                    return Err(anyhow!("{} failed Luhn check", field.name));
                }
            }
            Ok(())
        }
        ValidationType::MacAddress => {
            if let Some(s) = value.as_str() {
                let mac_regex = Regex::new(r"^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$").unwrap();
                if !mac_regex.is_match(s) {
                    return Err(anyhow!("{} must be a valid MAC address", field.name));
                }
            }
            Ok(())
        }
        ValidationType::Isbn => {
            if let Some(s) = value.as_str() {
                // Simplified ISBN validation - checks for ISBN-10 or ISBN-13 format
                let isbn_regex = Regex::new(r"^(?:ISBN(?:-1[03])?:?\s*)?(?:[0-9]{9}[0-9X]|(?:97[89])?[0-9]{10})$").unwrap();
                let normalized = s.replace(&['-', ' '][..], "");
                if !isbn_regex.is_match(&normalized) {
                    return Err(anyhow!("{} must be a valid ISBN", field.name));
                }
            }
            Ok(())
        }
        ValidationType::Iban => {
            if let Some(s) = value.as_str() {
                let iban_regex = Regex::new(r"^[A-Z]{2}[0-9]{2}[A-Z0-9]{1,30}$").unwrap();
                if !iban_regex.is_match(s) {
                    return Err(anyhow!("{} must be a valid IBAN", field.name));
                }
            }
            Ok(())
        }
        ValidationType::Ssn => {
            if let Some(s) = value.as_str() {
                let ssn_regex = Regex::new(r"^\d{3}-\d{2}-\d{4}$").unwrap();
                if !ssn_regex.is_match(s) {
                    return Err(anyhow!("{} must be a valid SSN (XXX-XX-XXXX)", field.name));
                }
            }
            Ok(())
        }
        ValidationType::PostalCode { country_code } => {
            if let Some(s) = value.as_str() {
                let is_valid = match country_code.as_str() {
                    "US" => Regex::new(r"^\d{5}(-\d{4})?$").unwrap().is_match(s),
                    "UK" => Regex::new(r"^[A-Z]{1,2}\d{1,2}[A-Z]?\s?\d[A-Z]{2}$")
                        .unwrap()
                        .is_match(s),
                    "CA" => Regex::new(r"^[A-Z]\d[A-Z]\s?\d[A-Z]\d$")
                        .unwrap()
                        .is_match(s),
                    _ => true, // Unknown country codes pass
                };
                if !is_valid {
                    return Err(anyhow!(
                        "{} must be a valid {} postal code",
                        field.name,
                        country_code
                    ));
                }
            }
            Ok(())
        }
        ValidationType::Base64 => {
            if let Some(s) = value.as_str() {
                let base64_regex = Regex::new(r"^[A-Za-z0-9+/]*={0,2}$").unwrap();
                if !base64_regex.is_match(s) {
                    return Err(anyhow!("{} must be valid Base64", field.name));
                }
            }
            Ok(())
        }
        ValidationType::Json => {
            if let Some(s) = value.as_str() {
                if serde_json::from_str::<Value>(s).is_err() {
                    return Err(anyhow!("{} must be valid JSON", field.name));
                }
            }
            Ok(())
        }
        ValidationType::Hex => {
            if let Some(s) = value.as_str() {
                let hex_regex = Regex::new(r"^[0-9a-fA-F]+$").unwrap();
                if !hex_regex.is_match(s) {
                    return Err(anyhow!("{} must be valid hexadecimal", field.name));
                }
            }
            Ok(())
        }
        ValidationType::Ascii => {
            if let Some(s) = value.as_str() {
                if !s.is_ascii() {
                    return Err(anyhow!("{} must contain only ASCII characters", field.name));
                }
            }
            Ok(())
        }
        ValidationType::NotEmpty => {
            if let Some(s) = value.as_str() {
                if s.trim().is_empty() {
                    return Err(anyhow!("{} must not be empty", field.name));
                }
            }
            Ok(())
        }
        ValidationType::Future => {
            if let Some(s) = value.as_str() {
                if let Ok(date) = DateTime::parse_from_rfc3339(s) {
                    if date <= Utc::now() {
                        return Err(anyhow!("{} must be a future date", field.name));
                    }
                } else if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                    let now = Utc::now().date_naive();
                    if date <= now {
                        return Err(anyhow!("{} must be a future date", field.name));
                    }
                }
            }
            Ok(())
        }
        ValidationType::Past => {
            if let Some(s) = value.as_str() {
                if let Ok(date) = DateTime::parse_from_rfc3339(s) {
                    if date >= Utc::now() {
                        return Err(anyhow!("{} must be a past date", field.name));
                    }
                } else if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                    let now = Utc::now().date_naive();
                    if date >= now {
                        return Err(anyhow!("{} must be a past date", field.name));
                    }
                }
            }
            Ok(())
        }
        ValidationType::MinAge { years } => {
            if let Some(s) = value.as_str() {
                if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                    let now = Utc::now().date_naive();
                    let age_days = (now - date).num_days();
                    let min_days = (*years as i64) * 365;
                    if age_days < min_days {
                        return Err(anyhow!("Must be at least {} years old", years));
                    }
                }
            }
            Ok(())
        }
        ValidationType::MaxAge { years } => {
            if let Some(s) = value.as_str() {
                if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                    let now = Utc::now().date_naive();
                    let age_days = (now - date).num_days();
                    let max_days = (*years as i64) * 365;
                    if age_days > max_days {
                        return Err(anyhow!("Must be at most {} years old", years));
                    }
                }
            }
            Ok(())
        }
        ValidationType::Between { min, max } => {
            if let Some(n) = value.as_f64() {
                if n < *min || n > *max {
                    return Err(anyhow!(
                        "{} must be between {} and {}",
                        field.name,
                        min,
                        max
                    ));
                }
            }
            Ok(())
        }
    }
}

/// Luhn algorithm implementation for credit card and similar validation
fn validate_luhn(s: &str) -> bool {
    let digits: Vec<u32> = s
        .chars()
        .filter(|c| c.is_numeric())
        .filter_map(|c| c.to_digit(10))
        .collect();

    if digits.len() < 2 {
        return false;
    }

    let checksum: u32 = digits
        .iter()
        .rev()
        .enumerate()
        .map(|(idx, &digit)| {
            if idx % 2 == 1 {
                let doubled = digit * 2;
                if doubled > 9 {
                    doubled - 9
                } else {
                    doubled
                }
            } else {
                digit
            }
        })
        .sum();

    checksum.is_multiple_of(10)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{FieldConfig, FieldType, TextFieldConfig, ValidationRule};

    #[test]
    fn test_email_validation() {
        let field = FieldConfig {
            id: "email".to_string(),
            name: "Email".to_string(),
            field_type: FieldType::Email {
                config: Default::default(),
            },
            required: false,
            editable: true,
            visible: true,
            default_value: None,
            placeholder: None,
            help_text: None,
            validations: vec![ValidationRule {
                rule_type: ValidationType::Email,
                message: None,
                condition: None,
            }],
            relationship_id: None,
        };

        let mut data = HashMap::new();
        data.insert(
            "email".to_string(),
            Value::String("test@example.com".to_string()),
        );

        let errors = validate_data(&data, &[field.clone()]).unwrap();
        assert_eq!(errors.len(), 0);

        data.insert("email".to_string(), Value::String("invalid".to_string()));
        let errors = validate_data(&data, &[field]).unwrap();
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_luhn_algorithm() {
        assert!(validate_luhn("4532015112830366")); // Valid Visa
        assert!(!validate_luhn("1234567890123456")); // Invalid
    }

    #[test]
    fn test_required_validation() {
        let field = FieldConfig {
            id: "name".to_string(),
            name: "Name".to_string(),
            field_type: FieldType::Text {
                config: TextFieldConfig::default(),
            },
            required: true,
            editable: true,
            visible: true,
            default_value: None,
            placeholder: None,
            help_text: None,
            validations: vec![],
            relationship_id: None,
        };

        let data = HashMap::new();
        let errors = validate_data(&data, &[field]).unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "name");
    }
}
