.PHONY: build_bot test_bot run_bot

build_bot:
	@echo "Building bot..."
	@cd obot && cargo build --release

test_bot:
	@echo "Testing bot..."
	@cd obot && cargo test

run_bot:
	@echo "Running bot..."
	@cd obot && cargo run