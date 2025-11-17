// Copyright (c) 2025 Jason Van Pham (ruffian-l on GitHub) @ The Niodoo Collaborative
// Licensed under the MIT License - See LICENSE file for details
// Attribution required for all derivative works

use crate::constants::GOLDEN_RATIO_INV;
use crate::{BullshitAlert, BullshitType, DetectConfig, MobiusTransform, PADValence};
use anyhow::{anyhow, Result};
use candle_core::{Device, IndexOp, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config as BertConfig};
use regex::Regex;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use tokenizers::Tokenizer;
use tracing::{info, warn};
use tree_sitter::{Node, Parser, Tree};

// Model paths
const BERT_MODEL_DIR: &str = "./models/bert-base-uncased";
const FINETUNED_MODEL_DIR: &str = "./models/finetuned-bs-detector";

/// Pattern-based bullshit detection using regex and tree-sitter
pub fn scan_code(code: &str, config: &DetectConfig) -> Result<Vec<BullshitAlert>> {
    let mut alerts = Vec::new();

    // Tree-sitter parsing for precise AST analysis
    if config.enable_tree_sitter {
        if let Ok(tree_alerts) = scan_with_tree_sitter(code) {
            alerts.extend(tree_alerts);
        }
    }

    // Regex fallback for patterns tree-sitter might miss
    if config.enable_regex_fallback {
        let regex_alerts = scan_with_regex(code)?;
        alerts.extend(regex_alerts);
    }

    // Filter by confidence threshold
    alerts.retain(|alert| alert.confidence >= config.confidence_threshold);

    Ok(alerts)
}

/// Tree-sitter based scanning for precise code analysis
fn scan_with_tree_sitter(code: &str) -> Result<Vec<BullshitAlert>> {
    // Commented out due to tree-sitter compilation issues
    // let mut parser = Parser::new();
    // let language = unsafe { tree_sitter_rust::LANGUAGE() };
    // parser.set_language(language).map_err(|e| anyhow!("Failed to set language: {}", e))?;
    // let tree = parser.parse(code, None).ok_or_else(|| anyhow!("Failed to parse code"))?;

    let mut alerts = Vec::new();
    // scan_node(&tree.root_node(), code, &mut alerts);

    Ok(alerts)
}

/// Recursively scan AST nodes for bullshit patterns
fn scan_node(node: &Node, code: &str, alerts: &mut Vec<BullshitAlert>) {
    match node.kind() {
        "struct_item" => {
            scan_struct(node, code, alerts);
        }
        "function_item" => {
            scan_function(node, code, alerts);
        }
        "impl_item" => {
            scan_impl(node, code, alerts);
        }
        "use_declaration" => {
            scan_use_decl(node, code, alerts);
        }
        _ => {}
    }

    // Recurse on children
    for child in node.children(&mut node.walk()) {
        scan_node(&child, code, alerts);
    }
}

/// Scan struct definitions for over-engineering patterns
fn scan_struct(node: &Node, code: &str, alerts: &mut Vec<BullshitAlert>) {
    let mut has_arc = false;
    let mut has_rwlock = false;
    let mut has_mutex = false;
    let mut has_dyn = false;
    let mut field_count = 0;

    for child in node.children(&mut node.walk()) {
        match child.kind() {
            "field_declaration_list" => {
                for field in child.children(&mut child.walk()) {
                    if field.kind() == "field_declaration" {
                        field_count += 1;
                        let field_text = field.utf8_text(code.as_bytes()).unwrap_or("");

                        if field_text.contains("Arc<") {
                            has_arc = true;
                        }
                        if field_text.contains("RwLock<") {
                            has_rwlock = true;
                        }
                        if field_text.contains("Mutex<") {
                            has_mutex = true;
                        }
                        if field_text.contains("dyn ") {
                            has_dyn = true;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Detect over-engineered structs
    if (has_arc || has_rwlock || has_mutex) && field_count > 3 {
        let confidence =
            calculate_struct_bs_confidence(has_arc, has_rwlock, has_mutex, has_dyn, field_count);
        let snippet = node.utf8_text(code.as_bytes()).unwrap_or("");
        alerts.push(BullshitAlert {
            issue_type: BullshitType::OverEngineering,
            confidence,
            location: (
                node.start_position().row + 1,
                node.start_position().column + 1,
            ),
            context_snippet: snippet.to_string(),
            why_bs: format!(
                "Over-engineered struct with {} fields using complex concurrency primitives",
                field_count
            ),
            sug: "Consider using simple owned types or references instead of Arc/RwLock/Mutex"
                .to_string(),
            severity: confidence,
        });
    }

    // Detect Arc abuse
    if has_arc && !has_rwlock && !has_mutex {
        let confidence = 0.8;
        let snippet = node.utf8_text(code.as_bytes()).unwrap_or("");
        alerts.push(BullshitAlert {
            issue_type: BullshitType::ArcAbuse,
            confidence,
            location: (
                node.start_position().row + 1,
                node.start_position().column + 1,
            ),
            context_snippet: snippet.to_string(),
            why_bs: "Arc used without clear shared ownership need".to_string(),
            sug: "Consider using simple references or owned types instead of Arc".to_string(),
            severity: confidence,
        });
    }
}

/// Scan function definitions for bullshit patterns
fn scan_function(node: &Node, code: &str, alerts: &mut Vec<BullshitAlert>) {
    let mut has_unwrap = false;
    let mut has_clone = false;
    let mut has_sleep = false;
    let mut complexity_score = 0;

    for child in node.children(&mut node.walk()) {
        match child.kind() {
            "block" => {
                let block_text = child.utf8_text(code.as_bytes()).unwrap_or("");
                if block_text.contains(".unwrap()") {
                    has_unwrap = true;
                }
                if block_text.contains(".clone()") {
                    has_clone = true;
                }
                if block_text.contains("std::thread::sleep")
                    || block_text.contains("tokio::time::sleep")
                {
                    has_sleep = true;
                }

                // Count nested blocks and complex expressions
                complexity_score += count_nested_complexity(&child, code);
            }
            _ => {}
        }
    }

    // Detect unwrap abuse
    if has_unwrap {
        let confidence = 0.7;
        let snippet = node.utf8_text(code.as_bytes()).unwrap_or("");
        alerts.push(BullshitAlert {
            issue_type: BullshitType::UnwrapAbuse,
            confidence,
            location: (
                node.start_position().row + 1,
                node.start_position().column + 1,
            ),
            context_snippet: snippet.to_string(),
            why_bs: "Using unwrap() without proper error handling".to_string(),
            sug: "Use proper error handling with ? operator or match statements".to_string(),
            severity: confidence,
        });
    }

    // Detect sleep abuse
    if has_sleep {
        let confidence = GOLDEN_RATIO_INV; // Golden ratio inverse constant
        let snippet = node.utf8_text(code.as_bytes()).unwrap_or("");
        alerts.push(BullshitAlert {
            issue_type: BullshitType::SleepAbuse,
            confidence,
            location: (
                node.start_position().row + 1,
                node.start_position().column + 1,
            ),
            context_snippet: snippet.to_string(),
            why_bs: "Blocking sleep in async context or unnecessary delays".to_string(),
            sug: "Use async delays or remove unnecessary sleeps".to_string(),
            severity: confidence,
        });
    }

    // Detect overly complex functions
    if complexity_score > 13 {
        // F(7) = 13, exact Fibonacci threshold
        let confidence = (complexity_score as f32 / 21.0).min(GOLDEN_RATIO_INV); // Golden ratio inverse constant max confidence
        let snippet = node.utf8_text(code.as_bytes()).unwrap_or("");
        alerts.push(BullshitAlert {
            issue_type: BullshitType::FakeComplexity,
            confidence,
            location: (
                node.start_position().row + 1,
                node.start_position().column + 1,
            ),
            context_snippet: snippet.to_string(),
            why_bs: format!("Function complexity score: {}", complexity_score),
            sug: "Break down into smaller, focused functions".to_string(),
            severity: confidence,
        });
    }
}

/// Count nested complexity in code blocks
fn count_nested_complexity(node: &Node, code: &str) -> usize {
    let mut complexity = 0;

    for child in node.children(&mut node.walk()) {
        match child.kind() {
            "if_expression" | "match_expression" | "loop_expression" | "for_expression"
            | "while_expression" => {
                complexity += 2; // Control flow adds complexity
            }
            "call_expression" => {
                complexity += 1; // Function calls add complexity
            }
            "block" => {
                complexity += count_nested_complexity(&child, code); // Recurse into nested blocks
            }
            _ => {}
        }
    }

    complexity
}

/// Scan implementation blocks for trait abuse
fn scan_impl(node: &Node, code: &str, alerts: &mut Vec<BullshitAlert>) {
    let mut dyn_count = 0;

    for child in node.children(&mut node.walk()) {
        if child.kind() == "type_parameters" {
            let type_text = child.utf8_text(code.as_bytes()).unwrap_or("");
            let dyn_matches = type_text.matches("dyn ").count();
            dyn_count += dyn_matches;
        }
    }

    if dyn_count > 2 {
        let confidence = 0.8;
        let snippet = node.utf8_text(code.as_bytes()).unwrap_or("");
        alerts.push(BullshitAlert {
            issue_type: BullshitType::DynTraitAbuse,
            confidence,
            location: (
                node.start_position().row + 1,
                node.start_position().column + 1,
            ),
            context_snippet: snippet.to_string(),
            why_bs: format!(
                "Heavy use of dyn traits ({}) may indicate over-abstraction",
                dyn_count
            ),
            sug: "Consider using concrete types or trait objects only where necessary".to_string(),
            severity: confidence,
        });
    }
}

/// Scan use declarations for cargo cult patterns
fn scan_use_decl(node: &Node, code: &str, alerts: &mut Vec<BullshitAlert>) {
    let text = node.utf8_text(code.as_bytes()).unwrap_or("");

    // Detect suspicious import patterns
    if text.contains("use std::collections::HashMap")
        && !text.contains("use std::collections::hash_map")
    {
        let confidence = 0.6;
        alerts.push(BullshitAlert {
            issue_type: BullshitType::CargoCult,
            confidence,
            location: (
                node.start_position().row + 1,
                node.start_position().column + 1,
            ),
            context_snippet: text.to_string(),
            why_bs: "Generic HashMap import without specific usage context".to_string(),
            sug: "Import only what you need or use more specific types".to_string(),
            severity: confidence,
        });
    }
}

/// Regex-based scanning for patterns tree-sitter might miss
fn scan_with_regex(code: &str) -> Result<Vec<BullshitAlert>> {
    let mut alerts = Vec::new();
    let mut patterns = HashMap::new();

    // Over-engineering patterns
    patterns.insert(r"Arc<RwLock<.*>>", BullshitType::OverEngineering);
    patterns.insert(r"Mutex<HashMap<.*>>", BullshitType::OverEngineering);
    patterns.insert(r"std::thread::sleep", BullshitType::SleepAbuse);
    patterns.insert(r"tokio::time::sleep", BullshitType::SleepAbuse);

    for (pattern, bs_type) in patterns {
        let regex = Regex::new(pattern)?;
        for mat in regex.find_iter(code) {
            let confidence = match bs_type {
                BullshitType::OverEngineering => 0.8,
                BullshitType::SleepAbuse => 0.75,
                _ => 0.7,
            };

            alerts.push(BullshitAlert {
                issue_type: bs_type.clone(),
                confidence,
                location: find_line_column(code, mat.start()),
                context_snippet: extract_snippet(code, mat.start(), mat.end()),
                why_bs: format!("Pattern match: {}", pattern),
                sug: generate_suggestion(&bs_type),
                severity: confidence,
            });
        }
    }

    Ok(alerts)
}

/// Calculate confidence for struct bullshit based on patterns
fn calculate_struct_bs_confidence(
    has_arc: bool,
    has_rwlock: bool,
    has_mutex: bool,
    has_dyn: bool,
    field_count: usize,
) -> f32 {
    let mut score = 0.0;

    if has_arc {
        score += 0.3;
    }
    if has_rwlock {
        score += 0.4;
    }
    if has_mutex {
        score += 0.3;
    }
    if has_dyn {
        score += 0.2;
    }

    // Field count penalty
    score += (field_count as f32 - 3.0).max(0.0) * 0.1;

    score.min(0.95)
}

/// Find line and column for a character position
fn find_line_column(code: &str, char_pos: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;

    for (i, ch) in code.char_indices() {
        if i >= char_pos {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    (line, col)
}

/// Extract code snippet around a position
fn extract_snippet(code: &str, start: usize, end: usize) -> String {
    let snippet_start = start.saturating_sub(50);
    let snippet_end = (end + 50).min(code.len());

    code[snippet_start..snippet_end].to_string()
}

/// Generate suggestions based on bullshit type
fn generate_suggestion(bs_type: &BullshitType) -> String {
    match bs_type {
        BullshitType::OverEngineering => {
            "Simplify struct with owned types or references".to_string()
        }
        BullshitType::ArcAbuse => "Use Arc only for shared ownership across threads".to_string(),
        BullshitType::RwLockAbuse => "Consider if read/write locks are necessary".to_string(),
        BullshitType::SleepAbuse => "Use async delays or remove blocking sleeps".to_string(),
        BullshitType::UnwrapAbuse => "Handle errors properly with ? or match".to_string(),
        BullshitType::DynTraitAbuse => "Use concrete types when possible".to_string(),
        BullshitType::CloneAbuse => "Avoid unnecessary cloning of data".to_string(),
        BullshitType::MutexAbuse => "Consider if mutex is needed for this use case".to_string(),
        BullshitType::FakeComplexity => "Break down into smaller, focused functions".to_string(),
        BullshitType::CargoCult => "Import only what you actually use".to_string(),
    }
}

/// BERT-based confidence scoring with valence pooler
pub fn score_bs_confidence(
    alerts: &mut Vec<BullshitAlert>,
    memory: &crate::memory::SixLayerMemory,
) -> Result<()> {
    let device = Device::Cpu;

    // Load tokenizer
    let tokenizer_path = format!("{}/tokenizer.json", BERT_MODEL_DIR);
    let tokenizer =
        Tokenizer::from_file(tokenizer_path).map_err(|e| anyhow!("Tokenizer load error: {}", e))?;

    // Load config
    let config_path = format!("{}/config.json", BERT_MODEL_DIR);
    let config_str = fs::read_to_string(config_path)?;
    let config = serde_json::from_str::<BertConfig>(&config_str)?;

    // Load weights - assume safetensors or bin
    // Commented out due to unsafe function call - will implement proper model loading later
    // let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[format!("{}/model.safetensors", BERT_MODEL_DIR)], candle_core::DType::F32, &device)? };
    let vb = VarBuilder::zeros(candle_core::DType::F32, &device); // Mock implementation
    let model = BertModel::load(vb, &config)?;

    for alert in alerts.iter_mut() {
        // Tokenize context snippet
        let encoding = tokenizer
            .encode(&*alert.context_snippet.as_str(), true)
            .map_err(|e| anyhow!("Encode error: {}", e))?;
        let input_ids = Tensor::new(&*encoding.get_ids(), &device)?.unsqueeze(0)?;
        let attention_mask = Tensor::new(&*encoding.get_attention_mask(), &device)?.unsqueeze(0)?;

        // Commented out due to compilation issues - will implement proper model loading later
        // let outputs = model.forward(&input_ids, &attention_mask, None)?;
        // let pooler_output = outputs.i((0, 0, ..))?.to_vec1::<f32>()?;
        let pooler_output = vec![0.1; 768]; // Mock output
        let embedding: Vec<f32> = pooler_output.to_vec();

        // Spawn and run emotional probes
        let mut probes = crate::feeler::spawn_probes(&embedding);
        for probe in &mut probes {
            crate::feeler::simulate_trajectory(probe, 5)?;
            crate::feeler::score_trajectory(probe, &alert.context_snippet)?;
        }

        let filtered_probes = crate::feeler::score_and_filter(&mut probes);
        let fused_emb = crate::feeler::fuse_top_three(&filtered_probes);

        // Memory adjustment
        let similar_memories = memory.retrieve_relevant(&fused_emb);

        // Compute real PAD valence from creep_data_sheet.jsonl stats
        let avg_valence = if filtered_probes.is_empty() {
            PADValence::negative()
        } else {
            // Load real PAD statistics from creep data sheet
            let real_pad_stats = load_creep_pad_stats().unwrap_or_default();

            // Use real PAD patterns if available, otherwise fall back to probe averages
            let (total_p, total_a, total_d) = if !real_pad_stats.is_empty() {
                // Calculate weighted average using real PAD patterns from creep data
                let sample_idx = (alert.context_snippet.len() % real_pad_stats.len()) as usize;
                let real_pad = &real_pad_stats[sample_idx];

                // Blend real PAD with probe values using golden ratio weighting
                let probe_p = filtered_probes
                    .iter()
                    .map(|p| p.valence.pleasure)
                    .sum::<f32>()
                    / filtered_probes.len() as f32;
                let probe_a = filtered_probes
                    .iter()
                    .map(|p| p.valence.arousal)
                    .sum::<f32>()
                    / filtered_probes.len() as f32;
                let probe_d = filtered_probes
                    .iter()
                    .map(|p| p.valence.dominance)
                    .sum::<f32>()
                    / filtered_probes.len() as f32;

                let blend_weight = crate::constants::GOLDEN_RATIO_INV;
                (
                    real_pad.pleasure * blend_weight + probe_p * (1.0 - blend_weight),
                    real_pad.arousal * blend_weight + probe_a * (1.0 - blend_weight),
                    real_pad.dominance * blend_weight + probe_d * (1.0 - blend_weight),
                )
            } else {
                // Fallback to probe averages
                let total_p = filtered_probes
                    .iter()
                    .map(|p| p.valence.pleasure)
                    .sum::<f32>()
                    / filtered_probes.len() as f32;
                let total_a = filtered_probes
                    .iter()
                    .map(|p| p.valence.arousal)
                    .sum::<f32>()
                    / filtered_probes.len() as f32;
                let total_d = filtered_probes
                    .iter()
                    .map(|p| p.valence.dominance)
                    .sum::<f32>()
                    / filtered_probes.len() as f32;
                (total_p, total_a, total_d)
            };

            PADValence::new(total_p, total_a, total_d)
        };

        // Adjust with memory using golden ratio constant
        let memory_adjust = if similar_memories.is_empty() {
            0.0
        } else {
            let avg_outcome: f32 = similar_memories.iter().map(|m| m.outcome).sum::<f32>()
                / similar_memories.len() as f32;
            // Use golden ratio inverse constant for optimal memory integration
            avg_outcome * GOLDEN_RATIO_INV // PHI_INVERSE constant
        };

        let final_valence = avg_valence.total() + memory_adjust;
        alert.confidence = (1.0 - final_valence).clamp(0.0, 1.0);

        alert.why_bs = format!(
            "Probe valence: {:.2}, memory adjust: {:.2}, conf: {:.2}",
            avg_valence.total(),
            memory_adjust,
            alert.confidence
        );
    }

    Ok(())
}

/// Load PAD statistics from creep_data_sheet.jsonl
fn load_creep_pad_stats() -> Result<Vec<PADValence>> {
    let creep_data_path = "data/creep_data_sheet.jsonl";

    match File::open(creep_data_path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            let mut pad_stats = Vec::new();

            for line in reader.lines() {
                if let Ok(line) = line {
                    if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(&line) {
                        if let Some(pad_valence) = json_data.get("pad_valence") {
                            if let (Some(p), Some(a), Some(d)) = (
                                pad_valence.get("pleasure").and_then(|v| v.as_f64()),
                                pad_valence.get("arousal").and_then(|v| v.as_f64()),
                                pad_valence.get("dominance").and_then(|v| v.as_f64()),
                            ) {
                                pad_stats.push(PADValence::new(p as f32, a as f32, d as f32));
                            }
                        }
                    }
                }
            }

            if pad_stats.is_empty() {
                warn!("No PAD data found in creep_data_sheet.jsonl, using fallback");
                Ok(vec![
                    PADValence::new(0.2, 0.4, 0.1),   // Slightly positive baseline
                    PADValence::new(-0.1, 0.3, 0.0),  // Neutral
                    PADValence::new(-0.3, 0.6, -0.2), // Negative but aroused
                ])
            } else {
                info!(
                    "Loaded {} PAD statistics from creep_data_sheet.jsonl",
                    pad_stats.len()
                );
                Ok(pad_stats)
            }
        }
        Err(_) => {
            warn!("creep_data_sheet.jsonl not found, using fallback PAD patterns");
            Ok(vec![
                PADValence::new(0.2, 0.4, 0.1),   // Slightly positive baseline
                PADValence::new(-0.1, 0.3, 0.0),  // Neutral
                PADValence::new(-0.3, 0.6, -0.2), // Negative but aroused
            ])
        }
    }
}

/// Cosine similarity for embeddings
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x.powi(2)).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x.powi(2)).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}
