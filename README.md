# PMP Backoffice Generator

A dynamic Rust application that generates multiple backoffice UIs from YAML configuration files. Built with Axum, Tailwind CSS, and jQuery.

## Features

- **Dynamic UI Generation**: Create multiple backoffice interfaces from YAML files
- **Multiple Data Sources**: Support for databases (PostgreSQL, MySQL, SQLite) and REST APIs
- **CRUD Operations**: Built-in support for List, Create, Update, Delete, and View actions
- **Flexible Field Types**: Support for text, number, email, password, date, boolean, select, textarea, and file inputs
- **Field Validation**: Configure validation rules (min/max length, patterns, min/max values, options)
- **Scope-based Authorization**: Define required scopes for each action
- **Modern UI**: Responsive design using Tailwind CSS
- **Interactive Forms**: Dynamic form generation with jQuery

## Architecture

```
├── config/
│   ├── config.yaml              # Main application configuration
│   └── backoffices/             # Directory for backoffice configurations
│       ├── users.yaml           # Example: User management backoffice
│       └── products.yaml        # Example: Product management backoffice
├── src/
│   ├── main.rs                  # Application entry point
│   ├── config.rs                # Configuration models and loaders
│   ├── server.rs                # Web server and API endpoints
│   └── data_source.rs           # Data source connectors (DB, API)
└── static/
    ├── index.html               # Main UI template
    └── app.js                   # jQuery-based dynamic rendering
```

## Quick Start

### Prerequisites

- Rust (latest stable version)
- Cargo

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd pmp-backoffice-generator
```

2. Build the project:
```bash
cargo build --release
```

3. Run the application:
```bash
cargo run
```

4. Open your browser and navigate to:
```
http://localhost:3000
```

## Configuration

### Main Configuration (`config/config.yaml`)

```yaml
server:
  host: "0.0.0.0"
  port: 3000

security:
  enabled: false
  jwt_secret: null
```

### Backoffice Configuration

Each backoffice is defined in a separate YAML file in the `config/backoffices/` directory (supports nested directories).

#### Basic Structure

```yaml
id: "unique-backoffice-id"
name: "Display Name"
description: "Optional description"

data_sources:
  source-name:
    type: api  # or database
    base_url: "https://api.example.com"
    headers:
      Content-Type: "application/json"

sections:
  - id: "section-id"
    name: "Section Name"
    icon: "fa-icon-name"  # Font Awesome icon
    actions:
      - id: "action-id"
        name: "Action Name"
        action_type: list  # list, create, update, delete, view
        data_source: "source-name"
        endpoint: "api/endpoint"  # for API sources
        query: "SQL query"         # for database sources
        required_scopes: ["scope:read"]
        fields:
          - id: "field-id"
            name: "Field Name"
            field_type: text  # text, number, email, password, date, datetime, boolean, select, textarea, file
            required: true
            editable: true
            visible: true
            default_value: null
            validation:
              min_length: 3
              max_length: 100
              pattern: "^[a-zA-Z]+$"
              min_value: 0
              max_value: 100
              options: ["option1", "option2"]
```

## Example Backoffices

### User Management (API-based)

The included `config/backoffices/users.yaml` demonstrates:
- API data source using JSONPlaceholder
- User listing with filtering
- Create, update, and delete operations
- Multiple sections (Users and Posts)
- Field validation

### Product Management (Database + API)

The included `config/backoffices/products.yaml` demonstrates:
- Mixed data sources (SQLite database and API)
- Product inventory management
- Category management
- Select fields with predefined options

## UI Layout

### Top Bar
- Displays all configured backoffices as tabs
- Click to switch between backoffices

### Left Sidebar
- Shows sections for the selected backoffice
- Icons for visual clarity
- Click to view section content

### Main Content Area
- Displays action buttons for the selected section
- Shows data tables for list actions
- Renders dynamic forms for create/update actions

## API Endpoints

The application exposes the following REST API endpoints:

- `GET /` - Main UI
- `GET /api/config` - Application configuration
- `GET /api/backoffices` - List all backoffices
- `GET /api/backoffices/:id` - Get specific backoffice
- `GET /api/backoffices/:backoffice_id/sections/:section_id/actions/:action_id` - Execute query action
- `POST /api/backoffices/:backoffice_id/sections/:section_id/actions/:action_id` - Execute mutation action

## Security Considerations

- Currently, the security system is a placeholder
- Scope checking is defined but not enforced
- For production use:
  - Implement JWT-based authentication
  - Add middleware for scope validation
  - Use HTTPS
  - Sanitize all user inputs
  - Use parameterized queries for database operations

## License

MIT
