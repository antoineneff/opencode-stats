# Opencode Status CLI - Agent Guidelines

## Project Overview

Build a CLI tool similar to Claude's status feature using Ratatui with inline viewport rendering. Displays:

1. GitHub-style contribution heatmap (past year's usage)
2. Rounded corner line chart (token usage by model over time)

## Key Decisions

| Aspect           | Decision                                                    |
| ---------------- | ----------------------------------------------------------- |
| Database Path    | Cross-platform (Windows/Linux/macOS default paths + custom) |
| Data Sources     | SQLite (primary) + JSON export (testing/migration)          |
| Heatmap Range    | Fixed 365 days                                              |
| Cost Calculation | Full implementation using models.json pricing               |
| UI Style         | No outer borders, clean separation with blank lines         |

## Development Phases

### Phase 1: Foundation

- Project structure and dependencies
- Cross-platform database connection
- Data models and inline viewport setup

### Phase 2: Data Processing

- Daily/weekly/monthly aggregation
- Heatmap data preparation
- Model statistics and cost calculation
- Time range filtering

### Phase 3: UI Implementation

- Heatmap widget (4-level intensity: `·` `░` `▒` `█`)
- Line chart widget (rounded corners, multi-color)
- Tab switching (Overview/Models)
- Theme system

### Phase 4: Polish

- Keyboard interaction
- Data caching
- Performance optimization
- Error handling

## Reference Project

Python implementation at `ref/ocmonitor-share/` provides:

- Database queries: `ocmonitor/utils/sqlite_utils.py`
- Data analysis: `ocmonitor/services/session_analyzer.py`
- UI components: `ocmonitor/ui/dashboard.py`
- Model pricing: `ocmonitor/models.json`

## Commands

```bash
# Build
cargo build

# Run
cargo run

# Run with custom database path
cargo run -- --db /path/to/opencode.db
```

## Key Conventions

- No comments in code unless requested
- Follow existing code style in Rust ecosystem
- Use `ratatui` 0.29 with `Viewport::Inline(N)`
- No outer borders/frames in UI
- Cross-platform compatibility required
