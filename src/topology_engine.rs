// Copyright (c) 2025 Jason Van Pham (ruffian-l on GitHub) @ The Niodoo Collaborative
// Licensed under the MIT License - See LICENSE file for details
// Attribution required for all derivative works

//! Topology Engine - Revolutionary Curved Space Similarity
//!
//! Replaces flat cosine similarity with geodesic distance on Möbius surfaces.
//! This is the mathematical foundation for proving curved space > flat space.

use crate::constants::GOLDEN_RATIO_INV;
use anyhow::Result;
use nalgebra::{DMatrix, Vector3};
use serde::{Deserialize, Serialize};
use std::f64::consts::{PI, TAU};
use tracing::{debug, info, warn};

/// Point on Möbius surface with parametric coordinates
#[derive(Debug, Clone, Copy)]
pub struct SurfacePoint {
    pub u: f64, // Angular parameter [0, 2π]
    pub v: f64, // Width parameter [-1, 1]
    pub x: f64, // 3D coordinates
    pub y: f64,
    pub z: f64,
}

/// Revolutionary topology-powered similarity engine
#[derive(Debug, Clone)]
pub struct TopologyEngine {
    /// Möbius surface parameters
    pub major_radius: f64,
    pub minor_radius: f64,

    /// Gaussian embedding parameters
    pub embedding_dim: usize,
    pub surface_scaling: f64,

    /// Performance optimization
    pub cache_size: usize,
    similarity_cache: std::collections::HashMap<String, f64>,
}

impl TopologyEngine {
    pub fn new(embedding_dim: usize) -> Self {
        Self {
            major_radius: 1.0,
            minor_radius: 0.5,
            embedding_dim,
            surface_scaling: GOLDEN_RATIO_INV as f64, // Use golden ratio for natural scaling
            cache_size: 10000,
            similarity_cache: std::collections::HashMap::new(),
        }
    }

    /// Map high-dimensional embedding to Möbius surface parameters
    pub fn embed_to_surface(&self, embedding: &[f32]) -> SurfacePoint {
        // Reduce embedding to 2D parametric space using PCA-like projection
        let u = self.embedding_to_u_param(embedding);
        let v = self.embedding_to_v_param(embedding);

        // Calculate 3D point on Möbius surface
        self.surface_point(u, v)
    }

    /// Calculate point on Möbius surface using parametric equations
    fn surface_point(&self, u: f64, v: f64) -> SurfacePoint {
        // Möbius strip parametric equations
        // x = (R + v*cos(u/2)) * cos(u)
        // y = (R + v*cos(u/2)) * sin(u)
        // z = v * sin(u/2)

        let half_u = u / 2.0;
        let cos_half_u = half_u.cos();
        let sin_half_u = half_u.sin();

        let radius_term = self.major_radius + v * self.minor_radius * cos_half_u;

        let x = radius_term * u.cos();
        let y = radius_term * u.sin();
        let z = v * self.minor_radius * sin_half_u;

        SurfacePoint { u, v, x, y, z }
    }

    /// THE REVOLUTIONARY ALGORITHM: Geodesic distance instead of cosine
    pub fn semantic_similarity(&mut self, a: &[f32], b: &[f32]) -> f64 {
        // Create cache key
        let key = format!("{:?}_{:?}", &a[..5.min(a.len())], &b[..5.min(b.len())]);

        // Check cache first
        if let Some(&cached) = self.similarity_cache.get(&key) {
            return cached;
        }

        // Map to Möbius surface
        let surface_a = self.embed_to_surface(a);
        let surface_b = self.embed_to_surface(b);

        // Calculate geodesic distance on curved surface
        let geodesic_dist = self.geodesic_distance(surface_a, surface_b);

        // Convert distance to similarity [0,1]
        let similarity = (-geodesic_dist / self.surface_scaling).exp();

        // Cache result
        if self.similarity_cache.len() < self.cache_size {
            self.similarity_cache.insert(key, similarity);
        }

        similarity
    }

    /// Calculate geodesic distance between two points on Möbius surface
    fn geodesic_distance(&self, p1: SurfacePoint, p2: SurfacePoint) -> f64 {
        // Approximation using metric tensor
        // For Möbius surface: ds² = (∂x/∂u)²du² + 2(∂x/∂u)(∂x/∂v)dudv + (∂x/∂v)²dv²

        let du = p2.u - p1.u;
        let dv = p2.v - p1.v;

        // Metric tensor components at midpoint
        let u_mid = (p1.u + p2.u) / 2.0;
        let v_mid = (p1.v + p2.v) / 2.0;

        let g_uu = self.metric_uu(u_mid, v_mid);
        let g_uv = self.metric_uv(u_mid, v_mid);
        let g_vv = self.metric_vv(u_mid, v_mid);

        // Geodesic distance approximation
        (g_uu * du * du + 2.0 * g_uv * du * dv + g_vv * dv * dv).sqrt()
    }

    /// Compute reverse metric tensor g^ij (inverse of metric tensor)
    /// This is the mathematical foundation for understanding topological transformations
    pub fn compute_reverse_metric(&self, u: f64, v: f64) -> Result<(f64, f64, f64)> {
        // Compute metric tensor components
        let g_uu = self.metric_uu(u, v);
        let g_uv = self.metric_uv(u, v);
        let g_vv = self.metric_vv(u, v);

        // Compute determinant of metric tensor
        let det_g = g_uu * g_vv - g_uv * g_uv;

        // Check for singular metric (non-invertible)
        if det_g.abs() < 1e-10 {
            return Err(anyhow::anyhow!("Singular metric tensor at ({}, {})", u, v));
        }

        // Compute inverse metric tensor components (reverse metric)
        // For 2x2 matrix: [a b; c d]^-1 = (1/det) * [d -b; -c a]
        let g_inv_uu = g_vv / det_g;
        let g_inv_uv = -g_uv / det_g;
        let g_inv_vv = g_uu / det_g;

        Ok((g_inv_uu, g_inv_uv, g_inv_vv))
    }

    /// Raise vector indices using reverse metric (covariant -> contravariant)
    /// This demonstrates how vectors transform under topological structure
    pub fn raise_vector_indices(
        &self,
        u: f64,
        v: f64,
        omega_u: f64,
        omega_v: f64,
    ) -> Result<(f64, f64)> {
        let (g_inv_uu, g_inv_uv, g_inv_vv) = self.compute_reverse_metric(u, v)?;

        // Raise indices: V^i = g^ij * ω_j
        let v_u = g_inv_uu * omega_u + g_inv_uv * omega_v;
        let v_v = g_inv_uv * omega_u + g_inv_vv * omega_v;

        Ok((v_u, v_v))
    }

    /// Compute Christoffel symbols using reverse metric
    /// These describe how geodesics curve on the Möbius surface
    pub fn compute_christoffel_symbols(&self, u: f64, v: f64) -> Result<(f64, f64, f64, f64)> {
        let (g_inv_uu, g_inv_uv, g_inv_vv) = self.compute_reverse_metric(u, v)?;

        // Finite difference step for numerical derivatives
        let epsilon = 1e-6;

        // Compute partial derivatives of metric tensor
        let g_uu_u =
            (self.metric_uu(u + epsilon, v) - self.metric_uu(u - epsilon, v)) / (2.0 * epsilon);
        let g_uu_v =
            (self.metric_uu(u, v + epsilon) - self.metric_uu(u, v - epsilon)) / (2.0 * epsilon);
        let g_uv_u =
            (self.metric_uv(u + epsilon, v) - self.metric_uv(u - epsilon, v)) / (2.0 * epsilon);
        let g_uv_v =
            (self.metric_uv(u, v + epsilon) - self.metric_uv(u, v - epsilon)) / (2.0 * epsilon);
        let g_vv_u =
            (self.metric_vv(u + epsilon, v) - self.metric_vv(u - epsilon, v)) / (2.0 * epsilon);
        let g_vv_v =
            (self.metric_vv(u, v + epsilon) - self.metric_vv(u, v - epsilon)) / (2.0 * epsilon);

        // Christoffel symbols: Γ^k_ij = (1/2) g^kl (∂g_lj/∂x^i + ∂g_li/∂x^j - ∂g_ij/∂x^l)
        // Γ^u_uu = (1/2) g^uu (2∂g_uu/∂u) + (1/2) g^uv (2∂g_uv/∂u - ∂g_uu/∂v)
        let gamma_uu_u = 0.5 * g_inv_uu * (2.0 * g_uu_u) + 0.5 * g_inv_uv * (2.0 * g_uv_u - g_uu_v);

        // Γ^u_uv = (1/2) g^uu (∂g_uv/∂u + ∂g_uu/∂v) + (1/2) g^uv (∂g_vv/∂u)
        let gamma_uv_u = 0.5 * g_inv_uu * (g_uv_u + g_uu_v) + 0.5 * g_inv_uv * g_vv_u;

        // Γ^v_uu = (1/2) g^vu (2∂g_uu/∂u) + (1/2) g^vv (2∂g_uv/∂u - ∂g_uu/∂v)
        let gamma_uu_v = 0.5 * g_inv_uv * (2.0 * g_uu_u) + 0.5 * g_inv_vv * (2.0 * g_uv_u - g_uu_v);

        // Γ^v_uv = (1/2) g^vu (∂g_uv/∂u + ∂g_uu/∂v) + (1/2) g^vv (∂g_vv/∂u)
        let gamma_uv_v = 0.5 * g_inv_uv * (g_uv_u + g_uu_v) + 0.5 * g_inv_vv * g_vv_u;

        Ok((gamma_uu_u, gamma_uv_u, gamma_uu_v, gamma_uv_v))
    }

    /// Demonstrate non-orientability using reverse metric
    /// Shows how vectors flip orientation after traversing the Möbius strip
    pub fn demonstrate_non_orientability(&self, v: f64) -> Result<()> {
        info!("🌀 Non-Orientability Demonstration using Reverse Metrics");
        info!("=====================================================");

        // Test vector: (1, 0) in u-direction
        let test_omega_u = 1.0;
        let test_omega_v = 0.0;

        // At u = 0
        let (v_u_0, v_v_0) = self.raise_vector_indices(0.0, v, test_omega_u, test_omega_v)?;
        info!("At u=0: V^u = {:.6}, V^v = {:.6}", v_u_0, v_v_0);

        // At u = 2π (after full twist)
        let (v_u_2pi, v_v_2pi) = self.raise_vector_indices(TAU, v, test_omega_u, test_omega_v)?;
        info!("At u=2π: V^u = {:.6}, V^v = {:.6}", v_u_2pi, v_v_2pi);

        // Check if orientation flipped
        let orientation_change = (v_u_2pi / v_u_0).signum();
        if orientation_change < 0.0 {
            info!("✅ Orientation FLIPPED! (Non-orientable surface confirmed)");
        } else {
            info!("❌ No orientation flip detected");
        }

        Ok(())
    }

    /// Metric tensor component g_uu
    pub fn metric_uu(&self, u: f64, v: f64) -> f64 {
        // ∂x/∂u and ∂y/∂u components
        let half_u = u / 2.0;
        let cos_u = u.cos();
        let sin_u = u.sin();
        let cos_half_u = half_u.cos();
        let sin_half_u = half_u.sin();

        let r = self.major_radius + v * self.minor_radius * cos_half_u;
        let dr_du = -v * self.minor_radius * sin_half_u / 2.0;

        let dx_du = dr_du * cos_u - r * sin_u;
        let dy_du = dr_du * sin_u + r * cos_u;
        let dz_du = v * self.minor_radius * cos_half_u / 2.0;

        dx_du * dx_du + dy_du * dy_du + dz_du * dz_du
    }

    /// Metric tensor component g_vv
    pub fn metric_vv(&self, u: f64, v: f64) -> f64 {
        let half_u = u / 2.0;
        let cos_half_u = half_u.cos();
        let sin_half_u = half_u.sin();

        let dx_dv = self.minor_radius * cos_half_u * u.cos();
        let dy_dv = self.minor_radius * cos_half_u * u.sin();
        let dz_dv = self.minor_radius * sin_half_u;

        dx_dv * dx_dv + dy_dv * dy_dv + dz_dv * dz_dv
    }

    /// Metric tensor component g_uv
    pub fn metric_uv(&self, u: f64, v: f64) -> f64 {
        // Mixed partial derivatives
        let half_u = u / 2.0;
        let cos_u = u.cos();
        let sin_u = u.sin();
        let cos_half_u = half_u.cos();
        let sin_half_u = half_u.sin();

        let dr_du = -v * self.minor_radius * sin_half_u / 2.0;

        let dx_du =
            dr_du * cos_u - (self.major_radius + v * self.minor_radius * cos_half_u) * sin_u;
        let dx_dv = self.minor_radius * cos_half_u * cos_u;

        let dy_du =
            dr_du * sin_u + (self.major_radius + v * self.minor_radius * cos_half_u) * cos_u;
        let dy_dv = self.minor_radius * cos_half_u * sin_u;

        let dz_du = v * self.minor_radius * cos_half_u / 2.0;
        let dz_dv = self.minor_radius * sin_half_u;

        dx_du * dx_dv + dy_du * dy_dv + dz_du * dz_dv
    }

    /// Map embedding vector to u parameter [0, 2π]
    fn embedding_to_u_param(&self, embedding: &[f32]) -> f64 {
        // Use primary component for angular position
        let sum: f32 = embedding.iter().take(embedding.len() / 2).sum();
        let normalized = (sum / (embedding.len() / 2) as f32).abs();

        // Map to [0, 2π] with some chaos for variety
        (normalized as f64 % 1.0) * TAU
    }

    /// Map embedding vector to v parameter [-1, 1]
    fn embedding_to_v_param(&self, embedding: &[f32]) -> f64 {
        // Use secondary component for width position
        let sum: f32 = embedding.iter().skip(embedding.len() / 2).sum();
        let normalized = sum / (embedding.len() / 2) as f32;

        // Map to [-1, 1] and clamp
        (normalized as f64).tanh() // tanh naturally bounds to [-1,1]
    }
}

/// Benchmark framework to prove topology superiority
pub struct TopologyBenchmark {
    pub topology_engine: TopologyEngine,
    pub test_embeddings: Vec<(Vec<f32>, String)>, // (embedding, code_snippet)
    pub ground_truth_pairs: Vec<(usize, usize, f64)>, // (idx1, idx2, human_similarity_score)
}

impl TopologyBenchmark {
    pub fn new(embedding_dim: usize) -> Self {
        Self {
            topology_engine: TopologyEngine::new(embedding_dim),
            test_embeddings: Vec::new(),
            ground_truth_pairs: Vec::new(),
        }
    }

    /// Load test data from our 420 code reviews
    pub fn load_test_data(&mut self, creep_data_path: &str) -> Result<()> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        // Load JSONL file containing code review embeddings
        let file = File::open(creep_data_path)
            .map_err(|e| anyhow::anyhow!("Failed to open creep data file: {}", e))?;

        let reader = BufReader::new(file);
        let mut loaded_count = 0;

        for (idx, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| anyhow::anyhow!("Failed to read line {}: {}", idx, e))?;

            // Parse JSONL entry
            let entry: serde_json::Value = serde_json::from_str(&line)
                .map_err(|e| anyhow::anyhow!("Failed to parse JSON at line {}: {}", idx, e))?;

            // Extract embedding vector
            let embedding_array = entry["embedding"]
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("Missing embedding field at line {}", idx))?;

            let embedding: Vec<f32> = embedding_array
                .iter()
                .filter_map(|v| v.as_f64().map(|f| f as f32))
                .collect();

            if embedding.len() != self.topology_engine.embedding_dim {
                warn!(
                    "Skipping line {} - embedding dimension mismatch: expected {}, got {}",
                    idx,
                    self.topology_engine.embedding_dim,
                    embedding.len()
                );
                continue;
            }

            // Extract code snippet for reference
            let code_snippet = entry["code"]
                .as_str()
                .unwrap_or("(no code snippet)")
                .to_string();

            self.test_embeddings.push((embedding, code_snippet));

            // Create ground truth pairs if similarity score is provided
            if let Some(similar_to_idx) = entry["similar_to"].as_u64() {
                if let Some(similarity_score) = entry["similarity_score"].as_f64() {
                    self.ground_truth_pairs.push((
                        loaded_count,
                        similar_to_idx as usize,
                        similarity_score,
                    ));
                }
            }

            loaded_count += 1;
        }

        info!(
            "Loaded {} test embeddings with {} ground truth pairs from {}",
            loaded_count,
            self.ground_truth_pairs.len(),
            creep_data_path
        );

        if loaded_count == 0 {
            return Err(anyhow::anyhow!(
                "No valid embeddings loaded from {}",
                creep_data_path
            ));
        }

        Ok(())
    }

    /// THE SCIENTIFIC TEST: Topology vs Cosine
    pub fn run_similarity_benchmark(&mut self) -> BenchmarkResults {
        let mut topology_wins = 0;
        let mut cosine_wins = 0;
        let mut ties = 0;

        let mut topology_times = Vec::new();
        let mut cosine_times = Vec::new();

        for (idx1, idx2, ground_truth) in &self.ground_truth_pairs {
            let emb1 = &self.test_embeddings[*idx1].0;
            let emb2 = &self.test_embeddings[*idx2].0;

            // Measure topology method
            let start = std::time::Instant::now();
            let topo_sim = self.topology_engine.semantic_similarity(emb1, emb2);
            topology_times.push(start.elapsed().as_nanos() as f64);

            // Measure cosine method
            let start = std::time::Instant::now();
            let cosine_sim = cosine_similarity_baseline(emb1, emb2);
            cosine_times.push(start.elapsed().as_nanos() as f64);

            // Compare accuracy to ground truth
            let topo_error = (topo_sim - ground_truth).abs();
            let cosine_error = (cosine_sim as f64 - ground_truth).abs();

            if topo_error < cosine_error {
                topology_wins += 1;
            } else if cosine_error < topo_error {
                cosine_wins += 1;
            } else {
                ties += 1;
            }
        }

        BenchmarkResults {
            topology_wins,
            cosine_wins,
            ties,
            avg_topology_time_ns: topology_times.iter().sum::<f64>() / topology_times.len() as f64,
            avg_cosine_time_ns: cosine_times.iter().sum::<f64>() / cosine_times.len() as f64,
        }
    }
}

#[derive(Debug)]
pub struct BenchmarkResults {
    pub topology_wins: usize,
    pub cosine_wins: usize,
    pub ties: usize,
    pub avg_topology_time_ns: f64,
    pub avg_cosine_time_ns: f64,
}

impl BenchmarkResults {
    pub fn print_scientific_results(&self) {
        let total = self.topology_wins + self.cosine_wins + self.ties;
        let topo_percent = (self.topology_wins as f64 / total as f64) * 100.0;
        let cosine_percent = (self.cosine_wins as f64 / total as f64) * 100.0;

        info!("\n🔬 SCIENTIFIC BENCHMARK RESULTS");
        info!("================================");
        info!("Total test pairs: {}", total);
        info!(
            "Topology wins: {} ({:.1}%)",
            self.topology_wins, topo_percent
        );
        info!("Cosine wins: {} ({:.1}%)", self.cosine_wins, cosine_percent);
        info!("Ties: {}", self.ties);
        info!("Performance:");
        info!("Topology avg: {:.0}ns", self.avg_topology_time_ns);
        info!("Cosine avg: {:.0}ns", self.avg_cosine_time_ns);
        info!(
            "Topology overhead: {:.1}x",
            self.avg_topology_time_ns / self.avg_cosine_time_ns
        );

        if topo_percent > cosine_percent + 5.0 {
            info!("🎉 TOPOLOGY WINS! Curved space is better than flat space!");
            info!("📄 Ready for academic publication");
        } else if cosine_percent > topo_percent + 5.0 {
            info!("😞 Cosine wins. Back to the drawing board.");
        } else {
            info!("🤔 Inconclusive. Need more data or better mapping.");
        }
    }
}

/// Baseline cosine similarity for comparison
pub fn cosine_similarity_baseline(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
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
    fn test_topology_vs_cosine() {
        let mut engine = TopologyEngine::new(384); // BERT embedding size

        // Simple test vectors
        let a = vec![1.0, 0.0, 0.5; 384];
        let b = vec![0.9, 0.1, 0.4; 384];
        let c = vec![-1.0, 0.0, -0.5; 384]; // Opposite

        let topo_ab = engine.semantic_similarity(&a, &b);
        let topo_ac = engine.semantic_similarity(&a, &c);

        let cosine_ab = cosine_similarity_baseline(&a, &b) as f64;
        let cosine_ac = cosine_similarity_baseline(&a, &c) as f64;

        info!("A-B: Topology={:.3}, Cosine={:.3}", topo_ab, cosine_ab);
        info!("A-C: Topology={:.3}, Cosine={:.3}", topo_ac, cosine_ac);

        // Both should agree that A-B more similar than A-C
        assert!(topo_ab > topo_ac);
        assert!(cosine_ab > cosine_ac);
    }
}
