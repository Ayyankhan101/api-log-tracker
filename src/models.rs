use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// One row in the CSV. Works for both "my own API" (server-side, via the
/// axum middleware) and "third-party API" (client-side, via LoggedClient).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: String,
    pub source: String, // "server" or "client" — which side generated this call
    pub method: String,
    pub endpoint: String,
    pub status_code: u16,
    pub latency_ms: u64,
    pub request_size: usize,
    pub response_size: usize,
    pub error: Option<String>,
}

impl LogEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source: &str,
        method: &str,
        endpoint: &str,
        status_code: u16,
        latency_ms: u64,
        request_size: usize,
        response_size: usize,
        error: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            source: source.to_string(),
            method: method.to_string(),
            endpoint: endpoint.to_string(),
            status_code,
            latency_ms,
            request_size,
            response_size,
            error,
        }
    }
}
