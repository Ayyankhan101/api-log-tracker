use crate::logger::ApiLogger;
use crate::models::LogEntry;
use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::time::Instant;

/// Drop this into an axum router with:
///   .layer(middleware::from_fn_with_state(logger.clone(), log_requests))
///
/// Note: for simplicity this reads sizes off Content-Length headers rather
/// than buffering the full body (buffering costs memory + extra complexity
/// for streaming bodies). Good enough for tracking, not byte-perfect for
/// chunked bodies without a Content-Length.
pub async fn log_requests(
    State(logger): State<ApiLogger>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let method = req.method().to_string();
    let endpoint = req.uri().path().to_string();
    let request_size = req
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);

    let start = Instant::now();
    let response = next.run(req).await;
    let latency_ms = start.elapsed().as_millis() as u64;

    let status_code = response.status().as_u16();
    let response_size = response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);

    let error = if status_code >= 400 {
        Some(format!("HTTP {}", status_code))
    } else {
        None
    };

    let entry = LogEntry::new(
        "server",
        &method,
        &endpoint,
        status_code,
        latency_ms,
        request_size,
        response_size,
        error,
    );

    // Fire-and-forget so a logging hiccup never blocks the response path.
    let logger = logger.clone();
    tokio::spawn(async move {
        if let Err(e) = logger.log(&entry).await {
            tracing::error!(error = %e, "failed to write log");
        }
    });

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
        Router,
    };
    use std::sync::Once;
    use tower::ServiceExt;

    static INIT: Once = Once::new();
    fn init_tracing() {
        INIT.call_once(|| {
            let _ = tracing_subscriber::fmt()
                .with_env_filter(tracing_subscriber::EnvFilter::new("off"))
                .try_init();
        });
    }

    #[tokio::test]
    async fn log_requests_writes_entry_for_ok_response() {
        init_tracing();
        let dir = std::env::temp_dir().join(format!("middleware_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let csv = dir.join("logs.csv");
        let csv_str = csv.to_string_lossy().to_string();
        let logger = ApiLogger::new(&csv_str);

        let app = Router::new()
            .route("/health", get(|| async { "healthy" }))
            .layer(middleware::from_fn_with_state(logger.clone(), log_requests))
            .with_state(logger);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Give the spawned task time to finish
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let content = std::fs::read_to_string(&csv).unwrap();
        assert!(content.contains("GET"));
        assert!(content.contains("/health"));
        assert!(content.contains("200"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn log_requests_captures_error_status() {
        init_tracing();
        let dir = std::env::temp_dir().join(format!("middleware_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let csv = dir.join("logs.csv");
        let csv_str = csv.to_string_lossy().to_string();
        let logger = ApiLogger::new(&csv_str);

        let app = Router::new()
            .route(
                "/error",
                get(|| async { (StatusCode::INTERNAL_SERVER_ERROR, "fail") }),
            )
            .layer(middleware::from_fn_with_state(logger.clone(), log_requests))
            .with_state(logger);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/error")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let content = std::fs::read_to_string(&csv).unwrap();
        assert!(content.contains("500"));
        assert!(content.contains("HTTP 500"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
