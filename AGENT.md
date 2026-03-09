# oc-stats Agent Context

## Product Goal

- Build a terminal status dashboard for OpenCode usage, similar to Claude status.
- Use `ratatui` inline viewport rendering, not the alt screen.
- Read usage primarily from the OpenCode SQLite database, with JSON export fallback.
- Show an Overview page and a Models page with cyan/blue themed visuals.

## Source Of Truth

- Product plan: `spec/plan.md`
- Reference implementation: `ref/ocmonitor-share/`
- Especially relevant reference files:
  - `ref/ocmonitor-share/ocmonitor/utils/sqlite_utils.py`
  - `ref/ocmonitor-share/ocmonitor/utils/data_loader.py`
  - `ref/ocmonitor-share/ocmonitor/utils/file_utils.py`
  - `ref/ocmonitor-share/ocmonitor/utils/time_utils.py`
  - `ref/ocmonitor-share/ocmonitor/services/session_analyzer.py`
  - `ref/ocmonitor-share/ocmonitor/services/price_fetcher.py`
  - `ref/ocmonitor-share/ocmonitor/ui/dashboard.py`
  - `ref/ocmonitor-share/ocmonitor/ui/theme.py`
  - `ref/ocmonitor-share/ocmonitor/models.json`

## Confirmed Data Rules

- Prefer SQLite; fall back to JSON input when explicitly provided.
- Default database path:
  - Windows: `%APPDATA%/opencode/opencode.db`
  - Linux: `~/.local/share/opencode/opencode.db`
  - macOS: `~/Library/Application Support/opencode/opencode.db`
- Only assistant messages count as billable usage.
- Extract from message JSON:
  - `modelID` or `model.modelID`
  - `tokens.input`
  - `tokens.output`
  - `tokens.cache.write`
  - `tokens.cache.read`
  - `time.created`
  - `time.completed`
  - `path.cwd` with `path.root` fallback
  - `agent`
  - `finish`
  - `cost` when present and positive
- Clamp token counts to non-negative values.
- Drop zero-token interactions.
- Drop sessions with no remaining token-bearing assistant interactions.

## Pricing Rules

- Bundled baseline pricing comes from `ref/ocmonitor-share/ocmonitor/models.json`.
- Local cache path is `~/.config/oc-stats/models.json`.
- Cache TTL is 1 hour, based on file modification time.
- If cache is stale, keep using it and refresh from `https://models.dev/api.json` asynchronously.
- Remote data only fills missing models or missing fields; it does not replace bundled pricing.
- Cache pricing fallback rules:
  - `cacheWrite = input` when absent
  - `cacheRead = input * 0.1` when absent
- If stored interaction cost exists and is positive, prefer it over recomputed cost.

## UI Rules

- Use inline viewport with a fixed height sized for both pages.
- No full-screen alt buffer.
- Overview page includes:
  - 365-day heatmap
  - range-sensitive stats
  - bottom comparison/fun fact
- Models page includes:
  - multi-model line chart
  - range-sensitive model totals and percentages
- Heatmap is always 365 days and ignores time-range switching.
- Time range affects summary stats and the line chart only.
- Primary visual direction is cyan/blue, with both dark and light themes.

## Interaction Rules

- `Left` / `Right` / `Tab`: switch page
- `r`: cycle range `All -> 30d -> 7d`
- `1` / `2` / `3`: direct range selection
- `Ctrl+S`: copy current page summary to clipboard
- `q` / `Esc`: quit

## Implementation Notes

- Keep analytics pure where possible.
- Keep data loading, pricing, analytics, and UI separated by module.
- Favor robust parsing over strict schema assumptions for JSON input.
- Preserve unknown models instead of failing; they should show zero computed cost unless stored cost exists.
- Optimize for correctness first; cache computed summaries inside app state only if needed.
