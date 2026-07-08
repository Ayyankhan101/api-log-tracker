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
            eprintln!("[api_log_tracker] failed to write log: {e}");
        }
    });

    response
}
