# Avoid inheriting SHELL from the env.
SHELL = /bin/sh

WAIT_FOR_IT_URL = https://raw.githubusercontent.com/eficode/wait-for/v2.2.3/wait-for

DB_USER           := postgres
DB_PASSWORD       := password
DB_NAME           := proximity_service
DB_HOST           := localhost
DB_PORT           := 5432
DB_CONTAINER_NAME := postgres_test
DB_URL            := postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}

.PHONY: Makefile

.PHONY: init_db
init_db:
	docker run \
		--name ${DB_CONTAINER_NAME} \
		-e POSTGRES_USER=${DB_USER} \
		-e POSTGRES_PASSWORD=${DB_PASSWORD} \
		-e POSTGRES_DB=${DB_NAME} \
		-h POSTGRES_HOST=${DB_HOST} \
		-p ${DB_PORT}:5432 \
		-d postgres \
		postgres -N 1000

.PHONY: ping_db
ping_db:
	curl ${WAIT_FOR_IT_URL} | sh -s -- ${DB_HOST}:${DB_PORT} -- echo Database is up

.PHONY: run_migrations
run_migrations:
ifeq (, $(shell which sqlx))
	cargo install sqlx-cli --no-default-features --features rustls,postgres; sqlx migrate run --database-url ${DB_URL}
else
	sqlx migrate run --database-url ${DB_URL}
endif

.PHONY: clean_up
clean_up:
	@docker stop ${DB_CONTAINER_NAME}; docker rm ${DB_CONTAINER_NAME}

.PHONY: test
test: init_db ping_db run_migrations ; cargo test; $(MAKE) clean_up
