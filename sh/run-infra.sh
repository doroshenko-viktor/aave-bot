#!/bin/sh
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
echo $SCRIPT_DIR

docker compose -f "$SCRIPT_DIR/../docker/infra.docker-compose.yml" up