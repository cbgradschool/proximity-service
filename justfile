# Settings
set export

DB_USER := "postgres"
DB_PASSWORD := "password"
DB_NAME := "proximity_service"
DB_HOST := "localhost"
DB_CONTAINER_NAME := "postgres"
DB_PORT := "5432"

_init-db:
	docker run \
		--name {{DB_CONTAINER_NAME}} \
		-e POSTGRES_USER={{DB_USER}} \
		-e POSTGRES_PASSWORD={{DB_PASSWORD}} \
		-e POSTGRES_DB={{DB_NAME}} \
		-h POSTGRES_HOST={{DB_HOST}} \
		-p {{DB_PORT}}:5432 \
		-d postgres \
		postgres -N 1000
	
_run-migrations:
	doppler run --command="sqlx migrate run --database-url \$APP_DATABASE_URL"
	
# ⏰ Wait for the availability of a host and TCP port
wait-for:
	wait-for-them $DB_HOST:$DB_PORT

# 🚀 Spins up service and it's dependencies. (Shuts and spins down service and dependencies on SIGINT)
run: _init-db wait-for _run-migrations
	doppler run --command="RUST_BACKTRACE=1 cargo watch -x check -x run"; just cleanup

# 🧪 Run test suite locally
test:
  #!/usr/bin/env bash
  set -euxo pipefail
  DB_URL="postgres://postgres:password@localhost:5433/proximity_service"
  just DB_CONTAINER_NAME="postgres_test" DB_PORT="5433" _init-db
  just DB_CONTAINER_NAME="postgres_test" DB_PORT="5433" wait-for
  sqlx migrate run --database-url ${DB_URL}
  APP_DATABASE_URL=${DB_URL} cargo nextest run
  just DB_CONTAINER_NAME="postgres_test" DB_PORT="5433" cleanup

# 🧹 Stop and remove Postgres docker container
cleanup:
	docker stop $DB_CONTAINER_NAME; docker rm $DB_CONTAINER_NAME

