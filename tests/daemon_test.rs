use api_log_tracker::daemon::{self, DaemonState};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use tower::util::ServiceExt;

fn test_app() -> Router {
    let tmp = std::env::temp_dir().join(format!("api_log_tracker_test_{}", uuid::Uuid::new_v4()));
    let csv_path = tmp.join("test.csv");
    std::fs::create_dir_all(&tmp).unwrap();

    let logger = api_log_tracker::ApiLogger::new(&csv_path);

    let state = DaemonState {
        logger,
        csv_path,
    };

    Router::new()
        .route("/api", axum::routing::get(daemon::api_index))
        .route("/api/log", axum::routing::post(daemon::post_log))
        .route("/api/health", axum::routing::get(daemon::health))
        .route("/api/logs", axum::routing::get(daemon::get_logs))
        .with_state(state)
}

#[tokio::test]
async fn health_returns_ok() {
    let app = test_app();
    let req = Request::builder()
        .uri("/api/health")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn api_index_returns_endpoints() {
    let app = test_app();
    let req = Request::builder()
        .uri("/api")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["service"], "api_log_tracker");
    assert!(json["endpoints"].is_object());
}

#[tokio::test]
async fn post_log_creates_entry() {
    let app = test_app();
    let payload = serde_json::json!({
        "source": "test",
        "method": "GET",
        "endpoint": "/test",
        "status_code": 200,
        "latency_ms": 42,
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/log")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn get_logs_returns_empty_when_no_data() {
    let app = test_app();
    let req = Request::builder()
        .uri("/api/logs")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn get_logs_with_limit() {
    let app = test_app();

    // Insert 3 entries
    for i in 0..3 {
        let payload = serde_json::json!({
            "source": "test",
            "method": "GET",
            "endpoint": format!("/test/{i}"),
            "status_code": 200,
            "latency_ms": 10,
        });

        let req = Request::builder()
            .method("POST")
            .uri("/api/log")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    // Query with limit=2
    let req = Request::builder()
        .uri("/api/logs?limit=2")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json.as_array().unwrap().len(), 2);
}
