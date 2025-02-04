#!/bin/bash

# Load environment variables
set -a
source .env
set +a

# Check if psql is installed
if ! command -v psql &> /dev/null; then
    echo "Error: PostgreSQL client (psql) is not installed"
    exit 1
fi

# Extract database name from DATABASE_URL
DB_NAME=$(echo $DATABASE_URL | sed 's/.*\///g')

# Create database if it doesn't exist
psql -h localhost -U postgres -c "CREATE DATABASE $DB_NAME;" 2>/dev/null || true

echo "Database setup complete!" 