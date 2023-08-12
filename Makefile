SHELL := /bin/bash

BUILD_DIR := build

install-server-bins:
	@mkdir -p $(PWD)/server/$(BUILD_DIR)
	@cargo install sqlx-cli --root $(PWD)/server/$(BUILD_DIR)
	@cargo install cargo-watch --root $(PWD)/server/$(BUILD_DIR)

.PHONY: install-server-bins

install-server-activator:
	@mkdir -p $(PWD)/server/$(BUILD_DIR)/bin
	@echo 'export PATH=$(PWD)/$(BUILD_DIR)/bin:$$PATH' > $(PWD)/server/$(BUILD_DIR)/bin/activate

.PHONY: install-server-activator

install-server-all: install-server-bins install-server-activator

migrate-local-database:
	@cd server; DATABASE_URL=postgres://punchcli:password@localhost:55432/punchcli $(PWD)/server/$(BUILD_DIR)/bin/sqlx migrate run

.PHONY: migrate-local-database

connect-local-database:
	@pgcli postgres://punchcli:password@localhost:55432/punchcli

.PHONY: connect-local-database

start-server:
	@cd server; $(PWD)/server/$(BUILD_DIR)/bin/cargo-watch watch -x run --env-file .env

.PHONY: start-server
