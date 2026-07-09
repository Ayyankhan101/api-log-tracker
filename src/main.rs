use api_log_tracker::{agent, middleware::log_requests, ApiLogger, LoggedClient};
use axum::{routing::get, Router};
use std::env;
use std::path::PathBuf;

const LOG_PATH: &str = "logs/api_logs.csv";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("serve");

    match mode {
        "serve" => run_server().await?,
        "demo-client" => run_demo_client().await?,
        "analyze" => run_analysis(&args).await?,
        "daemon" => {
            let port: u16 = args.get(2).and_then(|p| p.parse().ok()).unwrap_or(8080);
            let webhook = parse_flag(&args, "--webhook");
            api_log_tracker::start_daemon(port, webhook).await?;
        }
        _ => {
            print_usage();
        }
    }

    Ok(())
}

fn print_usage() {
    eprintln!("Usage: api_log_tracker <command> [options]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  serve                   Start axum server on :3000 with logging middleware");
    eprintln!("  demo-client             Make example calls through LoggedClient");
    eprintln!("  analyze [options]       Run LLM analysis on logs/api_logs.csv");
    eprintln!("  daemon [port]           Start HTTP daemon (default port 8080)");
    eprintln!();
    eprintln!("analyze options:");
    eprintln!("  --provider <name>       LLM provider (openai|anthropic|gemini|grok|groq|deepseek|qwen|baichuan|yi|stepfun|glm|ernie)");
    eprintln!("  --model <name>          Override default model for the provider");
    eprintln!();
    eprintln!("daemon options:");
    eprintln!("  [port]                  Port to listen on (default 8080)");
    eprintln!("  --webhook <url>         Webhook URL for anomaly reports");
    eprintln!();
    eprintln!("Environment variables:");
    eprintln!("  LLM_PROVIDER            Provider to use for analysis");
    eprintln!("  LLM_MODEL               Override default model");
    eprintln!("  OPENAI_API_KEY          OpenAI API key");
    eprintln!("  ANTHROPIC_API_KEY       Anthropic API key");
    eprintln!("  GEMINI_API_KEY          Google Gemini API key");
    eprintln!("  XAI_API_KEY             xAI Grok API key");
    eprintln!("  GROQ_API_KEY            Groq API key");
    eprintln!("  DEEPSEEK_API_KEY        DeepSeek API key");
    eprintln!("  DASHSCOPE_API_KEY       Qwen (Aliyun) API key");
    eprintln!("  BAICHUAN_API_KEY        Baichuan API key");
    eprintln!("  YI_API_KEY              Yi API key");
    eprintln!("  STEPFUN_API_KEY         StepFun API key");
    eprintln!("  ZHIPU_API_KEY           Zhipu GLM API key (format: id.secret)");
    eprintln!("  BAIDU_API_KEY           Baidu ERNIE client_id");
    eprintln!("  BAIDU_API_SECRET        Baidu ERNIE client_secret");
    eprintln!("  API_LOGGER_CSV          CSV log path (default: logs/api_logs.csv)");
}

fn parse_flag(args: &[String], flag: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == flag).map(|w| w[1].clone())
}

/// Starts a small axum server with the logging middleware attached.
async fn run_server() -> anyhow::Result<()> {
    let logger = ApiLogger::new(LOG_PATH);

    let app = Router::new()
        .route("/", get(|| async { "ok" }))
        .route("/health", get(|| async { "healthy" }))
        .layer(axum::middleware::from_fn_with_state(
            logger.clone(),
            log_requests,
        ))
        .with_state(logger);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("listening on http://0.0.0.0:3000 (logging to {LOG_PATH})");
    axum::serve(listener, app).await?;
    Ok(())
}

/// Makes example calls through LoggedClient.
async fn run_demo_client() -> anyhow::Result<()> {
    let logger = ApiLogger::new(LOG_PATH);
    let client = LoggedClient::new(logger);

    for _ in 0..5 {
        let _ = client.get("https://httpbin.org/get").await;
    }
    let _ = client.get("https://httpbin.org/status/500").await;

    println!("Made demo calls, check {LOG_PATH}");
    Ok(())
}

/// Runs the multi-provider agent analysis.
async fn run_analysis(args: &[String]) -> anyhow::Result<()> {
    // Set LLM_PROVIDER from --provider flag if provided
    if let Some(provider) = parse_flag(args, "--provider") {
        env::set_var("LLM_PROVIDER", &provider);
    }
    if let Some(model) = parse_flag(args, "--model") {
        env::set_var("LLM_MODEL", &model);
    }

    let result = agent::analyze_logs(&PathBuf::from(LOG_PATH)).await?;
    println!("\n=== Agent Analysis ===\n{result}\n");
    Ok(())
}
