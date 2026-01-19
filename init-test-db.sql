-- Initialize test database
-- This script is run when the PostgreSQL container starts

-- Create the test database if it doesn't exist
SELECT 'CREATE DATABASE spoons_test'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'spoons_test')\gexec

-- Grant privileges to the spoons user
GRANT ALL PRIVILEGES ON DATABASE spoons_test TO spoons;
