use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct WebhookConfig {
    pub url: String,
    pub error_threshold: f64,
    pub min_entries: usize,
}

#[derive(Debug)]
pub struct Stats {
    pub total: usize,
    pub errors: usize,
    pub avg_latency_ms: f64,
    pub max_latency_ms: u64,
    pub by_endpoint: HashMap<String, usize>,
    pub by_status: HashMap<u16, usize>,
}

impl Stats {
    pub fn error_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.errors as f64 / self.total as f64) * 100.0
        }
    }
}

#[derive(Serialize)]
pub struct WebhookPayload {
    pub event: String,
    pub timestamp: String,
    pub summary: String,
    pub error_rate: f64,
    pub total_requests: usize,
    pub error_count: usize,
    pub latency_avg_ms: f64,
    pub latency_max_ms: u64,
    pub anomalies: Vec<String>,
    pub recommendations: Vec<String>,
    pub error_clusters: HashMap<u16, usize>,
    pub latency_hotspots: Vec<LatencyHotspot>,
}

#[derive(Serialize)]
pub struct LatencyHotspot {
    pub endpoint: String,
    pub avg_ms: f64,
}

/// Check if current stats warrant a webhook report.
pub async fn check_and_report(current: &Stats, config: &WebhookConfig) -> Result<()> {
    if current.total < config.min_entries {
        return Ok(());
    }

    let error_rate = current.error_rate();
    if error_rate <= config.error_threshold {
        return Ok(());
    }

    // Find anomalies
    let mut anomalies = Vec::new();
    let mut recommendations = Vec::new();

    if error_rate > config.error_threshold {
        anomalies.push(format!(
            "Error rate {error_rate:.1}% exceeds threshold of {:.1}%",
            config.error_threshold
        ));
        recommendations.push("Investigate error-prone endpoints and add retry logic".to_string());
    }

    // Find latency hotspots
    let latency_hotspots: Vec<LatencyHotspot> = current
        .by_endpoint
        .iter()
        .filter(|(_, _)| current.avg_latency_ms > 1000.0)
        .map(|(ep, _)| LatencyHotspot {
            endpoint: ep.clone(),
            avg_ms: current.avg_latency_ms,
        })
        .collect();

    if current.max_latency_ms > 5000 {
        anomalies.push(format!(
            "Max latency of {}ms is dangerously high",
            current.max_latency_ms
        ));
        recommendations.push("Add connection pooling and consider caching".to_string());
    }

    let payload = WebhookPayload {
        event: "api_anomaly_detected".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        summary: format!(
            "API health alert: {:.1}% error rate across {} requests",
            error_rate, current.total
        ),
        error_rate,
        total_requests: current.total,
        error_count: current.errors,
        latency_avg_ms: current.avg_latency_ms,
        latency_max_ms: current.max_latency_ms,
        anomalies,
        recommendations,
        error_clusters: current.by_status.clone(),
        latency_hotspots,
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(&config.url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .context("webhook POST failed")?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        tracing::error!(status = %status, body = %body, "webhook error");
    } else {
        tracing::info!(url = %config.url, "anomaly report sent");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_error_rate_zero_entries() {
        let stats = Stats {
            total: 0,
            errors: 0,
            avg_latency_ms: 0.0,
            max_latency_ms: 0,
            by_endpoint: HashMap::new(),
            by_status: HashMap::new(),
        };
        assert_eq!(stats.error_rate(), 0.0);
    }

    #[test]
    fn stats_error_rate_calculation() {
        let stats = Stats {
            total: 100,
            errors: 25,
            avg_latency_ms: 150.0,
            max_latency_ms: 2000,
            by_endpoint: HashMap::new(),
            by_status: HashMap::new(),
        };
        assert!((stats.error_rate() - 25.0).abs() < 0.01);
    }

    #[test]
    fn stats_error_rate_all_errors() {
        let stats = Stats {
            total: 50,
            errors: 50,
            avg_latency_ms: 500.0,
            max_latency_ms: 5000,
            by_endpoint: HashMap::new(),
            by_status: HashMap::new(),
        };
        assert!((stats.error_rate() - 100.0).abs() < 0.01);
    }

    #[test]
    fn check_and_report_skips_when_below_min_entries() {
        let stats = Stats {
            total: 3,
            errors: 3,
            avg_latency_ms: 100.0,
            max_latency_ms: 500,
            by_endpoint: HashMap::new(),
            by_status: HashMap::new(),
        };
        let config = WebhookConfig {
            url: "http://localhost:9999/hook".to_string(),
            error_threshold: 5.0,
            min_entries: 10,
        };
        // Should return Ok without sending (below min_entries)
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let result = check_and_report(&stats, &config).await;
            assert!(result.is_ok());
        });
    }

    #[test]
    fn check_and_report_skips_when_below_threshold() {
        let stats = Stats {
            total: 20,
            errors: 0,
            avg_latency_ms: 50.0,
            max_latency_ms: 200,
            by_endpoint: HashMap::new(),
            by_status: HashMap::new(),
        };
        let config = WebhookConfig {
            url: "http://localhost:9999/hook".to_string(),
            error_threshold: 5.0,
            min_entries: 10,
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let result = check_and_report(&stats, &config).await;
            assert!(result.is_ok());
        });
    }
}
