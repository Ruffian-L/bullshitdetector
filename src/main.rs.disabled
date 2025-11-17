// Copyright (c) 2025 Jason Van Pham (ruffian-l on GitHub) @ The Niodoo Collaborative
// Licensed under the MIT License - See LICENSE file for details
// Attribution required for all derivative works

use anyhow::{Result, anyhow};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Json as AxumJson},
    routing::{get, post},
    Router,
};
use clap::{Parser, Subcommand};
use prometheus::{Histogram, Gauge, Encoder, TextEncoder, register_histogram, register_gauge};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use bullshitdetector::constants::{GOLDEN_RATIO, GOLDEN_RATIO_INV};
use bullshitdetector::{dataset, detect, feeler, integrate, lsp, memory, rag, suggest};

/// BullshitDetector CLI - NiodO.o's hardcore code quality companion
#[derive(Parser)]
#[command(name = "bullshitdetector")]
#[command(about = "Detects over-engineered, unnecessarily complex code patterns with snarky feedback")]
#[command(version = "φ ≈ 1.61803")] // Using golden ratio symbolically
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Configuration file path
    #[arg(short, long, default_value = "bullshitdetector.toml")]
    config: String,
}

/// Available CLI commands
#[derive(Subcommand)]
enum Commands {
    /// Generate synthetic dataset for training
    GenerateDataset {
        /// Output file path
        #[arg(short, long, default_value = "synthetic_bs_dataset.json")]
        output: String,

        /// Number of snippets to generate
        #[arg(short, long, default_value = "24000")]
        count: usize,

        /// Bullshit ratio (0.0-1.0)
        // Use golden ratio inverse constant φ⁻¹ = (√5 - 1)/2
        #[arg(short, long, default_value = "0.618033988749895")] // Default to exact φ⁻¹
        bs_ratio: f64,
    },

    /// Analyze code for bullshit patterns
    Detect {
        /// Code file or diff to analyze
        #[arg(short, long)]
        input: String,

        /// Output format (json, text)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Start web server for /evolve endpoint
    Server {
        /// Port to bind to
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Host to bind to
        #[arg(short, long, default_value = "0.0.0.0")]
        host: String,
    },

    /// Generate code review from diff
    Review {
        /// Diff input (file or stdin)
        #[arg(short, long)]
        diff: String,

        /// Enable assertive mode
        #[arg(short, long)]
        assertive: bool,
    },

    /// Run LSP server for IDE integration
    Lsp,

    /// Run benchmarks and performance tests
    Bench,

    /// Run all tests
    Test,
}

/// Web server state
#[derive(Clone)]
struct AppState {
    memory: Arc<Mutex<memory::SixLayerMemory>>,
    metrics: Arc<Metrics>,
}

/// Prometheus metrics
#[derive(Clone)]
pub struct Metrics {
    pub lat_hist: Histogram,
    pub coh_gauge: Gauge,
    pub bs_risk_gauge: Gauge,
    pub alert_count: prometheus::IntCounter,
}

impl Metrics {
    pub fn new() -> Result<Self> {
        let lat_hist = register_histogram!("bs_detector_latency_ms", "Latency in ms")?;
        let coh_gauge = register_gauge!("bs_detector_coherence", "Coherence score")?;
        let bs_risk_gauge = register_gauge!("bs_detector_risk", "Bullshit risk score")?;
        let alert_count = prometheus::register_int_counter!("bs_detector_alerts_total", "Total alerts detected")?;

        Ok(Self {
            lat_hist,
            coh_gauge,
            bs_risk_gauge,
            alert_count,
        })
    }
}

/// Main entry point
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    if cli.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::init();
    }

    // Execute command
    match cli.command {
        Commands::GenerateDataset { output, count, bs_ratio } => {
            run_dataset_generation(output, count, bs_ratio).await?;
        }
        Commands::Detect { input, format } => {
            run_detection(input, format).await?;
        }
        Commands::Server { port, host } => {
            run_server(host, port).await?;
        }
        Commands::Review { diff, assertive } => {
            run_review(diff, assertive).await?;
        }
        Commands::Lsp => {
            run_lsp_server().await?;
        }
        Commands::Bench => {
            run_benchmarks().await?;
        }
        Commands::Test => {
            run_tests().await?;
        }
    }

    Ok(())
}

/// Generate synthetic dataset
async fn run_dataset_generation(output: String, count: usize, bs_ratio: f64) -> Result<()> {
    tracing::info!("🚀 Generating synthetic bullshit detection dataset...");
    let golden_inv_f64 = (5.0f64.sqrt() - 1.0) / 2.0; // Actual mathematical constant for f64
    let effective_ratio = if bs_ratio == 0.618033988749895 { golden_inv_f64 } else { bs_ratio };

    let config = dataset::DatasetConfig {
        total_snippets: count,
        bs_ratio: effective_ratio,
        output_file: output,
        ..Default::default()
    };

    dataset::run_dataset_generation(config)?;

    tracing::info!("✅ Dataset generation complete!");

    Ok(())
}

/// Run code detection on input
async fn run_detection(input: String, format: String) -> Result<()> {
    tracing::info!("🔍 Analyzing code for bullshit patterns...");

    let code = std::fs::read_to_string(&input)
        .map_err(|e| anyhow!("Failed to read input file '{}': {}", input, e))?;

    let config = detect::DetectConfig::default();
    let mut alerts = detect::scan_code(&code, &config)?;

    // Score alerts with emotional probes
    let memory = memory::SixLayerMemory::new();
    detect::score_bs_confidence(&mut alerts, &memory)?;

    match format.as_str() {
        "json" => {
            tracing::info!("{}", serde_json::to_string_pretty(&alerts)?);
        }
        "text" => {
            print_detection_results(&alerts);
        }
        _ => {
            return Err(anyhow!("Unknown format: {}", format));
        }
    }

    Ok(())
}

/// Print detection results in human-readable format
fn print_detection_results(alerts: &[detect::BullshitAlert]) {
    if alerts.is_empty() {
        tracing::info!("✅ No bullshit detected! Clean code detected.");
        return;
    }

    tracing::warn!("🚨 BULLSHIT DETECTED!");
    tracing::warn!("Found {} bullshit patterns:\n", alerts.len());

    for (i, alert) in alerts.iter().enumerate() {
        tracing::info!("{}. 🔥 {} (confidence: {:.2})", i + 1, alert.issue_type, alert.confidence);
        tracing::info!("   Location: line {}, column {}", alert.location.0, alert.location.1);
        tracing::info!("   Why: {}", alert.why_bs);
        tracing::info!("   Suggestion: {}", alert.sug);
        tracing::info!("");
    }

    let total_xp = alerts.len() as u32 * 55; // F(10) = 55, exact Fibonacci number
    tracing::info!("🎮 XP REWARD: +{} points for refactoring!", total_xp);
}

/// Run web server with /evolve endpoint
async fn run_server(host: String, port: u16) -> Result<()> {
    tracing::info!("🌐 Starting BullshitDetector server on {}:{}", host, port);

    // Initialize shared state
    let memory = Arc::new(Mutex::new(memory::SixLayerMemory::new()));
    let metrics = Arc::new(Metrics::new()?);

    let state = AppState {
        memory: memory.clone(),
        metrics: metrics.clone(),
    };

    // Build router
    let app = Router::new()
        .route("/evolve", post(evolve_handler))
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_handler))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .timeout(std::time::Duration::from_secs(30))
        )
        .with_state(state);

    // Start server
    let addr = format!("{}:{}", host, port).parse::<SocketAddr>()?;
    tracing::info!("🚀 Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Handle /evolve POST requests
async fn evolve_handler(
    State(state): State<AppState>,
    Json(request): Json<integrate::DiffReq>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();

    // Scan code for bullshit
    let config = detect::DetectConfig::default();
    let mut alerts = match detect::scan_code(&request.diff, &config) {
        Ok(alerts) => alerts,
        Err(e) => {
            log::error!("Detection error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Detection failed").into_response();
        }
    };

    // Score with emotional probes
    let memory = state.memory.lock().await;
    if let Err(e) = detect::score_bs_confidence(&mut alerts, &memory) {
        log::error!("Scoring error: {}", e);
        // Continue without scoring if it fails
    }
    drop(memory);

    // Generate suggestions
    let suggest_config = suggest::SuggestConfig::default();
    let suggestions = match suggest::generate_suggestions(&alerts, &suggest_config) {
        Ok(sugs) => sugs,
        Err(e) => {
            log::error!("Suggestion error: {}", e);
            Vec::new()
        }
    };

    // Generate RAG review
    let rag_request = rag::RagReviewRequest { alerts, suggestions };
    let rag_config = rag::RagConfig::default();
    let review = match rag::run_enhanced_rag_generation(rag_request, rag_config).await {
        Ok(review) => review,
        Err(e) => {
            log::error!("RAG generation error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Review generation failed").into_response();
        }
    };

    // Update metrics
    let latency_ms = start.elapsed().as_millis() as f64;
    state.metrics.lat_hist.observe(latency_ms);
    state.metrics.coh_gauge.set(review.coherence as f64);
    state.metrics.bs_risk_gauge.set(review.severity as f64);
    state.metrics.alert_count.inc();

    (StatusCode::OK, AxumJson(review)).into_response()
}

/// Handle /metrics GET requests for Prometheus
async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();

    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        log::error!("Metrics encoding error: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Metrics encoding failed");
    }

    ([("content-type", "text/plain; charset=utf-8")], buffer)
}

/// Health check endpoint
async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "healthy")
}

/// Generate code review from diff
async fn run_review(diff: String, assertive: bool) -> Result<()> {
    tracing::info!("📝 Generating code review...");

    let code = std::fs::read_to_string(&diff)
        .map_err(|e| anyhow!("Failed to read diff file '{}': {}", diff, e))?;

    let config = detect::DetectConfig::default();
    let mut alerts = detect::scan_code(&code, &config)?;

    let memory = memory::SixLayerMemory::new();
    detect::score_bs_confidence(&mut alerts, &memory)?;

    let rag_config = rag::RagConfig {
        enable_assertive_mode: assertive,
        ..Default::default()
    };

    let request = rag::RagReviewRequest {
        alerts,
        suggestions: Vec::new(),
    };

    let response = rag::run_enhanced_rag_generation(request, rag_config).await?;

    tracing::info!("\n🎯 CODE REVIEW:");
    tracing::info!("Summary: {}", response.summary);
    tracing::info!("Severity: {:.2}", response.severity);
    tracing::info!("Coherence: {:.2}", response.coherence);
    tracing::info!("\n📋 RECOMMENDATIONS:");
    for rec in &response.recommendations {
        tracing::info!("• {}", rec);
    }

    Ok(())
}

/// Run LSP server for IDE integration
async fn run_lsp_server() -> Result<()> {
    tracing::info!("🔧 Starting LSP server for IDE integration...");

    lsp::run_lsp_server().await?;

    Ok(())
}

/// Run performance benchmarks
async fn run_benchmarks() -> Result<()> {
    tracing::info!("⚡ Running performance benchmarks...");

    // Run criterion benchmarks
    let output = std::process::Command::new("cargo")
        .args(&["criterion"])
        .output()?;

    if !output.status.success() {
        tracing::info!("Benchmark failed: {}", String::from_utf8_lossy(&output.stderr));
        return Err(anyhow!("Benchmark execution failed"));
    }

    tracing::info!("✅ Benchmarks completed successfully!");

    Ok(())
}

/// Run all tests
async fn run_tests() -> Result<()> {
    tracing::info!("🧪 Running test suite...");

    let output = std::process::Command::new("cargo")
        .args(&["test", "--all"])
        .output()?;

    if !output.status.success() {
        tracing::info!("Tests failed: {}", String::from_utf8_lossy(&output.stderr));
        return Err(anyhow!("Test execution failed"));
    }

    tracing::info!("✅ All tests passed!");

    Ok(())
}
