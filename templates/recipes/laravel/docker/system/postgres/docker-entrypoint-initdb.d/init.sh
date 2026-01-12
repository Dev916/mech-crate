#!/bin/bash
# PostgreSQL initialization script
# This runs when the database container is first created

set -e

# Create application database if it doesn't exist
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    -- Enable useful extensions
    CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
    CREATE EXTENSION IF NOT EXISTS "pg_trgm";
    
    -- Create application schema
    CREATE SCHEMA IF NOT EXISTS app;
    
    -- Grant permissions
    GRANT ALL ON SCHEMA app TO "$POSTGRES_USER";
    GRANT ALL ON ALL TABLES IN SCHEMA app TO "$POSTGRES_USER";
    ALTER DEFAULT PRIVILEGES IN SCHEMA app GRANT ALL ON TABLES TO "$POSTGRES_USER";
EOSQL

echo "Database initialization complete!"
