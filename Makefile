### Prerequisite:
# - llvm-tools-preview: rustup component add llvm-tools-preview --toolchain nightly
# - grcov: cargo install grcov

RUST_TOOLCHAIN := +nightly
OUT_DIR := target/debug
PROFILE_PATTERN := $(OUT_DIR)/coverage-%p-%m.profraw
PROFDATA := $(OUT_DIR)/coverage.profdata
LCOV := $(OUT_DIR)/lcov.info

SYSROOT := $(shell rustc $(RUST_TOOLCHAIN) --print sysroot 2>/dev/null)
LLVM_PROFDATA := $(firstword $(wildcard $(SYSROOT)/bin/llvm-profdata $(SYSROOT)/lib/rustlib/*/bin/llvm-profdata))

ifeq ($(LLVM_PROFDATA),)
	$(error llvm-profdata not found. Install with: rustup component add llvm-tools-preview --toolchain nightly)
endif

install:
	@echo "install grcov..."
	@cargo install grcov

test:
	@echo "running tests with coverage instrumentation..."
	@CARGO_INCREMENTAL=0 \
	RUSTFLAGS="-C instrument-coverage -Zpanic_abort_tests -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Cpanic=abort" \
	RUSTDOCFLAGS="-Cpanic=abort" \
	LLVM_PROFILE_FILE="$(PROFILE_PATTERN)" \
	cargo $(RUST_TOOLCHAIN) test --tests -- --test-threads=1

cover:
	@echo "merging profraw -> profdata..."
	@$(LLVM_PROFDATA) merge -sparse $(OUT_DIR)/*.profraw -o $(PROFDATA)
	@echo "generating lcov with grcov..."
	@grcov $(OUT_DIR) -s . --binary-path ./$(OUT_DIR) -t lcov --branch --ignore-not-existing -o $(LCOV) --ignore '/*' --ignore 'examples/*' --ignore 'tests/*' --ignore 'target/*'
	@echo "lcov saved to $(LCOV)"

build-linux-gnu:
	@echo "Building kartel for linux gnu..."
	@cargo build -p kartel --release --target x86_64-unknown-linux-gnu

# test:
# 	@echo "Running pfm-core unit test..."
# 	@cargo test -p pfm-core --lib

# test-integ:
# 	@echo "Running pfm-core integration test..."
# 	@cargo test --test '*' -- --test-threads=1
# 	@cargo test --release -p pfm-core --test test_storage -- test_storage_get_historical_range --exact --show-output --ignored

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
