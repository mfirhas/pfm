test:
	@echo "Running pfm-core unit test..."
	@cargo test -p pfm-core --lib

test-integ:
	@echo "Running pfm-core integration test..."
	@cargo test --test '*' -- --test-threads=1
	@cargo test --release -p pfm-core --test test_storage -- test_storage_get_historical_range --exact --show-output --ignored

P ?= pfm-http

build-linux-gnu:
	@echo "Building $(P) for linux gnu..."
	@cargo build -p $(P) --release --target x86_64-unknown-linux-gnu

build-linux-musl:
	@echo "Building $(P) for linux musl..."
	@cargo build -p $(P) --release --target x86_64-unknown-linux-musl

build-deploy-linux-gnu-all:
	@echo "Building all binary packages..."
	@echo "Building pfm-http..."
	@make build-linux-gnu P=pfm-http 
	@echo "Building pfm-cron..."
	@make build-linux-gnu P=pfm-cron
	@echo "Deploying pfm-http..."
	@./deploy.sh mfirhas 2345 target/x86_64-unknown-linux-gnu/release/pfm-http .env api_keys.json
	@echo "Deploying pfm-cron..."
	@./deploy.sh mfirhas 2345 target/x86_64-unknown-linux-gnu/release/pfm-cron .env api_keys.json
