use crate::logger::ApiLogger;
use crate::models::LogEntry;
use anyhow::Result;
use reqwest::{Client, Method, Response};
use serde_json::Value;
use std::time::Instant;

/// Wraps reqwest::Client so every call you make through it is timed and
/// logged to CSV automatically. Use this for calls TO third-party APIs.
#[derive(Clone)]
pub struct LoggedClient {
    client: Client,
    logger: ApiLogger,
}

impl LoggedClient {
    pub fn new(logger: ApiLogger) -> Self {
        Self {
            client: Client::new(),
            logger,
        }
    }

    pub async fn get(&self, url: &str) -> Result<Response> {
        self.request(Method::GET, url, None).await
    }

    pub async fn post_json(&self, url: &str, body: &Value) -> Result<Response> {
        self.request(Method::POST, url, Some(body.clone())).await
    }

    pub async fn query(&self, url: &str, body: &Value) -> Result<Response> {
        self.request(Method::from_bytes(b"QUERY")?, url, Some(body.clone()))
            .await
    }

    async fn request(&self, method: Method, url: &str, body: Option<Value>) -> Result<Response> {
        let request_size = body
            .as_ref()
            .map(|b| serde_json::to_vec(b).map(|v| v.len()).unwrap_or(0))
            .unwrap_or(0);

        let mut builder = self.client.request(method.clone(), url);
        if let Some(b) = &body {
            builder = builder.json(b);
        }

        let start = Instant::now();
        let result = builder.send().await;
        let latency_ms = start.elapsed().as_millis() as u64;

        let (status_code, response_size, error) = match &result {
            Ok(resp) => {
                let size = resp.content_length().unwrap_or(0) as usize;
                (resp.status().as_u16(), size, None)
            }
            Err(e) => (0, 0, Some(e.to_string())),
        };

        let entry = LogEntry::new(
            "client",
            method.as_str(),
            url,
            status_code,
            latency_ms,
            request_size,
            response_size,
            error,
        );

        if let Err(e) = self.logger.log(&entry).await {
            eprintln!("[api_log_tracker] failed to write log: {e}");
        }

        Ok(result?)
    }
}
