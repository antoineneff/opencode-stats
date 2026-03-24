# AGENTS.md

Context for AI assistants working on oc-stats.

## Project Overview

oc-stats is a terminal dashboard for tracking OpenCode usage statistics. It reads usage data from the OpenCode SQLite database (or JSON export) and displays token usage, costs, model breakdown, and activity heatmap in a ratatui-based TUI.

**Rust edition: 2024**

## Build & Test Commands

```bash
cargo build              # Build the project
cargo test               # Run all tests
cargo test <test_name>   # Run a single test (e.g., cargo test normalizes_date_suffixes)
cargo clippy             # Lint checks
cargo clippy --fix       # Auto-fix lint warnings
cargo fmt                # Format code
cargo run -- --db /path/to/opencode.db   # Run with custom database
cargo run -- --json /path/to/export.json # Run with JSON export
```

## Code Style

### Imports

Group imports in this order, separated by blank lines:
1. `std::` imports first
2. External crate imports
3. `crate::` imports last

### Formatting

- No comments unless explicitly requested
- Use `cargo fmt` before committing
- Use `#[allow(dead_code)]` for intentionally unused code

### Naming Conventions

- Types/structs/enums: `PascalCase`
- Functions/methods: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Test functions: descriptive snake_case (e.g., `normalizes_date_suffixes`)

### Error Handling

Each domain module has its own `errors.rs`:
- Define error types with `thiserror::Error`
- Provide a `Result<T>` type alias for the module
- Use constructor methods for common errors
- Use `anyhow::Result` with `.context()` in `main.rs` for user-facing errors

### Testing

- Place tests in a `#[cfg(test)] mod tests` block at the end of the file
- Import `super::*` for convenience in test modules

### Types

- Use `u64` for token counts (use `saturating_add` to avoid overflow)
- Use `Decimal` from `rust_decimal` for monetary values
- Use `DateTime<Local>` for timestamps displayed to users

## Key Rules

### Data Loading

- Default database: `%APPDATA%/opencode/opencode.db` (Windows), `~/.local/share/opencode/opencode.db` (Linux), `~/Library/Application Support/opencode/opencode.db` (macOS)
- Only assistant messages count toward usage

### Pricing

- Local cache: `~/.cache/oc-stats/models.json`
- Remote source: `https://models.dev/api.json`
- Cache TTL: 1 hour
- User overrides in OpenCode config take priority
- Fallback: `cacheWrite = input`, `cacheRead = input * 0.1`
- Prefer stored cost from database when available

### Theme Configuration

- App config: `~/.config/oc-stats/config.toml`
- Theme index: `~/.config/oc-stats/themes.toml`
- Theme overrides: `~/.config/oc-stats/themes/*.toml`