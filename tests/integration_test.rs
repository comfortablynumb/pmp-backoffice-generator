// Simple integration tests for the backoffice generator

use pmp_backoffice_generator::config::{
    ActionConfig, ActionType, AppConfig, BackofficeConfig, DataSourceConfig, FieldConfig,
    FieldType, ListActionConfig, SectionConfig, SecurityConfig, ServerConfig,
};
use std::collections::HashMap;

#[test]
fn test_create_app_config() {
    let config = AppConfig {
        server: ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 3000,
        },
        security: Some(SecurityConfig {
            enabled: false,
            jwt_secret: None,
        }),
    };

    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.server.port, 3000);
    assert!(config.security.is_some());
    assert!(!config.security.unwrap().enabled);
}

#[test]
fn test_create_backoffice_config() {
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
            id: "users".to_string(),
            name: "Users".to_string(),
            icon: Some("fa-users".to_string()),
            actions: vec![],
            audit: None,
        }],
    };

    assert_eq!(backoffice.id, "test");
    assert_eq!(backoffice.name, "Test Backoffice");
    assert_eq!(backoffice.sections.len(), 1);
    assert_eq!(backoffice.data_sources.len(), 1);
}

#[test]
fn test_create_action_config() {
    let action = ActionConfig {
        id: "list_items".to_string(),
        name: "List Items".to_string(),
        action_type: ActionType::List {
            fields: vec![FieldConfig {
                id: "id".to_string(),
                name: "ID".to_string(),
                field_type: FieldType::Text {
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
            config: ListActionConfig::default(),
        },
        data_source: "test_api".to_string(),
        required_scopes: vec!["read:items".to_string()],
        query: None,
        endpoint: Some("/items".to_string()),
    };

    assert_eq!(action.id, "list_items");
    assert_eq!(action.required_scopes.len(), 1);
    assert_eq!(action.required_scopes[0], "read:items");
}

#[test]
fn test_field_config_with_validation() {
    let field = FieldConfig {
        id: "email".to_string(),
        name: "Email Address".to_string(),
        field_type: FieldType::Email {
            config: Default::default(),
        },
        required: true,
        editable: true,
        visible: true,
        default_value: None,
        placeholder: Some("Enter your email".to_string()),
        help_text: Some("We'll never share your email".to_string()),
        validations: vec![],
        relationship_id: None,
    };

    assert_eq!(field.id, "email");
    assert!(field.required);
    assert_eq!(field.placeholder.as_ref().unwrap(), "Enter your email");
}

#[test]
fn test_section_with_multiple_actions() {
    let section = SectionConfig {
        id: "products".to_string(),
        name: "Products".to_string(),
        icon: Some("fa-box".to_string()),
        actions: vec![
            ActionConfig {
                id: "list_products".to_string(),
                name: "List Products".to_string(),
                action_type: ActionType::List {
                    fields: vec![],
                    config: ListActionConfig::default(),
                },
                data_source: "db".to_string(),
                required_scopes: vec![],
                query: Some("SELECT * FROM products".to_string()),
                endpoint: None,
            },
            ActionConfig {
                id: "create_product".to_string(),
                name: "Create Product".to_string(),
                action_type: ActionType::Form {
                    fields: vec![],
                    config: Default::default(),
                },
                data_source: "db".to_string(),
                required_scopes: vec![],
                query: Some("INSERT INTO products".to_string()),
                endpoint: None,
            },
        ],
        audit: None,
    };

    assert_eq!(section.actions.len(), 2);
    assert_eq!(section.actions[0].id, "list_products");
    assert_eq!(section.actions[1].id, "create_product");
}
