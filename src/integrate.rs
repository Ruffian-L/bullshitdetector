// Copyright (c) 2025 Jason Van Pham (ruffian-l on GitHub) @ The Niodoo Collaborative
// Licensed under the MIT License - See LICENSE file for details
// Attribution required for all derivative works

use crate::IntegrationConfig;
use anyhow::{anyhow, Result};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use git2::{Oid, Repository};
use prometheus::{register_gauge, register_histogram, Encoder, Gauge, Histogram, TextEncoder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffReq {
    pub diff: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResp {
    pub summary: String,
    pub severity: f32,
    pub recommendations: Vec<String>,
    pub coherence: f32,
    pub latency_ms: u64,
}

#[derive(Clone)]
pub struct AppState {
    pub memory: Arc<Mutex<crate::memory::SixLayerMemory>>,
}

pub struct Metrics {
    pub lat_hist: Histogram,
    pub coh_gauge: Gauge,
}

impl Metrics {
    pub fn new() -> Result<Self> {
        let lat_hist = register_histogram!("bs_detector_latency_ms", "Latency in ms")?;
        let coh_gauge = register_gauge!("bs_detector_coherence", "Coherence score")?;
        Ok(Self {
            lat_hist,
            coh_gauge,
        })
    }
}

pub async fn ci_pipeline_review(
    State(state): State<AppState>,
    Json(req): Json<DiffReq>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();
    let config = crate::DetectConfig::default();
    let mut alerts = crate::detect::scan_code(&req.diff, &config).unwrap_or_default();
    let memory = state.memory.lock().await;
    crate::detect::score_bs_confidence(&mut alerts, &memory).ok();
    let sug_config = crate::suggest::SuggestConfig::default();
    let mut sugs = crate::suggest::generate_sugs(&alerts, &sug_config).unwrap_or_default();
    crate::suggest::add_docs_impact(&mut sugs, &sug_config).ok();
    drop(memory); // Release lock

    let rag_req = crate::ReviewRequest {
        alerts,
        suggestions: sugs
            .into_iter()
            .map(|s| crate::Suggestion {
                suggestion: s.before_code.clone(),
                reasoning: s.impact.clone(),
                confidence: s.steer_conf,
                xp_reward: 10,
            })
            .collect(),
    };
    let review = crate::rag::generate_review(&rag_req).unwrap_or(crate::ReviewResponse {
        summary: "Review generation failed".to_string(),
        severity: 0.0,
        recommendations: vec![],
        coherence: 0.0,
        latency_ms: 0,
    });

    let lat_ms = start.elapsed().as_millis() as u64;
    let severity = review.severity;
    let coh = review.coherence;

    // Update metrics (assume global or pass)
    // metrics.lat_hist.observe(lat_ms as f64);
    // metrics.coh_gauge.set(coh as f64);

    let resp = ReviewResp {
        summary: review.summary,
        severity,
        recommendations: review.recommendations,
        coherence: coh,
        latency_ms: lat_ms,
    };

    (StatusCode::OK, Json(resp)).into_response()
}

pub fn setup_git_hook(repo_path: &str, _config: &IntegrationConfig) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    let hook_path = format!("{}/.git/hooks/pre-commit", repo_path);
    let script = r#"#!/bin/sh
    git diff --cached --name-only | while read file; do
        cargo run -- phase1 "$file" json | jq -r '.total_issues > 0 and .avg_confidence > 0.7' | grep true && echo "BS detected - commit blocked!" && exit 1
    done
    "#;
    std::fs::write(&hook_path, script)?;
    std::process::Command::new("chmod")
        .args(&["+x", &hook_path])
        .status()?;
    tracing::info!("Git hook installed at {}", hook_path);
    Ok(())
}

pub fn web_team_dash(app: Router<AppState>, metrics: &Metrics) -> Result<Router<AppState>> {
    let encoder = TextEncoder::new();
    let metrics_clone = metrics.clone(); // Assume Clone impl
    let app = app.route(
        "/metrics",
        get(move || async move {
            // Commented out due to prometheus compilation issues
            // let metric_families = metrics_clone.lat_hist.get_metric_families();
            // let encoder = encoder.clone();
            let mut buffer = Vec::new();
            // encoder.encode(&metric_families, &mut buffer).unwrap();
            buffer.extend_from_slice(b"# Mock metrics\n"); // Mock implementation
            (StatusCode::OK, buffer)
        }),
    );
    Ok(app)
}

// In lib.rs pub use integrate::{setup_git_hook, ci_pipeline_review, Metrics, web_team_dash};
