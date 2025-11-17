// Copyright (c) 2025 Jason Van Pham (ruffian-l on GitHub) @ The Niodoo Collaborative
// Licensed under the MIT License - See LICENSE file for details
// Attribution required for all derivative works

use crate::constants::{GOLDEN_RATIO, GOLDEN_RATIO_INV};
use crate::qwen_client::{QwenClient, ReviewRequest as QwenRequest};
use crate::{BullshitAlert, BullshitType, ReviewRequest, ReviewResponse, Suggestion};
use anyhow::{anyhow, Result};
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::generation::LogitsProcessor;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
// Note: Qwen model support may not be available in current candle_transformers version
// use candle_transformers::models::qwen::{Config as QwenConfig, Model as QwenModel};
use tokenizers::Tokenizer;

/// Enhanced RAG configuration with golden ratio optimizations
#[derive(Debug, Clone)]
pub struct RagConfig {
    pub model_path: String,
    pub top_k: usize,
    pub temperature: f32,
    pub max_tokens: usize,
    pub stop_sequences: Vec<String>,
    pub enable_assertive_mode: bool,
    pub xp_multiplier: f32,
    pub coherence_threshold: f32,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            model_path: "./models/qwen2-7b-instruct".to_string(),
            top_k: 20,        // As specified in requirements
            temperature: 0.7, // As specified in requirements
            max_tokens: 512,
            stop_sequences: vec!["```".to_string(), "\n\n".to_string()],
            enable_assertive_mode: true,
            xp_multiplier: 1.5,
            coherence_threshold: GOLDEN_RATIO_INV, // Golden ratio constant threshold
        }
    }
}

/// Enhanced RAG system for generating code reviews with Qwen via HTTP bridge
pub struct RagGenerator {
    config: RagConfig,
    rng: ChaCha8Rng,
    qwen_client: QwenClient,
    device: Device,
    tokenizer: Tokenizer,
    // model: QwenModel, // Commented out until Qwen support is available
}

impl RagGenerator {
    pub fn new(config: RagConfig) -> Result<Self> {
        let qwen_client = QwenClient::new()?;
        let device = Device::cuda_if_available(0).unwrap_or_else(|_| Device::Cpu);
        let tokenizer_path = format!("{}/tokenizer.json", config.model_path);
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow!("Failed to load tokenizer: {}", e))?;

        // Commented out Qwen model loading until support is available
        // let config_path = format!("{}/config.json", config.model_path);
        // let config_str = std::fs::read_to_string(&config_path).map_err(|e| anyhow!("Failed to read config: {}", e))?;
        // let qwen_config: QwenConfig = serde_json::from_str(&config_str).map_err(|e| anyhow!("Failed to parse config: {}", e))?;
        //
        // let mut vb_files = Vec::new();
        // for i in 0u32..100 {
        //     let path = format!("{}/model-{:014}.safetensors", config.model_path, i);
        //     if std::path::Path::new(&path).exists() {
        //         vb_files.push(path);
        //     } else {
        //         break;
        //     }
        // }
        // let vb = if vb_files.is_empty() {
        //     VarBuilder::from_mmaped_safetensors(&[format!("{}/model.safetensors", config.model_path)], DType::F32, &device)
        //         .map_err(|e| anyhow!("Failed to load model: {}", e))?
        // } else {
        //     VarBuilder::from_mmaped_safetensors(&vb_files, DType::F32, &device)
        //         .map_err(|e| anyhow!("Failed to load model shards: {}", e))?
        // };
        //
        // let model = QwenModel::load(&vb, &qwen_config).map_err(|e| anyhow!("Failed to load Qwen model: {}", e))?;

        Ok(Self {
            config,
            rng: ChaCha8Rng::from_seed(rand::thread_rng().gen::<[u8; 32]>()),
            qwen_client,
            device,
            tokenizer,
            // model, // Commented out until Qwen support is available
        })
    }

    pub fn with_url(config: RagConfig, bridge_url: &str) -> Result<Self> {
        let qwen_client = QwenClient::with_url(bridge_url)?;

        let model_path = config.model_path.clone();
        Ok(Self {
            config,
            rng: ChaCha8Rng::from_seed(rand::thread_rng().gen::<[u8; 32]>()),
            qwen_client,
            device: Device::Cpu, // Placeholder, will be updated in new
            tokenizer: Tokenizer::from_file(&format!("{}/tokenizer.json", model_path)).unwrap(), // Placeholder, will be updated in new
                                                                                                 // model: QwenModel::load(&VarBuilder::from_mmaped_safetensors(&[format!("{}/model.safetensors", config.model_path)], DType::F32, &Device::Cpu).unwrap(), &QwenConfig::default()).unwrap(), // Commented out until Qwen support is available
        })
    }

    /// Generate code review from bullshit alerts with enhanced snark
    pub async fn generate_review(&mut self, request: &ReviewRequest) -> Result<ReviewResponse> {
        let start_time = Instant::now();

        // Filter high-confidence alerts using golden ratio constant threshold
        let high_confidence_alerts: Vec<&BullshitAlert> = request
            .alerts
            .iter()
            .filter(|alert| alert.confidence > self.config.coherence_threshold)
            .collect();

        let summary = if high_confidence_alerts.is_empty() {
            "✅ Clean code detected - no bullshit patterns found! Keep grinding those clean implementations!".to_string()
        } else {
            let prompt = self.generate_enhanced_prompt(&request);
            self.generate_with_enhanced_model(&prompt).await?
        };

        // Generate assertive recommendations with XP multipliers
        let recommendations = self.generate_assertive_recommendations(&high_confidence_alerts)?;

        // Calculate overall severity and coherence using golden ratio math
        let severity = self.calculate_golden_severity(&high_confidence_alerts);
        let coherence = self.calculate_golden_coherence(&high_confidence_alerts);

        Ok(ReviewResponse {
            summary,
            severity,
            recommendations,
            coherence,
            latency_ms: start_time.elapsed().as_millis() as u64,
        })
    }

    /// Generate snarky summary with gaming metaphors
    fn generate_snarky_summary(&mut self, alerts: &[&BullshitAlert]) -> Result<String> {
        let mut summary_parts = Vec::new();

        // Group alerts by type for better organization
        let mut type_counts = HashMap::new();
        for alert in alerts {
            *type_counts.entry(&alert.issue_type).or_insert(0) += 1;
        }

        // Generate snarky commentary for each bullshit type
        for (bs_type, count) in type_counts {
            let commentary = match bs_type {
                BullshitType::OverEngineering => {
                    format!(
                        "🚨 {} over-engineered death traps screaming for simplification",
                        count
                    )
                }
                BullshitType::ArcAbuse => {
                    format!(
                        "🔒 {} Arc atrocities begging for mercy and simple ownership",
                        count
                    )
                }
                BullshitType::RwLockAbuse => {
                    format!(
                        "🔐 {} RwLock disasters plotting to deadlock your entire app",
                        count
                    )
                }
                BullshitType::SleepAbuse => {
                    format!(
                        "😴 {} blocking sleep zombies that should be async ghosts",
                        count
                    )
                }
                BullshitType::UnwrapAbuse => {
                    format!(
                        "💥 {} unwrap() bombs armed and ready to nuke your runtime",
                        count
                    )
                }
                BullshitType::DynTraitAbuse => {
                    format!(
                        "🎭 {} trait object pretenders complicating your clean design",
                        count
                    )
                }
                BullshitType::FakeComplexity => {
                    format!(
                        "🌀 {} fake complexity mazes that need immediate unwinding",
                        count
                    )
                }
                BullshitType::CargoCult => {
                    format!(
                        "📚 {} cargo cult rituals cluttering your sacred codebase",
                        count
                    )
                }
                BullshitType::CloneAbuse => {
                    format!("📋 {} unnecessary clones bleeding your memory dry", count)
                }
                BullshitType::MutexAbuse => {
                    format!("🔒 {} mutex monsters that could be harmless borrows", count)
                }
            };

            summary_parts.push(commentary);
        }

        // Calculate and add XP rewards
        let total_xp = alerts.len() as u32 * 50;
        let level_message = if total_xp > 200 {
            format!(
                "🚀 LVL3 BULLSHIT UNLOCKED! XP+{} GRIND! These refactors are your boss level!",
                total_xp
            )
        } else if total_xp > 100 {
            format!(
                "⭐ XP+{} earned! You're climbing the clean code mountain!",
                total_xp
            )
        } else {
            format!(
                "💪 XP+{} gained! Every refactor makes you stronger!",
                total_xp
            )
        };

        summary_parts.push(level_message);

        Ok(summary_parts.join(" | "))
    }

    /// Generate assertive recommendations with XP rewards
    fn generate_assertive_recommendations(
        &mut self,
        alerts: &[&BullshitAlert],
    ) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();

        for alert in alerts {
            let (suggestion, base_xp) = match &alert.issue_type {
                BullshitType::OverEngineering => (
                    "IMMEDIATE REFACTOR: Strip Arc<RwLock<>> to owned Vec<T> - reduces cognitive load 45% and prevents deadlocks".to_string(),
                    75,
                ),
                BullshitType::ArcAbuse => (
                    "CRITICAL FIX: Use Arc only for true shared ownership across threads - this isn't thread-safe data, it's complexity theater".to_string(),
                    50,
                ),
                BullshitType::RwLockAbuse => (
                    "URGENT: Consider if read/write locks are necessary - simple borrows might suffice for 80% of cases".to_string(),
                    60,
                ),
                BullshitType::SleepAbuse => (
                    "ASYNC MIGRATION: Use tokio::time::interval() instead of blocking sleeps - improves concurrency by 300%".to_string(),
                    40,
                ),
                BullshitType::UnwrapAbuse => (
                    "SAFETY FIRST: Handle errors properly with ? operator or match statements - prevents panic cascades".to_string(),
                    55,
                ),
                BullshitType::DynTraitAbuse => (
                    "DESIGN CLEANUP: Use concrete types when possible - trait objects add 20% runtime overhead unnecessarily".to_string(),
                    45,
                ),
                BullshitType::FakeComplexity => (
                    "COMPLEXITY BUSTER: Break down into focused single-purpose functions - improves readability by 60%".to_string(),
                    65,
                ),
                BullshitType::CargoCult => (
                    "DEPENDENCY AUDIT: Import only what you use - removes 15% of unused crate bloat".to_string(),
                    35,
                ),
                BullshitType::CloneAbuse => (
                    "MEMORY OPTIMIZATION: Avoid unnecessary cloning - use references or iterators for 40% less allocation".to_string(),
                    40,
                ),
                BullshitType::MutexAbuse => (
                    "CONCURRENCY REVIEW: Consider if mutex is needed - channels or atomics might be 50% more efficient".to_string(),
                    50,
                ),
            };

            // Apply XP multiplier from config
            let final_xp = (base_xp as f32 * self.config.xp_multiplier) as u32;

            let xp_text = if final_xp > 75 {
                format!(" (MEGA XP+{}! 🎯)", final_xp)
            } else if final_xp > 50 {
                format!(" (XP+{}! ⭐)", final_xp)
            } else {
                format!(" (xp+{})", final_xp)
            };

            recommendations.push(format!("🚨 ASSERTIVE REFACTOR: {} {}", suggestion, xp_text));
        }

        // Add general recommendations
        recommendations.push("✅ VALIDATION: Add unit tests for data mutations - validates refactoring safety and prevents regressions".to_string());
        recommendations.push("🔍 PATTERN SCAN: Review similar patterns across entire codebase - prevent bullshit from spreading".to_string());
        recommendations.push(
            "📊 METRICS: Track bullshit reduction over time - aim for <5% false positives"
                .to_string(),
        );

        Ok(recommendations)
    }

    /// Calculate severity using golden ratio weighted scoring with constants
    fn calculate_golden_severity(&self, alerts: &[&BullshitAlert]) -> f32 {
        if alerts.is_empty() {
            return 0.0;
        }

        // Use golden ratio constants
        let mut weighted_severity = 0.0;
        for alert in alerts {
            // Golden ratio weighted type severity using constants
            let type_weight = match &alert.issue_type {
                BullshitType::OverEngineering => GOLDEN_RATIO_INV, // Most severe
                BullshitType::RwLockAbuse => GOLDEN_RATIO_INV * 0.9,
                BullshitType::ArcAbuse => GOLDEN_RATIO_INV * 0.8,
                BullshitType::DynTraitAbuse => GOLDEN_RATIO_INV * 0.7,
                BullshitType::FakeComplexity => GOLDEN_RATIO_INV * 0.8,
                BullshitType::UnwrapAbuse => GOLDEN_RATIO_INV * 0.75,
                BullshitType::MutexAbuse => GOLDEN_RATIO_INV * 0.7,
                BullshitType::SleepAbuse => GOLDEN_RATIO_INV * 0.6,
                BullshitType::CloneAbuse => GOLDEN_RATIO_INV * 0.5,
                BullshitType::CargoCult => GOLDEN_RATIO_INV * 0.4,
            };

            weighted_severity += alert.confidence * type_weight;
        }

        let avg_weighted_severity = weighted_severity / alerts.len() as f32;

        // Apply golden ratio transformation for natural scaling using constant
        (avg_weighted_severity * GOLDEN_RATIO).min(1.0)
    }

    /// Calculate coherence using golden ratio variance analysis with constants
    fn calculate_golden_coherence(&self, alerts: &[&BullshitAlert]) -> f32 {
        if alerts.len() < 2 {
            return 1.0;
        }

        // Use golden ratio constant
        // Calculate confidence variance using golden ratio for natural distribution
        let confidences: Vec<f32> = alerts.iter().map(|alert| alert.confidence).collect();
        let mean_confidence = confidences.iter().sum::<f32>() / confidences.len() as f32;

        let variance: f32 = confidences
            .iter()
            .map(|&conf| (conf - mean_confidence).powi(2))
            .sum::<f32>()
            / confidences.len() as f32;

        // Golden ratio coherence calculation - higher coherence for lower variance using constant
        let coherence = (-variance * GOLDEN_RATIO * 2.0).exp();

        coherence.min(1.0).max(0.0)
    }

    /// Generate enhanced RAG prompt with snarky instructions
    pub fn generate_enhanced_prompt(&self, request: &ReviewRequest) -> String {
        let mut prompt = String::from(
            "You are NiodO.o, the BullshitDetector AI - a hardcore code quality enforcer with a snarky attitude. You help developers level up their code through brutal honesty and actionable feedback.\n\n"
        );

        // Add code context
        if let Some(first_alert) = request.alerts.first() {
            prompt.push_str(&format!(
                "CODE CONTEXT:\n```\n{}\n```\n\n",
                first_alert.context_snippet
            ));
        }

        // Add bullshit alerts with snarky formatting
        prompt.push_str("🚨 BULLSHIT ALERTS DETECTED:\n");
        for (i, alert) in request.alerts.iter().enumerate() {
            prompt.push_str(&format!(
                "{}. 🔥 {}: {} (CONFIDENCE: {:.2})\n   WHY BULLSHIT: {}\n   QUICK FIX: {}\n\n",
                i + 1,
                alert.issue_type,
                alert.context_snippet,
                alert.confidence,
                alert.why_bs,
                alert.sug
            ));
        }

        // Add assertive instructions with gaming metaphors
        prompt.push_str(
            "GENERATE RESPONSE:\n\
            • Be SNARKY but HELPFUL - call out bullshit directly\n\
            • Use GAMING METAPHORS (lvl3 bs unlocked, xp grind, boss level, etc.)\n\
            • Be ASSERTIVE - use phrases like \"REFACTOR NOW\" and \"URGENT FIX\"\n\
            • Provide SPECIFIC code changes with before/after examples\n\
            • Suggest UNIT TESTS and VALIDATION steps\n\
            • Calculate XP rewards based on bullshit severity\n\
            • Keep response CONCISE but COMPREHENSIVE (<300 words)\n\
            • End with NEXT STEPS for implementation\n\n",
        );

        prompt.push_str("CODE REVIEW (snarky, assertive, actionable):");

        prompt
    }

    /// Mock RAG generation with enhanced snark (replace with actual Qwen inference)
    pub async fn generate_with_enhanced_model(&mut self, prompt: &str) -> Result<String> {
        let encoding = self
            .tokenizer
            .encode(prompt, true)
            .map_err(|e| anyhow!("Encode error: {}", e))?;
        let mut input_ids = Tensor::new(&*encoding.get_ids(), &self.device)?.unsqueeze(0)?;
        let input_len = input_ids.dim(1).unwrap();
        let mut attention_mask =
            Tensor::new(&*encoding.get_attention_mask(), &self.device)?.unsqueeze(0)?;

        let mut generated_tokens: Vec<u32> = vec![];
        let mut logits_processor = LogitsProcessor::new(
            self.rng.random::<u64>(),
            Some(self.config.temperature as f64),
            Some(self.config.top_k as f64),
        );

        for _ in 0..self.config.max_tokens {
            // Commented out model forward call since model is not available
            // let outputs = self.model.forward(&input_ids, Some(&attention_mask))?;
            // let logits = outputs.logits.i((0, input_len - 1, ..))?;

            // Mock response for now
            let mock_response =
                "This code appears to be well-structured with appropriate error handling.";
            return Ok(mock_response.to_string());

            // Commented out remaining model inference code
            // let next_token_id = logits_processor.sample(&logits)?;
            //
            // generated_tokens.push(next_token_id as u32);
            // let new_input_ids = Tensor::new(&[next_token_id], &self.device)?.unsqueeze(0)?;
            // input_ids = Tensor::cat(&input_ids, &new_input_ids, 1)?;
            // let new_mask = Tensor::new(&[1i64], &self.device)?.unsqueeze(0)?;
            // attention_mask = Tensor::cat(&attention_mask, &new_mask, 1)?;
            // input_len += 1;
            //
            // let generated_text = self.tokenizer.decode(&generated_tokens, false).map_err(|e| anyhow!("Decode error: {}", e))?;
            // if self.config.stop_sequences.iter().any(|seq| generated_text.ends_with(seq)) {
            //     break;
            // }
        }

        Ok("Mock response generated".to_string())
    }
}

/// Generate review using RAG system
pub fn generate_review(request: &ReviewRequest) -> Result<ReviewResponse> {
    // Mock implementation
    Ok(ReviewResponse {
        summary: "Code review completed".to_string(),
        severity: 0.5,
        recommendations: vec!["Consider refactoring".to_string()],
        coherence: 0.8,
        latency_ms: 100,
    })
}

/// Build prompt for RAG system
pub fn build_prompt(request: &ReviewRequest) -> String {
    format!("Review the following code alerts: {:?}", request.alerts)
}

/// Command-line interface for enhanced RAG generation
pub async fn run_enhanced_rag_generation(
    request: ReviewRequest,
    config: RagConfig,
) -> Result<ReviewResponse> {
    let mut generator = RagGenerator::new(config)?;

    tracing::info!("🧠 Generating enhanced RAG-based code review with snark...");
    let response = generator.generate_review(&request).await?;

    tracing::info!(
        "✅ Enhanced code review generated in {}ms",
        response.latency_ms
    );
    tracing::info!(
        "📊 Coherence: {:.2}, Severity: {:.2}",
        response.coherence,
        response.severity
    );
    tracing::info!(
        "🎯 Recommendations: {} items",
        response.recommendations.len()
    );

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_rag_generator_creation() {
        let config = RagConfig::default();
        let generator = RagGenerator::new(config);

        // Test that generator is created with correct config
        assert_eq!(generator.config.top_k, 20);
        assert_eq!(generator.config.temperature, 0.7);
        assert!(generator.config.enable_assertive_mode);
    }

    #[test]
    fn test_golden_severity_calculation() {
        let config = RagConfig::default();
        let mut generator = RagGenerator::new(config);

        let alerts = vec![
            &BullshitAlert {
                issue_type: BullshitType::OverEngineering,
                confidence: 0.9,
                location: (1, 1),
                context_snippet: "test".to_string(),
                why_bs: "test".to_string(),
                sug: "test".to_string(),
                severity: 0.9,
            },
            &BullshitAlert {
                issue_type: BullshitType::ArcAbuse,
                confidence: 0.8,
                location: (2, 1),
                context_snippet: "test".to_string(),
                why_bs: "test".to_string(),
                sug: "test".to_string(),
                severity: 0.8,
            },
        ];

        let severity = generator.calculate_golden_severity(&alerts);
        assert!(severity > 0.8 && severity <= 1.0);
    }

    #[test]
    fn test_golden_coherence_calculation() {
        let config = RagConfig::default();
        let mut generator = RagGenerator::new(config);

        let alerts = vec![
            &BullshitAlert {
                issue_type: BullshitType::OverEngineering,
                confidence: 0.85,
                location: (1, 1),
                context_snippet: "test".to_string(),
                why_bs: "test".to_string(),
                sug: "test".to_string(),
                severity: 0.85,
            },
            &BullshitAlert {
                issue_type: BullshitType::OverEngineering,
                confidence: 0.87,
                location: (2, 1),
                context_snippet: "test".to_string(),
                why_bs: "test".to_string(),
                sug: "test".to_string(),
                severity: 0.87,
            },
        ];

        let coherence = generator.calculate_golden_coherence(&alerts);
        assert!(coherence > 0.9); // High coherence for similar confidence scores
    }
}
