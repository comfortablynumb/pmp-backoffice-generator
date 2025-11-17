# PMP Backoffice Generator

A powerful, dynamic Rust application that generates multiple backoffice UIs from YAML configuration files. Built with Axum, Tailwind CSS, and jQuery.

[![CI](https://github.com/YOUR_USERNAME/pmp-backoffice-generator/workflows/CI/badge.svg)](https://github.com/YOUR_USERNAME/pmp-backoffice-generator/actions/workflows/ci.yml)

## ✨ Features

### Core Features
- **Dynamic UI Generation**: Create unlimited backoffice interfaces from YAML files
- **30+ Field Types**: Text, email, URL, phone, rich text, color picker, signature, video, audio, JSON, markdown, and more
- **24+ Validation Types**: ISBN, IBAN, credit card, IP addresses, MAC addresses, coordinates, and more
- **10+ Data Sources**: Database, REST API, GraphQL, MongoDB, Redis, Elasticsearch, gRPC, Kafka, S3, Firebase, Supabase, WebSocket
- **CRUD Operations**: Built-in support for List, Create, Update, Delete, and View actions
- **Scope-based Authorization**: Define required scopes for each action

### UI/UX Features
- 🎨 **Dark Mode**: Toggle-able dark theme with localStorage persistence
- 🔍 **Real-time Search**: Instant filtering across all table columns
- 📊 **Column Sorting**: Multi-column sorting with visual indicators
- 📝 **Inline Editing**: Double-click to edit cells directly in tables
- 🎯 **Advanced Filtering**: Column-based filters with multiple operators
- 💾 **Filter Presets**: Save and load filter configurations
- 🔔 **Toast Notifications**: Modern notification system (success, error, warning, info)
- ⌨️ **Keyboard Shortcuts**: Power-user shortcuts for common actions

### Bulk Operations
- ☑️ **Bulk Selection**: Select multiple rows with checkboxes
- 🗑️ **Bulk Delete**: Delete multiple records at once
- 📤 **Bulk Export**: Export selected rows to CSV
- 📥 **CSV Import**: Import data from CSV files
- 📊 **Progress Indicators**: Real-time progress tracking for bulk operations

### Developer Features
- 🐳 **Docker Support**: Easy deployment with Docker and docker-compose
- 📚 **OpenAPI/Swagger**: Interactive API documentation at `/api/docs`
- 🧪 **Unit & Integration Tests**: Comprehensive test coverage
- 🔧 **Modern Stack**: Rust, Axum, Tailwind CSS, jQuery
- 📖 **Extensive Documentation**: Complete guides and examples

## 📖 Documentation

- **[Quick Start Guide](docs/QUICK_START.md)** - Get started in 5 minutes
- **[Complete Feature Documentation](docs/FEATURES.md)** - All field types, validations, and data sources with examples
- **[Real-World Examples](docs/EXAMPLES.md)** - Production-ready configurations for common use cases
- **[API Documentation](http://localhost:3000/api/docs)** - Interactive Swagger UI (when running)
- **[OpenAPI Specification](openapi.yaml)** - Full API spec in OpenAPI 3.0 format

## Quick Start

### Using Docker (Recommended)

```bash
# Clone the repository
git clone <repository-url>
cd pmp-backoffice-generator

# Start with docker-compose
docker-compose up

# Open browser to http://localhost:3000
```

### Local Development

```bash
# Prerequisites: Rust (latest stable)
cargo build --release
cargo run

# Open browser to http://localhost:3000
```

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
│   └── data_source.rs           # Data source connectors
└── static/
    ├── index.html               # Main UI template
    └── app.js                   # jQuery-based dynamic rendering
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

Each backoffice is defined in a YAML file in `config/backoffices/` (supports nested directories).

## Field Types (30+)

### Basic Fields
- `text` - Single-line text input
- `textarea` - Multi-line text input
- `number` - Numeric input
- `email` - Email input with validation
- `password` - Password input (masked)
- `tel` - Phone number input
- `url` - URL input with validation

### Date & Time
- `date` - Date picker
- `datetime` - Date and time picker
- `time` - Time picker
- `datetime_range` - Date/time range selector
- `month` - Month picker
- `week` - Week picker

### Selection & Choice
- `select` - Dropdown selection
- `radio` - Radio button group
- `checkbox` - Checkbox group
- `toggle` - Toggle switch
- `tags` - Tag input with autocomplete
- `multiselect` - Multiple selection dropdown

### Rich Content
- `richtext` - WYSIWYG editor
- `markdown` - Markdown editor with preview
- `code` - Code editor with syntax highlighting
- `json` - JSON editor with validation
- `html` - HTML editor

### Media & Files
- `file` - File upload
- `image` - Image upload with preview
- `video` - Video upload
- `audio` - Audio upload
- `signature` - Digital signature pad

### Advanced
- `color` - Color picker
- `rating` - Star rating input
- `slider` - Range slider
- `geolocation` - GPS coordinates
- `currency` - Currency input
- `percentage` - Percentage input

## Validation Types (24+)

### String Validations
- `min_length` / `max_length` - String length constraints
- `pattern` - Regular expression pattern
- `email` - Email format validation
- `url` - URL format validation
- `uuid` - UUID format validation
- `slug` - URL-friendly slug validation
- `alpha` - Alphabetic characters only
- `alphanumeric` - Alphanumeric characters only
- `lowercase` / `uppercase` - Case validation

### Numeric Validations
- `min_value` / `max_value` - Numeric range
- `integer` - Integer validation
- `positive` / `negative` - Sign validation
- `even` / `odd` - Parity validation

### Financial & Identity
- `credit_card` - Credit card number validation
- `iban` - IBAN validation
- `isbn` - ISBN validation
- `issn` - ISSN validation

### Network & Location
- `ipv4` / `ipv6` - IP address validation
- `mac_address` - MAC address validation
- `latitude` / `longitude` - GPS coordinates
- `port` - Port number validation

### Other
- `phone` - Phone number validation
- `json` - Valid JSON validation
- `base64` - Base64 encoding validation

## Data Sources (10+)

### Database
```yaml
data_sources:
  my_db:
    type: database
    connection_string: "postgresql://user:pass@localhost/db"
```

### REST API
```yaml
data_sources:
  my_api:
    type: api
    base_url: "https://api.example.com"
    headers:
      Authorization: "Bearer token"
      Content-Type: "application/json"
```

### GraphQL
```yaml
data_sources:
  my_graphql:
    type: graphql
    endpoint: "https://api.example.com/graphql"
    headers:
      Authorization: "Bearer token"
```

### MongoDB
```yaml
data_sources:
  my_mongo:
    type: mongodb
    connection_string: "mongodb://localhost:27017"
    database: "mydb"
```

### Redis
```yaml
data_sources:
  my_redis:
    type: redis
    connection_string: "redis://localhost:6379"
```

### Elasticsearch
```yaml
data_sources:
  my_search:
    type: elasticsearch
    url: "http://localhost:9200"
```

### gRPC
```yaml
data_sources:
  my_grpc:
    type: grpc
    endpoint: "localhost:50051"
```

### Kafka
```yaml
data_sources:
  my_kafka:
    type: kafka
    brokers: ["localhost:9092"]
```

### S3
```yaml
data_sources:
  my_s3:
    type: s3
    bucket: "my-bucket"
    region: "us-east-1"
```

### Firebase
```yaml
data_sources:
  my_firebase:
    type: firebase
    project_id: "my-project"
    credentials_path: "/path/to/credentials.json"
```

### Supabase
```yaml
data_sources:
  my_supabase:
    type: supabase
    url: "https://xxx.supabase.co"
    anon_key: "your-anon-key"
```

### WebSocket
```yaml
data_sources:
  my_websocket:
    type: websocket
    url: "wss://api.example.com/ws"
```

## Complete Example: E-commerce Backoffice

```yaml
id: "ecommerce"
name: "E-commerce Admin"
description: "Complete e-commerce management system"

data_sources:
  products_db:
    type: database
    connection_string: "postgresql://localhost/products"

  analytics_api:
    type: api
    base_url: "https://analytics.example.com"
    headers:
      Authorization: "Bearer ${API_KEY}"

sections:
  - id: "products"
    name: "Products"
    icon: "fa-box"
    actions:
      - id: "list_products"
        name: "Product List"
        action_type: list
        data_source: "products_db"
        query: "SELECT * FROM products WHERE deleted_at IS NULL ORDER BY created_at DESC"
        required_scopes: ["products:read"]
        fields:
          - id: "id"
            name: "ID"
            field_type: text
            visible: true
            editable: false

          - id: "name"
            name: "Product Name"
            field_type: text
            required: true
            visible: true
            editable: true
            validation:
              min_length: 3
              max_length: 200

          - id: "sku"
            name: "SKU"
            field_type: text
            required: true
            validation:
              pattern: "^[A-Z0-9-]+$"
              uppercase: true

          - id: "description"
            name: "Description"
            field_type: richtext
            required: false

          - id: "price"
            name: "Price"
            field_type: currency
            required: true
            validation:
              min_value: 0
              positive: true

          - id: "stock"
            name: "Stock"
            field_type: number
            required: true
            validation:
              integer: true
              min_value: 0

          - id: "category"
            name: "Category"
            field_type: select
            required: true
            options:
              - value: "electronics"
                label: "Electronics"
              - value: "clothing"
                label: "Clothing"
              - value: "books"
                label: "Books"

          - id: "tags"
            name: "Tags"
            field_type: tags
            required: false

          - id: "image"
            name: "Product Image"
            field_type: image
            required: false

          - id: "active"
            name: "Active"
            field_type: toggle
            default_value: true

          - id: "rating"
            name: "Rating"
            field_type: rating
            validation:
              min_value: 0
              max_value: 5

          - id: "created_at"
            name: "Created"
            field_type: datetime
            editable: false
            visible: true

      - id: "create_product"
        name: "Create Product"
        action_type: create
        data_source: "products_db"
        query: "INSERT INTO products (name, sku, description, price, stock, category, tags, image, active, rating) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
        required_scopes: ["products:write"]
        fields:
          # Same fields as list, excluding id and timestamps

      - id: "update_product"
        name: "Update Product"
        action_type: update
        data_source: "products_db"
        query: "UPDATE products SET name=$1, sku=$2, description=$3, price=$4, stock=$5, category=$6, tags=$7, image=$8, active=$9, rating=$10, updated_at=NOW() WHERE id=$11"
        required_scopes: ["products:write"]
        fields:
          # Same fields as list

      - id: "delete_product"
        name: "Delete Product"
        action_type: delete
        data_source: "products_db"
        query: "UPDATE products SET deleted_at=NOW() WHERE id=$1"
        required_scopes: ["products:delete"]

  - id: "customers"
    name: "Customers"
    icon: "fa-users"
    actions:
      - id: "list_customers"
        name: "Customer List"
        action_type: list
        data_source: "products_db"
        query: "SELECT * FROM customers ORDER BY created_at DESC"
        required_scopes: ["customers:read"]
        fields:
          - id: "email"
            name: "Email"
            field_type: email
            required: true
            validation:
              email: true

          - id: "phone"
            name: "Phone"
            field_type: tel
            required: false
            validation:
              phone: true

          - id: "address"
            name: "Address"
            field_type: textarea

          - id: "coordinates"
            name: "Location"
            field_type: geolocation
            required: false

  - id: "analytics"
    name: "Analytics"
    icon: "fa-chart-line"
    actions:
      - id: "sales_data"
        name: "Sales Dashboard"
        action_type: view
        data_source: "analytics_api"
        endpoint: "/api/v1/sales/summary"
        required_scopes: ["analytics:read"]
```

## Development

### Running Tests

```bash
cargo test --all-features --verbose
```

### Format Code

```bash
cargo fmt --all
```

### Run Linter

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Code Coverage

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --all-features --workspace --html
```

## Deployment

### Docker

```bash
# Build image
docker build -t pmp-backoffice-generator .

# Run container
docker run -p 3000:3000 -v $(pwd)/config:/app/config pmp-backoffice-generator
```

### Docker Compose

```bash
# Start services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### Production Build

```bash
cargo build --release
./target/release/pmp-backoffice-generator
```

## API Endpoints

- `GET /` - Main UI
- `GET /api/config` - Application configuration
- `GET /api/backoffices` - List all backoffices
- `GET /api/backoffices/:id` - Get specific backoffice
- `GET /api/backoffices/:backoffice_id/sections/:section_id/actions/:action_id` - Execute query action
- `POST /api/backoffices/:backoffice_id/sections/:section_id/actions/:action_id` - Execute mutation action

## Security Considerations

For production use:
- Implement JWT-based authentication
- Add middleware for scope validation
- Use HTTPS with valid certificates
- Sanitize all user inputs
- Use parameterized queries for database operations
- Enable CORS with proper origin restrictions
- Set up rate limiting
- Implement audit logging

## Roadmap

- [ ] Authentication & Authorization (JWT, OAuth)
- [ ] Real-time updates via WebSocket
- [ ] Advanced filtering and search
- [ ] Data export (CSV, Excel, PDF)
- [ ] Bulk operations
- [ ] Audit trail logging
- [ ] Multi-language support (i18n)
- [ ] Mobile responsive improvements
- [ ] Plugin system
- [ ] Custom themes

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

MIT

## Support

For issues and questions, please open an issue on GitHub.
