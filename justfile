# FreqModa Development Commands

# List available recipes
default:
    @just --list

# Install cargo-watch if not present
install-tools:
    cargo install cargo-watch

# Initialize PostgreSQL database (Docker)
init-db:
    #!/usr/bin/env bash
    ./scripts/init_db.sh

# Initialize Redis (Docker)
init-redis:
    #!/usr/bin/env bash
    ./scripts/init_redis.sh

# Initialize MinIO (Docker)
init-minio:
    #!/usr/bin/env bash
    ./scripts/init_minio.sh

# Initialize all development services (Database, Redis, MinIO)
init-services:
    just init-db
    just init-redis
    # just init-minio

# Run web UI with auto-reload
dev-web-ui:
    #!/usr/bin/env bash
    cd web-ui && cargo watch -x 'run' -w src -w Cargo.toml -w config -w templates

# Run streaming engine with auto-reload
dev-streaming:
    #!/usr/bin/env bash
    cd streaming-engine && cargo watch -x 'run' -w src -w Cargo.toml -w config

# Run both services in parallel with auto-reload
dev-all:
    #!/usr/bin/env bash
    trap 'kill 0' INT
    just dev-streaming &
    just dev-web-ui &
    wait

# Initialize services and run both services in parallel with graceful teardown
dev-full:
    #!/usr/bin/env bash
    echo "🚀 Initializing development services..."
    just init-services
    echo "✅ Services initialized. Starting applications..."
    sleep 2
    
    cleanup() {
        echo -e "\n🛑 Gracefully tearing down..."
        # Kill the background jobs (dev-web-ui and dev-streaming)
        kill $(jobs -p) 2>/dev/null || true
        # Stop docker services
        just stop-services
        echo "✅ Teardown complete"
        exit 0
    }

    trap cleanup INT TERM
    
    just dev-web-ui &
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
test-web-ui:
    cargo test --package web-ui

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
    just init-services
    just build

# Run web UI without auto-reload
run-web-ui:
    #!/usr/bin/env bash
    cd web-ui && cargo run

# Run streaming engine without auto-reload
run-streaming:
    #!/usr/bin/env bash
    cd streaming-engine && cargo run

# Show project structure
tree:
    tree -I target

# Stop all development services (Docker containers)
stop-services:
    #!/usr/bin/env bash
    echo "🛑 Stopping and removing development services..."
    docker ps -a --filter "name=redis" -q | xargs -r docker rm -f
    docker ps -a --filter "name=postgres" -q | xargs -r docker rm -f
    docker ps -a --filter "name=minio" -q | xargs -r docker rm -f
    echo "✅ All development services cleaned up"

# Teardown the development environment (apps and services)
teardown:
    #!/usr/bin/env bash
    echo "🧨 Full teardown initiated..."
    # Attempt to kill any orphaned cargo-watch processes related to this project
    pkill -f "cargo-watch.*web-ui" || true
    pkill -f "cargo-watch.*streaming-engine" || true
    just stop-services
    echo "✨ Environment cleaned"

# Check status of development services
status:
    #!/usr/bin/env bash
    echo "📊 Development Services Status"
    echo "=============================="
    echo ""
    echo "🐘 PostgreSQL:"
    if docker ps --filter "name=postgres" -q | grep -q .; then
        echo "   ✅ Running"
        echo "   📍 Port: 5432"
    else
        echo "   ❌ Not running"
    fi
    echo ""
    echo "🔴 Redis:"
    if docker ps --filter "name=redis" -q | grep -q .; then
        echo "   ✅ Running"
        echo "   📍 Port: 6379"
    else
        echo "   ❌ Not running"
    fi
    echo ""
    echo "📦 MinIO:"
    if docker ps --filter "name=minio" -q | grep -q .; then
        echo "   ✅ Running"
        echo "   📍 API: http://localhost:9000"
        echo "   📍 Console: http://localhost:9001"
    else
        echo "   ❌ Not running"
    fi

# Reset development environment (stop services, clean, rebuild)
reset:
    just stop-services
    just clean
    just setup
