.PHONY: all build test lint fmt clean

# Default target
all: fmt lint test

# Build all crates
build:
	cargo build --workspace

# Run all tests
test:
	cargo test --workspace

# Run tests with output
test-verbose:
	cargo test --workspace -- --nocapture

# Run clippy
lint:
	cargo clippy --workspace --all-targets -- -D warnings

# Format code
fmt:
	cargo fmt --all

# Format check
fmt-check:
	cargo fmt --all -- --check

# Clean build artifacts
clean:
	cargo clean
