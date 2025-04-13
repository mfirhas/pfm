test:
	@echo "Running pfm-core unit test..."
	@cargo test -p pfm-core --lib

test-integ:
	@echo "Running pfm-core integration test..."
	@cargo test --test '*' -- --test-threads=1
	@cargo test --release -p pfm-core --test test_storage -- test_storage_get_historical_range --exact --show-output --ignored

P ?= pfm-http

build-linux-gnu:
	@echo "Building $(p) for linux gnu..."
	@cargo build -p $(P) --release --target x86_64-unknown-linux-gnu

build-linux-musl:
	@echo "Building $(p) for linux musl..."
	@cargo build -p $(P) --release --target x86_64-unknown-linux-musl

