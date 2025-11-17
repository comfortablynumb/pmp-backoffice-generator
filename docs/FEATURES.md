# PMP Backoffice Generator - Complete Feature Documentation

## Table of Contents

1. [Overview](#overview)
2. [Field Types (30+)](#field-types)
3. [Validation Types (24+)](#validation-types)
4. [Data Sources (10+)](#data-sources)
5. [UI Features](#ui-features)
6. [Advanced Features](#advanced-features)
7. [Keyboard Shortcuts](#keyboard-shortcuts)
8. [API Documentation](#api-documentation)
9. [Configuration Examples](#configuration-examples)
10. [Best Practices](#best-practices)

---

## Overview

The PMP Backoffice Generator is a powerful, dynamic admin interface generator that creates full-featured CRUD applications from YAML configuration files. It supports 30+ field types, 24+ validation types, 10+ data sources, and includes advanced features like bulk operations, inline editing, and real-time search.

---

## Field Types (30+)

### Basic Text Fields

#### 1. Text Field
Single-line text input for general text data.

```yaml
fields:
  - id: username
    name: Username
    field_type: text
    config:
      min_length: 3
      max_length: 50
    required: true
    placeholder: "Enter username"
    help_text: "Your unique identifier"
```

**Example 2: Optional Description**
```yaml
  - id: bio
    name: Biography
    field_type: text
    config:
      max_length: 500
    required: false
    placeholder: "Tell us about yourself"
```

**Example 3: Read-only Display**
```yaml
  - id: created_by
    name: Created By
    field_type: text
    editable: false
    visible: true
```

#### 2. Textarea Field
Multi-line text input for longer content.

```yaml
fields:
  - id: description
    name: Product Description
    field_type: textarea
    config:
      rows: 5
      max_length: 2000
    required: true
    placeholder: "Describe your product in detail"
```

**Example 2: Comments Field**
```yaml
  - id: notes
    name: Internal Notes
    field_type: textarea
    config:
      rows: 10
    required: false
    help_text: "Private notes visible only to staff"
```

**Example 3: Address Field**
```yaml
  - id: address
    name: Shipping Address
    field_type: textarea
    config:
      rows: 3
    required: true
    placeholder: "Street, City, State, ZIP"
```

### Numeric Fields

#### 3. Number Field
Numeric input with validation.

```yaml
fields:
  - id: quantity
    name: Quantity
    field_type: number
    config:
      min: 0
      max: 999999
      step: 1
    required: true
    default_value: 1
```

**Example 2: Decimal Numbers**
```yaml
  - id: weight
    name: Weight (kg)
    field_type: number
    config:
      min: 0.01
      max: 1000
      step: 0.01
    required: true
```

**Example 3: Rating**
```yaml
  - id: priority
    name: Priority Level
    field_type: number
    config:
      min: 1
      max: 10
      step: 1
    default_value: 5
```

#### 4. Currency Field
Monetary values with proper formatting.

```yaml
fields:
  - id: price
    name: Price
    field_type: currency
    config:
      currency: "USD"
      min: 0
      max: 999999.99
    required: true
    validations:
      - type: positive
```

**Example 2: Multiple Currencies**
```yaml
  - id: budget
    name: Budget
    field_type: currency
    config:
      currency: "EUR"
      allow_negative: false
    required: true
```

**Example 3: Discount Amount**
```yaml
  - id: discount
    name: Discount Amount
    field_type: currency
    config:
      currency: "USD"
      min: 0
      max: 1000
```

#### 5. Percentage Field
Percentage values with validation.

```yaml
fields:
  - id: tax_rate
    name: Tax Rate
    field_type: percentage
    config:
      min: 0
      max: 100
      decimals: 2
    required: true
```

**Example 2: Commission**
```yaml
  - id: commission
    name: Sales Commission
    field_type: percentage
    config:
      min: 0
      max: 50
      decimals: 1
    default_value: 10
```

**Example 3: Discount Percentage**
```yaml
  - id: discount_percent
    name: Discount %
    field_type: percentage
    config:
      min: 0
      max: 100
      step: 5
```

### Contact & Communication Fields

#### 6. Email Field
Email input with validation.

```yaml
fields:
  - id: email
    name: Email Address
    field_type: email
    required: true
    placeholder: "user@example.com"
    validations:
      - type: email
```

**Example 2: Optional CC Email**
```yaml
  - id: cc_email
    name: CC Email
    field_type: email
    required: false
    help_text: "Copy notifications to this address"
```

**Example 3: Multiple Emails**
```yaml
  - id: contact_emails
    name: Contact Emails
    field_type: tags
    config:
      validation: email
    help_text: "Enter multiple email addresses"
```

#### 7. Phone/Tel Field
Phone number input with formatting.

```yaml
fields:
  - id: phone
    name: Phone Number
    field_type: tel
    config:
      format: "US"
    required: true
    placeholder: "(555) 123-4567"
    validations:
      - type: phone
```

**Example 2: International Phone**
```yaml
  - id: mobile
    name: Mobile Number
    field_type: tel
    config:
      international: true
    placeholder: "+1 555 123 4567"
```

**Example 3: Emergency Contact**
```yaml
  - id: emergency_phone
    name: Emergency Contact
    field_type: tel
    required: true
    help_text: "Available 24/7"
```

#### 8. URL Field
URL input with validation.

```yaml
fields:
  - id: website
    name: Website
    field_type: url
    required: false
    placeholder: "https://example.com"
    validations:
      - type: url
```

**Example 2: Social Media Link**
```yaml
  - id: linkedin
    name: LinkedIn Profile
    field_type: url
    placeholder: "https://linkedin.com/in/username"
```

**Example 3: Portfolio URL**
```yaml
  - id: portfolio
    name: Portfolio URL
    field_type: url
    help_text: "Link to your work samples"
```

### Date & Time Fields

#### 9. Date Field
Date picker for selecting dates.

```yaml
fields:
  - id: birth_date
    name: Date of Birth
    field_type: date
    required: true
    config:
      min: "1900-01-01"
      max: "2010-12-31"
```

**Example 2: Expiration Date**
```yaml
  - id: expiry_date
    name: Expiration Date
    field_type: date
    config:
      min_today: true
    required: true
```

**Example 3: Start Date**
```yaml
  - id: start_date
    name: Project Start Date
    field_type: date
    default_value: "today"
```

#### 10. DateTime Field
Date and time picker.

```yaml
fields:
  - id: appointment
    name: Appointment Time
    field_type: datetime
    required: true
    config:
      min_today: true
      step: 15  # 15-minute intervals
```

**Example 2: Event Timestamp**
```yaml
  - id: event_time
    name: Event Date & Time
    field_type: datetime
    config:
      timezone: "UTC"
    required: true
```

**Example 3: Last Login**
```yaml
  - id: last_login
    name: Last Login
    field_type: datetime
    editable: false
    visible: true
```

#### 11. Time Field
Time picker for time-only values.

```yaml
fields:
  - id: opening_time
    name: Opening Time
    field_type: time
    required: true
    config:
      format: "24h"
```

**Example 2: Closing Time**
```yaml
  - id: closing_time
    name: Closing Time
    field_type: time
    config:
      format: "12h"
    default_value: "17:00"
```

#### 12. DateTime Range Field
Select a range of dates/times.

```yaml
fields:
  - id: campaign_period
    name: Campaign Period
    field_type: datetime_range
    required: true
    config:
      allow_same_day: false
```

**Example 2: Vacation Dates**
```yaml
  - id: vacation
    name: Vacation Dates
    field_type: datetime_range
    config:
      min_duration_days: 1
      max_duration_days: 30
```

#### 13. Month Field
Month picker.

```yaml
fields:
  - id: reporting_month
    name: Reporting Month
    field_type: month
    required: true
```

#### 14. Week Field
Week picker.

```yaml
fields:
  - id: week
    name: Week Number
    field_type: week
    required: true
```

### Selection Fields

#### 15. Select Field
Dropdown selection from predefined options.

```yaml
fields:
  - id: status
    name: Status
    field_type: select
    required: true
    config:
      options:
        - value: draft
          label: Draft
        - value: published
          label: Published
        - value: archived
          label: Archived
    default_value: draft
```

**Example 2: Priority Select**
```yaml
  - id: priority
    name: Priority
    field_type: select
    config:
      options:
        - value: low
          label: "Low Priority"
          color: "green"
        - value: medium
          label: "Medium Priority"
          color: "yellow"
        - value: high
          label: "High Priority"
          color: "red"
```

**Example 3: Country Select**
```yaml
  - id: country
    name: Country
    field_type: select
    config:
      options:
        - value: US
          label: "United States"
        - value: UK
          label: "United Kingdom"
        - value: CA
          label: "Canada"
      searchable: true
```

#### 16. Radio Field
Radio button group.

```yaml
fields:
  - id: payment_method
    name: Payment Method
    field_type: radio
    required: true
    config:
      options:
        - value: card
          label: "Credit Card"
        - value: paypal
          label: "PayPal"
        - value: bank
          label: "Bank Transfer"
```

**Example 2: Subscription Type**
```yaml
  - id: subscription
    name: Subscription Plan
    field_type: radio
    config:
      options:
        - value: free
          label: "Free (0$/month)"
        - value: pro
          label: "Pro (29$/month)"
        - value: enterprise
          label: "Enterprise (99$/month)"
```

#### 17. Checkbox Field
Multiple checkbox selection.

```yaml
fields:
  - id: features
    name: Features
    field_type: checkbox
    required: false
    config:
      options:
        - value: ssl
          label: "SSL Certificate"
        - value: backup
          label: "Daily Backup"
        - value: cdn
          label: "CDN"
```

**Example 2: Permissions**
```yaml
  - id: permissions
    name: User Permissions
    field_type: checkbox
    config:
      options:
        - value: read
          label: "Read"
        - value: write
          label: "Write"
        - value: delete
          label: "Delete"
        - value: admin
          label: "Admin"
```

#### 18. Boolean/Toggle Field
Simple on/off toggle.

```yaml
fields:
  - id: active
    name: Active Status
    field_type: toggle
    default_value: true
```

**Example 2: Email Notifications**
```yaml
  - id: email_notifications
    name: Email Notifications
    field_type: boolean
    default_value: false
    help_text: "Receive email updates"
```

**Example 3: Public Visibility**
```yaml
  - id: is_public
    name: Public
    field_type: toggle
    required: true
```

#### 19. Tags Field
Tag input with autocomplete.

```yaml
fields:
  - id: tags
    name: Tags
    field_type: tags
    config:
      allow_custom: true
      suggestions:
        - "important"
        - "urgent"
        - "follow-up"
      max_tags: 10
```

**Example 2: Skills**
```yaml
  - id: skills
    name: Skills
    field_type: tags
    config:
      allow_custom: true
      suggestions:
        - "JavaScript"
        - "Python"
        - "React"
        - "Node.js"
```

#### 20. Multiselect Field
Multiple selection dropdown.

```yaml
fields:
  - id: categories
    name: Categories
    field_type: multiselect
    config:
      options:
        - value: tech
          label: "Technology"
        - value: business
          label: "Business"
        - value: lifestyle
          label: "Lifestyle"
      max_selections: 5
```

### Rich Content Fields

#### 21. Rich Text Field
WYSIWYG editor for formatted text.

```yaml
fields:
  - id: content
    name: Article Content
    field_type: richtext
    required: true
    config:
      toolbar:
        - bold
        - italic
        - underline
        - link
        - image
        - list
      max_length: 50000
```

**Example 2: Product Description**
```yaml
  - id: product_desc
    name: Product Description
    field_type: richtext
    config:
      toolbar:
        - bold
        - italic
        - list
      simple: true
```

#### 22. Markdown Field
Markdown editor with preview.

```yaml
fields:
  - id: readme
    name: README
    field_type: markdown
    required: true
    config:
      preview: true
      syntax_highlighting: true
```

**Example 2: Blog Post**
```yaml
  - id: post_content
    name: Post Content
    field_type: markdown
    config:
      preview: true
      auto_save: true
```

#### 23. Code Field
Code editor with syntax highlighting.

```yaml
fields:
  - id: script
    name: Custom Script
    field_type: code
    config:
      language: javascript
      theme: monokai
      line_numbers: true
```

**Example 2: CSS Styles**
```yaml
  - id: custom_css
    name: Custom CSS
    field_type: code
    config:
      language: css
      theme: github
```

**Example 3: SQL Query**
```yaml
  - id: query
    name: SQL Query
    field_type: code
    config:
      language: sql
      read_only: false
```

#### 24. JSON Field
JSON editor with validation.

```yaml
fields:
  - id: metadata
    name: Metadata
    field_type: json
    required: false
    config:
      validate: true
      format: true
```

**Example 2: API Response**
```yaml
  - id: api_config
    name: API Configuration
    field_type: json
    config:
      schema_validation: true
      default_value: "{}"
```

#### 25. HTML Field
HTML editor.

```yaml
fields:
  - id: template
    name: Email Template
    field_type: html
    config:
      sanitize: true
      preview: true
```

### File & Media Fields

#### 26. File Field
File upload.

```yaml
fields:
  - id: document
    name: Document
    field_type: file
    required: false
    config:
      allowed_types:
        - pdf
        - doc
        - docx
      max_size_mb: 10
      storage: s3
```

**Example 2: Multiple Files**
```yaml
  - id: attachments
    name: Attachments
    field_type: file
    config:
      multiple: true
      max_files: 5
      allowed_types:
        - pdf
        - jpg
        - png
```

#### 27. Image Field
Image upload with preview.

```yaml
fields:
  - id: avatar
    name: Profile Picture
    field_type: image
    config:
      max_size_mb: 5
      allowed_types:
        - jpg
        - png
        - webp
      resize:
        width: 800
        height: 800
      preview: true
```

**Example 2: Product Images**
```yaml
  - id: product_images
    name: Product Images
    field_type: image
    config:
      multiple: true
      max_files: 10
      thumbnail: true
      crop: true
```

#### 28. Video Field
Video upload.

```yaml
fields:
  - id: promo_video
    name: Promotional Video
    field_type: video
    config:
      max_size_mb: 100
      allowed_types:
        - mp4
        - webm
      thumbnail: true
```

#### 29. Audio Field
Audio upload.

```yaml
fields:
  - id: podcast
    name: Podcast Episode
    field_type: audio
    config:
      max_size_mb: 50
      allowed_types:
        - mp3
        - wav
        - ogg
```

### Specialized Fields

#### 30. Signature Field
Digital signature pad.

```yaml
fields:
  - id: signature
    name: Signature
    field_type: signature
    required: true
    config:
      width: 400
      height: 150
      pen_color: "#000000"
```

**Example 2: Agreement Signature**
```yaml
  - id: agreement_signature
    name: I agree to the terms
    field_type: signature
    config:
      background_color: "#f0f0f0"
      save_format: png
```

#### 31. Color Field
Color picker.

```yaml
fields:
  - id: brand_color
    name: Brand Color
    field_type: color
    required: true
    default_value: "#0066cc"
```

**Example 2: Theme Colors**
```yaml
  - id: primary_color
    name: Primary Color
    field_type: color
    config:
      format: hex
      alpha: false
```

#### 32. Rating Field
Star rating input.

```yaml
fields:
  - id: rating
    name: Product Rating
    field_type: rating
    config:
      max_stars: 5
      allow_half: true
    validations:
      - type: min_value
        value: 1
      - type: max_value
        value: 5
```

**Example 2: Satisfaction Score**
```yaml
  - id: satisfaction
    name: Satisfaction Score
    field_type: rating
    config:
      max_stars: 10
      icon: heart
```

#### 33. Slider Field
Range slider.

```yaml
fields:
  - id: volume
    name: Volume
    field_type: slider
    config:
      min: 0
      max: 100
      step: 5
      show_value: true
    default_value: 50
```

**Example 2: Price Range**
```yaml
  - id: price_range
    name: Price Range
    field_type: slider
    config:
      min: 0
      max: 1000
      step: 10
      range: true
```

#### 34. Geolocation Field
GPS coordinates.

```yaml
fields:
  - id: location
    name: Location
    field_type: geolocation
    config:
      map_provider: google
      allow_current_location: true
```

**Example 2: Delivery Address**
```yaml
  - id: delivery_coords
    name: Delivery Coordinates
    field_type: geolocation
    config:
      show_map: true
      zoom: 15
```

#### 35. Password Field
Password input (masked).

```yaml
fields:
  - id: password
    name: Password
    field_type: password
    required: true
    config:
      min_length: 8
      require_uppercase: true
      require_number: true
      require_special: true
    validations:
      - type: min_length
        value: 8
```

**Example 2: Confirm Password**
```yaml
  - id: password_confirm
    name: Confirm Password
    field_type: password
    config:
      match_field: password
```

---

## Validation Types (24+)

### String Validations

#### 1. Min Length
```yaml
validations:
  - type: min_length
    value: 3
    message: "Must be at least 3 characters"
```

#### 2. Max Length
```yaml
validations:
  - type: max_length
    value: 100
    message: "Cannot exceed 100 characters"
```

#### 3. Pattern (Regex)
```yaml
validations:
  - type: pattern
    value: "^[A-Z0-9-]+$"
    message: "Only uppercase letters, numbers, and hyphens"
```

#### 4. Email Format
```yaml
validations:
  - type: email
    message: "Invalid email format"
```

#### 5. URL Format
```yaml
validations:
  - type: url
    message: "Must be a valid URL"
```

#### 6. UUID Format
```yaml
validations:
  - type: uuid
    message: "Must be a valid UUID"
```

#### 7. Slug Format
```yaml
validations:
  - type: slug
    message: "Must be a valid URL slug (lowercase, hyphens only)"
```

#### 8. Alpha (Letters Only)
```yaml
validations:
  - type: alpha
    message: "Only alphabetic characters allowed"
```

#### 9. Alphanumeric
```yaml
validations:
  - type: alphanumeric
    message: "Only letters and numbers allowed"
```

#### 10. Lowercase
```yaml
validations:
  - type: lowercase
    message: "Must be lowercase"
```

#### 11. Uppercase
```yaml
validations:
  - type: uppercase
    message: "Must be uppercase"
```

### Numeric Validations

#### 12. Min Value
```yaml
validations:
  - type: min_value
    value: 0
    message: "Must be at least 0"
```

#### 13. Max Value
```yaml
validations:
  - type: max_value
    value: 100
    message: "Cannot exceed 100"
```

#### 14. Integer
```yaml
validations:
  - type: integer
    message: "Must be a whole number"
```

#### 15. Positive
```yaml
validations:
  - type: positive
    message: "Must be positive"
```

#### 16. Negative
```yaml
validations:
  - type: negative
    message: "Must be negative"
```

#### 17. Even
```yaml
validations:
  - type: even
    message: "Must be an even number"
```

#### 18. Odd
```yaml
validations:
  - type: odd
    message: "Must be an odd number"
```

### Financial Validations

#### 19. Credit Card
```yaml
validations:
  - type: credit_card
    message: "Invalid credit card number"
```

#### 20. IBAN
```yaml
validations:
  - type: iban
    message: "Invalid IBAN"
```

#### 21. ISBN
```yaml
validations:
  - type: isbn
    message: "Invalid ISBN"
```

#### 22. ISSN
```yaml
validations:
  - type: issn
    message: "Invalid ISSN"
```

### Network Validations

#### 23. IPv4
```yaml
validations:
  - type: ipv4
    message: "Invalid IPv4 address"
```

#### 24. IPv6
```yaml
validations:
  - type: ipv6
    message: "Invalid IPv6 address"
```

#### 25. MAC Address
```yaml
validations:
  - type: mac_address
    message: "Invalid MAC address"
```

#### 26. Port Number
```yaml
validations:
  - type: port
    message: "Invalid port number (1-65535)"
```

### Geographic Validations

#### 27. Latitude
```yaml
validations:
  - type: latitude
    message: "Invalid latitude (-90 to 90)"
```

#### 28. Longitude
```yaml
validations:
  - type: longitude
    message: "Invalid longitude (-180 to 180)"
```

### Other Validations

#### 29. Phone Number
```yaml
validations:
  - type: phone
    message: "Invalid phone number"
```

#### 30. JSON
```yaml
validations:
  - type: json
    message: "Must be valid JSON"
```

#### 31. Base64
```yaml
validations:
  - type: base64
    message: "Must be valid Base64"
```

### Combined Validation Example

```yaml
fields:
  - id: username
    name: Username
    field_type: text
    required: true
    validations:
      - type: min_length
        value: 3
      - type: max_length
        value: 20
      - type: pattern
        value: "^[a-z0-9_]+$"
        message: "Only lowercase letters, numbers, and underscores"
      - type: lowercase
```

---

## Data Sources (10+)

### 1. Database

**PostgreSQL Example:**
```yaml
data_sources:
  main_db:
    type: database
    connection_string: "postgresql://user:password@localhost:5432/mydb"
    db_type: postgres
```

**MySQL Example:**
```yaml
data_sources:
  mysql_db:
    type: database
    connection_string: "mysql://user:password@localhost:3306/mydb"
    db_type: mysql
```

**SQLite Example:**
```yaml
data_sources:
  local_db:
    type: database
    connection_string: "sqlite://data/local.db"
    db_type: sqlite
```

**Usage in Action:**
```yaml
actions:
  - id: list_users
    name: List Users
    action_type: list
    data_source: main_db
    query: |
      SELECT id, username, email, created_at
      FROM users
      WHERE deleted_at IS NULL
      ORDER BY created_at DESC
```

### 2. REST API

**Basic API:**
```yaml
data_sources:
  external_api:
    type: api
    base_url: "https://api.example.com"
    headers:
      Authorization: "Bearer ${API_TOKEN}"
      Content-Type: "application/json"
```

**API with Custom Auth:**
```yaml
data_sources:
  secure_api:
    type: api
    base_url: "https://secure-api.example.com"
    auth:
      type: bearer
      token: "${SECRET_TOKEN}"
```

**Usage in Action:**
```yaml
actions:
  - id: get_products
    name: Get Products
    action_type: list
    data_source: external_api
    endpoint: "/api/v1/products"
    query: "limit=50&sort=created_at"
```

### 3. GraphQL

**Basic GraphQL:**
```yaml
data_sources:
  graphql_api:
    type: graphql
    endpoint: "https://api.example.com/graphql"
    headers:
      Authorization: "Bearer ${GRAPHQL_TOKEN}"
```

**Usage in Action:**
```yaml
actions:
  - id: get_posts
    name: Get Posts
    action_type: list
    data_source: graphql_api
    query: |
      query GetPosts($limit: Int!) {
        posts(limit: $limit) {
          id
          title
          author {
            name
            email
          }
          createdAt
        }
      }
```

### 4. MongoDB

**MongoDB Connection:**
```yaml
data_sources:
  mongo_db:
    type: mongodb
    connection_string: "mongodb://localhost:27017"
    database: "myapp"
```

**MongoDB with Authentication:**
```yaml
data_sources:
  mongo_cloud:
    type: mongodb
    connection_string: "mongodb+srv://user:pass@cluster.mongodb.net"
    database: "production"
```

**Usage in Action:**
```yaml
actions:
  - id: list_documents
    name: List Documents
    action_type: list
    data_source: mongo_db
    query: |
      {
        "collection": "users",
        "filter": { "active": true },
        "sort": { "created_at": -1 }
      }
```

### 5. Redis

**Redis Connection:**
```yaml
data_sources:
  cache:
    type: redis
    connection_string: "redis://localhost:6379"
    database: 0
```

**Redis with Password:**
```yaml
data_sources:
  redis_secure:
    type: redis
    connection_string: "redis://:password@localhost:6379"
    database: 1
```

### 6. Elasticsearch

**Elasticsearch Connection:**
```yaml
data_sources:
  search_engine:
    type: elasticsearch
    url: "http://localhost:9200"
    index: "products"
```

**Elasticsearch with Auth:**
```yaml
data_sources:
  elastic_cloud:
    type: elasticsearch
    url: "https://elasticsearch.example.com"
    username: "elastic"
    password: "${ELASTIC_PASSWORD}"
```

**Usage in Action:**
```yaml
actions:
  - id: search_products
    name: Search Products
    action_type: list
    data_source: search_engine
    query: |
      {
        "query": {
          "match_all": {}
        },
        "size": 100
      }
```

### 7. gRPC

**gRPC Connection:**
```yaml
data_sources:
  grpc_service:
    type: grpc
    endpoint: "localhost:50051"
    proto_file: "service.proto"
```

**Secure gRPC:**
```yaml
data_sources:
  grpc_secure:
    type: grpc
    endpoint: "api.example.com:443"
    tls: true
    cert_file: "/path/to/cert.pem"
```

### 8. Kafka

**Kafka Connection:**
```yaml
data_sources:
  event_stream:
    type: kafka
    brokers:
      - "localhost:9092"
      - "localhost:9093"
    topic: "events"
```

**Kafka with SASL:**
```yaml
data_sources:
  kafka_secure:
    type: kafka
    brokers:
      - "kafka1.example.com:9092"
      - "kafka2.example.com:9092"
    sasl:
      mechanism: "PLAIN"
      username: "user"
      password: "${KAFKA_PASSWORD}"
```

### 9. S3 (Object Storage)

**AWS S3:**
```yaml
data_sources:
  file_storage:
    type: s3
    bucket: "my-bucket"
    region: "us-east-1"
    access_key: "${AWS_ACCESS_KEY}"
    secret_key: "${AWS_SECRET_KEY}"
```

**MinIO (S3-compatible):**
```yaml
data_sources:
  minio:
    type: s3
    bucket: "uploads"
    endpoint: "http://localhost:9000"
    access_key: "minioadmin"
    secret_key: "minioadmin"
```

### 10. Firebase

**Firebase Connection:**
```yaml
data_sources:
  firebase_db:
    type: firebase
    project_id: "my-project-123"
    credentials_path: "/path/to/serviceAccount.json"
```

**Usage in Action:**
```yaml
actions:
  - id: get_users
    name: Get Users
    action_type: list
    data_source: firebase_db
    query: |
      {
        "collection": "users",
        "orderBy": "createdAt",
        "limit": 100
      }
```

### 11. Supabase

**Supabase Connection:**
```yaml
data_sources:
  supabase:
    type: supabase
    url: "https://xxxxx.supabase.co"
    anon_key: "${SUPABASE_ANON_KEY}"
```

**Usage in Action:**
```yaml
actions:
  - id: list_posts
    name: List Posts
    action_type: list
    data_source: supabase
    query: |
      {
        "table": "posts",
        "select": "*",
        "order": { "created_at": "desc" }
      }
```

### 12. WebSocket

**WebSocket Connection:**
```yaml
data_sources:
  realtime:
    type: websocket
    url: "wss://api.example.com/ws"
    protocol: "json"
```

---

## UI Features

### 1. Toast Notifications

**Success Toast:**
```javascript
showSuccess('Record created successfully');
```

**Error Toast:**
```javascript
showError('Failed to save: Validation error');
```

**Warning Toast:**
```javascript
showWarning('This action cannot be undone');
```

**Info Toast:**
```javascript
showInfo('Processing your request...');
```

### 2. Search Functionality

Global search automatically added to all list views:
- Real-time filtering as you type
- Case-insensitive search
- Searches across all visible fields
- Shows result count
- Clear button to reset

**Usage:**
1. Type in search box above table
2. Rows are filtered instantly
3. Count shows "Showing X of Y results"

### 3. Dark Mode

**Toggle Dark Mode:**
- Click moon/sun icon in top-right
- Persists via localStorage
- Applies to all UI elements

**Programmatic Toggle:**
```javascript
// Enable dark mode
$('body').addClass('dark-mode');
localStorage.setItem('theme', 'dark');

// Disable dark mode
$('body').removeClass('dark-mode');
localStorage.setItem('theme', 'light');
```

### 4. Inline Editing

Edit table cells directly:
- Double-click any editable cell
- Edit value inline
- Press Enter to save
- Press Escape to cancel
- Auto-saves on blur (click outside)

**Supported Field Types:**
- Text, Number, Email, Date
- Boolean (dropdown)
- Textarea (2 rows)

### 5. CSV Export

Export table data to CSV:
- Click "Export" button
- Exports all visible columns
- Handles dates, booleans, objects
- Filename: `section-name-export-YYYY-MM-DD.csv`
- RFC 4180 compliant format

### 6. Column Sorting

Sort by any column:
- Click column header to sort
- First click: Ascending ↑
- Second click: Descending ↓
- Visual indicators on active column
- Smart sorting (detects numbers vs text)

### 7. Advanced Filtering

**Open Filter Panel:**
Click "Filters" button in toolbar

**Filter Options:**
- Column-based filters
- Multiple filter operators
- Date range pickers
- Saved filter presets

**Save Filter Preset:**
1. Set your filters
2. Click "Save Preset"
3. Enter preset name
4. Reload anytime from dropdown

### 8. Bulk Operations

**Select Rows:**
- Check boxes next to rows
- Or use "Select All" checkbox

**Bulk Actions:**
- **Delete Selected**: Delete multiple rows at once
- **Export Selected**: Export only selected rows
- **Deselect All**: Clear all selections

**Progress Tracking:**
- Real-time progress bar
- Shows completion percentage
- Displays success/failure count

### 9. CSV Import

**Import Data:**
1. Click "Import" button
2. Select CSV file
3. First row must be headers
4. Click "Import" in dialog
5. Watch progress indicator

**Requirements:**
- CSV format
- Header row with column names
- Matching field IDs

### 10. Keyboard Shortcuts

**Available Shortcuts:**
- `Ctrl/Cmd + K`: Focus search
- `Ctrl/Cmd + E`: Export to CSV
- `Ctrl/Cmd + D`: Toggle dark mode
- `Ctrl/Cmd + N`: Open create form
- `Ctrl/Cmd + /`: Show shortcuts help
- `Esc`: Close modal

**View All Shortcuts:**
Press `Ctrl/Cmd + /` to see help

---

## Advanced Features

### 1. Pagination

**Configure Pagination:**
```yaml
actions:
  - id: list_items
    name: List Items
    action_type: list
    config:
      pagination:
        enabled: true
        page_size: 20
        show_size_options: true
        size_options: [10, 20, 50, 100]
```

**Usage:**
- Navigate with prev/next buttons
- Jump to specific page
- Change page size
- Shows total pages and items

### 2. Filter Configuration

**Define Filters:**
```yaml
actions:
  - id: list_products
    action_type: list
    config:
      filters:
        - field: status
          label: Status
          type: select
          options:
            - draft
            - published
            - archived
        - field: created_at
          label: Created Date
          type: date_range
        - field: price
          label: Price Range
          type: number_range
          min: 0
          max: 10000
```

### 3. Bulk Update

```yaml
actions:
  - id: bulk_update_status
    name: Bulk Update Status
    action_type: update
    bulk: true
    fields:
      - id: status
        name: New Status
        field_type: select
        config:
          options:
            - value: approved
              label: Approved
            - value: rejected
              label: Rejected
```

### 4. Conditional Fields

**Show/Hide Based on Other Fields:**
```yaml
fields:
  - id: payment_method
    name: Payment Method
    field_type: select
    config:
      options:
        - value: card
          label: Credit Card
        - value: bank
          label: Bank Transfer

  - id: card_number
    name: Card Number
    field_type: text
    conditional:
      field: payment_method
      value: card
      operator: equals
```

### 5. Field Dependencies

**Dependent Validation:**
```yaml
fields:
  - id: password
    name: Password
    field_type: password
    validations:
      - type: min_length
        value: 8

  - id: password_confirm
    name: Confirm Password
    field_type: password
    validations:
      - type: match
        field: password
        message: "Passwords must match"
```

### 6. Custom Actions

**Define Custom Action:**
```yaml
actions:
  - id: export_report
    name: Export Report
    action_type: custom
    icon: "fa-file-export"
    endpoint: "/api/reports/export"
    method: POST
    confirm_message: "Generate and download report?"
```

### 7. Audit Trail

**Track Changes:**
```yaml
sections:
  - id: users
    audit: true
    audit_fields:
      - created_by
      - created_at
      - updated_by
      - updated_at
```

### 8. Row Actions

**Per-Row Actions:**
```yaml
actions:
  - id: view_details
    name: View
    action_type: view
    icon: "fa-eye"
    show_in_row: true

  - id: edit_item
    name: Edit
    action_type: form
    icon: "fa-edit"
    show_in_row: true

  - id: delete_item
    name: Delete
    action_type: delete
    icon: "fa-trash"
    show_in_row: true
    confirm_message: "Are you sure?"
```

---

## Keyboard Shortcuts

### Global Shortcuts

| Shortcut | Action | Description |
|----------|--------|-------------|
| `Ctrl/Cmd + K` | Focus Search | Jump to search input |
| `Ctrl/Cmd + E` | Export CSV | Export current table |
| `Ctrl/Cmd + D` | Toggle Dark Mode | Switch theme |
| `Ctrl/Cmd + N` | New Record | Open create form |
| `Ctrl/Cmd + /` | Show Help | Display shortcuts |
| `Esc` | Close Modal | Close any open dialog |

### Table Navigation

| Shortcut | Action |
|----------|--------|
| `↑` / `↓` | Navigate rows |
| `Enter` | Edit selected row |
| `Space` | Toggle row selection |
| `Ctrl + A` | Select all rows |

### Form Shortcuts

| Shortcut | Action |
|----------|--------|
| `Tab` | Next field |
| `Shift + Tab` | Previous field |
| `Ctrl/Cmd + Enter` | Submit form |
| `Esc` | Cancel/Close |

---

## API Documentation

### REST API Endpoints

#### 1. Get Configuration
```http
GET /api/config
```

**Response:**
```json
{
  "server": {
    "host": "0.0.0.0",
    "port": 3000
  },
  "security": {
    "enabled": false
  }
}
```

#### 2. List Backoffices
```http
GET /api/backoffices
```

**Response:**
```json
[
  {
    "id": "users",
    "name": "User Management",
    "description": "Manage users",
    "sections": [...]
  }
]
```

#### 3. Get Specific Backoffice
```http
GET /api/backoffices/{id}
```

**Example:**
```http
GET /api/backoffices/users
```

#### 4. Execute List Action
```http
GET /api/backoffices/{backoffice_id}/sections/{section_id}/actions/{action_id}?page=1&page_size=20
```

**Example:**
```http
GET /api/backoffices/users/sections/users/actions/list_users?page=1&page_size=20&sort_by=created_at&sort_order=desc
```

**Response:**
```json
{
  "data": [...],
  "fields": [...],
  "pagination": {
    "page": 1,
    "page_size": 20,
    "total_items": 150,
    "total_pages": 8
  }
}
```

#### 5. Execute Mutation (Create/Update/Delete)
```http
POST /api/backoffices/{backoffice_id}/sections/{section_id}/actions/{action_id}
```

**Create Example:**
```http
POST /api/backoffices/users/sections/users/actions/create_user
Content-Type: application/json

{
  "username": "johndoe",
  "email": "john@example.com",
  "role": "admin"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": 123,
    "username": "johndoe",
    "email": "john@example.com"
  }
}
```

### Swagger UI

Access interactive API documentation:
```
http://localhost:3000/api/docs
```

Download OpenAPI spec:
```
http://localhost:3000/openapi.yaml
```

---

## Configuration Examples

### Complete E-Commerce Backoffice

```yaml
id: "ecommerce"
name: "E-Commerce Admin"
description: "Complete e-commerce management"

data_sources:
  db:
    type: database
    connection_string: "postgresql://localhost/ecommerce"
    db_type: postgres

  storage:
    type: s3
    bucket: "product-images"
    region: "us-east-1"

sections:
  # Products Section
  - id: products
    name: Products
    icon: "fa-box"
    actions:
      - id: list_products
        name: Product List
        action_type: list
        data_source: db
        query: |
          SELECT p.*, c.name as category_name
          FROM products p
          LEFT JOIN categories c ON p.category_id = c.id
          WHERE p.deleted_at IS NULL
          ORDER BY p.created_at DESC
        config:
          pagination:
            enabled: true
            page_size: 50
          filters:
            - field: category_id
              label: Category
              type: select
            - field: status
              label: Status
              type: select
            - field: price
              label: Price
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
            validations:
              - type: pattern
                value: "^[A-Z0-9-]+$"

          - id: name
            name: Product Name
            field_type: text
            required: true
            visible: true
            editable: true
            validations:
              - type: min_length
                value: 3
              - type: max_length
                value: 200

          - id: description
            name: Description
            field_type: richtext
            required: true

          - id: price
            name: Price
            field_type: currency
            required: true
            config:
              currency: USD
            validations:
              - type: min_value
                value: 0
              - type: positive

          - id: stock
            name: Stock
            field_type: number
            required: true
            validations:
              - type: integer
              - type: min_value
                value: 0

          - id: images
            name: Images
            field_type: image
            config:
              multiple: true
              max_files: 10
              storage: storage

          - id: category_id
            name: Category
            field_type: select
            required: true
            config:
              options:
                - value: 1
                  label: Electronics
                - value: 2
                  label: Clothing
                - value: 3
                  label: Books

          - id: tags
            name: Tags
            field_type: tags
            config:
              allow_custom: true

          - id: featured
            name: Featured
            field_type: toggle
            default_value: false

          - id: status
            name: Status
            field_type: select
            required: true
            config:
              options:
                - value: draft
                  label: Draft
                - value: published
                  label: Published
                - value: archived
                  label: Archived
            default_value: draft

      - id: create_product
        name: Create Product
        action_type: form
        data_source: db
        config:
          form_mode: create
        query: |
          INSERT INTO products (sku, name, description, price, stock, category_id, images, tags, featured, status)
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
          RETURNING *

  # Orders Section
  - id: orders
    name: Orders
    icon: "fa-shopping-cart"
    actions:
      - id: list_orders
        name: Order List
        action_type: list
        data_source: db
        query: |
          SELECT o.*, u.email as customer_email
          FROM orders o
          LEFT JOIN users u ON o.user_id = u.id
          ORDER BY o.created_at DESC
        fields:
          - id: id
            name: Order #
            field_type: text
            editable: false

          - id: customer_email
            name: Customer
            field_type: email
            editable: false

          - id: total
            name: Total
            field_type: currency
            config:
              currency: USD

          - id: status
            name: Status
            field_type: select
            config:
              options:
                - value: pending
                  label: Pending
                - value: processing
                  label: Processing
                - value: shipped
                  label: Shipped
                - value: delivered
                  label: Delivered
                - value: cancelled
                  label: Cancelled

          - id: created_at
            name: Order Date
            field_type: datetime
            editable: false

  # Customers Section
  - id: customers
    name: Customers
    icon: "fa-users"
    actions:
      - id: list_customers
        name: Customer List
        action_type: list
        data_source: db
        query: "SELECT * FROM users WHERE role = 'customer' ORDER BY created_at DESC"
        fields:
          - id: email
            name: Email
            field_type: email
            validations:
              - type: email

          - id: first_name
            name: First Name
            field_type: text

          - id: last_name
            name: Last Name
            field_type: text

          - id: phone
            name: Phone
            field_type: tel
            validations:
              - type: phone

          - id: address
            name: Address
            field_type: textarea

          - id: created_at
            name: Joined
            field_type: datetime
            editable: false
```

### Blog Management System

```yaml
id: "blog"
name: "Blog CMS"

data_sources:
  db:
    type: database
    connection_string: "postgresql://localhost/blog"

sections:
  - id: posts
    name: Posts
    actions:
      - id: list_posts
        action_type: list
        data_source: db
        fields:
          - id: title
            name: Title
            field_type: text
            required: true

          - id: slug
            name: URL Slug
            field_type: text
            validations:
              - type: slug

          - id: content
            name: Content
            field_type: markdown
            config:
              preview: true

          - id: excerpt
            name: Excerpt
            field_type: textarea
            config:
              max_length: 500

          - id: featured_image
            name: Featured Image
            field_type: image

          - id: author_id
            name: Author
            field_type: select

          - id: category
            name: Category
            field_type: select

          - id: tags
            name: Tags
            field_type: tags

          - id: status
            name: Status
            field_type: select
            config:
              options:
                - draft
                - published
                - scheduled

          - id: published_at
            name: Publish Date
            field_type: datetime

          - id: seo_title
            name: SEO Title
            field_type: text

          - id: seo_description
            name: SEO Description
            field_type: textarea
```

---

## Best Practices

### 1. Configuration Organization

**Use Nested Directories:**
```
config/
  backoffices/
    ecommerce/
      products.yaml
      orders.yaml
      customers.yaml
    blog/
      posts.yaml
      comments.yaml
    admin/
      users.yaml
      settings.yaml
```

### 2. Security

**Always Validate Input:**
```yaml
fields:
  - id: email
    validations:
      - type: email
  - id: password
    validations:
      - type: min_length
        value: 8
```

**Use Required Scopes:**
```yaml
actions:
  - id: delete_user
    required_scopes:
      - "users:delete"
      - "admin"
```

**Sanitize Data:**
```yaml
fields:
  - id: html_content
    field_type: html
    config:
      sanitize: true
```

### 3. Performance

**Use Pagination:**
```yaml
config:
  pagination:
    enabled: true
    page_size: 50
```

**Index Database Columns:**
```sql
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_products_sku ON products(sku);
```

**Limit Query Results:**
```yaml
query: "SELECT * FROM products LIMIT 1000"
```

### 4. User Experience

**Provide Help Text:**
```yaml
fields:
  - id: api_key
    help_text: "Your API key for external integrations"
```

**Set Sensible Defaults:**
```yaml
fields:
  - id: status
    default_value: "draft"
  - id: active
    default_value: true
```

**Use Placeholders:**
```yaml
fields:
  - id: email
    placeholder: "user@example.com"
```

### 5. Error Handling

**Custom Error Messages:**
```yaml
validations:
  - type: min_length
    value: 8
    message: "Password must be at least 8 characters long"
```

**Validation Rules:**
```yaml
fields:
  - id: username
    validations:
      - type: min_length
        value: 3
      - type: max_length
        value: 20
      - type: pattern
        value: "^[a-z0-9_]+$"
        message: "Only lowercase letters, numbers, and underscores"
```

### 6. Testing

**Test Configuration:**
```bash
# Validate YAML syntax
yamllint config/backoffices/*.yaml

# Test database connection
cargo test test_database_connection

# Run all tests
cargo test --all-features
```

### 7. Documentation

**Document Custom Fields:**
```yaml
fields:
  - id: custom_config
    name: Custom Configuration
    field_type: json
    help_text: |
      JSON configuration for advanced settings.
      Example: {"theme": "dark", "language": "en"}
```

**Add Section Descriptions:**
```yaml
sections:
  - id: products
    name: Products
    description: "Manage your product catalog including inventory, pricing, and categories"
```

---

## Deployment

### Docker Deployment

**Build and Run:**
```bash
# Build Docker image
docker build -t pmp-backoffice .

# Run container
docker run -p 3000:3000 \
  -v $(pwd)/config:/app/config \
  pmp-backoffice
```

**Using Docker Compose:**
```bash
docker-compose up -d
```

### Production Checklist

- [ ] Enable JWT authentication
- [ ] Configure HTTPS
- [ ] Set up CORS properly
- [ ] Enable rate limiting
- [ ] Configure audit logging
- [ ] Set up monitoring
- [ ] Configure backups
- [ ] Review security settings
- [ ] Test all actions
- [ ] Document custom configurations

---

This documentation covers all features with extensive examples. For more specific use cases or questions, please refer to the README or open an issue on GitHub.
