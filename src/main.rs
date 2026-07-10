use api_log_tracker::{agent, middleware::log_requests, ApiLogger, LoggedClient};
use axum::{routing::get, Router};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, shells};
use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

// ── CLI definition ──────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "api_log_tracker",
    version,
    about = "API log tracker with multi-provider LLM anomaly analysis",
    long_about = "API log tracker with multi-provider LLM anomaly analysis, HTTP daemon, and TUI dashboard.\n\nRuns as an HTTP daemon that accepts API logs from any language, then optionally analyzes them with 13 LLM providers."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start axum server on :3000 with logging middleware
    Serve {
        /// Listen port
        #[arg(long, default_value_t = 3000, env = "API_LOGGER_PORT")]
        port: u16,

        /// CSV log path
        #[arg(long, default_value = "logs/api_logs.csv", env = "API_LOGGER_CSV")]
        csv: PathBuf,
    },

    /// Make example calls through LoggedClient
    DemoClient {
        /// CSV log path
        #[arg(long, default_value = "logs/api_logs.csv", env = "API_LOGGER_CSV")]
        csv: PathBuf,
    },

    /// Run LLM analysis on log entries
    Analyze {
        /// CSV log path
        #[arg(long, default_value = "logs/api_logs.csv", env = "API_LOGGER_CSV")]
        csv: PathBuf,

        /// LLM provider
        #[arg(long, env = "LLM_PROVIDER")]
        provider: Option<String>,

        /// Override default model for the provider
        #[arg(long, env = "LLM_MODEL")]
        model: Option<String>,
    },

    /// Start HTTP daemon (default port 8080)
    Daemon {
        /// Listen port
        #[arg(long, default_value_t = 8080, env = "API_LOGGER_PORT")]
        port: u16,

        /// CSV log path
        #[arg(long, default_value = "logs/api_logs.csv", env = "API_LOGGER_CSV")]
        csv: PathBuf,

        /// Webhook URL for anomaly reports
        #[arg(long)]
        webhook: Option<String>,
    },

    /// Launch the TUI dashboard (requires Node.js)
    Tui {
        /// CSV log path
        #[arg(long, default_value = "logs/api_logs.csv", env = "API_LOGGER_CSV")]
        csv: PathBuf,
    },

    /// Generate shell completions
    Completions {
        /// Target shell
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Clone, ValueEnum)]
enum Shell {
    Bash,
    Elvish,
    Fish,
    Powershell,
    Zsh,
}

impl From<Shell> for shells::Shell {
    fn from(s: Shell) -> Self {
        match s {
            Shell::Bash => shells::Shell::Bash,
            Shell::Elvish => shells::Shell::Elvish,
            Shell::Fish => shells::Shell::Fish,
            Shell::Powershell => shells::Shell::PowerShell,
            Shell::Zsh => shells::Shell::Zsh,
        }
    }
}

// ── Main ────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Serve {
        port: 3000,
        csv: PathBuf::from("logs/api_logs.csv"),
    }) {
        Commands::Serve { port, csv } => run_server(port, csv).await?,
        Commands::DemoClient { csv } => run_demo_client(csv).await?,
        Commands::Analyze { csv, provider, model } => run_analysis(csv, provider, model).await?,
        Commands::Daemon { port, csv, webhook } => run_daemon(port, csv, webhook).await?,
        Commands::Tui { csv } => run_tui(csv)?,
        Commands::Completions { shell } => run_completions(shell)?,
    }

    Ok(())
}

// ── Commands ────────────────────────────────────────────────────────────────

async fn run_server(port: u16, csv: PathBuf) -> anyhow::Result<()> {
    let csv_str = csv.to_string_lossy().to_string();
    let logger = ApiLogger::new(&csv_str);

    let app = Router::new()
        .route("/", get(|| async { "ok" }))
        .route("/health", get(|| async { "healthy" }))
        .layer(axum::middleware::from_fn_with_state(
            logger.clone(),
            log_requests,
        ))
        .with_state(logger);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    tracing::info!("listening on http://0.0.0.0:{port} (logging to {csv_str})");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn run_demo_client(csv: PathBuf) -> anyhow::Result<()> {
    let csv_str = csv.to_string_lossy().to_string();
    let logger = ApiLogger::new(&csv_str);
    let client = LoggedClient::new(logger);

    for _ in 0..5 {
        let _ = client.get("https://httpbin.org/get").await;
    }
    let _ = client.get("https://httpbin.org/status/500").await;

    tracing::info!("made demo calls, check {csv_str}");
    Ok(())
}

async fn run_analysis(
    csv: PathBuf,
    provider: Option<String>,
    model: Option<String>,
) -> anyhow::Result<()> {
    if let Some(ref p) = provider {
        env::set_var("LLM_PROVIDER", p);
    }
    if let Some(ref m) = model {
        env::set_var("LLM_MODEL", m);
    }

    let result = agent::analyze_logs(&csv).await?;
    println!("\n=== Agent Analysis ===\n{result}\n");
    Ok(())
}

async fn run_daemon(port: u16, csv: PathBuf, webhook: Option<String>) -> anyhow::Result<()> {
    // Set API_LOGGER_CSV so daemon picks it up
    env::set_var("API_LOGGER_CSV", csv.to_string_lossy().to_string());
    api_log_tracker::start_daemon(port, webhook).await
}

fn run_tui(csv: PathBuf) -> anyhow::Result<()> {
    let csv_str = csv.to_string_lossy().to_string();

    // 1. Try npx (user has npm installed globally)
    let npx = Command::new("npx").args(["api-log-tracker-tui"]).spawn();
    if let Ok(mut child) = npx {
        let status = child.wait()?;
        if status.success() {
            return Ok(());
        }
    }

    // 2. Try local development path
    let local_cli = PathBuf::from("my-terminal-ui/dist/cli.js");
    if local_cli.exists() {
        let status = Command::new("node")
            .arg(&local_cli)
            .arg("--csv")
            .arg(&csv_str)
            .spawn()?
            .wait()?;
        if status.success() {
            return Ok(());
        }
    }

    // 3. Fail with clear message
    eprintln!("error: TUI requires Node.js and the api-log-tracker-tui npm package.");
    eprintln!("       Install with: npm install -g api-log-tracker-tui");
    eprintln!("       Then run:     api_log_tracker tui");
    eprintln!();
    eprintln!("       Or from a clone of this repo:");
    eprintln!("         cd my-terminal-ui && npm install && npm run build");
    eprintln!("         api_log_tracker tui");
    std::process::exit(1);
}

fn run_completions(shell: Shell) -> anyhow::Result<()> {
    let mut cli = Cli::command();
    let shell: shells::Shell = shell.into();
    let name = "api_log_tracker";
    let mut buf = std::io::stdout();
    generate(shell, &mut cli, name, &mut buf);
    buf.flush()?;
    Ok(())
}
