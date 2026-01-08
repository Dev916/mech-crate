# Database Design Guide: Theory & Practice

**Version**: 1.0
**Last Updated**: 2026-01-07
**Purpose**: Comprehensive guide to database design theory, normalization, and practical patterns for PostgreSQL, Redis, and MongoDB

---

## Table of Contents

1. [Normalization Theory](#1-normalization-theory)
2. [PostgreSQL Design Patterns](#2-postgresql-design-patterns)
3. [Redis Design Patterns](#3-redis-design-patterns)
4. [MongoDB Design Patterns](#4-mongodb-design-patterns)
5. [Streaming Patterns](#5-streaming-patterns)
6. [Polyglot Persistence Strategy](#6-polyglot-persistence-strategy)

---

## 1. Normalization Theory

### 1.1 Overview

**Purpose**: Eliminate redundancy, ensure data integrity, optimize for consistency.

**Key Concepts**:
- **Functional Dependency**: Attribute B is functionally dependent on A if each value of A determines exactly one value of B (A → B)
- **Prime Attribute**: Attribute that is part of any candidate key
- **Non-Prime Attribute**: Attribute that is not part of any candidate key
- **Candidate Key**: Minimal set of attributes that uniquely identifies a tuple
- **Superkey**: Set of attributes that uniquely identifies a tuple (may contain extra attributes)

### 1.2 First Normal Form (1NF)

**Definition**: Each attribute contains only atomic (indivisible) values. No repeating groups or arrays.

**Violations**:
```sql
-- ❌ VIOLATES 1NF: Multiple phone numbers in single column
CREATE TABLE contacts_bad (
    id SERIAL PRIMARY KEY,
    name TEXT,
    phone_numbers TEXT  -- "555-1234, 555-5678, 555-9012"
);

-- ❌ VIOLATES 1NF: Array/composite values
CREATE TABLE contacts_bad2 (
    id SERIAL PRIMARY KEY,
    name TEXT,
    phones TEXT[]  -- PostgreSQL array
);
```

**Correct 1NF**:
```sql
-- ✅ 1NF: Atomic values only
CREATE TABLE contacts (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE contact_phones (
    id SERIAL PRIMARY KEY,
    contact_id INTEGER NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    phone_number TEXT NOT NULL,
    phone_type TEXT CHECK (phone_type IN ('mobile', 'home', 'work')),
    UNIQUE(contact_id, phone_number)
);
```

**Benefits**:
- Queryable individual values
- Enforceable constraints
- Indexable columns
- No parsing required

### 1.3 Second Normal Form (2NF)

**Definition**: Must be in 1NF AND every non-prime attribute must be fully functionally dependent on the entire candidate key (no partial dependencies).

**Rule**: Only applies to tables with composite keys. If key is single attribute, 2NF is automatically satisfied.

**Violations**:
```sql
-- ❌ VIOLATES 2NF: Partial dependency on composite key
CREATE TABLE order_items_bad (
    order_id INTEGER,
    product_id INTEGER,
    quantity INTEGER NOT NULL,
    product_name TEXT,      -- Depends only on product_id (partial dependency)
    product_price NUMERIC,  -- Depends only on product_id (partial dependency)
    PRIMARY KEY (order_id, product_id)
);
-- Problem: product_name and product_price depend on product_id only,
-- not on the full key (order_id, product_id)
```

**Correct 2NF**:
```sql
-- ✅ 2NF: Eliminate partial dependencies
CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    price NUMERIC(10, 2) NOT NULL CHECK (price >= 0)
);

CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    customer_id INTEGER NOT NULL REFERENCES customers(id),
    order_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status TEXT NOT NULL CHECK (status IN ('pending', 'paid', 'shipped', 'delivered', 'cancelled'))
);

CREATE TABLE order_items (
    order_id INTEGER REFERENCES orders(id) ON DELETE CASCADE,
    product_id INTEGER REFERENCES products(id),
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    unit_price NUMERIC(10, 2) NOT NULL,  -- Snapshot price at order time
    PRIMARY KEY (order_id, product_id)
);
```

**Benefits**:
- Update product name/price in one place
- No update anomalies
- Reduced redundancy

### 1.4 Third Normal Form (3NF)

**Definition**: Must be in 2NF AND no transitive dependencies (non-prime attributes must not depend on other non-prime attributes).

**Rule**: A → B → C means C transitively depends on A through B.

**Violations**:
```sql
-- ❌ VIOLATES 3NF: Transitive dependency
CREATE TABLE employees_bad (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    department_id INTEGER NOT NULL,
    department_name TEXT,    -- Transitively depends on id through department_id
    department_budget NUMERIC -- Transitively depends on id through department_id
);
-- Problem: department_name depends on department_id, which depends on id
-- Transitive: id → department_id → department_name
```

**Correct 3NF**:
```sql
-- ✅ 3NF: Eliminate transitive dependencies
CREATE TABLE departments (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    budget NUMERIC(12, 2) NOT NULL CHECK (budget >= 0)
);

CREATE TABLE employees (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    department_id INTEGER NOT NULL REFERENCES departments(id),
    hire_date DATE NOT NULL,
    salary NUMERIC(10, 2) NOT NULL CHECK (salary >= 0)
);
```

**Benefits**:
- Update department info in one place
- No anomalies when department changes
- Clear separation of concerns

### 1.5 Boyce-Codd Normal Form (BCNF)

**Definition**: Must be in 3NF AND for every functional dependency X → Y, X must be a superkey.

**Stricter than 3NF**: Eliminates anomalies when multiple overlapping candidate keys exist.

**Violations**:
```sql
-- ❌ VIOLATES BCNF: Professor determines course, but not a superkey
CREATE TABLE course_assignments_bad (
    student_id INTEGER,
    course_id INTEGER,
    professor_id INTEGER,
    PRIMARY KEY (student_id, course_id),
    -- Functional dependency: course_id → professor_id
    -- But course_id is not a superkey
    -- Problem: Same course must have same professor for all students
);
```

**Correct BCNF**:
```sql
-- ✅ BCNF: Decompose into proper relationships
CREATE TABLE courses (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    credits INTEGER NOT NULL CHECK (credits > 0)
);

CREATE TABLE course_sections (
    id SERIAL PRIMARY KEY,
    course_id INTEGER NOT NULL REFERENCES courses(id),
    professor_id INTEGER NOT NULL REFERENCES professors(id),
    semester TEXT NOT NULL,
    UNIQUE(course_id, professor_id, semester)
);

CREATE TABLE enrollments (
    student_id INTEGER REFERENCES students(id),
    section_id INTEGER REFERENCES course_sections(id),
    grade TEXT CHECK (grade IN ('A', 'B', 'C', 'D', 'F', 'W', 'I')),
    PRIMARY KEY (student_id, section_id)
);
```

**Benefits**:
- No update anomalies with overlapping keys
- Clear relationship modeling
- Proper constraint enforcement

### 1.6 Fourth Normal Form (4NF)

**Definition**: Must be in BCNF AND no multi-valued dependencies (MVDs) exist.

**Multi-Valued Dependency**: A →→ B means for each value of A, there exists a set of values for B that is independent of other attributes.

**Violations**:
```sql
-- ❌ VIOLATES 4NF: Multi-valued dependencies
CREATE TABLE employee_skills_languages_bad (
    employee_id INTEGER,
    skill TEXT,      -- MVD: employee_id →→ skill
    language TEXT,   -- MVD: employee_id →→ language
    PRIMARY KEY (employee_id, skill, language)
);
-- Problem: Skills and languages are independent facts about employee
-- Cartesian product: If employee knows 3 skills and 2 languages = 6 rows
-- Inserting new skill requires updating all language combinations
```

**Correct 4NF**:
```sql
-- ✅ 4NF: Separate independent multi-valued facts
CREATE TABLE employee_skills (
    employee_id INTEGER REFERENCES employees(id),
    skill TEXT NOT NULL,
    proficiency_level TEXT CHECK (proficiency_level IN ('beginner', 'intermediate', 'advanced', 'expert')),
    PRIMARY KEY (employee_id, skill)
);

CREATE TABLE employee_languages (
    employee_id INTEGER REFERENCES employees(id),
    language TEXT NOT NULL,
    fluency_level TEXT CHECK (fluency_level IN ('basic', 'conversational', 'fluent', 'native')),
    PRIMARY KEY (employee_id, language)
);
```

**Benefits**:
- No redundant combinations
- Independent updates
- Clearer semantics
- Reduced storage

### 1.7 Fifth Normal Form (5NF) / Project-Join Normal Form (PJNF)

**Definition**: Must be in 4NF AND no join dependencies exist (cannot be decomposed into smaller tables without loss of information).

**Join Dependency**: A table has a join dependency if it can only be reconstructed by joining multiple projections.

**Violations**:
```sql
-- ❌ VIOLATES 5NF: Hidden join dependency
CREATE TABLE supplier_part_project_bad (
    supplier_id INTEGER,
    part_id INTEGER,
    project_id INTEGER,
    PRIMARY KEY (supplier_id, part_id, project_id)
);
-- Problem: If the rules are:
-- 1. Supplier S supplies Part P
-- 2. Part P is used in Project J
-- 3. Supplier S supplies to Project J
-- Then (S, P, J) is implied by joins, storing it creates redundancy
```

**Correct 5NF**:
```sql
-- ✅ 5NF: Decompose to eliminate join dependencies
CREATE TABLE supplier_parts (
    supplier_id INTEGER REFERENCES suppliers(id),
    part_id INTEGER REFERENCES parts(id),
    PRIMARY KEY (supplier_id, part_id)
);

CREATE TABLE part_projects (
    part_id INTEGER REFERENCES parts(id),
    project_id INTEGER REFERENCES projects(id),
    PRIMARY KEY (part_id, project_id)
);

CREATE TABLE supplier_projects (
    supplier_id INTEGER REFERENCES suppliers(id),
    project_id INTEGER REFERENCES projects(id),
    PRIMARY KEY (supplier_id, project_id)
);

-- Valid combinations can be reconstructed via:
-- SELECT sp.supplier_id, sp.part_id, pp.project_id
-- FROM supplier_parts sp
-- JOIN part_projects pp ON sp.part_id = pp.part_id
-- JOIN supplier_projects sproj ON sp.supplier_id = sproj.supplier_id
--                              AND pp.project_id = sproj.project_id;
```

**Benefits**:
- No redundancy from implied relationships
- Each fact stored once
- Maximum flexibility for changes

**When to Use**: Rare in practice. Most systems stop at 3NF/BCNF.

### 1.8 Sixth Normal Form (6NF) / Domain-Key Normal Form (DKNF)

**Definition**: Decompose to eliminate all data redundancy. Every table has exactly one non-key attribute (or is all-key).

**Purpose**: Temporal databases, audit systems, bi-temporal data.

**Example**:
```sql
-- ✅ 6NF: Maximal decomposition for temporal data
CREATE TABLE employee_names (
    employee_id INTEGER REFERENCES employees(id),
    name TEXT NOT NULL,
    valid_from TIMESTAMPTZ NOT NULL,
    valid_to TIMESTAMPTZ,
    PRIMARY KEY (employee_id, valid_from),
    CHECK (valid_to IS NULL OR valid_to > valid_from)
);

CREATE TABLE employee_salaries (
    employee_id INTEGER REFERENCES employees(id),
    salary NUMERIC(10, 2) NOT NULL,
    valid_from TIMESTAMPTZ NOT NULL,
    valid_to TIMESTAMPTZ,
    PRIMARY KEY (employee_id, valid_from),
    CHECK (valid_to IS NULL OR valid_to > valid_from)
);

CREATE TABLE employee_departments (
    employee_id INTEGER REFERENCES employees(id),
    department_id INTEGER REFERENCES departments(id),
    valid_from TIMESTAMPTZ NOT NULL,
    valid_to TIMESTAMPTZ,
    PRIMARY KEY (employee_id, valid_from),
    CHECK (valid_to IS NULL OR valid_to > valid_from)
);
```

**Benefits**:
- Independent attribute history
- Update one attribute without affecting others
- Perfect for audit trails

**Drawbacks**:
- Many tables
- Complex queries
- Performance overhead

**When to Use**: Temporal databases, regulatory compliance, complete audit trails.

### 1.9 Denormalization: When to Break the Rules

**Strategic Denormalization** for performance when:

1. **Read-Heavy Workloads**:
```sql
-- Denormalized for reporting
CREATE MATERIALIZED VIEW order_summary AS
SELECT
    o.id,
    o.order_date,
    c.name AS customer_name,
    c.email AS customer_email,
    COUNT(oi.product_id) AS item_count,
    SUM(oi.quantity * oi.unit_price) AS total_amount
FROM orders o
JOIN customers c ON o.customer_id = c.id
JOIN order_items oi ON o.id = oi.order_id
GROUP BY o.id, o.order_date, c.name, c.email;

CREATE UNIQUE INDEX ON order_summary(id);
REFRESH MATERIALIZED VIEW CONCURRENTLY order_summary;
```

2. **Computed Aggregates**:
```sql
-- Maintain aggregate for performance
CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    customer_id INTEGER NOT NULL REFERENCES customers(id),
    order_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    total_amount NUMERIC(10, 2) NOT NULL DEFAULT 0,  -- Denormalized sum
    item_count INTEGER NOT NULL DEFAULT 0            -- Denormalized count
);

-- Keep denormalized data in sync with triggers
CREATE OR REPLACE FUNCTION update_order_totals()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE orders
    SET
        total_amount = (
            SELECT COALESCE(SUM(quantity * unit_price), 0)
            FROM order_items
            WHERE order_id = NEW.order_id
        ),
        item_count = (
            SELECT COUNT(*)
            FROM order_items
            WHERE order_id = NEW.order_id
        )
    WHERE id = NEW.order_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER maintain_order_totals
AFTER INSERT OR UPDATE OR DELETE ON order_items
FOR EACH ROW EXECUTE FUNCTION update_order_totals();
```

3. **Common Joins**:
```sql
-- Cache frequently joined data
CREATE TABLE order_items (
    order_id INTEGER REFERENCES orders(id),
    product_id INTEGER REFERENCES products(id),
    quantity INTEGER NOT NULL,
    unit_price NUMERIC(10, 2) NOT NULL,
    product_name TEXT NOT NULL,  -- Denormalized from products
    PRIMARY KEY (order_id, product_id)
);
```

**Rules for Safe Denormalization**:
1. Document the denormalization and rationale
2. Use triggers/application logic to maintain consistency
3. Monitor for staleness
4. Measure performance impact (before/after)
5. Consider materialized views instead of manual denormalization
6. Never denormalize without profiling first

---

## 2. PostgreSQL Design Patterns

### 2.1 Schema Design Principles

**Core Guidelines**:
1. Start with 3NF, denormalize only with measurement
2. Use appropriate data types (don't use TEXT for everything)
3. Add constraints at database level, not just application
4. Use DOMAIN types for reusable constraints
5. Leverage PostgreSQL-specific features (arrays, JSON, ranges, etc.)

**Type Selection**:
```sql
-- ✅ Use appropriate types
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email CITEXT NOT NULL UNIQUE,  -- Case-insensitive email
    username VARCHAR(50) NOT NULL UNIQUE CHECK (username ~* '^[a-z0-9_]{3,50}$'),
    age SMALLINT CHECK (age BETWEEN 0 AND 150),
    balance NUMERIC(12, 2) NOT NULL DEFAULT 0 CHECK (balance >= 0),
    metadata JSONB,  -- Flexible schema data
    ip_address INET,  -- IP addresses
    active_period TSTZRANGE,  -- Time ranges
    tags TEXT[],  -- Arrays
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ❌ Avoid generic types
CREATE TABLE users_bad (
    id TEXT PRIMARY KEY,  -- Should be UUID or BIGSERIAL
    email TEXT,  -- Should be CITEXT
    age TEXT,  -- Should be SMALLINT
    balance TEXT,  -- Should be NUMERIC
    created_at TEXT  -- Should be TIMESTAMPTZ
);
```

**DOMAIN Types for Reusability**:
```sql
-- Define reusable constrained types
CREATE DOMAIN email AS CITEXT
    CHECK (VALUE ~ '^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$');

CREATE DOMAIN phone AS TEXT
    CHECK (VALUE ~ '^\+?[1-9]\d{1,14}$');

CREATE DOMAIN url AS TEXT
    CHECK (VALUE ~ '^https?://[^\s/$.?#].[^\s]*$');

CREATE DOMAIN slug AS VARCHAR(100)
    CHECK (VALUE ~ '^[a-z0-9]+(?:-[a-z0-9]+)*$');

CREATE DOMAIN money AS NUMERIC(12, 2)
    CHECK (VALUE >= 0);

-- Use domains
CREATE TABLE products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    slug slug UNIQUE NOT NULL,
    price money NOT NULL,
    website url
);
```

### 2.2 Indexing Strategies

**Index Types**:

**B-Tree (Default)** - Equality, range queries, sorting:
```sql
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_orders_customer_date ON orders(customer_id, order_date DESC);
CREATE INDEX idx_products_price ON products(price) WHERE active = true;  -- Partial index
```

**Hash** - Exact equality only (faster than B-tree for equality):
```sql
CREATE INDEX idx_users_uuid_hash ON users USING HASH(id);
```

**GIN (Generalized Inverted Index)** - Full-text search, JSONB, arrays:
```sql
-- Full-text search
CREATE INDEX idx_posts_search ON posts USING GIN(to_tsvector('english', title || ' ' || body));

-- JSONB
CREATE INDEX idx_users_metadata ON users USING GIN(metadata);
CREATE INDEX idx_users_metadata_path ON users USING GIN(metadata jsonb_path_ops);

-- Arrays
CREATE INDEX idx_posts_tags ON posts USING GIN(tags);
```

**GiST (Generalized Search Tree)** - Geometric types, ranges, full-text:
```sql
-- Ranges
CREATE INDEX idx_bookings_period ON bookings USING GIST(period);

-- Geometric
CREATE INDEX idx_locations_point ON locations USING GIST(coordinates);
```

**BRIN (Block Range Index)** - Very large tables with natural ordering:
```sql
-- Time-series data
CREATE INDEX idx_events_created_brin ON events USING BRIN(created_at);
```

**Covering Indexes** (INCLUDE):
```sql
-- Index includes extra columns for index-only scans
CREATE INDEX idx_orders_customer_covering
ON orders(customer_id) INCLUDE (order_date, total_amount);
```

**Multi-Column Index Order**:
```sql
-- ✅ High cardinality first, query order matches index
CREATE INDEX idx_orders_customer_date_status
ON orders(customer_id, order_date DESC, status);

-- Supports queries:
-- WHERE customer_id = X
-- WHERE customer_id = X AND order_date > Y
-- WHERE customer_id = X AND order_date > Y AND status = Z

-- ❌ Low cardinality first
CREATE INDEX idx_orders_bad ON orders(status, customer_id);
-- Less efficient for filtering
```

**Partial Indexes**:
```sql
-- Index only active records
CREATE INDEX idx_active_users ON users(email) WHERE active = true;

-- Index only recent data
CREATE INDEX idx_recent_orders ON orders(created_at)
WHERE created_at > NOW() - INTERVAL '90 days';

-- Index only non-null values
CREATE INDEX idx_users_phone ON users(phone) WHERE phone IS NOT NULL;
```

**Expression Indexes**:
```sql
-- Index on computed expression
CREATE INDEX idx_users_lower_email ON users(LOWER(email));

-- Index on JSON field
CREATE INDEX idx_users_subscription_level
ON users((metadata->>'subscription_level'));

-- Trigram index for fuzzy search
CREATE EXTENSION pg_trgm;
CREATE INDEX idx_products_name_trgm ON products USING GIN(name gin_trgm_ops);
```

### 2.3 Constraints & Data Integrity

**NOT NULL**:
```sql
-- ✅ Be explicit about required fields
CREATE TABLE orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES customers(id),
    order_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status TEXT NOT NULL DEFAULT 'pending'
);
```

**CHECK Constraints**:
```sql
CREATE TABLE products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL CHECK (LENGTH(name) > 0),
    price NUMERIC(10, 2) NOT NULL CHECK (price > 0),
    discount_pct NUMERIC(5, 2) CHECK (discount_pct BETWEEN 0 AND 100),
    stock INTEGER NOT NULL DEFAULT 0 CHECK (stock >= 0),
    weight_kg NUMERIC(8, 3) CHECK (weight_kg > 0),
    published_at TIMESTAMPTZ,
    archived_at TIMESTAMPTZ,
    CHECK (archived_at IS NULL OR archived_at > published_at)
);
```

**UNIQUE Constraints**:
```sql
-- Single column
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email CITEXT NOT NULL UNIQUE,
    username VARCHAR(50) NOT NULL UNIQUE
);

-- Multi-column (composite unique)
CREATE TABLE product_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id),
    user_id UUID NOT NULL REFERENCES users(id),
    rating SMALLINT NOT NULL CHECK (rating BETWEEN 1 AND 5),
    UNIQUE(product_id, user_id)  -- One review per product per user
);

-- Partial unique constraint
CREATE UNIQUE INDEX idx_users_username_active
ON users(username) WHERE active = true;
```

**EXCLUSION Constraints**:
```sql
-- Prevent overlapping time ranges
CREATE EXTENSION btree_gist;

CREATE TABLE room_bookings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id UUID NOT NULL REFERENCES rooms(id),
    guest_id UUID NOT NULL REFERENCES guests(id),
    period TSTZRANGE NOT NULL,
    EXCLUDE USING GIST (room_id WITH =, period WITH &&)
);

-- Prevent overlapping IP ranges
CREATE TABLE ip_allocations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    network INET NOT NULL,
    allocated_to TEXT NOT NULL,
    EXCLUDE USING GIST (network inet_ops WITH &&)
);
```

**Foreign Keys with Actions**:
```sql
CREATE TABLE orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES customers(id) ON DELETE RESTRICT,
    -- Cannot delete customer with orders
);

CREATE TABLE order_items (
    order_id UUID REFERENCES orders(id) ON DELETE CASCADE,
    -- Delete items when order is deleted
    product_id UUID REFERENCES products(id) ON DELETE RESTRICT,
    -- Cannot delete product that's been ordered
    PRIMARY KEY (order_id, product_id)
);

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    -- Keep audit log but clear user reference
    action TEXT NOT NULL
);
```

### 2.4 Partitioning

**Range Partitioning** (time-series, numeric ranges):
```sql
-- Parent table
CREATE TABLE events (
    id UUID DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    event_type TEXT NOT NULL,
    payload JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

-- Create partitions
CREATE TABLE events_2024_01 PARTITION OF events
    FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');

CREATE TABLE events_2024_02 PARTITION OF events
    FOR VALUES FROM ('2024-02-01') TO ('2024-03-01');

CREATE TABLE events_2024_03 PARTITION OF events
    FOR VALUES FROM ('2024-03-01') TO ('2024-04-01');

-- Indexes on partitions
CREATE INDEX ON events_2024_01(user_id);
CREATE INDEX ON events_2024_02(user_id);
CREATE INDEX ON events_2024_03(user_id);

-- Automatic partition creation (requires extension)
CREATE EXTENSION pg_partman;
SELECT partman.create_parent(
    'public.events',
    'created_at',
    'native',
    'monthly'
);
```

**List Partitioning** (discrete values):
```sql
CREATE TABLE orders (
    id UUID DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, status)
) PARTITION BY LIST (status);

CREATE TABLE orders_pending PARTITION OF orders
    FOR VALUES IN ('pending', 'processing');

CREATE TABLE orders_completed PARTITION OF orders
    FOR VALUES IN ('completed', 'delivered');

CREATE TABLE orders_cancelled PARTITION OF orders
    FOR VALUES IN ('cancelled', 'refunded');
```

**Hash Partitioning** (uniform distribution):
```sql
CREATE TABLE user_events (
    id UUID DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    event_data JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, user_id)
) PARTITION BY HASH (user_id);

CREATE TABLE user_events_0 PARTITION OF user_events
    FOR VALUES WITH (MODULUS 4, REMAINDER 0);

CREATE TABLE user_events_1 PARTITION OF user_events
    FOR VALUES WITH (MODULUS 4, REMAINDER 1);

CREATE TABLE user_events_2 PARTITION OF user_events
    FOR VALUES WITH (MODULUS 4, REMAINDER 2);

CREATE TABLE user_events_3 PARTITION OF user_events
    FOR VALUES WITH (MODULUS 4, REMAINDER 3);
```

**Partition Maintenance**:
```sql
-- Detach old partition
ALTER TABLE events DETACH PARTITION events_2023_01;

-- Archive to separate tablespace
ALTER TABLE events_2023_01 SET TABLESPACE archive;

-- Drop old partitions
DROP TABLE events_2022_01;

-- Attach pre-created partition
CREATE TABLE events_2024_12 (LIKE events INCLUDING ALL);
ALTER TABLE events ATTACH PARTITION events_2024_12
    FOR VALUES FROM ('2024-12-01') TO ('2025-01-01');
```

### 2.5 Materialized Views

**Basic Materialized View**:
```sql
CREATE MATERIALIZED VIEW daily_sales_summary AS
SELECT
    DATE_TRUNC('day', o.created_at) AS sale_date,
    COUNT(DISTINCT o.id) AS order_count,
    COUNT(DISTINCT o.customer_id) AS unique_customers,
    SUM(oi.quantity * oi.unit_price) AS total_revenue,
    AVG(oi.quantity * oi.unit_price) AS avg_order_value
FROM orders o
JOIN order_items oi ON o.id = oi.order_id
WHERE o.status = 'completed'
GROUP BY DATE_TRUNC('day', o.created_at);

CREATE UNIQUE INDEX ON daily_sales_summary(sale_date);

-- Refresh (blocking)
REFRESH MATERIALIZED VIEW daily_sales_summary;

-- Refresh concurrently (requires unique index, non-blocking)
REFRESH MATERIALIZED VIEW CONCURRENTLY daily_sales_summary;
```

**Incremental Refresh Pattern**:
```sql
-- Track last refresh
CREATE TABLE mv_refresh_log (
    view_name TEXT PRIMARY KEY,
    last_refresh TIMESTAMPTZ NOT NULL
);

-- Materialized view with incremental data
CREATE MATERIALIZED VIEW user_activity_summary AS
SELECT
    user_id,
    COUNT(*) AS event_count,
    MAX(created_at) AS last_event_at,
    MIN(created_at) AS first_event_at
FROM events
WHERE created_at > COALESCE(
    (SELECT last_refresh FROM mv_refresh_log WHERE view_name = 'user_activity_summary'),
    '1970-01-01'::TIMESTAMPTZ
)
GROUP BY user_id;

-- Refresh procedure
CREATE OR REPLACE FUNCTION refresh_user_activity()
RETURNS VOID AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY user_activity_summary;

    INSERT INTO mv_refresh_log (view_name, last_refresh)
    VALUES ('user_activity_summary', NOW())
    ON CONFLICT (view_name)
    DO UPDATE SET last_refresh = NOW();
END;
$$ LANGUAGE plpgsql;

-- Schedule with pg_cron
CREATE EXTENSION pg_cron;
SELECT cron.schedule('refresh-user-activity', '*/15 * * * *', 'SELECT refresh_user_activity()');
```

### 2.6 Triggers and Stored Procedures

**Audit Trail Trigger**:
```sql
-- Audit table
CREATE TABLE audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_name TEXT NOT NULL,
    record_id UUID NOT NULL,
    action TEXT NOT NULL CHECK (action IN ('INSERT', 'UPDATE', 'DELETE')),
    old_data JSONB,
    new_data JSONB,
    changed_by UUID REFERENCES users(id),
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Generic audit trigger function
CREATE OR REPLACE FUNCTION audit_trigger_func()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        INSERT INTO audit_log (table_name, record_id, action, old_data)
        VALUES (TG_TABLE_NAME, OLD.id, TG_OP, row_to_json(OLD));
        RETURN OLD;
    ELSIF (TG_OP = 'UPDATE') THEN
        INSERT INTO audit_log (table_name, record_id, action, old_data, new_data)
        VALUES (TG_TABLE_NAME, NEW.id, TG_OP, row_to_json(OLD), row_to_json(NEW));
        RETURN NEW;
    ELSIF (TG_OP = 'INSERT') THEN
        INSERT INTO audit_log (table_name, record_id, action, new_data)
        VALUES (TG_TABLE_NAME, NEW.id, TG_OP, row_to_json(NEW));
        RETURN NEW;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Apply to tables
CREATE TRIGGER audit_users
AFTER INSERT OR UPDATE OR DELETE ON users
FOR EACH ROW EXECUTE FUNCTION audit_trigger_func();

CREATE TRIGGER audit_orders
AFTER INSERT OR UPDATE OR DELETE ON orders
FOR EACH ROW EXECUTE FUNCTION audit_trigger_func();
```

**Updated At Trigger**:
```sql
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_updated_at
BEFORE UPDATE ON users
FOR EACH ROW EXECUTE FUNCTION update_updated_at();
```

**Soft Delete Pattern**:
```sql
CREATE TABLE products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    price NUMERIC(10, 2) NOT NULL,
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Prevent hard deletes
CREATE OR REPLACE FUNCTION prevent_delete()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'Hard deletes are not allowed. Use soft delete.';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER prevent_product_delete
BEFORE DELETE ON products
FOR EACH ROW EXECUTE FUNCTION prevent_delete();

-- Soft delete function
CREATE OR REPLACE FUNCTION soft_delete_product(product_id UUID)
RETURNS VOID AS $$
BEGIN
    UPDATE products
    SET deleted_at = NOW()
    WHERE id = product_id AND deleted_at IS NULL;
END;
$$ LANGUAGE plpgsql;

-- View for active products
CREATE VIEW active_products AS
SELECT * FROM products WHERE deleted_at IS NULL;
```

**Cascading Status Updates**:
```sql
-- Update order status based on all items shipped
CREATE OR REPLACE FUNCTION check_order_completion()
RETURNS TRIGGER AS $$
DECLARE
    all_shipped BOOLEAN;
BEGIN
    SELECT NOT EXISTS(
        SELECT 1 FROM order_items
        WHERE order_id = NEW.order_id AND status != 'shipped'
    ) INTO all_shipped;

    IF all_shipped THEN
        UPDATE orders
        SET status = 'completed', completed_at = NOW()
        WHERE id = NEW.order_id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER order_item_shipped
AFTER UPDATE OF status ON order_items
FOR EACH ROW
WHEN (NEW.status = 'shipped')
EXECUTE FUNCTION check_order_completion();
```

### 2.7 Advanced Querying Patterns

**CTEs (Common Table Expressions)**:
```sql
-- Recursive CTE for hierarchical data
WITH RECURSIVE category_tree AS (
    -- Base case: root categories
    SELECT id, name, parent_id, name AS path, 0 AS depth
    FROM categories
    WHERE parent_id IS NULL

    UNION ALL

    -- Recursive case: child categories
    SELECT c.id, c.name, c.parent_id,
           ct.path || ' > ' || c.name AS path,
           ct.depth + 1 AS depth
    FROM categories c
    JOIN category_tree ct ON c.parent_id = ct.id
)
SELECT * FROM category_tree
ORDER BY path;

-- Multiple CTEs for complex queries
WITH
monthly_sales AS (
    SELECT
        DATE_TRUNC('month', order_date) AS month,
        SUM(total_amount) AS revenue
    FROM orders
    WHERE status = 'completed'
    GROUP BY DATE_TRUNC('month', order_date)
),
monthly_growth AS (
    SELECT
        month,
        revenue,
        LAG(revenue) OVER (ORDER BY month) AS prev_month_revenue,
        (revenue - LAG(revenue) OVER (ORDER BY month)) /
            NULLIF(LAG(revenue) OVER (ORDER BY month), 0) * 100 AS growth_pct
    FROM monthly_sales
)
SELECT
    month,
    revenue,
    prev_month_revenue,
    ROUND(growth_pct, 2) AS growth_percentage
FROM monthly_growth
WHERE prev_month_revenue IS NOT NULL
ORDER BY month DESC;
```

**Window Functions**:
```sql
-- Running totals and rankings
SELECT
    o.order_date,
    o.total_amount,
    SUM(o.total_amount) OVER (ORDER BY o.order_date) AS running_total,
    AVG(o.total_amount) OVER (
        ORDER BY o.order_date
        ROWS BETWEEN 6 PRECEDING AND CURRENT ROW
    ) AS moving_avg_7day,
    RANK() OVER (PARTITION BY DATE_TRUNC('month', o.order_date) ORDER BY o.total_amount DESC) AS monthly_rank,
    NTILE(4) OVER (ORDER BY o.total_amount) AS quartile
FROM orders o
WHERE o.status = 'completed'
ORDER BY o.order_date;

-- Top N per group
SELECT *
FROM (
    SELECT
        p.*,
        ROW_NUMBER() OVER (PARTITION BY p.category_id ORDER BY p.sales DESC) AS rn
    FROM products p
) ranked
WHERE rn <= 5;
```

**LATERAL Joins**:
```sql
-- Get latest order for each customer
SELECT c.id, c.name, latest.order_id, latest.order_date, latest.total_amount
FROM customers c
LEFT JOIN LATERAL (
    SELECT id AS order_id, order_date, total_amount
    FROM orders
    WHERE customer_id = c.id
    ORDER BY order_date DESC
    LIMIT 1
) latest ON true;

-- Top 3 products per category with details
SELECT cat.name AS category_name, top_products.*
FROM categories cat
LEFT JOIN LATERAL (
    SELECT p.id, p.name, p.price, p.sales
    FROM products p
    WHERE p.category_id = cat.id
    ORDER BY p.sales DESC
    LIMIT 3
) top_products ON true;
```

**JSONB Queries**:
```sql
-- Query JSONB fields
SELECT
    id,
    metadata->>'subscription_level' AS subscription,
    metadata->'preferences'->>'theme' AS theme,
    jsonb_array_elements_text(metadata->'tags') AS tag
FROM users
WHERE metadata @> '{"subscription_level": "premium"}'
  AND metadata ? 'preferences'
  AND jsonb_typeof(metadata->'tags') = 'array';

-- Update JSONB fields
UPDATE users
SET metadata = jsonb_set(
    metadata,
    '{preferences, notifications}',
    'true'::jsonb,
    true
)
WHERE id = 'some-uuid';

-- Remove JSONB key
UPDATE users
SET metadata = metadata - 'old_key'
WHERE id = 'some-uuid';
```

### 2.8 Performance Optimization

**EXPLAIN ANALYZE**:
```sql
-- Always profile before optimizing
EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON)
SELECT c.name, COUNT(o.id) AS order_count
FROM customers c
LEFT JOIN orders o ON c.id = o.customer_id
WHERE c.created_at > NOW() - INTERVAL '1 year'
GROUP BY c.id, c.name
HAVING COUNT(o.id) > 10
ORDER BY order_count DESC;

-- Look for:
-- 1. Sequential Scans on large tables (add indexes)
-- 2. High actual time vs planned time (statistics out of date)
-- 3. High buffer reads (add indexes, optimize queries)
-- 4. Nested loops with high row counts (rewrite join)
```

**Query Optimization Checklist**:
```sql
-- ✅ Use covering indexes
CREATE INDEX idx_orders_customer_summary
ON orders(customer_id, status) INCLUDE (order_date, total_amount);

-- ✅ Avoid SELECT *
SELECT id, name, email FROM users;  -- Good
SELECT * FROM users;  -- Bad (retrieves unnecessary columns)

-- ✅ Use LIMIT for pagination
SELECT * FROM products ORDER BY created_at DESC LIMIT 20 OFFSET 0;

-- ✅ Filter early in CTEs
WITH recent_orders AS (
    SELECT * FROM orders
    WHERE order_date > NOW() - INTERVAL '30 days'  -- Filter early
)
SELECT * FROM recent_orders WHERE status = 'completed';

-- ❌ Don't filter late
WITH all_orders AS (
    SELECT * FROM orders  -- Loads everything
)
SELECT * FROM all_orders
WHERE order_date > NOW() - INTERVAL '30 days'  -- Too late
  AND status = 'completed';
```

**Connection Pooling** (application-side with pgbouncer):
```ini
# /etc/pgbouncer/pgbouncer.ini
[databases]
mydb = host=localhost dbname=mydb

[pgbouncer]
listen_port = 6432
listen_addr = *
auth_type = md5
auth_file = /etc/pgbouncer/userlist.txt
pool_mode = transaction
max_client_conn = 1000
default_pool_size = 25
reserve_pool_size = 5
reserve_pool_timeout = 3
```

**Vacuuming and Maintenance**:
```sql
-- Autovacuum configuration (postgresql.conf)
autovacuum = on
autovacuum_max_workers = 4
autovacuum_vacuum_threshold = 50
autovacuum_vacuum_scale_factor = 0.1
autovacuum_analyze_threshold = 50
autovacuum_analyze_scale_factor = 0.05

-- Manual vacuum for maintenance windows
VACUUM ANALYZE VERBOSE;

-- Aggressive vacuum to reclaim space
VACUUM FULL ANALYZE users;  -- Locks table, use sparingly

-- Update statistics
ANALYZE users;
ANALYZE orders;

-- Check bloat
SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size,
    n_tup_upd + n_tup_del AS modifications,
    last_vacuum,
    last_autovacuum
FROM pg_stat_user_tables
ORDER BY modifications DESC;
```

---

## 3. Redis Design Patterns

### 3.1 Overview

**Redis Use Cases**:
- **Cache Layer**: Session storage, query results, computed data
- **Real-Time Analytics**: Counters, rate limiting, leaderboards
- **Message Broker**: Pub/Sub, task queues, event streaming
- **Distributed Lock**: Coordination across services
- **Geospatial**: Location-based queries

**Key Characteristics**:
- In-memory (fast reads/writes)
- Single-threaded (atomic operations)
- Data structures (not just key-value)
- Persistence optional (RDB snapshots, AOF log)

### 3.2 Data Structure Patterns

**Strings** - Simple values, counters, bitmaps:
```redis
# Basic key-value
SET user:1000:name "Alice"
GET user:1000:name

# With expiration (session storage)
SETEX session:abc123 3600 '{"user_id": 1000, "role": "admin"}'

# Atomic counter
INCR page:views:homepage
INCRBY product:1000:stock -5

# Bit operations for flags
SETBIT user:1000:features 0 1  # Enable feature flag 0
GETBIT user:1000:features 0  # Check feature flag 0
BITCOUNT user:1000:features  # Count enabled features
```

**Hashes** - Objects, structured data:
```redis
# Store user object
HSET user:1000 name "Alice" email "alice@example.com" age 30
HGETALL user:1000
HGET user:1000 email
HINCRBY user:1000 age 1

# Partial updates
HMSET product:2000 name "Laptop" price 999.99 stock 50
HGET product:2000 price
HINCRBY product:2000 stock -1
```

**Lists** - Queues, activity feeds, recent items:
```redis
# Task queue (FIFO)
LPUSH queue:emails '{"to": "user@example.com", "subject": "Welcome"}'
BRPOP queue:emails 0  # Blocking pop (worker pattern)

# Recent activity (capped list)
LPUSH user:1000:recent_views product:5000
LTRIM user:1000:recent_views 0 49  # Keep only 50 items
LRANGE user:1000:recent_views 0 9  # Get 10 most recent

# Stack (LIFO)
LPUSH stack:undo "action1"
LPOP stack:undo
```

**Sets** - Unique collections, tags, relationships:
```redis
# Tags
SADD post:100:tags "redis" "database" "nosql"
SMEMBERS post:100:tags
SISMEMBER post:100:tags "redis"  # Check membership

# Set operations
SADD user:1000:following user:2000 user:3000 user:4000
SADD user:2000:following user:1000 user:3000 user:5000
SINTER user:1000:following user:2000:following  # Mutual follows
SDIFF user:1000:following user:2000:following  # Follows 1000 but not 2000
SUNION user:1000:following user:2000:following  # All follows

# Random elements
SRANDMEMBER recommendations:products 5  # Get 5 random recommendations
```

**Sorted Sets** - Leaderboards, priority queues, time-based data:
```redis
# Leaderboard
ZADD leaderboard:global 1500 "player1" 1200 "player2" 1800 "player3"
ZREVRANGE leaderboard:global 0 9 WITHSCORES  # Top 10
ZRANK leaderboard:global "player1"  # Player rank
ZINCRBY leaderboard:global 100 "player1"  # Add score

# Time-series (score = timestamp)
ZADD events:user:1000 1704067200 "login" 1704070800 "purchase"
ZRANGEBYSCORE events:user:1000 1704067200 1704153600  # Events in time range
ZREMRANGEBYSCORE events:user:1000 0 1672531200  # Delete old events

# Priority queue
ZADD tasks:priority 1 "urgent-task" 5 "normal-task" 10 "low-priority-task"
ZPOPMIN tasks:priority  # Get highest priority task
```

**Streams** - Event log, message queue, time-series:
```redis
# Add to stream
XADD events:orders * user_id 1000 product_id 5000 amount 99.99

# Read from stream
XREAD COUNT 10 STREAMS events:orders 0-0  # Read from beginning
XREAD COUNT 10 BLOCK 5000 STREAMS events:orders $  # Block for new messages

# Consumer groups (multiple consumers)
XGROUP CREATE events:orders order-processor $ MKSTREAM
XREADGROUP GROUP order-processor consumer1 COUNT 1 STREAMS events:orders >
XACK events:orders order-processor <message-id>

# Trimming
XTRIM events:orders MAXLEN ~ 10000  # Keep ~10k messages
```

### 3.3 Caching Patterns

**Cache-Aside (Lazy Loading)**:
```python
def get_user(user_id):
    # Try cache first
    cache_key = f"user:{user_id}"
    cached = redis.get(cache_key)

    if cached:
        return json.loads(cached)

    # Cache miss - load from database
    user = db.query("SELECT * FROM users WHERE id = %s", (user_id,))

    # Store in cache with TTL
    redis.setex(cache_key, 3600, json.dumps(user))

    return user

def update_user(user_id, data):
    # Update database
    db.execute("UPDATE users SET ... WHERE id = %s", (user_id,))

    # Invalidate cache
    redis.delete(f"user:{user_id}")
```

**Write-Through**:
```python
def update_user(user_id, data):
    # Update database
    db.execute("UPDATE users SET ... WHERE id = %s", (user_id,))

    # Update cache immediately
    cache_key = f"user:{user_id}"
    user = db.query("SELECT * FROM users WHERE id = %s", (user_id,))
    redis.setex(cache_key, 3600, json.dumps(user))
```

**Write-Behind (Write-Back)**:
```python
def update_user(user_id, data):
    # Update cache immediately
    cache_key = f"user:{user_id}"
    redis.hset(cache_key, mapping=data)

    # Queue for async database write
    redis.lpush("write_queue:users", json.dumps({"id": user_id, "data": data}))

# Background worker
def write_worker():
    while True:
        _, payload = redis.brpop("write_queue:users", timeout=5)
        if payload:
            data = json.loads(payload)
            db.execute("UPDATE users SET ... WHERE id = %s", (data["id"],))
```

**Cache Stampede Prevention (Locking)**:
```python
import time

def get_expensive_data(key):
    cache_key = f"data:{key}"
    lock_key = f"lock:{key}"

    # Try cache
    cached = redis.get(cache_key)
    if cached:
        return json.loads(cached)

    # Acquire lock
    lock_acquired = redis.set(lock_key, "1", nx=True, ex=10)

    if lock_acquired:
        try:
            # Compute expensive data
            data = compute_expensive_data(key)
            redis.setex(cache_key, 3600, json.dumps(data))
            return data
        finally:
            redis.delete(lock_key)
    else:
        # Another process is computing, wait and retry
        time.sleep(0.1)
        return get_expensive_data(key)
```

### 3.4 Rate Limiting

**Fixed Window**:
```python
def rate_limit_fixed_window(user_id, limit=100, window=60):
    key = f"rate_limit:{user_id}:{int(time.time() // window)}"
    current = redis.incr(key)

    if current == 1:
        redis.expire(key, window)

    return current <= limit
```

**Sliding Window Log**:
```python
def rate_limit_sliding_log(user_id, limit=100, window=60):
    key = f"rate_limit:log:{user_id}"
    now = time.time()
    window_start = now - window

    # Remove old entries
    redis.zremrangebyscore(key, 0, window_start)

    # Count requests in window
    count = redis.zcard(key)

    if count < limit:
        # Add current request
        redis.zadd(key, {str(now): now})
        redis.expire(key, window)
        return True

    return False
```

**Token Bucket**:
```lua
-- token_bucket.lua
local key = KEYS[1]
local capacity = tonumber(ARGV[1])
local refill_rate = tonumber(ARGV[2])
local now = tonumber(ARGV[3])

local tokens = tonumber(redis.call('HGET', key, 'tokens') or capacity)
local last_refill = tonumber(redis.call('HGET', key, 'last_refill') or now)

-- Refill tokens
local elapsed = now - last_refill
local refilled = math.floor(elapsed * refill_rate)
tokens = math.min(capacity, tokens + refilled)

if tokens >= 1 then
    tokens = tokens - 1
    redis.call('HSET', key, 'tokens', tokens)
    redis.call('HSET', key, 'last_refill', now)
    redis.call('EXPIRE', key, math.ceil(capacity / refill_rate))
    return 1
else
    return 0
end
```

```python
# Usage
script_sha = redis.script_load(token_bucket_lua)
allowed = redis.evalsha(script_sha, 1, f"bucket:{user_id}", 100, 10, time.time())
```

### 3.5 Pub/Sub Patterns

**Basic Pub/Sub**:
```python
# Publisher
redis.publish("notifications", json.dumps({
    "type": "new_message",
    "user_id": 1000,
    "message": "Hello"
}))

# Subscriber
pubsub = redis.pubsub()
pubsub.subscribe("notifications")

for message in pubsub.listen():
    if message['type'] == 'message':
        data = json.loads(message['data'])
        process_notification(data)
```

**Pattern Subscriptions**:
```python
# Subscribe to multiple channels with pattern
pubsub = redis.pubsub()
pubsub.psubscribe("user:*:notifications")

for message in pubsub.listen():
    if message['type'] == 'pmessage':
        channel = message['channel']  # e.g., "user:1000:notifications"
        data = json.loads(message['data'])
        process_notification(channel, data)
```

**Fan-Out Pattern**:
```python
# Notify all connected clients
def notify_all(event_type, payload):
    redis.publish("broadcast", json.dumps({
        "event": event_type,
        "data": payload,
        "timestamp": time.time()
    }))
```

### 3.6 Distributed Locking

**Redlock Algorithm**:
```python
import uuid
import time

def acquire_lock(lock_name, timeout=10):
    identifier = str(uuid.uuid4())
    lock_key = f"lock:{lock_name}"

    # SET with NX (only if not exists) and EX (expiration)
    acquired = redis.set(lock_key, identifier, nx=True, ex=timeout)

    return identifier if acquired else None

def release_lock(lock_name, identifier):
    lock_key = f"lock:{lock_name}"

    # Lua script for atomic check-and-delete
    lua_script = """
    if redis.call("GET", KEYS[1]) == ARGV[1] then
        return redis.call("DEL", KEYS[1])
    else
        return 0
    end
    """

    return redis.eval(lua_script, 1, lock_key, identifier)

# Usage
lock_id = acquire_lock("process:critical_section")
if lock_id:
    try:
        # Critical section
        perform_critical_operation()
    finally:
        release_lock("process:critical_section", lock_id)
else:
    # Lock not acquired, handle accordingly
    pass
```

**Lock with Auto-Renewal**:
```python
import threading

class AutoRenewLock:
    def __init__(self, redis_client, lock_name, timeout=10):
        self.redis = redis_client
        self.lock_name = lock_name
        self.timeout = timeout
        self.identifier = str(uuid.uuid4())
        self.renew_thread = None
        self.stop_renew = threading.Event()

    def acquire(self):
        lock_key = f"lock:{self.lock_name}"
        acquired = self.redis.set(lock_key, self.identifier, nx=True, ex=self.timeout)

        if acquired:
            # Start renewal thread
            self.renew_thread = threading.Thread(target=self._renew_lock)
            self.renew_thread.daemon = True
            self.renew_thread.start()
            return True

        return False

    def _renew_lock(self):
        while not self.stop_renew.is_set():
            time.sleep(self.timeout / 2)  # Renew at half the timeout
            lock_key = f"lock:{self.lock_name}"

            lua_script = """
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return redis.call("EXPIRE", KEYS[1], ARGV[2])
            else
                return 0
            end
            """

            renewed = self.redis.eval(lua_script, 1, lock_key, self.identifier, self.timeout)
            if not renewed:
                break

    def release(self):
        self.stop_renew.set()
        if self.renew_thread:
            self.renew_thread.join(timeout=1)

        lock_key = f"lock:{self.lock_name}"
        lua_script = """
        if redis.call("GET", KEYS[1]) == ARGV[1] then
            return redis.call("DEL", KEYS[1])
        else
            return 0
        end
        """
        self.redis.eval(lua_script, 1, lock_key, self.identifier)
```

### 3.7 Persistence Configuration

**RDB (Snapshotting)**:
```conf
# redis.conf
save 900 1      # Save if 1 key changed in 900s
save 300 10     # Save if 10 keys changed in 300s
save 60 10000   # Save if 10000 keys changed in 60s

dbfilename dump.rdb
dir /var/lib/redis

# Compression
rdbcompression yes
rdbchecksum yes
```

**AOF (Append-Only File)**:
```conf
# redis.conf
appendonly yes
appendfilename "appendonly.aof"

# Fsync policy
appendfsync everysec    # Good balance (default)
# appendfsync always    # Slowest, safest
# appendfsync no        # Fastest, least safe

# AOF rewrite
auto-aof-rewrite-percentage 100
auto-aof-rewrite-min-size 64mb
```

**Hybrid Persistence** (RDB + AOF):
```conf
# Use both for maximum durability
save 900 1
appendonly yes
appendfsync everysec

# Load AOF on startup if available (more complete)
aof-use-rdb-preamble yes
```

---

## 4. MongoDB Design Patterns

### 4.1 Overview

**MongoDB Use Cases**:
- **Document Storage**: Content management, catalogs, user profiles
- **Real-Time Analytics**: Event tracking, metrics aggregation
- **Flexible Schema**: Rapid prototyping, evolving data models
- **Hierarchical Data**: Nested structures, trees, graphs

**Key Characteristics**:
- Document-oriented (BSON format)
- Schema-less (flexible structure)
- Horizontal scaling (sharding)
- Rich query language
- Secondary indexes

### 4.2 Document Modeling

**Embedding vs. Referencing**:

**Embed When** (One-to-Few, One-to-Many with bounded growth):
```javascript
// ✅ Embed comments (bounded, accessed together)
{
  "_id": ObjectId("..."),
  "title": "MongoDB Design Patterns",
  "author": "Alice",
  "content": "...",
  "comments": [
    {
      "author": "Bob",
      "text": "Great article!",
      "date": ISODate("2024-01-07T10:00:00Z")
    },
    {
      "author": "Charlie",
      "text": "Very helpful",
      "date": ISODate("2024-01-07T11:00:00Z")
    }
  ],
  "created_at": ISODate("2024-01-07T09:00:00Z")
}
```

**Reference When** (One-to-Many unbounded, Many-to-Many):
```javascript
// ✅ Reference users (unbounded, independent access)
// Users collection
{
  "_id": ObjectId("user_123"),
  "name": "Alice",
  "email": "alice@example.com"
}

// Posts collection
{
  "_id": ObjectId("post_456"),
  "title": "MongoDB Design",
  "author_id": ObjectId("user_123"),  // Reference
  "content": "..."
}

// Comments collection (unbounded growth)
{
  "_id": ObjectId("comment_789"),
  "post_id": ObjectId("post_456"),  // Reference
  "author_id": ObjectId("user_123"),  // Reference
  "text": "Great post!",
  "created_at": ISODate("2024-01-07T10:00:00Z")
}
```

**Hybrid Approach** (Embed summary, reference full):
```javascript
// ✅ Embed author summary, reference for full profile
{
  "_id": ObjectId("post_456"),
  "title": "MongoDB Design",
  "author": {
    "id": ObjectId("user_123"),
    "name": "Alice",
    "avatar_url": "https://..."
  },
  "content": "...",
  "stats": {
    "views": 1500,
    "likes": 42,
    "comment_count": 10
  }
}
```

### 4.3 Schema Patterns

**Polymorphic Pattern** (Multiple types in one collection):
```javascript
// Products with different types
{
  "_id": ObjectId("..."),
  "type": "book",
  "name": "MongoDB Patterns",
  "price": 29.99,
  // Book-specific fields
  "isbn": "978-1234567890",
  "author": "Alice Smith",
  "pages": 350
}

{
  "_id": ObjectId("..."),
  "type": "electronics",
  "name": "Laptop",
  "price": 999.99,
  // Electronics-specific fields
  "brand": "TechCo",
  "warranty_months": 24,
  "specs": {
    "cpu": "Intel i7",
    "ram_gb": 16
  }
}

// Query by type
db.products.find({ type: "book" })
db.products.createIndex({ type: 1, name: 1 })
```

**Attribute Pattern** (Sparse, varying attributes):
```javascript
// Instead of flat fields that are mostly null
// ❌ Bad (sparse)
{
  "_id": ObjectId("..."),
  "name": "Product",
  "color": "red",
  "size": null,
  "weight": null,
  "material": "plastic",
  "voltage": null
}

// ✅ Good (attribute array)
{
  "_id": ObjectId("..."),
  "name": "Product",
  "attributes": [
    { "k": "color", "v": "red" },
    { "k": "material", "v": "plastic" }
  ]
}

// Index on attributes
db.products.createIndex({ "attributes.k": 1, "attributes.v": 1 })

// Query
db.products.find({ attributes: { $elemMatch: { k: "color", v: "red" } } })
```

**Bucket Pattern** (Time-series, IoT data):
```javascript
// Instead of one document per reading
// ❌ Bad (millions of documents)
{
  "_id": ObjectId("..."),
  "sensor_id": "sensor_001",
  "temperature": 22.5,
  "timestamp": ISODate("2024-01-07T10:00:00Z")
}

// ✅ Good (bucketed by hour)
{
  "_id": ObjectId("..."),
  "sensor_id": "sensor_001",
  "date": ISODate("2024-01-07T10:00:00Z"),  // Start of hour
  "measurements": [
    { "ts": ISODate("2024-01-07T10:00:15Z"), "temp": 22.5 },
    { "ts": ISODate("2024-01-07T10:01:15Z"), "temp": 22.7 },
    { "ts": ISODate("2024-01-07T10:02:15Z"), "temp": 22.6 }
    // ... up to 3600 readings per hour
  ],
  "summary": {
    "count": 3,
    "avg_temp": 22.6,
    "min_temp": 22.5,
    "max_temp": 22.7
  }
}

// Index
db.sensor_data.createIndex({ sensor_id: 1, date: 1 })
```

**Computed Pattern** (Pre-computed aggregations):
```javascript
// Real-time updates with pre-computed stats
{
  "_id": ObjectId("..."),
  "product_id": ObjectId("prod_123"),
  "reviews": [
    { "user_id": ObjectId("..."), "rating": 5, "text": "Excellent!" },
    { "user_id": ObjectId("..."), "rating": 4, "text": "Good" }
  ],
  "stats": {
    "total_reviews": 2,
    "avg_rating": 4.5,
    "rating_distribution": {
      "5": 1,
      "4": 1,
      "3": 0,
      "2": 0,
      "1": 0
    }
  },
  "last_updated": ISODate("2024-01-07T10:00:00Z")
}

// Update with computed stats
db.products.updateOne(
  { _id: ObjectId("...") },
  {
    $push: { reviews: newReview },
    $inc: {
      "stats.total_reviews": 1,
      [`stats.rating_distribution.${newReview.rating}`]: 1
    },
    $set: { last_updated: new Date() }
  }
)

// Recalculate avg_rating periodically or with change streams
```

**Outlier Pattern** (Handle rare large documents separately):
```javascript
// Most orders have few items
{
  "_id": ObjectId("order_123"),
  "customer_id": ObjectId("..."),
  "items": [
    { "product_id": ObjectId("..."), "qty": 2, "price": 29.99 },
    { "product_id": ObjectId("..."), "qty": 1, "price": 49.99 }
  ],
  "total": 109.97
}

// Large orders (outliers) split into separate documents
{
  "_id": ObjectId("order_456"),
  "customer_id": ObjectId("..."),
  "has_overflow": true,
  "items": [/* first 100 items */],
  "total": 5499.99
}

// Overflow documents
{
  "_id": ObjectId("order_456_overflow_1"),
  "parent_order_id": ObjectId("order_456"),
  "items": [/* next 100 items */]
}
```

### 4.4 Indexing Strategies

**Single Field Index**:
```javascript
db.users.createIndex({ email: 1 })  // Ascending
db.products.createIndex({ price: -1 })  // Descending
```

**Compound Index**:
```javascript
// Order matters for query support
db.orders.createIndex({ customer_id: 1, order_date: -1, status: 1 })

// Supports:
// { customer_id: X }
// { customer_id: X, order_date: Y }
// { customer_id: X, order_date: Y, status: Z }

// Does NOT support:
// { order_date: Y }  // Missing prefix (customer_id)
// { status: Z }  // Missing prefix
```

**Multikey Index** (Arrays):
```javascript
// Index array elements
db.posts.createIndex({ tags: 1 })

// Query
db.posts.find({ tags: "mongodb" })  // Matches if "mongodb" in tags array
```

**Text Index** (Full-text search):
```javascript
// Create text index
db.articles.createIndex({
  title: "text",
  content: "text"
})

// Search
db.articles.find({ $text: { $search: "mongodb design patterns" } })

// With score
db.articles.find(
  { $text: { $search: "mongodb design patterns" } },
  { score: { $meta: "textScore" } }
).sort({ score: { $meta: "textScore" } })
```

**Geospatial Index**:
```javascript
// 2dsphere for Earth-like geometry
db.locations.createIndex({ coordinates: "2dsphere" })

// Store location
db.locations.insertOne({
  name: "Coffee Shop",
  coordinates: {
    type: "Point",
    coordinates: [-73.97, 40.77]  // [longitude, latitude]
  }
})

// Query nearby
db.locations.find({
  coordinates: {
    $near: {
      $geometry: {
        type: "Point",
        coordinates: [-73.98, 40.76]
      },
      $maxDistance: 1000  // meters
    }
  }
})
```

**Partial Index** (Filtered):
```javascript
// Index only active users
db.users.createIndex(
  { email: 1 },
  { partialFilterExpression: { active: true } }
)

// Index only premium subscriptions
db.users.createIndex(
  { subscription_end_date: 1 },
  { partialFilterExpression: { subscription_level: "premium" } }
)
```

**TTL Index** (Auto-expiration):
```javascript
// Expire documents after 30 days
db.sessions.createIndex(
  { created_at: 1 },
  { expireAfterSeconds: 2592000 }  // 30 days
)

// Document will be deleted when created_at + 30 days < now
```

### 4.5 Aggregation Pipeline

**Basic Aggregation**:
```javascript
// Group and aggregate
db.orders.aggregate([
  { $match: { status: "completed" } },
  { $group: {
      _id: "$customer_id",
      total_orders: { $sum: 1 },
      total_spent: { $sum: "$total_amount" },
      avg_order_value: { $avg: "$total_amount" }
  }},
  { $sort: { total_spent: -1 } },
  { $limit: 10 }
])
```

**Lookup (Join)**:
```javascript
// Left join orders with customers
db.orders.aggregate([
  {
    $lookup: {
      from: "customers",
      localField: "customer_id",
      foreignField: "_id",
      as: "customer"
    }
  },
  { $unwind: "$customer" },
  {
    $project: {
      order_id: "$_id",
      order_date: 1,
      total_amount: 1,
      customer_name: "$customer.name",
      customer_email: "$customer.email"
    }
  }
])
```

**Faceted Search**:
```javascript
// Multiple aggregations in one query
db.products.aggregate([
  {
    $facet: {
      // Price ranges
      "price_ranges": [
        {
          $bucket: {
            groupBy: "$price",
            boundaries: [0, 50, 100, 500, 1000, 5000],
            default: "Other",
            output: { count: { $sum: 1 }, avg_price: { $avg: "$price" } }
          }
        }
      ],
      // Top categories
      "categories": [
        { $sortByCount: "$category" },
        { $limit: 5 }
      ],
      // Rating distribution
      "ratings": [
        {
          $group: {
            _id: { $floor: "$rating" },
            count: { $sum: 1 }
          }
        },
        { $sort: { _id: -1 } }
      ]
    }
  }
])
```

**Time-Series Aggregation**:
```javascript
// Daily sales summary
db.orders.aggregate([
  { $match: {
      order_date: {
        $gte: ISODate("2024-01-01"),
        $lt: ISODate("2024-02-01")
      }
  }},
  {
    $group: {
      _id: {
        year: { $year: "$order_date" },
        month: { $month: "$order_date" },
        day: { $dayOfMonth: "$order_date" }
      },
      daily_revenue: { $sum: "$total_amount" },
      order_count: { $sum: 1 },
      unique_customers: { $addToSet: "$customer_id" }
    }
  },
  {
    $project: {
      date: {
        $dateFromParts: {
          year: "$_id.year",
          month: "$_id.month",
          day: "$_id.day"
        }
      },
      daily_revenue: 1,
      order_count: 1,
      customer_count: { $size: "$unique_customers" }
    }
  },
  { $sort: { date: 1 } }
])
```

---

## 5. Streaming Patterns

### 5.1 PostgreSQL Logical Replication & Change Data Capture

**Logical Replication Setup**:
```sql
-- On primary server (publisher)
ALTER SYSTEM SET wal_level = 'logical';
ALTER SYSTEM SET max_replication_slots = 10;
-- Restart PostgreSQL

-- Create publication
CREATE PUBLICATION my_publication FOR TABLE users, orders, products;

-- Or publish all tables
CREATE PUBLICATION all_tables FOR ALL TABLES;

-- On replica server (subscriber)
CREATE SUBSCRIPTION my_subscription
CONNECTION 'host=primary.example.com port=5432 dbname=mydb user=repl_user password=...'
PUBLICATION my_publication;

-- Check replication status
SELECT * FROM pg_stat_subscription;
SELECT * FROM pg_replication_slots;
```

**CDC with Debezium (Kafka Connect)**:
```yaml
# debezium-postgres-connector.json
{
  "name": "postgres-connector",
  "config": {
    "connector.class": "io.debezium.connector.postgresql.PostgresConnector",
    "database.hostname": "localhost",
    "database.port": "5432",
    "database.user": "debezium",
    "database.password": "secret",
    "database.dbname": "mydb",
    "database.server.name": "mydb",
    "table.include.list": "public.users,public.orders",
    "plugin.name": "pgoutput",
    "publication.name": "dbz_publication",
    "slot.name": "dbz_slot"
  }
}
```

**Change Stream Consumer** (Python):
```python
from kafka import KafkaConsumer
import json

consumer = KafkaConsumer(
    'mydb.public.users',
    'mydb.public.orders',
    bootstrap_servers=['localhost:9092'],
    value_deserializer=lambda m: json.loads(m.decode('utf-8')),
    group_id='cdc-processor'
)

for message in consumer:
    event = message.value
    
    if event['op'] == 'c':  # Create
        handle_insert(event['after'])
    elif event['op'] == 'u':  # Update
        handle_update(event['before'], event['after'])
    elif event['op'] == 'd':  # Delete
        handle_delete(event['before'])
```

**Logical Decoding with pg_recvlogical**:
```bash
# Create replication slot
psql -c "SELECT * FROM pg_create_logical_replication_slot('my_slot', 'test_decoding');"

# Stream changes
pg_recvlogical -d mydb --slot my_slot --start -f - | while read line; do
    echo "Change: $line"
    # Process change event
done
```

**PostgreSQL LISTEN/NOTIFY**:
```sql
-- Trigger for notifications
CREATE OR REPLACE FUNCTION notify_order_changes()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        PERFORM pg_notify('order_channel', json_build_object(
            'operation', 'INSERT',
            'order_id', NEW.id,
            'customer_id', NEW.customer_id,
            'amount', NEW.total_amount
        )::text);
    ELSIF TG_OP = 'UPDATE' THEN
        PERFORM pg_notify('order_channel', json_build_object(
            'operation', 'UPDATE',
            'order_id', NEW.id,
            'old_status', OLD.status,
            'new_status', NEW.status
        )::text);
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER order_changes
AFTER INSERT OR UPDATE ON orders
FOR EACH ROW EXECUTE FUNCTION notify_order_changes();
```

```python
# Python consumer
import psycopg2
import select
import json

conn = psycopg2.connect("dbname=mydb")
conn.set_isolation_level(psycopg2.extensions.ISOLATION_LEVEL_AUTOCOMMIT)

cursor = conn.cursor()
cursor.execute("LISTEN order_channel;")

print("Listening for order changes...")

while True:
    if select.select([conn], [], [], 5) == ([], [], []):
        continue  # Timeout
    
    conn.poll()
    while conn.notifies:
        notify = conn.notifies.pop(0)
        payload = json.loads(notify.payload)
        print(f"Received: {payload}")
        process_order_change(payload)
```

### 5.2 Redis Streams

**Producer**:
```python
import redis
import json
import time

r = redis.Redis(host='localhost', port=6379, decode_responses=True)

# Add events to stream
def publish_event(event_type, data):
    event_id = r.xadd(
        'events:orders',
        {
            'type': event_type,
            'data': json.dumps(data),
            'timestamp': time.time()
        },
        maxlen=100000  # Trim to 100k messages
    )
    return event_id

# Usage
event_id = publish_event('order_created', {
    'order_id': '12345',
    'customer_id': '67890',
    'amount': 99.99
})
```

**Consumer (Single)**:
```python
# Read from stream (blocking)
def consume_events():
    last_id = '0-0'  # Start from beginning
    
    while True:
        # Block for 5 seconds waiting for new messages
        messages = r.xread(
            {'events:orders': last_id},
            count=10,
            block=5000
        )
        
        if not messages:
            continue
        
        for stream_name, stream_messages in messages:
            for message_id, data in stream_messages:
                event_type = data['type']
                payload = json.loads(data['data'])
                
                print(f"Processing {event_type}: {payload}")
                process_event(event_type, payload)
                
                last_id = message_id
```

**Consumer Groups** (Multiple consumers, load balanced):
```python
# Create consumer group
try:
    r.xgroup_create('events:orders', 'order-processor', id='0', mkstream=True)
except redis.exceptions.ResponseError:
    pass  # Group already exists

def consume_with_group(consumer_name):
    while True:
        # Read new messages for this consumer
        messages = r.xreadgroup(
            'order-processor',
            consumer_name,
            {'events:orders': '>'},
            count=1,
            block=5000
        )
        
        if not messages:
            # Check for pending messages
            pending = r.xpending_range(
                'events:orders',
                'order-processor',
                '-', '+',
                count=10,
                consumername=consumer_name
            )
            
            if pending:
                # Reclaim abandoned messages
                for p in pending:
                    messages = r.xclaim(
                        'events:orders',
                        'order-processor',
                        consumer_name,
                        min_idle_time=60000,  # 60 seconds
                        message_ids=[p['message_id']]
                    )
                    # Process reclaimed messages
            continue
        
        for stream_name, stream_messages in messages:
            for message_id, data in stream_messages:
                try:
                    event_type = data['type']
                    payload = json.loads(data['data'])
                    
                    process_event(event_type, payload)
                    
                    # Acknowledge successful processing
                    r.xack('events:orders', 'order-processor', message_id)
                except Exception as e:
                    print(f"Error processing {message_id}: {e}")
                    # Message remains pending for retry

# Run multiple consumers
import threading

for i in range(3):
    t = threading.Thread(target=consume_with_group, args=(f'consumer-{i}',))
    t.daemon = True
    t.start()
```

**Stream Monitoring**:
```python
# Get stream info
info = r.xinfo_stream('events:orders')
print(f"Length: {info['length']}")
print(f"First entry: {info['first-entry']}")
print(f"Last entry: {info['last-entry']}")

# Get consumer group info
groups = r.xinfo_groups('events:orders')
for group in groups:
    print(f"Group: {group['name']}, Pending: {group['pending']}")
    
    # Get consumers in group
    consumers = r.xinfo_consumers('events:orders', group['name'])
    for consumer in consumers:
        print(f"  Consumer: {consumer['name']}, Pending: {consumer['pending']}")
```

### 5.3 MongoDB Change Streams

**Watch Collection**:
```javascript
// Node.js
const { MongoClient } = require('mongodb');

async function watchOrders() {
  const client = new MongoClient('mongodb://localhost:27017');
  await client.connect();
  
  const db = client.db('mydb');
  const collection = db.collection('orders');
  
  // Open change stream
  const changeStream = collection.watch([
    { $match: { 'operationType': { $in: ['insert', 'update', 'delete'] } } }
  ]);
  
  console.log('Watching for changes...');
  
  changeStream.on('change', (change) => {
    console.log('Change detected:', change);
    
    switch (change.operationType) {
      case 'insert':
        handleInsert(change.fullDocument);
        break;
      case 'update':
        handleUpdate(change.documentKey, change.updateDescription);
        break;
      case 'delete':
        handleDelete(change.documentKey);
        break;
    }
  });
  
  changeStream.on('error', (error) => {
    console.error('Change stream error:', error);
  });
}

watchOrders();
```

**Resume After Interruption**:
```javascript
const fs = require('fs');

async function watchWithResume() {
  const client = new MongoClient('mongodb://localhost:27017');
  await client.connect();
  
  const collection = client.db('mydb').collection('orders');
  
  // Load resume token from file
  let resumeToken;
  try {
    resumeToken = JSON.parse(fs.readFileSync('resume-token.json', 'utf8'));
  } catch (e) {
    // No resume token, start from now
  }
  
  const options = resumeToken ? { resumeAfter: resumeToken } : {};
  const changeStream = collection.watch([], options);
  
  changeStream.on('change', (change) => {
    // Process change
    processChange(change);
    
    // Save resume token
    fs.writeFileSync('resume-token.json', JSON.stringify(change._id));
  });
}
```

**Aggregation Pipeline Filtering**:
```javascript
// Watch with aggregation pipeline
const pipeline = [
  // Only watch updates to specific fields
  {
    $match: {
      'operationType': 'update',
      'updateDescription.updatedFields.status': { $exists: true }
    }
  },
  // Project only needed fields
  {
    $project: {
      '_id': 1,
      'fullDocument._id': 1,
      'fullDocument.status': 1,
      'fullDocument.customer_id': 1,
      'updateDescription': 1
    }
  }
];

const changeStream = collection.watch(pipeline);

changeStream.on('change', (change) => {
  console.log(`Order ${change.fullDocument._id} status changed to ${change.fullDocument.status}`);
  notifyCustomer(change.fullDocument.customer_id, change.fullDocument.status);
});
```

**Watch Entire Database**:
```javascript
// Watch all collections in database
const db = client.db('mydb');
const changeStream = db.watch();

changeStream.on('change', (change) => {
  console.log(`Change in ${change.ns.coll}: ${change.operationType}`);
  
  // Route to appropriate handler based on collection
  switch (change.ns.coll) {
    case 'users':
      handleUserChange(change);
      break;
    case 'orders':
      handleOrderChange(change);
      break;
    case 'products':
      handleProductChange(change);
      break;
  }
});
```

**Python Change Streams**:
```python
from pymongo import MongoClient
from pymongo.errors import PyMongoError

client = MongoClient('mongodb://localhost:27017/')
db = client['mydb']
collection = db['orders']

# Watch with pipeline
pipeline = [
    {'$match': {'operationType': {'$in': ['insert', 'update']}}}
]

try:
    with collection.watch(pipeline) as stream:
        for change in stream:
            operation = change['operationType']
            
            if operation == 'insert':
                doc = change['fullDocument']
                print(f"New order: {doc['_id']}")
                process_new_order(doc)
            elif operation == 'update':
                doc_id = change['documentKey']['_id']
                updated_fields = change['updateDescription']['updatedFields']
                print(f"Order {doc_id} updated: {updated_fields}")
                process_order_update(doc_id, updated_fields)
except PyMongoError as e:
    print(f"Change stream error: {e}")
```

### 5.4 Cross-Database Event Streaming

**Event-Driven Architecture Pattern**:
```
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│  PostgreSQL │─────>│    Kafka    │<─────│   MongoDB   │
│    (OLTP)   │      │  (Events)   │      │  (Analytics)│
└─────────────┘      └─────────────┘      └─────────────┘
       │                    │                     │
       │                    v                     │
       │             ┌─────────────┐             │
       └────────────>│    Redis    │<────────────┘
                     │   (Cache)   │
                     └─────────────┘
```

**Unified Event Producer** (Python):
```python
from kafka import KafkaProducer
import json

producer = KafkaProducer(
    bootstrap_servers=['localhost:9092'],
    value_serializer=lambda v: json.dumps(v).encode('utf-8')
)

class EventBus:
    def publish(self, topic, event):
        producer.send(topic, event)
        producer.flush()
    
    def publish_order_created(self, order):
        self.publish('orders.created', {
            'order_id': str(order['id']),
            'customer_id': str(order['customer_id']),
            'amount': float(order['total_amount']),
            'timestamp': order['created_at'].isoformat()
        })
    
    def publish_user_updated(self, user_id, changes):
        self.publish('users.updated', {
            'user_id': str(user_id),
            'changes': changes,
            'timestamp': datetime.utcnow().isoformat()
        })
```

**Unified Event Consumer**:
```python
from kafka import KafkaConsumer
import redis
import psycopg2
from pymongo import MongoClient

# Initialize connections
redis_client = redis.Redis(host='localhost', decode_responses=True)
pg_conn = psycopg2.connect("dbname=mydb")
mongo_client = MongoClient('mongodb://localhost:27017/')

consumer = KafkaConsumer(
    'orders.created',
    'users.updated',
    bootstrap_servers=['localhost:9092'],
    group_id='event-sync-processor',
    value_deserializer=lambda m: json.loads(m.decode('utf-8'))
)

for message in consumer:
    topic = message.topic
    event = message.value
    
    if topic == 'orders.created':
        # Invalidate cache
        redis_client.delete(f"order:{event['order_id']}")
        redis_client.delete(f"customer:{event['customer_id']}:orders")
        
        # Update analytics in MongoDB
        mongo_client.mydb.order_events.insert_one(event)
        
        # Trigger notifications
        notify_order_created(event)
    
    elif topic == 'users.updated':
        # Invalidate user cache
        redis_client.delete(f"user:{event['user_id']}")
        
        # Sync to MongoDB for analytics
        mongo_client.mydb.user_events.insert_one(event)
```

---

## 6. Polyglot Persistence Strategy

### 6.1 Database Selection Matrix

| Use Case | PostgreSQL | Redis | MongoDB |
|----------|-----------|-------|---------|
| **Transactional Data** | ✅ Primary | ❌ | ⚠️ Secondary |
| **User Sessions** | ❌ | ✅ Primary | ❌ |
| **Real-time Analytics** | ⚠️ Heavy | ⚠️ Limited | ✅ Primary |
| **Full-text Search** | ⚠️ Basic | ❌ | ✅ Good |
| **Geospatial Queries** | ✅ PostGIS | ⚠️ Basic | ✅ Native |
| **Time-series Data** | ⚠️ TimescaleDB | ❌ | ✅ Primary |
| **Graph Relationships** | ⚠️ Recursive | ❌ | ⚠️ References |
| **Cache Layer** | ❌ | ✅ Primary | ❌ |
| **Message Queue** | ⚠️ LISTEN/NOTIFY | ✅ Streams | ⚠️ Capped |
| **Document Storage** | ⚠️ JSONB | ❌ | ✅ Primary |

### 6.2 Consistency Models

**PostgreSQL** - ACID, Strong Consistency:
- Immediate consistency
- SERIALIZABLE isolation
- Use for: Financial transactions, inventory, user accounts

**Redis** - AP (Availability + Partition Tolerance):
- Eventual consistency (with replication)
- Single-threaded atomicity
- Use for: Caching, rate limiting, leaderboards

**MongoDB** - Tunable Consistency:
- Write Concern: `majority`, `1`, `0`
- Read Concern: `majority`, `local`, `available`
- Use for: Content management, analytics, flexible schemas

### 6.3 Integration Patterns

**Pattern 1: PostgreSQL (Source of Truth) + Redis (Cache)**:
```python
class UserService:
    def get_user(self, user_id):
        # Try cache
        cached = redis.get(f"user:{user_id}")
        if cached:
            return json.loads(cached)
        
        # Load from PostgreSQL
        user = pg.query_one("SELECT * FROM users WHERE id = %s", (user_id,))
        
        # Cache for 1 hour
        redis.setex(f"user:{user_id}", 3600, json.dumps(user))
        
        return user
    
    def update_user(self, user_id, data):
        # Update PostgreSQL (source of truth)
        pg.execute("UPDATE users SET ... WHERE id = %s", (user_id,))
        
        # Invalidate cache
        redis.delete(f"user:{user_id}")
```

**Pattern 2: PostgreSQL (OLTP) + MongoDB (Analytics)**:
```python
# CDC from PostgreSQL to MongoDB
def sync_order_to_analytics(order):
    # Denormalized document for analytics
    mongo.db.orders_analytics.insert_one({
        '_id': order['id'],
        'customer': {
            'id': order['customer_id'],
            'name': order['customer_name'],
            'email': order['customer_email']
        },
        'items': order['items'],
        'total': order['total_amount'],
        'date': order['created_at'],
        'year': order['created_at'].year,
        'month': order['created_at'].month,
        'day': order['created_at'].day
    })
```

**Pattern 3: Three-Tier (PostgreSQL + Redis + MongoDB)**:
```python
class OrderService:
    def create_order(self, order_data):
        # 1. Write to PostgreSQL (source of truth)
        with pg.transaction():
            order_id = pg.insert("INSERT INTO orders (...) VALUES (...) RETURNING id")
            pg.insert("INSERT INTO order_items (...) VALUES (...)")
        
        # 2. Invalidate related caches
        redis.delete(f"customer:{order_data['customer_id']}:orders")
        
        # 3. Publish event for MongoDB analytics
        event_bus.publish('orders.created', {
            'order_id': order_id,
            'customer_id': order_data['customer_id'],
            'amount': order_data['total_amount']
        })
        
        return order_id
```

### 6.4 Migration Strategies

**Zero-Downtime Migration**:
```
Phase 1: Dual Writes
┌─────────┐
│   App   │─────> PostgreSQL (old)
└─────────┘   └─> MongoDB (new)

Phase 2: Verify & Compare
┌─────────┐
│   App   │─────> PostgreSQL ──┐
└─────────┘   └─> MongoDB      │
                                └─> Validator

Phase 3: Read Migration
┌─────────┐     ┌──> PostgreSQL (fallback)
│   App   │─────┤
└─────────┘     └──> MongoDB (primary)

Phase 4: Deprecate Old
┌─────────┐
│   App   │─────> MongoDB (only)
└─────────┘
```

### 6.5 Monitoring & Observability

**Unified Metrics Collection**:
```python
from prometheus_client import Counter, Histogram, Gauge

# Database operation metrics
db_queries = Counter('db_queries_total', 'Total database queries', ['database', 'operation'])
db_latency = Histogram('db_query_duration_seconds', 'Query latency', ['database', 'operation'])
cache_hits = Counter('cache_hits_total', 'Cache hits', ['cache_type'])
cache_misses = Counter('cache_misses_total', 'Cache misses', ['cache_type'])

# Stream processing metrics
stream_events_processed = Counter('stream_events_processed_total', 'Events processed', ['stream', 'event_type'])
stream_lag = Gauge('stream_consumer_lag', 'Consumer lag', ['stream', 'consumer_group'])

# Usage
with db_latency.labels(database='postgresql', operation='SELECT').time():
    result = pg.query("SELECT ...")
db_queries.labels(database='postgresql', operation='SELECT').inc()
```

**Health Checks**:
```python
def health_check():
    health = {
        'postgres': check_postgres(),
        'redis': check_redis(),
        'mongodb': check_mongodb(),
        'kafka': check_kafka()
    }
    
    overall_healthy = all(health.values())
    
    return {
        'status': 'healthy' if overall_healthy else 'degraded',
        'details': health
    }

def check_postgres():
    try:
        pg.query_one("SELECT 1")
        return True
    except:
        return False
```

---

**Document Complete**: This guide covers normalization theory (1NF-6NF), PostgreSQL patterns (indexing, partitioning, triggers, CTEs), Redis patterns (caching, rate limiting, pub/sub, streams), MongoDB patterns (document modeling, aggregation, change streams), streaming architectures (CDC, event-driven), and polyglot persistence strategies.

