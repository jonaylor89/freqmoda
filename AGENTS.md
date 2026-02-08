# FreqModa Development Guide

## Build & Test Commands
- `just` - List available recipes
- `just dev-web-ui` - Run web UI with auto-reload
- `just dev-streaming` - Run streaming engine with auto-reload
- `just dev-all` - Run both services with auto-reload in parallel
- `just dev-full` - Initialize services and run both with auto-reload
- `just teardown` - Gracefully stop all services and cleanup containers
- `just build` - Build the workspace
- `just test` - Run all tests
- `just test-name <name>` - Run a specific test
- `just test-web-ui` - Run tests for web UI
- `just lint` - Run linter (clippy)
- `just fmt` - Format code
- `just check` - Full check: format, lint, build, test

## Project Structure
- **Workspace**: Multi-service Rust workspace with `web-ui/` and `streaming-engine/`
- **Services**: Independent services with shared dependencies in workspace Cargo.toml
- **Database**: PostgreSQL with SQLx migrations in `migrations/`

## Code Style
- **Imports**: Group std, external crates, then local modules
- **Error Handling**: Use `color_eyre::Result`, `thiserror` for custom errors
- **Logging**: Use `tracing` with structured logging and `#[instrument]` for functions
- **Types**: Prefer explicit types, use `Uuid` for IDs, `DateTime<Utc>` for timestamps
- **Naming**: snake_case for functions/variables, PascalCase for types, modules in snake_case
- **Async**: Use `tokio::main` and async/await throughout
- **Database**: Use SQLx with `FromRow` derive, parameterized queries
- **API**: Use Axum with `State` extraction and `Json` responses
