use crate::agent;
use crate::logger::ApiLogger;
use crate::models::LogEntry;
use crate::webhook;
use axum::{
    extract::{Json, Query, State},
    http::{Method, StatusCode},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use tower_http::cors::{Any, CorsLayer};

// ── Request / Response types ─────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct LogRequest {
    pub id: Option<String>,
    pub timestamp: Option<String>,
    pub source: String,
    pub method: String,
    pub endpoint: String,
    pub status_code: u16,
    pub latency_ms: u64,
    #[serde(default)]
    pub request_size: usize,
    #[serde(default)]
    pub response_size: usize,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub total_entries: usize,
    pub provider: String,
}

#[derive(Deserialize)]
pub struct AnalyzeRequest {
    pub provider: Option<String>,
    pub model: Option<String>,
}

#[derive(Serialize)]
pub struct AnalyzeResponse {
    pub status: String,
    pub provider: String,
    pub model: String,
    pub analysis: String,
    pub error_rate: f64,
    pub total_requests: usize,
    pub error_count: usize,
}

#[derive(Deserialize)]
pub struct LogQuery {
    pub limit: Option<usize>,
    pub source: Option<String>,
}

#[derive(Serialize)]
pub struct LogEntryResponse {
    pub id: String,
    pub timestamp: String,
    pub source: String,
    pub method: String,
    pub endpoint: String,
    pub status_code: u16,
    pub latency_ms: u64,
    pub error: Option<String>,
}

// ── State ────────────────────────────────────────────────────────────────────

const MAX_CSV_SIZE: u64 = 100 * 1024 * 1024; // 100 MB
const MAX_ROTATED_FILES: usize = 3;

#[derive(Clone)]
pub struct DaemonState {
    pub logger: ApiLogger,
    pub csv_path: PathBuf,
}

fn rotate_csv_if_needed(csv_path: &Path) {
    let meta = match std::fs::metadata(csv_path) {
        Ok(m) => m,
        Err(_) => return,
    };
    if meta.len() < MAX_CSV_SIZE {
        return;
    }

    // Shift rotated files: .3 -> delete, .2 -> .3, .1 -> .2, current -> .1
    for i in (1..=MAX_ROTATED_FILES).rev() {
        let prev = csv_path.with_extension(format!("csv.{}", i));
        if i == MAX_ROTATED_FILES {
            let _ = std::fs::remove_file(&prev);
        } else {
            let next = csv_path.with_extension(format!("csv.{}", i + 1));
            let _ = std::fs::rename(&prev, &next);
        }
    }
    let rotated = csv_path.with_extension("csv.1");
    let _ = std::fs::rename(csv_path, &rotated);
}

// ── Main entry ───────────────────────────────────────────────────────────────

pub async fn start_daemon(port: u16, webhook_url: Option<String>) -> anyhow::Result<()> {
    let csv_path_str =
        std::env::var("API_LOGGER_CSV").unwrap_or_else(|_| "logs/api_logs.csv".to_string());
    let csv_path = PathBuf::from(&csv_path_str);
    let logger = ApiLogger::new(&csv_path_str);

    let state = DaemonState {
        logger: logger.clone(),
        csv_path: csv_path.clone(),
    };

    // CORS layer — allow any origin, any header, for browser clients
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        .route("/api", get(api_index))
        .route("/api/log", post(post_log))
        .route("/api/health", get(health))
        .route("/api/logs", get(get_logs))
        .route("/api/analyze", post(post_analyze))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("daemon listening on {addr}");

    // Webhook loop
    let webhook_handle = if let Some(url) = webhook_url {
        let csv = csv_path.clone();
        let config = webhook::WebhookConfig {
            url,
            error_threshold: std::env::var("WEBHOOK_THRESHOLD")
                .unwrap_or_else(|_| "5.0".to_string())
                .parse()
                .unwrap_or(5.0),
            min_entries: std::env::var("WEBHOOK_MIN_ENTRIES")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
        };
        Some(tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                match agent::get_stats(&csv) {
                    Ok(stats) => {
                        if let Err(e) = webhook::check_and_report(&stats, &config).await {
                            tracing::error!(error = %e, "webhook error");
                        }
                    }
                    Err(e) => tracing::error!(error = %e, "webhook stats error"),
                }
            }
        }))
    } else {
        None
    };

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let server = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal());

    if let Err(e) = server.await {
        tracing::error!(error = %e, "server error");
    }

    if let Some(handle) = webhook_handle {
        handle.abort();
    }

    tracing::info!("daemon shut down");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("received SIGINT, shutting down");
        }
        _ = terminate => {
            tracing::info!("received SIGTERM, shutting down");
        }
    }
}

// ── Handlers ─────────────────────────────────────────────────────────────────

pub async fn api_index() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "service": "api_log_tracker",
        "version": env!("CARGO_PKG_VERSION"),
        "endpoints": {
            "POST /api/log": "Submit an API log entry",
            "GET  /api/logs": "Retrieve recent log entries (?limit=N&source=client|server)",
            "GET  /api/health": "Health check",
            "POST /api/analyze": "Analyze logs with LLM anomaly detection",
            "GET  /api": "This endpoint — API discovery",
        },
    }))
}

pub async fn post_log(
    State(state): State<DaemonState>,
    Json(req): Json<LogRequest>,
) -> (StatusCode, &'static str) {
    let entry = LogEntry {
        id: req.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        timestamp: req
            .timestamp
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
        source: req.source,
        method: req.method,
        endpoint: req.endpoint,
        status_code: req.status_code,
        latency_ms: req.latency_ms,
        request_size: req.request_size,
        response_size: req.response_size,
        error: req.error,
    };

    if let Err(e) = state.logger.log(&entry).await {
        tracing::error!(error = %e, "failed to write log");
        return (StatusCode::INTERNAL_SERVER_ERROR, "write failed");
    }

    rotate_csv_if_needed(&state.csv_path);

    (StatusCode::CREATED, "ok")
}

pub async fn health(State(state): State<DaemonState>) -> Json<HealthResponse> {
    let total_entries = if state.csv_path.exists() {
        std::fs::read_to_string(&state.csv_path)
            .map(|c| c.lines().count().saturating_sub(1))
            .unwrap_or(0)
    } else {
        0
    };

    let provider_name = std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "auto-detect".to_string());

    Json(HealthResponse {
        status: "ok".to_string(),
        total_entries,
        provider: provider_name,
    })
}

pub async fn get_logs(
    State(state): State<DaemonState>,
    Query(params): Query<LogQuery>,
) -> Json<Vec<LogEntryResponse>> {
    let limit = params.limit.unwrap_or(50);
    let source_filter = params.source.as_deref();

    let entries = if state.csv_path.exists() {
        let mut rdr = csv::Reader::from_path(&state.csv_path).ok();
        rdr.as_mut()
            .map(|r| {
                r.deserialize::<LogEntry>()
                    .filter_map(|r| r.ok())
                    .filter(|e| source_filter.is_none_or(|s| e.source == s))
                    .take(limit)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    } else {
        vec![]
    };

    Json(
        entries
            .into_iter()
            .map(|e| LogEntryResponse {
                id: e.id,
                timestamp: e.timestamp,
                source: e.source,
                method: e.method,
                endpoint: e.endpoint,
                status_code: e.status_code,
                latency_ms: e.latency_ms,
                error: e.error,
            })
            .collect(),
    )
}

async fn post_analyze(
    State(state): State<DaemonState>,
    Json(req): Json<AnalyzeRequest>,
) -> Result<Json<AnalyzeResponse>, (StatusCode, String)> {
    let (analysis, provider_name, model_name) = agent::analyze_logs_with(
        &state.csv_path,
        req.provider.as_deref(),
        req.model.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let entries = agent::read_entries(&state.csv_path).unwrap_or_default();
    let stats = agent::compute_stats(&entries);

    Ok(Json(AnalyzeResponse {
        status: "ok".to_string(),
        provider: provider_name,
        model: model_name,
        analysis,
        error_rate: stats.error_rate(),
        total_requests: stats.total,
        error_count: stats.errors,
    }))
}
