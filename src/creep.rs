// Copyright (c) 2025 Jason Van Pham (ruffian-l on GitHub) @ The Niodoo Collaborative
// Licensed under the MIT License - See LICENSE file for details
// Attribution required for all derivative works

use anyhow::Result;
use git2::{Repository, DiffOptions, DiffDeltaType, Diff, DiffHunk, DiffLine}; // Add
use nalgebra::{DMatrix, DVector};
use crate::creep_swarm::{CreepSwarm, CodeDelta, ChangeType}; // Assume creep_swarm as dependency or copy code
use crate::graph::build_graph; // Assume
use crate::constants::{GOLDEN_RATIO_F64, GOLDEN_RATIO_INV_F64};
use std::collections::HashMap;
use regex;

#[derive(Debug, Clone)]
pub struct CreepAgent {
    pub id: String,
    pub embedding: Vec<f32>,
    pub tumor_size: f64,
    pub activation: f64,
    pub pad_state: super::pad::PADState, // From creep_swarm
}

impl CreepAgent {
    pub fn new(id: String, embedding: Vec<f32>) -> Self {
        Self {
            id,
            embedding,
            // Use exact golden ratio inverse φ⁻¹ = (√5 - 1)/2 for tumor size scaling
            tumor_size: embedding.len() as f64 * GOLDEN_RATIO_INV_F64, // Exact φ⁻¹
            activation: GOLDEN_RATIO_F64, // Exact φ = (1 + √5)/2
            pad_state: super::pad::PADState::zero(),
        }
    }

    pub fn from_result(result: &super::AgentResult) -> Self {
        Self {
            id: result.agent_id.clone(),
            embedding: result.embedding.clone(),
            tumor_size: GOLDEN_RATIO_F64, // Exact φ tumor size
            activation: result.activation,
            pad_state: result.pad_state,
        }
    }

    pub fn process(&self) -> super::AgentResult {
        super::AgentResult {
            agent_id: self.id.clone(),
            embedding: self.embedding.clone(),
            pad_state: self.pad_state,
            activation: self.activation,
            // Use Fibonacci number for processing time estimation
            processing_time_ms: 13, // Fibonacci number for optimal timing
        }
    }
}

fn parse_diff_hatchery(repo: &git2::Repository, diff_opts: &mut git2::DiffOptions) -> Result<DVector<f64>> {
    let head = repo.head()?;
    let tree = head.peel_to_tree()?;
    let diff = repo.diff_tree_to_index(Some(&tree), None, Some(diff_opts))?;
    let mut h = DVector::zeros(/* n from graph */ 10); // Assume n
    let phi = GOLDEN_RATIO_F64;
    
    for delta in diff.deltas() {
        if delta.status() == git2::DeltaType::Modified {
            for hunk in delta.hunks()? {
                for line in hunk.lines()? {
                    if line.origin() == b'+' || line.origin() == b'-' {
                        let content = std::str::from_utf8(line.content())?;
                        if let Some(cap) = regex::Regex::new(r"\s*(fn|struct)\s+(\w+)").unwrap().captures(content) {
                            if let Some(name_cap) = cap.get(2) {
                                let name = name_cap.as_str();
                                // Assume node_map.get(name) -> idx
                                if let Some(&idx) = /* node_map */ std::collections::HashMap::new().get(name) { // Stub map
                                    h[idx] += phi;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(h)
}

pub async fn spawn_from_diff(diff_content: &str, num_agents: usize, _model: &str) -> Result<Vec<CreepAgent>> {
    // Build graph for node_map
    let code = "dummy code"; // Need full codebase; assume passed or global
    let (_, _, node_map) = build_graph(code);
    
    let repo = Repository::open(".")?;
    let mut opts = DiffOptions::new();
    opts.include_untracked(true);
    let h = parse_diff_hatchery(&repo, &mut opts)?;
    
    let mut deltas = Vec::new();
    for line in diff_content.lines() {
        if line.starts_with("+") || line.starts_with("-") {
            deltas.push(CodeDelta {
                id: format!("delta_{}", deltas.len()),
                content: line.to_string(),
                file_path: "unknown.rs".to_string(),
                change_type: if line.starts_with("+") { ChangeType::Added } else { ChangeType::Modified },
                timestamp: chrono::Utc::now(),
            });
        }
    }
    
    let swarm = CreepSwarm::new().await?;
    // Pass h to swarm or set initial heat
    let results = swarm.spawn_creep_swarm(deltas).await?;
    // After spawn, apply h to heat_state if needed
    
    let agents: Vec<CreepAgent> = results.into_iter().map(|r| CreepAgent::from_result(&r)).collect();
    Ok(agents.into_iter().take(num_agents).collect())
}

#[tokio::test]
async fn test_spawn_from_diff() {
    let diff = "+fn foo() {}";
    let agents = spawn_from_diff(diff, 1, "").await.unwrap();
    assert_eq!(agents.len(), 1);
    // h parse test separate
    let repo = // Mock repo stub
    let h = parse_diff_hatchery(&repo, &mut DiffOptions::new())?;
    assert!(h[/* foo idx */ 0] > 1.618);
}

pub fn spread_activation(agents: &mut [CreepAgent], graph_laplacian: &DMatrix<f64>, timestep: f32) -> Result<()> {
    // Use creep_swarm graph heat diffusion
    // Assume graph from swarm
    Ok(())
}

pub fn cache_review(agents: &[CreepAgent]) -> std::collections::HashMap<String, Vec<f32>> {
    agents.iter().map(|a| (a.id.clone(), a.embedding.clone())).collect()
}
