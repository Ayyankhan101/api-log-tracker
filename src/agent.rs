use crate::models::LogEntry;
use crate::provider;
use crate::webhook::Stats;
use anyhow::{Context, Result};
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

// ── Cache ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct CachedResult {
    text: String,
    provider: String,
    model: String,
    created_at: Instant,
}

const CACHE_TTL: Duration = Duration::from_secs(60);

struct AnalysisCache {
    inner: Mutex<HashMap<String, CachedResult>>,
}

impl AnalysisCache {
    fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    fn get(&self, key: &str) -> Option<CachedResult> {
        let cache = self.inner.lock().unwrap();
        cache.get(key).and_then(|v| {
            if v.created_at.elapsed() < CACHE_TTL {
                Some(v.clone())
            } else {
                None
            }
        })
    }

    fn insert(&self, key: String, value: CachedResult) {
        self.inner.lock().unwrap().insert(key, value);
    }
}

static CACHE: once_cell::sync::Lazy<AnalysisCache> = once_cell::sync::Lazy::new(AnalysisCache::new);

// ── Stats computation ────────────────────────────────────────────────────────

pub fn compute_stats(entries: &[LogEntry]) -> Stats {
    let total = entries.len();
    let errors = entries.iter().filter(|e| e.error.is_some()).count();
    let avg_latency_ms = if total > 0 {
        entries.iter().map(|e| e.latency_ms as f64).sum::<f64>() / total as f64
    } else {
        0.0
    };
    let max_latency_ms = entries.iter().map(|e| e.latency_ms).max().unwrap_or(0);

    let mut by_endpoint: HashMap<String, usize> = HashMap::new();
    let mut by_status: HashMap<u16, usize> = HashMap::new();
    for e in entries {
        *by_endpoint.entry(e.endpoint.clone()).or_insert(0) += 1;
        *by_status.entry(e.status_code).or_insert(0) += 1;
    }

    Stats {
        total,
        errors,
        avg_latency_ms,
        max_latency_ms,
        by_endpoint,
        by_status,
    }
}

pub fn read_entries(csv_path: &Path) -> Result<Vec<LogEntry>> {
    let mut rdr = csv::Reader::from_path(csv_path)
        .with_context(|| format!("could not open {}", csv_path.display()))?;
    let mut entries = Vec::new();
    for result in rdr.deserialize() {
        let entry: LogEntry = result?;
        entries.push(entry);
    }
    Ok(entries)
}

fn stats_hash(stats: &Stats) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(format!("{:?}", stats).as_bytes());
    hex::encode(hasher.finalize())
}

// ── Prompt ───────────────────────────────────────────────────────────────────

fn build_prompt(stats: &Stats) -> String {
    let mut top_endpoints: Vec<_> = stats.by_endpoint.iter().collect();
    top_endpoints.sort_by(|a, b| b.1.cmp(a.1));
    top_endpoints.truncate(10);

    let summary = json!({
        "total_requests": stats.total,
        "error_count": stats.errors,
        "error_rate_pct": stats.error_rate(),
        "avg_latency_ms": stats.avg_latency_ms,
        "max_latency_ms": stats.max_latency_ms,
        "top_endpoints": top_endpoints.iter().map(|(k, v)| json!({"endpoint": k, "count": v})).collect::<Vec<_>>(),
        "status_code_breakdown": stats.by_status,
    });

    format!(
        "You are monitoring API traffic for a system. Here is a stats summary \
         (not raw logs) covering the current CSV log file:\n\n{}\n\n\
         Give a short analysis: 1) anything anomalous (latency spikes, error \
         concentration on specific endpoints), 2) overall health verdict, \
         3) one concrete recommendation if something looks off. Be concise.",
        serde_json::to_string_pretty(&summary).unwrap_or_default()
    )
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Analyze logs using the configured LLM provider.
/// Reads CSV → computes stats → hashes stats for cache → calls provider.
pub async fn analyze_logs(csv_path: &Path) -> Result<String> {
    let entries = read_entries(csv_path)?;
    if entries.is_empty() {
        return Ok("No log entries found yet — nothing to analyze.".to_string());
    }

    let stats = compute_stats(&entries);
    let cache_key = stats_hash(&stats);

    // Check cache
    if let Some(cached) = CACHE.get(&cache_key) {
        return Ok(format!(
            "{} [cached, provider={}]",
            cached.text, cached.provider
        ));
    }

    let (provider_name, provider_impl) = provider::resolve_provider()?;
    let prompt = build_prompt(&stats);
    let model_override = std::env::var("LLM_MODEL").ok();
    let model_ref = model_override.as_deref();

    let text = provider_impl
        .analyze(&prompt, "", model_ref)
        .await
        .with_context(|| format!("analysis failed with provider {provider_name}"))?;

    let model_name = model_override.unwrap_or_else(|| provider_impl.default_model().to_string());

    // Store in cache
    CACHE.insert(
        cache_key,
        CachedResult {
            text: text.clone(),
            provider: provider_name.to_string(),
            model: model_name,
            created_at: Instant::now(),
        },
    );

    Ok(text)
}

/// Analyze logs using a specific provider and API key (for daemon endpoint).
pub async fn analyze_logs_with(
    csv_path: &Path,
    provider_name: Option<&str>,
    model: Option<&str>,
) -> Result<(String, String, String)> {
    let entries = read_entries(csv_path)?;
    if entries.is_empty() {
        return Ok((
            "No log entries found yet — nothing to analyze.".to_string(),
            "none".to_string(),
            "none".to_string(),
        ));
    }

    let stats = compute_stats(&entries);
    let cache_key = stats_hash(&stats);

    if let Some(cached) = CACHE.get(&cache_key) {
        return Ok((cached.text, cached.provider, cached.model));
    }

    let (pname, provider_impl) = if let Some(name) = provider_name {
        provider::resolve_named_provider(name)?
    } else {
        provider::resolve_provider()?
    };

    let prompt = build_prompt(&stats);
    let text = provider_impl.analyze(&prompt, "", model).await?;

    let model_name = model.unwrap_or(provider_impl.default_model()).to_string();
    let provider_name_str = pname.to_string();

    CACHE.insert(
        cache_key,
        CachedResult {
            text: text.clone(),
            provider: provider_name_str.clone(),
            model: model_name.clone(),
            created_at: Instant::now(),
        },
    );

    Ok((text, provider_name_str, model_name))
}

/// Get current stats (for daemon health and webhook).
pub fn get_stats(csv_path: &Path) -> Result<Stats> {
    let entries = read_entries(csv_path)?;
    Ok(compute_stats(&entries))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::LogEntry;

    fn make_entry(endpoint: &str, status: u16, latency: u64, error: Option<&str>) -> LogEntry {
        LogEntry::new(
            "server",
            "GET",
            endpoint,
            status,
            latency,
            0,
            0,
            error.map(String::from),
        )
    }

    #[test]
    fn compute_stats_empty() {
        let stats = compute_stats(&[]);
        assert_eq!(stats.total, 0);
        assert_eq!(stats.errors, 0);
        assert_eq!(stats.avg_latency_ms, 0.0);
        assert_eq!(stats.max_latency_ms, 0);
    }

    #[test]
    fn compute_stats_basic() {
        let entries = vec![
            make_entry("/a", 200, 100, None),
            make_entry("/a", 200, 200, None),
            make_entry("/b", 500, 300, Some("err")),
        ];
        let stats = compute_stats(&entries);
        assert_eq!(stats.total, 3);
        assert_eq!(stats.errors, 1);
        assert_eq!(stats.max_latency_ms, 300);
        assert!((stats.avg_latency_ms - 200.0).abs() < 0.01);
        assert_eq!(*stats.by_endpoint.get("/a").unwrap(), 2);
        assert_eq!(*stats.by_endpoint.get("/b").unwrap(), 1);
        assert_eq!(*stats.by_status.get(&200).unwrap(), 2);
        assert_eq!(*stats.by_status.get(&500).unwrap(), 1);
    }

    #[test]
    fn error_rate_calculation() {
        let entries = vec![
            make_entry("/", 200, 10, None),
            make_entry("/", 200, 10, None),
            make_entry("/", 200, 10, None),
            make_entry("/", 200, 10, None),
            make_entry("/", 500, 10, Some("err")),
        ];
        let stats = compute_stats(&entries);
        let rate = stats.error_rate();
        assert!((rate - 20.0).abs() < 0.01);
    }

    #[test]
    fn error_rate_zero_entries() {
        let stats = compute_stats(&[]);
        assert_eq!(stats.error_rate(), 0.0);
    }
}
