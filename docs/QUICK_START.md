# Quick Start Guide

Get up and running with PMP Backoffice Generator in 5 minutes.

## Installation

### Using Docker (Recommended)

```bash
# Clone the repository
git clone <repository-url>
cd pmp-backoffice-generator

# Start with docker-compose
docker-compose up

# Open browser
open http://localhost:3000
```

### Local Development

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build and run
cargo build --release
cargo run
```

## Create Your First Backoffice

### 1. Create Configuration File

Create `config/backoffices/myapp.yaml`:

```yaml
id: "myapp"
name: "My Application"
description: "My first backoffice"

data_sources:
  db:
    type: database
    connection_string: "postgresql://localhost:5432/mydb"
    db_type: postgres

sections:
  - id: users
    name: Users
    icon: "fa-users"
    actions:
      - id: list_users
        name: User List
        action_type: list
        data_source: db
        query: "SELECT * FROM users ORDER BY created_at DESC"
        fields:
          - id: id
            name: ID
            field_type: text
            visible: true
            editable: false

          - id: email
            name: Email
            field_type: email
            visible: true
            required: true

          - id: name
            name: Full Name
            field_type: text
            visible: true
            required: true

          - id: created_at
            name: Joined
            field_type: datetime
            visible: true
            editable: false
```

### 2. Restart Application

```bash
# If using Docker
docker-compose restart

# If running locally
cargo run
```

### 3. Access Your Backoffice

1. Open http://localhost:3000
2. Click on "My Application" tab
3. Click on "Users" in the sidebar
4. You'll see your user list!

## Common Tasks

### Adding a New Field

```yaml
fields:
  - id: status
    name: Status
    field_type: select
    visible: true
    required: true
    config:
      options:
        - value: active
          label: Active
        - value: inactive
          label: Inactive
```

### Adding a Create Form

```yaml
actions:
  - id: create_user
    name: Create User
    action_type: form
    data_source: db
    config:
      form_mode: create
    query: |
      INSERT INTO users (email, name, status)
      VALUES ($1, $2, $3)
      RETURNING *
    fields:
      # ... same fields as list
```

### Adding Validation

```yaml
fields:
  - id: email
    name: Email
    field_type: email
    required: true
    validations:
      - type: email
        message: "Invalid email format"
```

### Adding Filters

```yaml
config:
  filters:
    - field: status
      label: Status
      type: select
      options:
        - active
        - inactive
```

## Using Features

### Search
- Type in the search box above the table
- Results filter in real-time

### Sort
- Click any column header to sort
- Click again to reverse sort order

### Bulk Operations
1. Check boxes next to rows
2. Click "Delete Selected" or "Export Selected"
3. Confirm action

### Export to CSV
- Click "Export" button
- File downloads automatically

### Import from CSV
1. Click "Import" button
2. Select CSV file (with headers)
3. Click "Import"
4. Watch progress

### Dark Mode
- Click moon/sun icon in top-right
- Setting persists across sessions

### Keyboard Shortcuts
- Press `Ctrl/Cmd + /` to see all shortcuts

## Next Steps

- Read [FEATURES.md](./FEATURES.md) for all field types
- Check [EXAMPLES.md](./EXAMPLES.md) for complete examples
- See [API Documentation](http://localhost:3000/api/docs) for REST API

## Troubleshooting

**Problem: Can't connect to database**
```
Solution: Check connection string in data source configuration
```

**Problem: No data showing**
```
Solution: Verify SQL query returns results and field IDs match column names
```

**Problem: Validation not working**
```
Solution: Ensure validation type is correct and validations array is properly formatted
```

## Getting Help

- Check documentation in `docs/` folder
- Open issue on GitHub
- Read inline help text (hover over ? icons)
