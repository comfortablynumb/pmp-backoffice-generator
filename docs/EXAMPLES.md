# Real-World Configuration Examples

This document provides complete, production-ready examples for common use cases.

## Table of Contents

1. [User Management System](#user-management-system)
2. [E-Commerce Platform](#e-commerce-platform)
3. [Content Management System (CMS)](#content-management-system)
4. [Project Management Tool](#project-management-tool)
5. [Customer Support Ticketing](#customer-support-ticketing)
6. [Inventory Management](#inventory-management)
7. [Event Management](#event-management)
8. [HR Management](#hr-management)

---

## User Management System

Complete user management with roles, permissions, and authentication.

### Configuration: `config/backoffices/users.yaml`

```yaml
id: "user_management"
name: "User Management"
description: "Manage users, roles, and permissions"

data_sources:
  users_db:
    type: database
    connection_string: "postgresql://localhost:5432/auth_db"
    db_type: postgres

sections:
  - id: users
    name: Users
    icon: "fa-users"
    actions:
      # List Users
      - id: list_users
        name: User Directory
        action_type: list
        data_source: users_db
        required_scopes: ["users:read"]
        query: |
          SELECT
            u.id,
            u.username,
            u.email,
            u.first_name,
            u.last_name,
            u.phone,
            u.avatar_url,
            u.status,
            u.email_verified,
            u.last_login_at,
            u.created_at,
            r.name as role_name
          FROM users u
          LEFT JOIN roles r ON u.role_id = r.id
          WHERE u.deleted_at IS NULL
          ORDER BY u.created_at DESC
        config:
          pagination:
            enabled: true
            page_size: 25
            show_size_options: true
          filters:
            - field: status
              label: Status
              type: select
              options:
                - active
                - inactive
                - suspended
            - field: email_verified
              label: Email Verified
              type: boolean
            - field: role_name
              label: Role
              type: select
            - field: created_at
              label: Registration Date
              type: date_range
        fields:
          - id: id
            name: ID
            field_type: text
            visible: true
            editable: false

          - id: avatar_url
            name: Avatar
            field_type: image
            visible: true
            editable: false

          - id: username
            name: Username
            field_type: text
            visible: true
            editable: true
            required: true
            validations:
              - type: min_length
                value: 3
              - type: max_length
                value: 30
              - type: pattern
                value: "^[a-z0-9_]+$"
                message: "Lowercase letters, numbers, and underscores only"

          - id: email
            name: Email
            field_type: email
            visible: true
            editable: true
            required: true
            validations:
              - type: email

          - id: first_name
            name: First Name
            field_type: text
            visible: true
            editable: true
            required: true

          - id: last_name
            name: Last Name
            field_type: text
            visible: true
            editable: true
            required: true

          - id: phone
            name: Phone
            field_type: tel
            visible: false
            editable: true
            validations:
              - type: phone

          - id: role_name
            name: Role
            field_type: text
            visible: true
            editable: false

          - id: status
            name: Status
            field_type: select
            visible: true
            editable: true
            config:
              options:
                - value: active
                  label: Active
                  color: green
                - value: inactive
                  label: Inactive
                  color: gray
                - value: suspended
                  label: Suspended
                  color: red
            default_value: active

          - id: email_verified
            name: Email Verified
            field_type: toggle
            visible: true
            editable: true
            default_value: false

          - id: last_login_at
            name: Last Login
            field_type: datetime
            visible: true
            editable: false

          - id: created_at
            name: Joined
            field_type: datetime
            visible: true
            editable: false

      # Create User
      - id: create_user
        name: Create User
        action_type: form
        data_source: users_db
        required_scopes: ["users:write"]
        config:
          form_mode: create
        query: |
          INSERT INTO users (
            username, email, password_hash, first_name, last_name,
            phone, role_id, status, email_verified
          ) VALUES (
            $1, $2, crypt($3, gen_salt('bf')), $4, $5, $6, $7, $8, $9
          ) RETURNING *
        fields:
          - id: username
            name: Username
            field_type: text
            required: true
            placeholder: "johndoe"
            validations:
              - type: min_length
                value: 3
              - type: pattern
                value: "^[a-z0-9_]+$"

          - id: email
            name: Email Address
            field_type: email
            required: true
            placeholder: "john@example.com"

          - id: password
            name: Password
            field_type: password
            required: true
            help_text: "Minimum 8 characters, must include uppercase, number, and special character"
            validations:
              - type: min_length
                value: 8

          - id: password_confirm
            name: Confirm Password
            field_type: password
            required: true

          - id: first_name
            name: First Name
            field_type: text
            required: true

          - id: last_name
            name: Last Name
            field_type: text
            required: true

          - id: phone
            name: Phone Number
            field_type: tel
            required: false
            placeholder: "+1 (555) 123-4567"

          - id: role_id
            name: Role
            field_type: select
            required: true
            config:
              options:
                - value: 1
                  label: User
                - value: 2
                  label: Moderator
                - value: 3
                  label: Admin
            default_value: 1

          - id: status
            name: Initial Status
            field_type: select
            required: true
            config:
              options:
                - value: active
                  label: Active
                - value: inactive
                  label: Inactive
            default_value: active

          - id: email_verified
            name: Mark Email as Verified
            field_type: toggle
            default_value: false
            help_text: "Enable if verifying email manually"

      # Update User
      - id: update_user
        name: Update User
        action_type: form
        data_source: users_db
        required_scopes: ["users:write"]
        config:
          form_mode: update
        query: |
          UPDATE users SET
            username = $1,
            email = $2,
            first_name = $3,
            last_name = $4,
            phone = $5,
            role_id = $6,
            status = $7,
            email_verified = $8,
            updated_at = NOW()
          WHERE id = $9
          RETURNING *

      # Delete User
      - id: delete_user
        name: Delete User
        action_type: delete
        data_source: users_db
        required_scopes: ["users:delete"]
        query: |
          UPDATE users SET deleted_at = NOW() WHERE id = $1

  # Roles Section
  - id: roles
    name: Roles & Permissions
    icon: "fa-shield-alt"
    actions:
      - id: list_roles
        name: Role List
        action_type: list
        data_source: users_db
        query: "SELECT * FROM roles ORDER BY name"
        fields:
          - id: name
            name: Role Name
            field_type: text
            required: true

          - id: description
            name: Description
            field_type: textarea

          - id: permissions
            name: Permissions
            field_type: tags
            help_text: "Comma-separated permissions"

  # Activity Log
  - id: activity
    name: Activity Log
    icon: "fa-history"
    actions:
      - id: list_activity
        name: Recent Activity
        action_type: list
        data_source: users_db
        query: |
          SELECT
            a.id,
            a.user_id,
            u.username,
            a.action,
            a.resource_type,
            a.resource_id,
            a.ip_address,
            a.user_agent,
            a.created_at
          FROM activity_logs a
          LEFT JOIN users u ON a.user_id = u.id
          ORDER BY a.created_at DESC
          LIMIT 1000
        config:
          pagination:
            enabled: true
            page_size: 50
        fields:
          - id: username
            name: User
            field_type: text

          - id: action
            name: Action
            field_type: text

          - id: resource_type
            name: Resource
            field_type: text

          - id: ip_address
            name: IP Address
            field_type: text

          - id: created_at
            name: Timestamp
            field_type: datetime
```

### Database Schema

```sql
-- Users table
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(30) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    phone VARCHAR(20),
    avatar_url TEXT,
    role_id INTEGER REFERENCES roles(id),
    status VARCHAR(20) DEFAULT 'active',
    email_verified BOOLEAN DEFAULT false,
    last_login_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    deleted_at TIMESTAMP
);

-- Roles table
CREATE TABLE roles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT,
    permissions TEXT[],
    created_at TIMESTAMP DEFAULT NOW()
);

-- Activity logs
CREATE TABLE activity_logs (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    action VARCHAR(100),
    resource_type VARCHAR(100),
    resource_id VARCHAR(100),
    ip_address VARCHAR(45),
    user_agent TEXT,
    metadata JSONB,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_status ON users(status);
CREATE INDEX idx_activity_user ON activity_logs(user_id);
CREATE INDEX idx_activity_created ON activity_logs(created_at);
```

---

## E-Commerce Platform

Complete e-commerce management with products, orders, and customers.

### Configuration: `config/backoffices/ecommerce.yaml`

```yaml
id: "ecommerce"
name: "E-Commerce Admin"
description: "Manage products, orders, and customers"

data_sources:
  shop_db:
    type: database
    connection_string: "postgresql://localhost:5432/shop"
    db_type: postgres

  product_images:
    type: s3
    bucket: "product-images"
    region: "us-east-1"

sections:
  # Products
  - id: products
    name: Products
    icon: "fa-box"
    actions:
      - id: list_products
        name: Product Catalog
        action_type: list
        data_source: shop_db
        query: |
          SELECT
            p.*,
            c.name as category_name,
            b.name as brand_name,
            (SELECT COUNT(*) FROM order_items oi WHERE oi.product_id = p.id) as total_sold
          FROM products p
          LEFT JOIN categories c ON p.category_id = c.id
          LEFT JOIN brands b ON p.brand_id = b.id
          WHERE p.deleted_at IS NULL
          ORDER BY p.created_at DESC
        config:
          pagination:
            enabled: true
            page_size: 25
          filters:
            - field: category_id
              label: Category
              type: select
            - field: brand_id
              label: Brand
              type: select
            - field: status
              label: Status
              type: select
              options:
                - draft
                - published
                - out_of_stock
                - discontinued
            - field: price
              label: Price Range
              type: number_range
              min: 0
              max: 10000
            - field: stock
              label: Stock Level
              type: number_range
        fields:
          - id: id
            name: ID
            field_type: text
            visible: true
            editable: false

          - id: sku
            name: SKU
            field_type: text
            visible: true
            editable: true
            required: true
            validations:
              - type: pattern
                value: "^[A-Z0-9-]+$"
                message: "Uppercase letters, numbers, and hyphens only"

          - id: name
            name: Product Name
            field_type: text
            visible: true
            editable: true
            required: true
            validations:
              - type: min_length
                value: 3
              - type: max_length
                value: 200

          - id: description
            name: Description
            field_type: richtext
            visible: false
            required: true
            config:
              toolbar:
                - bold
                - italic
                - list
                - link

          - id: short_description
            name: Short Description
            field_type: textarea
            visible: false
            config:
              rows: 3
              max_length: 500

          - id: price
            name: Price
            field_type: currency
            visible: true
            editable: true
            required: true
            config:
              currency: USD
            validations:
              - type: min_value
                value: 0

          - id: compare_at_price
            name: Compare Price
            field_type: currency
            visible: false
            config:
              currency: USD
            help_text: "Original price for showing discounts"

          - id: cost
            name: Cost
            field_type: currency
            visible: false
            config:
              currency: USD
            help_text: "Your cost (not shown to customers)"

          - id: stock
            name: Stock
            field_type: number
            visible: true
            editable: true
            required: true
            validations:
              - type: integer
              - type: min_value
                value: 0

          - id: low_stock_threshold
            name: Low Stock Alert
            field_type: number
            visible: false
            default_value: 10
            help_text: "Alert when stock falls below this number"

          - id: category_name
            name: Category
            field_type: text
            visible: true
            editable: false

          - id: brand_name
            name: Brand
            field_type: text
            visible: true
            editable: false

          - id: images
            name: Images
            field_type: image
            visible: false
            config:
              multiple: true
              max_files: 10
              storage: product_images
              resize:
                width: 1200
                height: 1200

          - id: weight
            name: Weight (kg)
            field_type: number
            visible: false
            config:
              step: 0.01

          - id: dimensions
            name: Dimensions (L×W×H cm)
            field_type: text
            visible: false
            placeholder: "30x20x10"

          - id: tags
            name: Tags
            field_type: tags
            visible: false
            config:
              allow_custom: true
              suggestions:
                - "new-arrival"
                - "bestseller"
                - "sale"
                - "featured"

          - id: seo_title
            name: SEO Title
            field_type: text
            visible: false
            config:
              max_length: 60

          - id: seo_description
            name: SEO Description
            field_type: textarea
            visible: false
            config:
              max_length: 160

          - id: status
            name: Status
            field_type: select
            visible: true
            editable: true
            required: true
            config:
              options:
                - value: draft
                  label: Draft
                - value: published
                  label: Published
                - value: out_of_stock
                  label: Out of Stock
                - value: discontinued
                  label: Discontinued
            default_value: draft

          - id: featured
            name: Featured
            field_type: toggle
            visible: true
            editable: true
            default_value: false

          - id: total_sold
            name: Total Sold
            field_type: number
            visible: true
            editable: false

          - id: created_at
            name: Created
            field_type: datetime
            visible: true
            editable: false

  # Orders
  - id: orders
    name: Orders
    icon: "fa-shopping-cart"
    actions:
      - id: list_orders
        name: Order Management
        action_type: list
        data_source: shop_db
        query: |
          SELECT
            o.*,
            u.email as customer_email,
            u.first_name || ' ' || u.last_name as customer_name,
            (SELECT COUNT(*) FROM order_items WHERE order_id = o.id) as item_count
          FROM orders o
          LEFT JOIN users u ON o.user_id = u.id
          ORDER BY o.created_at DESC
        config:
          pagination:
            enabled: true
            page_size: 50
          filters:
            - field: status
              label: Order Status
              type: select
              options:
                - pending
                - processing
                - shipped
                - delivered
                - cancelled
            - field: payment_status
              label: Payment Status
              type: select
              options:
                - pending
                - paid
                - failed
                - refunded
            - field: created_at
              label: Order Date
              type: date_range
        fields:
          - id: order_number
            name: Order #
            field_type: text
            visible: true
            editable: false

          - id: customer_name
            name: Customer
            field_type: text
            visible: true
            editable: false

          - id: customer_email
            name: Email
            field_type: email
            visible: true
            editable: false

          - id: item_count
            name: Items
            field_type: number
            visible: true
            editable: false

          - id: subtotal
            name: Subtotal
            field_type: currency
            visible: true
            editable: false
            config:
              currency: USD

          - id: tax
            name: Tax
            field_type: currency
            visible: false
            config:
              currency: USD

          - id: shipping
            name: Shipping
            field_type: currency
            visible: false
            config:
              currency: USD

          - id: total
            name: Total
            field_type: currency
            visible: true
            editable: false
            config:
              currency: USD

          - id: status
            name: Status
            field_type: select
            visible: true
            editable: true
            config:
              options:
                - value: pending
                  label: Pending
                  color: yellow
                - value: processing
                  label: Processing
                  color: blue
                - value: shipped
                  label: Shipped
                  color: purple
                - value: delivered
                  label: Delivered
                  color: green
                - value: cancelled
                  label: Cancelled
                  color: red

          - id: payment_status
            name: Payment
            field_type: select
            visible: true
            editable: true
            config:
              options:
                - value: pending
                  label: Pending
                - value: paid
                  label: Paid
                - value: failed
                  label: Failed
                - value: refunded
                  label: Refunded

          - id: shipping_address
            name: Shipping Address
            field_type: textarea
            visible: false
            editable: false

          - id: tracking_number
            name: Tracking #
            field_type: text
            visible: false
            editable: true

          - id: notes
            name: Order Notes
            field_type: textarea
            visible: false
            editable: true

          - id: created_at
            name: Order Date
            field_type: datetime
            visible: true
            editable: false

  # Customers
  - id: customers
    name: Customers
    icon: "fa-users"
    actions:
      - id: list_customers
        name: Customer Directory
        action_type: list
        data_source: shop_db
        query: |
          SELECT
            u.*,
            COUNT(DISTINCT o.id) as total_orders,
            COALESCE(SUM(o.total), 0) as lifetime_value
          FROM users u
          LEFT JOIN orders o ON u.id = o.user_id
          WHERE u.role = 'customer'
          GROUP BY u.id
          ORDER BY lifetime_value DESC
        fields:
          - id: email
            name: Email
            field_type: email

          - id: first_name
            name: First Name
            field_type: text

          - id: last_name
            name: Last Name
            field_type: text

          - id: phone
            name: Phone
            field_type: tel

          - id: total_orders
            name: Orders
            field_type: number
            editable: false

          - id: lifetime_value
            name: Lifetime Value
            field_type: currency
            editable: false
            config:
              currency: USD

          - id: created_at
            name: Customer Since
            field_type: datetime
            editable: false
```

This is just the first part. Would you like me to continue with the remaining examples (CMS, Project Management, etc.)?
