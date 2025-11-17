// Copyright (c) 2025 Jason Van Pham (ruffian-l on GitHub) @ The Niodoo Collaborative
// Licensed under the MIT License - See LICENSE file for details
// Attribution required for all derivative works

//! Qwen LoRA Client - Bridge to Python inference server
//!
//! Rust client that calls the Python Qwen LoRA bridge over HTTP

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const QWEN_BRIDGE_URL: &str = "http://127.0.0.1:8765";

#[derive(Debug, Clone, Serialize)]
pub struct ReviewRequest {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReviewResponse {
    pub review: String,
    pub tokens_generated: usize,
    pub latency_ms: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub model_loaded: bool,
    pub cuda_available: bool,
    pub device: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatsResponse {
    pub total_params: usize,
    pub trainable_params: usize,
    pub trainable_percent: f64,
    pub lora_active: bool,
    pub gpu_memory_allocated_mb: f64,
}

pub struct QwenClient {
    client: Client,
    base_url: String,
}

impl QwenClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        Ok(Self {
            client,
            base_url: QWEN_BRIDGE_URL.to_string(),
        })
    }

    pub fn with_url(url: &str) -> Result<Self> {
        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        Ok(Self {
            client,
            base_url: url.to_string(),
        })
    }

    pub async fn health_check(&self) -> Result<HealthResponse> {
        let url = format!("{}/health", self.base_url);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("Health check failed: {}", response.status()));
        }

        let health: HealthResponse = response.json().await?;
        Ok(health)
    }

    pub async fn get_stats(&self) -> Result<StatsResponse> {
        let url = format!("{}/stats", self.base_url);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("Stats request failed: {}", response.status()));
        }

        let stats: StatsResponse = response.json().await?;
        Ok(stats)
    }

    pub async fn generate_review(&self, request: ReviewRequest) -> Result<ReviewResponse> {
        let url = format!("{}/review", self.base_url);
        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Review generation failed: {}", error_text));
        }

        let review: ReviewResponse = response.json().await?;
        Ok(review)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_qwen_client() -> Result<()> {
        let client = QwenClient::new()?;

        // Test health
        let health = client.health_check().await?;
        assert_eq!(health.status, "healthy");
        assert!(health.model_loaded);

        // Test review generation
        let request = ReviewRequest {
            code: "let x = Arc::new(RwLock::new(vec![]));\nx.write().unwrap().push(1);".to_string(),
            max_tokens: Some(256),
            temperature: Some(0.7),
            top_p: Some(0.9),
        };

        let response = client.generate_review(request).await?;
        assert!(!response.review.is_empty());
        assert!(response.tokens_generated > 0);

        Ok(())
    }
}
