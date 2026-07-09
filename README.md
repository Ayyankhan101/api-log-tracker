# API Logger

[![CI](https://github.com/Ayyankhan101/api-log-tracker/actions/workflows/ci.yml/badge.svg)](https://github.com/Ayyankhan101/api-log-tracker/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/api_log_tracker)](https://crates.io/crates/api_log_tracker)
[![npm](https://img.shields.io/npm/v/api-log-tracker-tui)](https://www.npmjs.com/package/api-log-tracker-tui)

Real-time API traffic logging + multi-provider LLM analysis + TUI dashboard + webhook reporting.

## Quick Start

```bash
# Build
cargo build --release

# Serve (axum with logging middleware)
cargo run -- serve

# Demo client (generates test log entries)
cargo run -- demo-client

# Daemon (HTTP API — integrate from any language)
cargo run -- daemon [port]

# TUI dashboard
./tui

# Analysis (requires LLM_PROVIDER + API key)
export LLM_PROVIDER=openai
export OPENAI_API_KEY=sk-...
cargo run -- analyze
```

## Daemon Endpoints

| Method | Path | Description |
|---|---|---|
| POST | `/api/log` | Log an API call |
| GET | `/api/health` | Status + entry count |
| GET | `/api/logs?limit=50&source=client` | Query stored logs |
| POST | `/api/analyze` | Trigger LLM analysis |

### POST /api/log

```bash
curl -X POST http://localhost:8080/api/log \
  -H "Content-Type: application/json" \
  -d '{
    "source": "my-app",
    "method": "GET",
    "endpoint": "/api/users",
    "status_code": 200,
    "latency_ms": 45,
    "request_size": 128,
    "response_size": 4096,
    "error": null
  }'
```

### POST /api/analyze

```bash
curl -X POST http://localhost:8080/api/analyze \
  -d '{"provider": "openai", "model": "gpt-4o"}' \
  -H "Content-Type: application/json"
```

Returns:

```json
{
  "status": "ok",
  "provider": "openai",
  "model": "gpt-4o",
  "analysis": "...",
  "error_rate": 4.2,
  "total_requests": 150,
  "error_count": 6
}
```

## CLI

```
Usage: api_log_tracker <command> [options]

Commands:
  serve                   Start axum server on :3000
  demo-client             Generate test log entries
  analyze [options]       Run LLM analysis
  daemon [port]           Start HTTP daemon (default 8080)

analyze options:
  --provider <name>       Force a provider
  --model <name>          Override default model

daemon options:
  [port]                  Port (default 8080)
  --webhook <url>         Webhook for anomaly reports
```

## LLM Providers (13)

Set `LLM_PROVIDER` + the matching API key:

| Provider | Env Var | Default Model |
|---|---|---|
| OpenAI | `OPENAI_API_KEY` | `gpt-4o` |
| Anthropic | `ANTHROPIC_API_KEY` | `claude-sonnet-4-5` |
| Google Gemini | `GEMINI_API_KEY` | `gemini-2.0-flash` |
| Grok (xAI) | `XAI_API_KEY` | `grok-2` |
| Groq | `GROQ_API_KEY` | `llama-3.3-70b` |
| DeepSeek | `DEEPSEEK_API_KEY` | `deepseek-chat` |
| Qwen | `DASHSCOPE_API_KEY` | `qwen-plus` |
| Baichuan | `BAICHUAN_API_KEY` | `Baichuan4` |
| Yi | `YI_API_KEY` | `yi-large` |
| StepFun | `STEPFUN_API_KEY` | `step-2-16k` |
| Zhipu GLM | `ZHIPU_API_KEY` | `glm-4-plus` |
| Baidu ERNIE | `BAIDU_API_KEY` + `BAIDU_API_SECRET` | `ernie-4.0-8k` |

Optional: `LLM_MODEL=gpt-4o` to override default model.

## Webhook Reporting

```bash
cargo run -- daemon 8080 --webhook https://hooks.slack.com/...
```

Triggers when error rate exceeds 5% (configurable via `WEBHOOK_THRESHOLD`). Posts structured JSON with anomalies, latency hotspots, and recommendations.

## Deploy

```bash
# Docker
./deploy.sh docker

# Fly.io
./deploy.sh fly

# Railway
./deploy.sh railway

# Render (push to GitHub, connect in dashboard)
./deploy.sh render

# Hugging Face Spaces
./deploy.sh hf

# Local build
./deploy.sh local
```

Docker Compose starts the daemon on port 8080 with health checks:

```bash
docker compose up -d
curl http://localhost:8080/api/health
```

## CSV Log

Default location: `logs/api_logs.csv`. Override with `API_LOGGER_CSV` env var.

| Column | Type | Description |
|---|---|---|
| id | UUID | Unique request ID |
| timestamp | RFC3339 | When the request was made |
| source | `server` or `client` | Which side generated the log |
| method | string | HTTP method |
| endpoint | string | Path or full URL |
| status_code | u16 | HTTP status (0 if request itself failed) |
| latency_ms | u64 | Round-trip time in milliseconds |
| request_size | usize | Bytes (from Content-Length) |
| response_size | usize | Bytes (from Content-Length) |
| error | Option | Error message if any |

## Cache

LLM analysis results are cached for 60s (keyed by SHA256 of stats). Prevents redundant API calls when multiple clients hit `/api/analyze`.

## TUI Dashboard

```bash
cd my-terminal-ui && npm run build && node dist/cli.js
# or: ./tui
```

Tabs:
- **Dashboard** — live stats, status bars, sparkline
- **Live Logs** — scrollable log table with filter
- **Analysis** — LLM provider selector + run analysis
- **Controls** — start/stop server, run demo client
- **Integration** — copy-paste snippets for Python, C, C++, Rust, Go, Erlang + LLM provider env var reference

Use `1-5` to switch tabs, `j/k` to navigate within tabs, `r` to run analysis.

## Using as a Library

```toml
[dependencies]
api_log_tracker = { path = "../api_log_tracker" }
```

```rust
use api_log_tracker::{ApiLogger, LoggedClient};

let logger = ApiLogger::new("logs/api_logs.csv");
let client = LoggedClient::new(logger.clone());

let resp = client.get("https://api.example.com/data").await?;
```

For axum middleware:

```rust
use api_log_tracker::{ApiLogger, middleware::log_requests};

let logger = ApiLogger::new("logs/api_logs.csv");
let app = Router::new()
    .route("/", get(handler))
    .layer(axum::middleware::from_fn_with_state(logger.clone(), log_requests))
    .with_state(logger);
```

## Supported Languages (Integration Snippets)

The TUI Integration tab includes copy-paste snippets for:

| Language | Method |
|---|---|
| Python | `requests.post()` |
| C | `libcurl` |
| C++ | `libcurl` + `std::format` |
| Rust | `reqwest` |
| Go | `net/http` |
| Erlang | `httpc` + `jsx` |

All snippets POST to `http://localhost:8080/api/log` with the same JSON schema.
