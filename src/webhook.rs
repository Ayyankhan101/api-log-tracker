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
        if self.total == 0 { 0.0 } else { (self.errors as f64 / self.total as f64) * 100.0 }
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
pub async fn check_and_report(
    current: &Stats,
    config: &WebhookConfig,
) -> Result<()> {
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
        anomalies.push(format!("Error rate {error_rate:.1}% exceeds threshold of {:.1}%", config.error_threshold));
        recommendations.push("Investigate error-prone endpoints and add retry logic".to_string());
    }

    // Find latency hotspots
    let latency_hotspots: Vec<LatencyHotspot> = current.by_endpoint.iter()
        .filter(|(_, _)| current.avg_latency_ms > 1000.0)
        .map(|(ep, _)| LatencyHotspot {
            endpoint: ep.clone(),
            avg_ms: current.avg_latency_ms,
        })
        .collect();

    if current.max_latency_ms > 5000 {
        anomalies.push(format!("Max latency of {}ms is dangerously high", current.max_latency_ms));
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
        eprintln!("[webhook] error {status}: {body}");
    } else {
        println!("[webhook] anomaly report sent to {}", config.url);
    }

    Ok(())
}
