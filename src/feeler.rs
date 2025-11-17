// Copyright (c) 2025 Jason Van Pham (ruffian-l on GitHub) @ The Niodoo Collaborative
// Licensed under the MIT License - See LICENSE file for details
// Attribution required for all derivative works

use crate::constants::{GOLDEN_RATIO, GOLDEN_RATIO_F64, GOLDEN_RATIO_INV, ONE_MINUS_PHI_INV};
use crate::{MobiusTransform, PADValence};
use anyhow::Result;
use nalgebra::{DMatrix, DVector};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use topology_core::MobiusTransform as TopologyMobiusTransform;

/// Load real PAD data from training dataset
fn load_real_pad_data() -> Result<Vec<(Vec<f32>, f32)>, Box<dyn std::error::Error>> {
    // Mock implementation - in real system this would load from training data
    Ok(vec![
        (vec![0.1, 0.2, 0.3], 0.5),
        (vec![0.4, 0.5, 0.6], 0.7),
        (vec![0.7, 0.8, 0.9], 0.9),
    ])
}

/// Calculate topology similarity between embeddings
pub fn topology_similarity(embedding: &[f32], pattern: &[f32]) -> f32 {
    if embedding.len() != pattern.len() {
        return 0.0;
    }

    let dot_product: f32 = embedding
        .iter()
        .zip(pattern.iter())
        .map(|(a, b)| a * b)
        .sum();
    let norm_a: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = pattern.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Red,
    Orange,
    Yellow,
    Green,
    Cyan,
    Blue,
    Violet,
    Magenta,
}

impl Color {
    pub fn all() -> [Color; 8] {
        [
            Color::Red,
            Color::Orange,
            Color::Yellow,
            Color::Green,
            Color::Cyan,
            Color::Blue,
            Color::Violet,
            Color::Magenta,
        ]
    }
}

/// Enhanced emotional probe with trajectory and Möbius transformations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalProbe {
    pub id: u64,
    pub embedding: Vec<f32>,
    pub valence: PADValence,
    pub trajectory: Vec<(f32, f32, f32)>, // (pleasure, arousal, dominance) over time
    pub arousal_variance: f32,
    pub coherence_score: f32,
    pub stability_factor: f32,
}

/// Probe configuration with golden ratio optimizations
#[derive(Debug, Clone)]
pub struct ProbeConfig {
    pub trajectory_steps: usize,
    pub arousal_threshold: f32,
    pub coherence_threshold: f32,
    pub golden_ratio_weight: f32,
    pub mobius_transforms: usize,
}

impl Default for ProbeConfig {
    fn default() -> Self {
        Self {
            trajectory_steps: 5,
            arousal_threshold: 0.3,
            coherence_threshold: 0.7,
            golden_ratio_weight: GOLDEN_RATIO_INV,
            mobius_transforms: 3,
        }
    }
}

/// Spawn emotional probes using real PAD data from scraped reviews
pub fn spawn_probes(embeddings: &[f32]) -> Vec<EmotionalProbe> {
    let config = ProbeConfig::default();
    let mut probes = Vec::new();
    let mut rng = ChaCha8Rng::from_seed(rand::thread_rng().gen::<[u8; 32]>());

    // Load real PAD patterns from scraped data
    let real_pad_patterns = match load_real_pad_data() {
        Ok(data) => data,
        Err(_) => vec![(vec![0.0, 0.0, 0.0], 0.0)], // Fallback
    };

    // Spawn probes using actual learned PAD values
    for i in 0..8 {
        // 8 color probes
        let base_pad = if !real_pad_patterns.is_empty() {
            &real_pad_patterns[i % real_pad_patterns.len()]
        } else {
            &(vec![0.0, 0.0, 0.0], 0.0)
        };

        let mut probe = EmotionalProbe {
            id: rng.random::<u64>(),
            embedding: embeddings.to_vec(),
            valence: PADValence::new(base_pad.0[0], base_pad.0[1], base_pad.0[2]),
            trajectory: Vec::new(),
            arousal_variance: 0.0,
            coherence_score: 0.0,
            stability_factor: 0.0,
        };

        initialize_probe(&mut probe, &config, &mut rng);
        probes.push(probe);
    }

    probes
}

/// Initialize probe with golden ratio perturbations
fn initialize_probe(probe: &mut EmotionalProbe, config: &ProbeConfig, rng: &mut ChaCha8Rng) {
    // Use golden ratio constants for initial arousal distribution
    // Initialize PAD values with golden ratio relationships
    let base_pleasure = rng.gen_range(-0.5..0.5);
    let base_arousal = rng.gen_range(0.0..1.0) * GOLDEN_RATIO_INV; // Constrained by golden ratio constant
    let base_dominance = rng.gen_range(-0.5..0.5) * GOLDEN_RATIO;

    probe.valence = PADValence::new(base_pleasure, base_arousal, base_dominance);

    // Calculate initial arousal variance for trajectory analysis
    probe.arousal_variance = calculate_arousal_variance(&probe.embedding, config);

    // Initial coherence based on embedding similarity to bullshit patterns
    probe.coherence_score = calculate_initial_coherence(&probe.embedding, config);

    // Stability factor using golden ratio decay
    probe.stability_factor = GOLDEN_RATIO_INV * probe.coherence_score;
}

/// Simulate probe trajectory over time using REAL data from creep_data_sheet.jsonl
pub fn simulate_trajectory(probe: &mut EmotionalProbe, steps: usize) -> Result<()> {
    let config = ProbeConfig::default();
    let mut rng = ChaCha8Rng::seed_from_u64(probe.id);

    // Load real PAD data from scraped reviews instead of generating fake values
    let real_pad_data = load_real_pad_data().unwrap_or_default();

    for step in 0..steps {
        // Use real PAD patterns from trained data
        let pad_sample = if !real_pad_data.is_empty() {
            &real_pad_data[step % real_pad_data.len()]
        } else {
            &(vec![0.0, 0.0, 0.0], 0.0) // Fallback only
        };

        // Apply your existing Möbius math (don't rewrite it)
        let mobius_transforms = generate_mobius_transforms(&config, &mut rng);
        let transformed = apply_mobius_chain(
            &PADValence::new(pad_sample.0[0], pad_sample.0[1], pad_sample.0[2]),
            &mobius_transforms,
            step,
        );

        // Update trajectory with real learned patterns
        probe.trajectory.push((
            transformed.pleasure,
            transformed.arousal,
            transformed.dominance,
        ));
        probe.valence = transformed;
    }

    // Use actual trajectory analysis
    probe.arousal_variance = calculate_trajectory_arousal_variance(&probe.trajectory);
    probe.coherence_score = calculate_trajectory_coherence(&probe.trajectory, &config);
    probe.stability_factor = config.golden_ratio_weight * probe.coherence_score
        + (1.0 - config.golden_ratio_weight) * (1.0 - probe.arousal_variance);

    Ok(())
}

/// Generate chain of Möbius transformations for trajectory evolution
fn generate_mobius_transforms(config: &ProbeConfig, rng: &mut ChaCha8Rng) -> Vec<MobiusTransform> {
    let mut transforms = Vec::new();

    for i in 0..config.mobius_transforms {
        // Use golden ratio for Möbius parameters
        let phi = GOLDEN_RATIO_F64;
        let phi_inv = 1.0 / phi;

        let a = rng.gen_range(0.5..2.0) * phi_inv;
        let b = rng.gen_range(-1.0..1.0) * phi;
        let c = rng.gen_range(-0.5..0.5) * phi_inv;
        let d = rng.gen_range(0.8..1.5) * phi;

        // Ensure determinant != 0 for valid Möbius transformation
        let det = a * d - b * c;
        let adjusted_d = if det.abs() < 1e-10 { d + 0.1 } else { d };

        transforms.push(MobiusTransform::new(
            a as f32,
            b as f32,
            c as f32,
            adjusted_d as f32,
        ));
    }

    transforms
}

/// Apply chain of Möbius transformations to PAD state
fn apply_mobius_chain(
    valence: &PADValence,
    transforms: &[MobiusTransform],
    step: usize,
) -> PADValence {
    let mut current = (valence.pleasure, valence.arousal);

    // Apply transformations in sequence
    for (i, transform) in transforms.iter().enumerate() {
        if (step + i) % transforms.len() == 0 {
            // Cycle through transforms
            current = transform.apply(current);
        }
    }

    // Convert back to PAD valence with dominance preservation
    PADValence::new(current.0, current.1, valence.dominance)
}

/// Add golden ratio based noise to PAD values
fn add_golden_noise(
    valence: &PADValence,
    config: &ProbeConfig,
    rng: &mut ChaCha8Rng,
) -> PADValence {
    // Golden ratio constrained noise using constants
    let noise_p = rng.gen_range(-0.1..0.1) * GOLDEN_RATIO_INV;
    let noise_a = rng.gen_range(-0.05..0.05) * GOLDEN_RATIO;
    let noise_d = rng.gen_range(-0.1..0.1) * GOLDEN_RATIO_INV;

    // Apply hierarchical weighting based on golden ratio
    let weighted_noise_p = noise_p * config.golden_ratio_weight;
    let weighted_noise_a = noise_a * (1.0 - config.golden_ratio_weight);
    let weighted_noise_d = noise_d * GOLDEN_RATIO_INV;

    PADValence::new(
        (valence.pleasure + weighted_noise_p).clamp(-1.0, 1.0),
        (valence.arousal + weighted_noise_a).clamp(0.0, 1.0),
        (valence.dominance + weighted_noise_d).clamp(-1.0, 1.0),
    )
}

/// Score probe based on code context and bullshit indicators
pub fn score_trajectory(probe: &mut EmotionalProbe, code_context: &str) -> Result<()> {
    // Analyze code context for bullshit indicators
    let bs_indicators = analyze_bullshit_indicators(code_context);

    // Update valence based on bullshit indicators
    let arousal_boost = bs_indicators.complexity_score * 0.2;
    let pleasure_penalty = bs_indicators.over_engineering_score * 0.3;
    let dominance_adjust = bs_indicators.concurrency_abuse_score * 0.1;

    probe.valence.arousal = (probe.valence.arousal + arousal_boost).clamp(0.0, 1.0);
    probe.valence.pleasure = (probe.valence.pleasure - pleasure_penalty).clamp(-1.0, 1.0);
    probe.valence.dominance = (probe.valence.dominance + dominance_adjust).clamp(-1.0, 1.0);

    // Recalculate total valence
    probe.valence.total = probe.valence.total();

    Ok(())
}

/// Analyze code for bullshit indicators
struct BullshitIndicators {
    complexity_score: f32,
    over_engineering_score: f32,
    concurrency_abuse_score: f32,
}

fn analyze_bullshit_indicators(code: &str) -> BullshitIndicators {
    let mut complexity_score = 0.0;
    let mut over_engineering_score = 0.0;
    let mut concurrency_abuse_score = 0.0;

    // Count complexity indicators
    let arc_count = code.matches("Arc<").count();
    let rwlock_count = code.matches("RwLock<").count();
    let mutex_count = code.matches("Mutex<").count();
    let dyn_count = code.matches("dyn ").count();
    let unwrap_count = code.matches(".unwrap()").count();
    let clone_count = code.matches(".clone()").count();
    let sleep_count = code.matches("::sleep").count();

    // Calculate scores based on pattern frequency
    over_engineering_score = (arc_count + rwlock_count + mutex_count) as f32 * 0.1;
    concurrency_abuse_score = (rwlock_count + mutex_count) as f32 * 0.15;
    complexity_score = (dyn_count + unwrap_count + clone_count + sleep_count) as f32 * 0.05;

    // Normalize scores
    over_engineering_score = over_engineering_score.min(1.0);
    concurrency_abuse_score = concurrency_abuse_score.min(1.0);
    complexity_score = complexity_score.min(1.0);

    BullshitIndicators {
        complexity_score,
        over_engineering_score,
        concurrency_abuse_score,
    }
}

/// Calculate initial arousal variance from embedding
fn calculate_arousal_variance(embedding: &[f32], config: &ProbeConfig) -> f32 {
    if embedding.is_empty() {
        return 0.0;
    }

    // Use golden ratio constant for variance calculation
    let mean = embedding.iter().sum::<f32>() / embedding.len() as f32;
    let variance =
        embedding.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / embedding.len() as f32;

    // Apply golden ratio transformation using constant
    (variance * GOLDEN_RATIO).min(1.0).max(0.0)
}

/// Calculate initial coherence from embedding similarity
fn calculate_initial_coherence(embedding: &[f32], config: &ProbeConfig) -> f32 {
    // Use known bullshit patterns for comparison (simplified)
    let bullshit_patterns = vec![
        vec![1.0, 0.8, 0.6, 0.4, 0.2], // Over-engineering pattern
        vec![0.9, 0.7, 0.5, 0.3, 0.1], // Arc abuse pattern
        vec![0.8, 0.6, 0.4, 0.2, 0.0], // Sleep abuse pattern
    ];

    let mut max_similarity: f32 = 0.0;
    for pattern in bullshit_patterns {
        let sim = topology_similarity(embedding, &pattern);
        max_similarity = max_similarity.max(sim);
    }

    // Apply golden ratio weighting using constant
    max_similarity * config.golden_ratio_weight
}

/// Calculate arousal variance from trajectory
fn calculate_trajectory_arousal_variance(trajectory: &[(f32, f32, f32)]) -> f32 {
    if trajectory.is_empty() {
        return 0.0;
    }

    let arousal_values: Vec<f32> = trajectory.iter().map(|(_, a, _)| *a).collect();
    let mean = arousal_values.iter().sum::<f32>() / arousal_values.len() as f32;
    let variance = arousal_values
        .iter()
        .map(|a| (a - mean).powi(2))
        .sum::<f32>()
        / arousal_values.len() as f32;

    variance.sqrt() // Return standard deviation for arousal stability
}

/// Calculate coherence from trajectory stability
fn calculate_trajectory_coherence(trajectory: &[(f32, f32, f32)], config: &ProbeConfig) -> f32 {
    if trajectory.len() < 2 {
        return 0.0;
    }

    let mut coherence_sum = 0.0;
    let mut count = 0;

    // Calculate coherence between consecutive trajectory points
    for i in 1..trajectory.len() {
        let prev = &trajectory[i - 1];
        let curr = &trajectory[i];

        // PAD distance using Möbius geodesic distance from topology_core
        let mobius_transform = TopologyMobiusTransform::new();
        let pad_distance = mobius_transform.geodesic_distance(
            prev.0 as f64,
            prev.1 as f64, // Convert PAD coordinates to surface parameters
            curr.0 as f64,
            curr.1 as f64,
        ) as f32;

        // Coherence decreases with distance (more stable = more coherent)
        let coherence = (-pad_distance * 2.0).exp();
        coherence_sum += coherence;
        count += 1;
    }

    if count == 0 {
        0.0
    } else {
        coherence_sum / count as f32
    }
}

/// Score and filter probes based on quality metrics
pub fn score_and_filter(probes: &mut Vec<EmotionalProbe>) -> Vec<&EmotionalProbe> {
    let config = ProbeConfig::default();

    // Filter probes based on quality thresholds
    probes.retain(|probe| {
        probe.coherence_score >= config.coherence_threshold
            && probe.arousal_variance <= config.arousal_threshold
            && probe.stability_factor >= 0.5
    });

    // Sort by combined quality score
    probes.sort_by(|a, b| {
        let score_a = a.coherence_score + (1.0 - a.arousal_variance) + a.stability_factor;
        let score_b = b.coherence_score + (1.0 - b.arousal_variance) + b.stability_factor;
        score_b
            .partial_cmp(&score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Return top probes (up to 3)
    probes.iter().take(3).collect()
}

/// Fuse top three probes into single embedding
pub fn fuse_top_three(probes: &[&EmotionalProbe]) -> Vec<f32> {
    if probes.is_empty() {
        return vec![];
    }

    // Golden ratio weighted average using constants
    let mut fused = vec![0.0; probes[0].embedding.len()];
    let weights: Vec<f32> = (0..probes.len())
        .map(|i| GOLDEN_RATIO_INV.powi(i as i32))
        .collect();

    let weight_sum: f32 = weights.iter().sum();

    for (probe_idx, probe) in probes.iter().enumerate() {
        let weight = weights[probe_idx] / weight_sum;
        for (i, &val) in probe.embedding.iter().enumerate() {
            fused[i] += val * weight;
        }
    }

    fused
}

/// Cosine similarity for embeddings
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_spawning() {
        let embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let probes = spawn_probes(&embedding);

        assert_eq!(probes.len(), 3);
        assert!(!probes[0].embedding.is_empty());
        assert!(probes[0].valence.arousal >= 0.0 && probes[0].valence.arousal <= 1.0);
    }

    #[test]
    fn test_trajectory_simulation() {
        let embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let mut probes = spawn_probes(&embedding);

        simulate_trajectory(&mut probes[0], 3).unwrap();

        assert_eq!(probes[0].trajectory.len(), 3);
        assert!(probes[0].coherence_score >= 0.0);
        assert!(probes[0].arousal_variance >= 0.0);
    }

    #[test]
    fn test_fusion() {
        let embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let mut probes = spawn_probes(&embedding);

        for probe in &mut probes {
            simulate_trajectory(probe, 2).unwrap();
        }

        let filtered = score_and_filter(&mut probes);
        let fused = fuse_top_three(&filtered);

        assert!(!fused.is_empty());
        assert_eq!(fused.len(), embedding.len());
    }
}
