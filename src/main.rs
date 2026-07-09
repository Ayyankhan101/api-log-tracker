use api_log_tracker::{agent, middleware::log_requests, ApiLogger, LoggedClient};
use axum::{routing::get, Router};
use std::env;
use std::path::PathBuf;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const LOG_PATH: &str = "logs/api_logs.csv";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

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
        "help" | "--help" | "-h" => {
            print_usage();
        }
        "--version" | "-v" => {
            println!("api_log_tracker v{VERSION}");
        }
        _ => {
            eprintln!("Unknown command: {mode}");
            eprintln!("Run 'api_log_tracker help' for usage.");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_usage() {
    println!("api_log_tracker v{VERSION}");
    println!("API log tracker with multi-provider LLM anomaly analysis");
    println!();
    println!("USAGE:");
    println!("  api_log_tracker <command> [options]");
    println!();
    println!("COMMANDS:");
    println!("  serve               Start axum server on :3000 with logging middleware");
    println!("  demo-client         Make example calls through LoggedClient");
    println!("  analyze [options]   Run LLM analysis on logs/api_logs.csv");
    println!("  daemon [port]       Start HTTP daemon (default port 8080)");
    println!("  help                Show this help message");
    println!();
    println!("OPTIONS:");
    println!("  --provider <name>   LLM provider (openai|anthropic|gemini|grok|groq|deepseek|qwen|baichuan|yi|stepfun|glm|ernie)");
    println!("  --model <name>      Override default model for the provider");
    println!("  --webhook <url>     Webhook URL for anomaly reports");
    println!("  --port <port>       Daemon listen port (default 8080)");
    println!();
    println!("ENVIRONMENT:");
    println!("  LLM_PROVIDER            Provider to use for analysis");
    println!("  LLM_MODEL               Override default model");
    println!("  RUST_LOG                Log level filter (default: info)");
    println!("  API_LOGGER_CSV          CSV log path (default: logs/api_logs.csv)");
    println!();
    println!("API KEYS:");
    println!("  OPENAI_API_KEY, ANTHROPIC_API_KEY, GEMINI_API_KEY, XAI_API_KEY,");
    println!("  GROQ_API_KEY, DEEPSEEK_API_KEY, DASHSCOPE_API_KEY, BAICHUAN_API_KEY,");
    println!("  YI_API_KEY, STEPFUN_API_KEY, ZHIPU_API_KEY, BAIDU_API_KEY, BAIDU_API_SECRET");
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
