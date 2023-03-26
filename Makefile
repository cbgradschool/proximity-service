include $(PWD)/Makefile.base

DB_USER      ?= postgres
DB_PASSWORD  ?= password
DB_NAME      ?= proximity_service
DB_HOST      ?= localhost

.PHONY: test
test: export DB_PORT           ?= 5433
test: export DB_CONTAINER_NAME ?= postgres_test
test: export DATABASE_URL      ?= postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
test: init_db ping_db run_migrations ; cargo test; $(MAKE) clean_up

.PHONY: run
run: export DB_PORT           ?= 5432
run: export DB_CONTAINER_NAME ?= postgres
run: export DATABASE_URL      ?= postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
run: init_db ping_db run_migrations
