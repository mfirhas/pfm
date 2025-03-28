test:
	@echo "Running pfm-core unit test..."
	@cargo test -p pfm-core --lib

test-all:
	@echo "Running pfm-core unit test..."
	@cargo test 

P ?= pfm-http

build-linux-gnu:
	@echo "Building $(p) for linux gnu..."
	@cargo build -p $(P) --release --target x86_64-unknown-linux-gnu

build-linux-musl:
	@echo "Building $(p) for linux musl..."
	@cargo build -p $(P) --release --target x86_64-unknown-linux-musl

