use anyhow::{Context, Result};
use serde_json::json;
use std::env;

// ── Trait ────────────────────────────────────────────────────────────────────

#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;
    fn default_model(&self) -> &str;
    async fn analyze(&self, prompt: &str, api_key: &str, model: Option<&str>) -> Result<String>;
}

// ── OpenAI-compatible group ──────────────────────────────────────────────────

struct OpenAiCompatProvider {
    name: &'static str,
    base_url: &'static str,
    default_model: &'static str,
}

#[async_trait::async_trait]
impl LlmProvider for OpenAiCompatProvider {
    fn name(&self) -> &str { self.name }
    fn default_model(&self) -> &str { self.default_model }

    async fn analyze(&self, prompt: &str, api_key: &str, model: Option<&str>) -> Result<String> {
        let model = model.unwrap_or(self.default_model);
        let client = reqwest::Client::new();
        let url = format!("{}/v1/chat/completions", self.base_url);

        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .json(&json!({
                "model": model,
                "messages": [{"role": "user", "content": prompt}],
                "max_tokens": 500,
            }))
            .send()
            .await
            .with_context(|| format!("request to {} failed", self.name))?;

        let status = resp.status();
        let body: serde_json::Value = resp.json().await?;

        if !status.is_success() {
            anyhow::bail!("{} API error ({status}): {body}", self.name);
        }

        Ok(body["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("(no text in response)")
            .to_string())
    }
}

// ── Anthropic Claude ────────────────────────────────────────────────────────

struct AnthropicProvider;

#[async_trait::async_trait]
impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str { "anthropic" }
    fn default_model(&self) -> &str { "claude-sonnet-4-5-20250929" }

    async fn analyze(&self, prompt: &str, api_key: &str, model: Option<&str>) -> Result<String> {
        let model = model.unwrap_or(self.default_model());
        let client = reqwest::Client::new();

        let resp = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&json!({
                "model": model,
                "max_tokens": 500,
                "messages": [{"role": "user", "content": prompt}],
            }))
            .send()
            .await
            .context("request to Anthropic API failed")?;

        let status = resp.status();
        let body: serde_json::Value = resp.json().await?;

        if !status.is_success() {
            anyhow::bail!("Anthropic API error ({status}): {body}");
        }

        Ok(body["content"][0]["text"]
            .as_str()
            .unwrap_or("(no text in response)")
            .to_string())
    }
}

// ── Google Gemini ───────────────────────────────────────────────────────────

struct GeminiProvider;

#[async_trait::async_trait]
impl LlmProvider for GeminiProvider {
    fn name(&self) -> &str { "gemini" }
    fn default_model(&self) -> &str { "gemini-2.0-flash" }

    async fn analyze(&self, prompt: &str, api_key: &str, model: Option<&str>) -> Result<String> {
        let model = model.unwrap_or(self.default_model());
        let client = reqwest::Client::new();
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={api_key}"
        );

        let resp = client
            .post(&url)
            .json(&json!({
                "contents": [{"parts": [{"text": prompt}]}],
                "generationConfig": {"maxOutputTokens": 500},
            }))
            .send()
            .await
            .context("request to Gemini API failed")?;

        let status = resp.status();
        let body: serde_json::Value = resp.json().await?;

        if !status.is_success() {
            anyhow::bail!("Gemini API error ({status}): {body}");
        }

        Ok(body["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("(no text in response)")
            .to_string())
    }
}

// ── Zhipu GLM ───────────────────────────────────────────────────────────────

struct ZhipuProvider;

#[async_trait::async_trait]
impl LlmProvider for ZhipuProvider {
    fn name(&self) -> &str { "glm" }
    fn default_model(&self) -> &str { "glm-4-plus" }

    async fn analyze(&self, prompt: &str, api_key: &str, model: Option<&str>) -> Result<String> {
        let model = model.unwrap_or(self.default_model());
        let parts: Vec<&str> = api_key.splitn(2, '.').collect();
        let (key_id, key_secret) = if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            anyhow::bail!("Zhipu API key must be in format 'id.secret'");
        };

        use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

        let now = chrono::Utc::now().timestamp() as u64;
        let payload = json!({
            "api_key": key_id,
            "exp": now + 3600,
            "timestamp": now,
        });

        let token = encode(
            &Header::new(Algorithm::HS256),
            &payload,
            &EncodingKey::from_secret(key_secret.as_bytes()),
        )
        .context("failed to create Zhipu JWT")?;

        let client = reqwest::Client::new();
        let resp = client
            .post("https://open.bigmodel.cn/api/paas/v4/chat/completions")
            .header("Authorization", format!("Bearer {token}"))
            .json(&json!({
                "model": model,
                "messages": [{"role": "user", "content": prompt}],
                "max_tokens": 500,
            }))
            .send()
            .await
            .context("request to Zhipu API failed")?;

        let status = resp.status();
        let body: serde_json::Value = resp.json().await?;

        if !status.is_success() {
            anyhow::bail!("Zhipu API error ({status}): {body}");
        }

        Ok(body["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("(no text in response)")
            .to_string())
    }
}

// ── Baidu ERNIE ─────────────────────────────────────────────────────────────

struct BaiduProvider;

#[async_trait::async_trait]
impl LlmProvider for BaiduProvider {
    fn name(&self) -> &str { "ernie" }
    fn default_model(&self) -> &str { "ernie-4.0-8k" }

    async fn analyze(&self, prompt: &str, api_key: &str, model: Option<&str>) -> Result<String> {
        let model = model.unwrap_or(self.default_model());

        let client = reqwest::Client::new();

        // OAuth: get access token
        let token_resp = client
            .post("https://aip.baidubce.com/oauth/2.0/token")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", api_key),
                ("client_secret", &std::env::var("BAIDU_API_SECRET")
                    .unwrap_or_default()),
            ])
            .send()
            .await
            .context("Baidu OAuth token request failed")?;

        let token_body: serde_json::Value = token_resp.json().await?;
        let access_token = token_body["access_token"]
            .as_str()
            .context("no access_token in Baidu response")?;

        // Chat completion
        let resp = client
            .post(format!(
                "https://aip.baidubce.com/rpc/2.0/ai_custom/v1/wenxinworkshop/chat/{model}?access_token={access_token}"
            ))
            .json(&json!({
                "messages": [{"role": "user", "content": prompt}],
                "max_output_tokens": 500,
            }))
            .send()
            .await
            .context("request to Baidu ERNIE failed")?;

        let status = resp.status();
        let body: serde_json::Value = resp.json().await?;

        if !status.is_success() {
            anyhow::bail!("Baidu ERNIE API error ({status}): {body}");
        }

        Ok(body["result"]
            .as_str()
            .unwrap_or("(no text in response)")
            .to_string())
    }
}

// ── Provider registry ───────────────────────────────────────────────────────

fn make_provider(name: &str) -> Option<Box<dyn LlmProvider>> {
    match name {
        // OpenAI-compatible
        "openai" => Some(Box::new(OpenAiCompatProvider { name: "openai", base_url: "https://api.openai.com", default_model: "gpt-4o" })),
        "grok" => Some(Box::new(OpenAiCompatProvider { name: "grok", base_url: "https://api.x.ai", default_model: "grok-2" })),
        "groq" => Some(Box::new(OpenAiCompatProvider { name: "groq", base_url: "https://api.groq.com/open", default_model: "llama-3.3-70b-versatile" })),
        "deepseek" => Some(Box::new(OpenAiCompatProvider { name: "deepseek", base_url: "https://api.deepseek.com", default_model: "deepseek-chat" })),
        "qwen" => Some(Box::new(OpenAiCompatProvider { name: "qwen", base_url: "https://dashscope.aliyuncs.com/compatible-mode", default_model: "qwen-plus" })),
        "baichuan" => Some(Box::new(OpenAiCompatProvider { name: "baichuan", base_url: "https://api.baichuan-ai.com", default_model: "Baichuan4" })),
        "yi" => Some(Box::new(OpenAiCompatProvider { name: "yi", base_url: "https://api.01.ai", default_model: "yi-large" })),
        "stepfun" => Some(Box::new(OpenAiCompatProvider { name: "stepfun", base_url: "https://api.stepfun.com", default_model: "step-2-16k" })),
        // Unique APIs
        "anthropic" => Some(Box::new(AnthropicProvider)),
        "gemini" => Some(Box::new(GeminiProvider)),
        // Custom auth
        "glm" => Some(Box::new(ZhipuProvider)),
        "ernie" => Some(Box::new(BaiduProvider)),
        _ => None,
    }
}

const ALL_PROVIDER_NAMES: &[&str] = &[
    "openai", "grok", "groq", "deepseek", "qwen", "baichuan", "yi", "stepfun",
    "anthropic", "gemini", "glm", "ernie",
];

/// Returns the provider name → env key mapping for documentation.
pub fn provider_env_keys() -> Vec<(&'static str, &'static str)> {
    vec![
        ("openai", "OPENAI_API_KEY"),
        ("grok", "XAI_API_KEY"),
        ("groq", "GROQ_API_KEY"),
        ("deepseek", "DEEPSEEK_API_KEY"),
        ("qwen", "DASHSCOPE_API_KEY"),
        ("baichuan", "BAICHUAN_API_KEY"),
        ("yi", "YI_API_KEY"),
        ("stepfun", "STEPFUN_API_KEY"),
        ("anthropic", "ANTHROPIC_API_KEY"),
        ("gemini", "GEMINI_API_KEY"),
        ("glm", "ZHIPU_API_KEY"),
        ("ernie", "BAIDU_API_KEY + BAIDU_API_SECRET"),
    ]
}

/// Resolve a provider by name.
pub fn resolve_named_provider(name: &str) -> Result<(&'static str, Box<dyn LlmProvider>)> {
    let name_lower = name.to_lowercase();
    if let Some(p) = make_provider(&name_lower) {
        return Ok((ALL_PROVIDER_NAMES.iter().find(|&&n| n == name_lower).unwrap_or(&"unknown"), p));
    }
    anyhow::bail!(
        "Unknown provider: '{}'. Supported: {}",
        name,
        ALL_PROVIDER_NAMES.join(", ")
    )
}

/// Resolve the provider from env vars. Checks `LLM_PROVIDER` first,
/// then falls back to whichever key is set.
pub fn resolve_provider() -> Result<(&'static str, Box<dyn LlmProvider>)> {
    // If LLM_PROVIDER is explicitly set, use it
    if let Ok(name) = env::var("LLM_PROVIDER") {
        let name_lower = name.to_lowercase();
        if let Some(p) = make_provider(&name_lower) {
            return Ok((ALL_PROVIDER_NAMES.iter().find(|&&n| n == name_lower).unwrap_or(&"unknown"), p));
        }
        anyhow::bail!(
            "Unknown LLM_PROVIDER: '{}'. Supported: {}",
            name,
            ALL_PROVIDER_NAMES.join(", ")
        );
    }

    // Auto-detect: use whichever key is set
    let auto_detect = [
        ("openai", "OPENAI_API_KEY"),
        ("anthropic", "ANTHROPIC_API_KEY"),
        ("gemini", "GEMINI_API_KEY"),
        ("grok", "XAI_API_KEY"),
        ("groq", "GROQ_API_KEY"),
        ("deepseek", "DEEPSEEK_API_KEY"),
        ("qwen", "DASHSCOPE_API_KEY"),
        ("baichuan", "BAICHUAN_API_KEY"),
        ("yi", "YI_API_KEY"),
        ("stepfun", "STEPFUN_API_KEY"),
        ("glm", "ZHIPU_API_KEY"),
        ("ernie", "BAIDU_API_KEY"),
    ];

    for (name, env_var) in &auto_detect {
        if env::var(env_var).is_ok() {
            if let Some(p) = make_provider(name) {
                return Ok((name, p));
            }
        }
    }

    anyhow::bail!(
        "No LLM provider configured. Set LLM_PROVIDER or one of the API key env vars. \
         Supported providers: {}",
        ALL_PROVIDER_NAMES.join(", ")
    )
}
