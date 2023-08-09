migrate-local-database:
	@docker compose exec server sqlx migrate run

.PHONY: migrate-local-database

connect-to-local-database:
	@pgcli postgres://punchcli:password@localhost:55432/punchcli

.PHONY: connect-to-local-database
