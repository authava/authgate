#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

# Check for .env and suggest copying from .env.dist
if [ ! -f .env ]; then
  echo "📦 .env file not found. Copying from .env.dist..."
  cp .env.dist .env
  echo "✅ .env file created from .env.dist"
fi

SERVICE_NAME=postgres
APP_DIR=authgate
MIGRATIONS_DIR="$APP_DIR/migrations"
DATABASE_URL="postgres://authgate:devpassword@host.docker.internal:54322/authgate"

echo "🔧 Shutting down any previous docker containers..."
docker-compose down -v

echo "🚀 Starting database..."
docker compose up -d $SERVICE_NAME

echo "⏳ Waiting for Postgres to be ready..."
until docker compose exec -T $SERVICE_NAME pg_isready -U postgres >/dev/null 2>&1; do
  sleep 1
done
echo "✅ Postgres is ready."

export DATABASE_URL

echo "📜 Running SQLx migrations..."
(
  cd $APP_DIR
  sqlx migrate run
)

docker compose up -d

echo "System up."