# PMP Backoffice Generator - Examples

This directory contains complete, production-ready example configurations demonstrating all advanced features of the PMP Backoffice Generator.

## Available Examples

### 1. Blog Management System (`blog-with-relationships.yaml`)

A complete blog platform showcasing:

**Relationships:**
- User → Posts (One-to-Many)
- User → Comments (One-to-Many with cascade delete)
- Post → Comments (One-to-Many with cascade delete)
- Post → Category (Many-to-One)
- Post ↔ Tags (Many-to-Many)

**Audit Features:**
- Full change tracking on all tables
- Different retention policies per section
- Selective rollback capabilities
- Created/Updated by tracking

**Key Features:**
- Markdown content editing
- Slug generation
- Image uploads
- Status workflows

**Use Case:** Content management, blogging platforms, documentation systems

---

### 2. E-Commerce Platform (`ecommerce-with-audit.yaml`)

A complete e-commerce system showcasing:

**Relationships:**
- Customer → Orders (One-to-Many)
- Customer → Addresses (One-to-Many with cascade delete)
- Order → Order Items (One-to-Many with cascade delete)
- Product → Order Items (One-to-Many)
- Product → Category (Many-to-One)

**Audit Features:**
- 7-year retention for compliance (customers, orders)
- 10-year retention for products
- Full change history with rollback
- Financial record tracking

**Key Features:**
- Multi-currency support
- Inventory management
- Order workflow
- Payment tracking
- Address management

**Use Case:** E-commerce platforms, retail management, order processing

---

## Feature Highlights

### Relationships

Define relationships between tables in the top-level `relationships` array:

```yaml
relationships:
  - id: user_posts
    name: User Posts
    relationship_type: onetomany
    from_section: users
    from_field: id
    to_section: posts
    to_field: author_id
    cascade_delete: false
    display_in_form: true
    display_in_list: true
    display_fields: ["id", "title", "status", "created_at"]
```

**Relationship Types:**
- `onetomany`: One parent record to many child records
- `manytoone`: Many child records to one parent record
- `onetoone`: One-to-one relationship
- `manytomany`: Many-to-many with junction table

**Options:**
- `cascade_delete`: Automatically delete related records
- `display_in_form`: Show related data in edit forms
- `display_in_list`: Show related data in list views
- `display_fields`: Which fields to show from related records

### Audit Trail

Configure audit tracking per section:

```yaml
sections:
  - id: customers
    name: Customers
    audit:
      track_changes: true       # Enable change tracking
      track_created: true       # Track creation timestamp/user
      track_updated: true       # Track update timestamp/user
      track_deleted: true       # Track deletion
      enable_rollback: true     # Allow version rollback
      retention_days: 2555      # Keep for 7 years
      created_by_field: created_by
      updated_by_field: updated_by
      created_at_field: created_at
      updated_at_field: updated_at
```

**Audit Features:**
- Change history with before/after comparison
- User tracking for all modifications
- Configurable retention periods
- Optional rollback capability
- System-wide activity log

### Advanced Export

All data can be exported in multiple formats:
- **CSV**: Standard CSV with proper escaping
- **Excel (XLSX)**: Formatted spreadsheets with styling
- **JSON**: Clean JSON export
- **PDF**: Professional PDF documents

Exports respect current filters and search state.

### Field-Level Relationships

Link fields to relationships:

```yaml
fields:
  - id: author_id
    name: Author
    field_type: number
    relationship_id: user_posts  # Links to relationship definition
```

This enables:
- Automatic foreign key lookups
- Related data display
- Validation of foreign keys

---

## Database Setup

### Blog System

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    role VARCHAR(20) NOT NULL DEFAULT 'author',
    created_at TIMESTAMP DEFAULT NOW(),
    created_by VARCHAR(100),
    updated_at TIMESTAMP,
    updated_by VARCHAR(100)
);

CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    title VARCHAR(200) NOT NULL,
    slug VARCHAR(250) UNIQUE NOT NULL,
    content TEXT NOT NULL,
    author_id INTEGER REFERENCES users(id),
    category_id INTEGER REFERENCES categories(id),
    status VARCHAR(20) DEFAULT 'draft',
    featured_image VARCHAR(500),
    created_at TIMESTAMP DEFAULT NOW(),
    created_by VARCHAR(100),
    updated_at TIMESTAMP,
    updated_by VARCHAR(100)
);

CREATE TABLE comments (
    id SERIAL PRIMARY KEY,
    post_id INTEGER REFERENCES posts(id) ON DELETE CASCADE,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMP DEFAULT NOW(),
    created_by VARCHAR(100),
    updated_at TIMESTAMP,
    updated_by VARCHAR(100)
);

CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    slug VARCHAR(50) UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE post_tags (
    post_id INTEGER REFERENCES posts(id) ON DELETE CASCADE,
    tag_id INTEGER REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (post_id, tag_id)
);

-- Audit history table (optional, for storing change history)
CREATE TABLE audit_log (
    id SERIAL PRIMARY KEY,
    table_name VARCHAR(50) NOT NULL,
    record_id INTEGER NOT NULL,
    action VARCHAR(20) NOT NULL,
    changes JSONB,
    user_email VARCHAR(100),
    timestamp TIMESTAMP DEFAULT NOW()
);
```

### E-Commerce System

```sql
CREATE TABLE customers (
    id SERIAL PRIMARY KEY,
    email VARCHAR(100) UNIQUE NOT NULL,
    first_name VARCHAR(50) NOT NULL,
    last_name VARCHAR(50) NOT NULL,
    phone VARCHAR(20),
    status VARCHAR(20) DEFAULT 'active',
    total_spent DECIMAL(10,2) DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW(),
    created_by VARCHAR(100),
    updated_at TIMESTAMP,
    updated_by VARCHAR(100)
);

CREATE TABLE addresses (
    id SERIAL PRIMARY KEY,
    customer_id INTEGER REFERENCES customers(id) ON DELETE CASCADE,
    type VARCHAR(20) NOT NULL,
    street VARCHAR(200) NOT NULL,
    city VARCHAR(100) NOT NULL,
    state VARCHAR(100) NOT NULL,
    postal_code VARCHAR(20) NOT NULL,
    country VARCHAR(100) NOT NULL,
    is_default BOOLEAN DEFAULT false,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    sku VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    category_id INTEGER REFERENCES categories(id),
    price DECIMAL(10,2) NOT NULL,
    cost DECIMAL(10,2),
    stock_quantity INTEGER DEFAULT 0,
    weight DECIMAL(8,2),
    status VARCHAR(20) DEFAULT 'active',
    images TEXT[],
    created_at TIMESTAMP DEFAULT NOW(),
    created_by VARCHAR(100),
    updated_at TIMESTAMP,
    updated_by VARCHAR(100)
);

CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    order_number VARCHAR(50) UNIQUE NOT NULL,
    customer_id INTEGER REFERENCES customers(id),
    status VARCHAR(20) DEFAULT 'pending',
    payment_status VARCHAR(20) DEFAULT 'pending',
    subtotal DECIMAL(10,2) NOT NULL,
    tax DECIMAL(10,2) NOT NULL,
    shipping DECIMAL(10,2) NOT NULL,
    total DECIMAL(10,2) NOT NULL,
    notes TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    created_by VARCHAR(100),
    updated_at TIMESTAMP,
    updated_by VARCHAR(100)
);

CREATE TABLE order_items (
    id SERIAL PRIMARY KEY,
    order_id INTEGER REFERENCES orders(id) ON DELETE CASCADE,
    product_id INTEGER REFERENCES products(id),
    quantity INTEGER NOT NULL,
    price DECIMAL(10,2) NOT NULL,
    subtotal DECIMAL(10,2) NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE audit_log (
    id SERIAL PRIMARY KEY,
    table_name VARCHAR(50) NOT NULL,
    record_id INTEGER NOT NULL,
    action VARCHAR(20) NOT NULL,
    changes JSONB,
    user_email VARCHAR(100),
    timestamp TIMESTAMP DEFAULT NOW()
);
```

---

## Usage

1. **Copy an example** to your `backoffices/` directory:
   ```bash
   cp examples/blog-with-relationships.yaml backoffices/
   ```

2. **Set up the database** using the SQL schemas above

3. **Update connection string** in the YAML:
   ```yaml
   data_sources:
     main_db:
       connection_string: postgresql://user:pass@localhost/your_db
   ```

4. **Start the server**:
   ```bash
   cargo run
   ```

5. **Access the backoffice** at `http://localhost:8080`

---

## UI Features

### Viewing Relationships

When viewing or editing a record with relationships:
- **Related Data Tabs**: View all related records in organized tabs
- **Relationship Map**: Visual diagram showing connections
- **Cascade Warnings**: Alerts before deleting records with dependencies

### Audit Trail

- **Audit Panel**: Shows created/updated timestamps and users
- **Change History**: Timeline of all modifications with diff view
- **Rollback**: Restore to any previous version
- **Activity Log**: System-wide audit trail (Ctrl+L)

### Keyboard Shortcuts

- `Ctrl+K`: Focus search
- `Ctrl+E`: Export data
- `Ctrl+D`: Toggle dark mode
- `Ctrl+N`: Create new record
- `Ctrl+L`: Open activity log
- `Ctrl+/`: Show shortcuts help
- `Esc`: Close modal

---

## Best Practices

### Relationships

1. **Cascade Delete**: Only enable for truly dependent data (e.g., order items)
2. **Display Fields**: Limit to 3-5 most important fields for performance
3. **Foreign Keys**: Always add database-level constraints

### Audit Trail

1. **Retention Periods**:
   - Financial records: 7+ years
   - User data: Per GDPR/compliance requirements
   - Simple logs: 90-365 days

2. **Rollback**:
   - Enable for critical data that may need recovery
   - Disable for append-only data (logs, analytics)

3. **Performance**:
   - Index audit timestamp fields
   - Archive old audit data to separate tables
   - Use partitioning for high-volume audit logs

### Security

1. Always validate foreign key relationships at database level
2. Implement proper authentication before enabling audit features
3. Restrict access to activity logs to administrators
4. Encrypt sensitive audit data

---

## Extending Examples

These examples can be extended with:
- Authentication & authorization
- File storage (S3, local)
- Email notifications
- Webhooks for changes
- Custom validation rules
- Workflow automation
- Real-time updates via WebSockets

See the main documentation for full feature reference.
