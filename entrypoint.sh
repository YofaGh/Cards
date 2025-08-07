#!/bin/sh

set -e

if [ -z "$DATABASE_URL" ]; then
  echo "Error: DATABASE_URL environment variable is not set."
  exit 1
fi

echo "Waiting for database to be ready..."
until sqlx database ping > /dev/null 2>&1; do
  echo "Database is unavailable - sleeping"
  sleep 1
done

echo "Database is up - running migrations"
sqlx migrate run

echo "Migrations complete - starting application"

exec "$@"