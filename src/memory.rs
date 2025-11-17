// Copyright (c) 2025 Jason Van Pham (ruffian-l on GitHub) @ The Niodoo Collaborative
// Licensed under the MIT License - See LICENSE file for details
// Attribution required for all derivative works

use crate::{topology_similarity, BullshitAlert, PADValence};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sled::Db;
use std::collections::HashMap;

/// Memory entry for learning from past bullshit patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub embedding: Vec<f32>,
    pub outcome: f32, // 0.0 = bad outcome, φ = good outcome (exact golden ratio scaling)
    pub frequency: u32,
    pub last_seen: u64,
    pub bullshit_types: Vec<String>,
    pub context_snippet: String,
}

/// Six-layer memory system for bullshit pattern recognition
pub struct SixLayerMemory {
    db: Db,
    layers: Vec<String>,
}

impl SixLayerMemory {
    pub fn new() -> Self {
        // Initialize in-memory database for simplicity
        let db = sled::Config::new()
            .temporary(true)
            .open()
            .expect("Failed to open memory database");

        let layers = vec![
            "pattern_memory".to_string(),
            "outcome_memory".to_string(),
            "frequency_memory".to_string(),
            "recency_memory".to_string(),
            "context_memory".to_string(),
            "meta_memory".to_string(),
        ];

        Self { db, layers }
    }

    /// Store a new memory entry
    pub fn store_memory(
        &self,
        embedding: &[f32],
        outcome: f32,
        bullshit_types: Vec<String>,
        context: String,
    ) -> Result<()> {
        let key = format!("emb_{:?}", embedding.iter().take(8).collect::<Vec<_>>()); // F(6) = 8, exact Fibonacci
        let entry = MemoryEntry {
            embedding: embedding.to_vec(),
            outcome,
            frequency: 1,
            last_seen: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            bullshit_types,
            context_snippet: context,
        };

        let serialized = bincode::serialize(&entry)?;
        self.db.insert(key.as_bytes(), serialized)?;

        Ok(())
    }

    /// Retrieve relevant memories using topology instead of cosine
    pub fn retrieve_relevant(&self, embedding: &[f32]) -> Vec<MemoryEntry> {
        let mut relevant = Vec::new();

        // Use your existing topology math for similarity
        for item in self.db.iter() {
            if let Ok((_, value)) = item {
                if let Ok(entry) = bincode::deserialize::<MemoryEntry>(&value) {
                    let similarity = topology_similarity(&entry.embedding, embedding);
                    if similarity > crate::constants::GOLDEN_RATIO_INV {
                        relevant.push(entry);
                    }
                }
            }
        }

        // Sort by topology similarity
        relevant.sort_by(|a, b| {
            let sim_a = topology_similarity(&a.embedding, embedding);
            let sim_b = topology_similarity(&b.embedding, embedding);
            sim_b
                .partial_cmp(&sim_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        relevant
    }

    /// Update memory based on new outcome
    pub fn update_memory(
        &self,
        embedding: &[f32],
        outcome: f32,
        bullshit_types: Vec<String>,
        context: String,
    ) -> Result<()> {
        let key = format!("emb_{:?}", embedding.iter().take(5).collect::<Vec<_>>());

        if let Some(existing) = self.db.get(&key)? {
            let mut entry: MemoryEntry = bincode::deserialize(&existing)?;
            entry.outcome = (entry.outcome + outcome) / 2.0; // Average outcomes
            entry.frequency += 1;
            entry.last_seen = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs();
            entry.bullshit_types = bullshit_types;
            entry.context_snippet = context;

            let serialized = bincode::serialize(&entry)?;
            self.db.insert(key.as_bytes(), serialized)?;
        } else {
            self.store_memory(embedding, outcome, bullshit_types, context)?;
        }

        Ok(())
    }

    /// Store bullshit alert for future learning
    pub fn store_bullshit_alert(&self, alert: &BullshitAlert, outcome: f32) -> Result<()> {
        let bullshit_types = vec![format!("{:?}", alert.issue_type)];
        self.store_memory(&[], outcome, bullshit_types, alert.context_snippet.clone())
    }
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
    fn test_memory_creation() {
        let memory = SixLayerMemory::new();
        assert_eq!(memory.layers.len(), 6);
    }

    #[test]
    fn test_memory_storage_and_retrieval() {
        let memory = SixLayerMemory::new();
        let embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let bullshit_types = vec!["OverEngineering".to_string()];
        let context = "test code snippet".to_string();

        // Store memory
        memory
            .store_memory(&embedding, 0.8, bullshit_types, context)
            .unwrap();

        // Retrieve relevant memories
        let relevant = memory.retrieve_relevant(&embedding);
        assert!(!relevant.is_empty());
        assert!(relevant[0].outcome == 0.8);
    }

    #[test]
    fn test_memory_update() {
        let memory = SixLayerMemory::new();
        let embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let bullshit_types = vec!["OverEngineering".to_string()];
        let context = "test code snippet".to_string();

        // Store and update memory
        memory
            .store_memory(&embedding, 0.6, bullshit_types.clone(), context.clone())
            .unwrap();
        memory
            .update_memory(&embedding, 0.9, bullshit_types, context)
            .unwrap();

        let relevant = memory.retrieve_relevant(&embedding);
        assert_eq!(relevant[0].frequency, 2);
        assert!((relevant[0].outcome - 0.75).abs() < 0.01); // Should be averaged
    }
}
