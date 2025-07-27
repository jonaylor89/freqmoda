# FreqModa Development Commands

# List available recipes
default:
    @just --list

# Install cargo-watch if not present
install-tools:
    cargo install cargo-watch

# Run gateway service with auto-reload
dev-gateway:
    #!/usr/bin/env bash
    cd gateway-service && cargo watch -x 'run' -w src -w Cargo.toml -w config

# Run streaming engine with auto-reload
dev-streaming:
    #!/usr/bin/env bash
    cd streaming-engine && cargo watch -x 'run' -w src -w Cargo.toml -w config

# Run both services in parallel with auto-reload
dev-all:
    #!/usr/bin/env bash
    trap 'kill 0' INT
    just dev-gateway &
    just dev-streaming &
    wait

# Build the entire workspace
build:
    cargo build

# Build with release optimizations
build-release:
    cargo build --release

# Run all tests
test:
    cargo test

# Run tests for specific service
test-gateway:
    cargo test --package gateway-service

test-streaming:
    cargo test --package streaming-engine

# Run specific test by name
test-name name:
    cargo test {{name}}

# Run linter
lint:
    cargo clippy

# Format code
fmt:
    cargo fmt

# Check formatting without changing files
fmt-check:
    cargo fmt --check

# Run database migrations
migrate:
    sqlx migrate run

# Create new migration
migrate-new name:
    sqlx migrate add {{name}}

# Clean build artifacts
clean:
    cargo clean

# Full check: format, lint, build, test
check:
    just fmt-check
    just lint
    just build
    just test

# Setup development environment
setup:
    just install-tools
    just migrate
    just build

# Run gateway service without auto-reload
run-gateway:
    #!/usr/bin/env bash
    cd gateway-service && cargo run

# Run streaming engine without auto-reload
run-streaming:
    #!/usr/bin/env bash
    cd streaming-engine && cargo run

# Show project structure
tree:
    tree -I target
