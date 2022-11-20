.PHONY: build_bot test_bot run_bot db_run db_revert

build_bot:
	@echo "Building bot..."
	@cd obot && cargo build --release

test_bot:
	@echo "Testing bot..."
	@cd obot && cargo test

run_bot:
	@echo "Running bot..."
	@cd obot && sqlx migrate run && cargo run

db_run:
	@echo "Running database..."
	@cd obot && sqlx migrate run