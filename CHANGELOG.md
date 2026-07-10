# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.4] - 2026-07-09

### Added
- `api_log_tracker tui` command — launches the TUI dashboard from the Rust binary (tries `npx`, falls back to local `my-terminal-ui/dist/cli.js`)
- `api_log_tracker completions <shell>` — generates shell completions for bash, zsh, fish, powershell, elvish
- `--csv <path>` flag on all subcommands (`serve`, `daemon`, `analyze`, `tui`, `demo-client`)
- `--port <port>` flag on `serve` command (was hardcoded to :3000)
- `--port <port>` flag on `daemon` command (replaces positional arg for consistency)
- Environment variable support for all flags (`API_LOGGER_PORT`, `API_LOGGER_CSV`, `LLM_PROVIDER`, `LLM_MODEL`)
- `clap` derive-based argument parser with auto-generated help and `--version`
- Subcommand-specific `--help` output

### Changed
- Migrated CLI from manual `env::args()` parsing to `clap` derive macros
- `daemon` now uses `--port` flag instead of positional argument
- Version bumped from 0.1.3 to 0.1.4

## [0.1.3] - 2026-07-09

### Added
- Google Sheets real-time export via `gsheets-exporter` (watchdog + gspread)
- `LoggedClient` Python httpx wrapper for outgoing HTTP request logging
- `ApiLoggerMiddleware` — drop-in FastAPI/Starlette middleware for Python services
- VisaARX integration: API logging daemon + Google Sheets sync in Docker Compose
- CSV rotation handling in gsheets-exporter (detects file recreation)

### Changed
- Version bumped from 0.1.2 to 0.1.3

## [0.1.2] - 2026-07-09

### Added
- `--version` / `-v` flag and `help` command in CLI
- `--help` / `-h` flag with full usage documentation
- Graceful shutdown on SIGTERM/SIGINT in daemon mode
- `GET /api` endpoint for API discovery (lists all available endpoints)
- Structured logging via `tracing` + `tracing-subscriber` (replaces raw `eprintln!`/`println!`)
- Daemon integration tests (`tests/daemon_test.rs`) — health, api index, post log, get logs
- `client.rs` unit tests (LoggedClient creation)
- `webhook.rs` unit tests (stats error rate calculation, threshold checks)
- `Analysis.test.tsx` — provider selector, idle/running/done/error states
- `ServerControls.test.tsx` — status display, start/stop/demo options
- `LiveLogs.test.tsx` — entries, filter, column headers, empty state
- Keyboard help overlay in TUI (press `?` for shortcuts reference)
- CHANGELOG.md

### Changed
- Replaced all `eprintln!`/`println!` with `tracing::info!`/`tracing::error!` across Rust codebase
- Version bumped from 0.1.1 to 0.1.2

### Fixed
- Unknown CLI arguments now exit with error code 1 and helpful message

## [0.1.1] - 2026-07-09

### Added
- `Swatinem/rust-cache@v2` to CI `workflow-lint` job for faster cached builds
- `@types/node` as explicit devDependency (needed after removing xo)
- Unit tests: 3 in `models.rs`, 4 in `agent.rs`, 8 in `provider.rs`
- `LICENSE` file (MIT)
- README badges (CI, crates.io, npm)

### Fixed
- `Dockerfile` — added `curl` for healthcheck + dep-caching layer for faster builds
- `Analysis.tsx` — fixed stale closure using `useRef` for output buffer
- `Analysis.tsx` — per-provider API key lookup (not hardcoded `ANTHROPIC_API_KEY`)
- `LiveLogs.tsx` — implemented functional up/down arrow scroll
- Removed `xo` and `prettier` from devDependencies (vitest handles ESM+TS natively)
- CI test script updated from `test:ava` to `test`

## [0.1.0] - 2026-07-09

### Added
- Initial release
- Rust binary with serve, demo-client, analyze, daemon modes
- 13 LLM providers: OpenAI, Anthropic, Gemini, Grok, Groq, DeepSeek, Qwen, Baichuan, Yi, StepFun, Zhipu GLM, Baidu ERNIE
- Multi-provider agent dispatcher with SHA256 60s TTL cache
- CORS-enabled HTTP daemon (POST /api/log, GET /api/health, GET /api/logs, POST /api/analyze)
- Anomaly webhook reporter with structured JSON payload
- TUI dashboard (ink + React) with 5 tabs: Dashboard, Logs, Analysis, Server Controls, Integration
- Integration snippets for Python, C, C++, Rust, Go, Erlang
- Deploy configs: Docker, docker-compose, fly.io, railway, render
- Published to crates.io (`api_log_tracker`) and npm (`api-log-tracker-tui`)
