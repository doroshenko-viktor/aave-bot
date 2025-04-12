#!/bin/sh
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
echo $SCRIPT_DIR

docker compose -f $SCRIPT_DIR/../docker/infra.docker-compose.yml down
docker volume rm docker_postgres_aave_bot
docker compose -f $SCRIPT_DIR/../docker/infra.docker-compose.yml up -d
sleep 5
sqlx migrate run