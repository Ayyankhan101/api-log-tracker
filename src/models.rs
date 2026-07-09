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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_entry_has_uuid_and_timestamp() {
        let entry = LogEntry::new("server", "GET", "/api/test", 200, 45, 10, 200, None);
        assert!(!entry.id.is_empty());
        assert!(!entry.timestamp.is_empty());
        assert_eq!(entry.source, "server");
        assert_eq!(entry.method, "GET");
        assert_eq!(entry.endpoint, "/api/test");
        assert_eq!(entry.status_code, 200);
        assert_eq!(entry.latency_ms, 45);
        assert_eq!(entry.request_size, 10);
        assert_eq!(entry.response_size, 200);
        assert!(entry.error.is_none());
    }

    #[test]
    fn new_entry_with_error() {
        let entry = LogEntry::new(
            "client",
            "POST",
            "/api/data",
            500,
            1200,
            50,
            0,
            Some("timeout".into()),
        );
        assert_eq!(entry.error.as_deref(), Some("timeout"));
        assert_eq!(entry.status_code, 500);
    }

    #[test]
    fn new_entries_have_unique_ids() {
        let a = LogEntry::new("server", "GET", "/", 200, 10, 0, 0, None);
        let b = LogEntry::new("server", "GET", "/", 200, 10, 0, 0, None);
        assert_ne!(a.id, b.id);
    }
}
