// Copyright (c) 2025 Jason Van Pham (ruffian-l on GitHub) @ The Niodoo Collaborative
// Licensed under the MIT License - See LICENSE file for details
// Attribution required for all derivative works

//! Bullshitdetector - Fast pattern detection for magic numbers and code smells
//! 
//! This crate provides blazing-fast regex-based detection of hardcoded values,
//! magic numbers, and common code smells in source code.
//! 
//! # Quick Start
//! 
//! ```rust
//! use bullshitdetector::{DetectConfig, scan_code};
//! 
//! let code = r#"
//!     if confidence > 0.85 {
//!         do_something();
//!     }
//! "#;
//! 
//! let config = DetectConfig::default();
//! let alerts = scan_code(code, &config).unwrap();
//! 
//! for alert in alerts {
//!     println!("Found {} at line {}", alert.issue_type, alert.location.0);
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

pub mod constants;

/// Bullshit alert types
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum BullshitType {
    FakeComplexity,
    CargoCult,
    OverEngineering,
    ArcAbuse,
    RwLockAbuse,
    SleepAbuse,
    UnwrapAbuse,
    DynTraitAbuse,
    CloneAbuse,
    MutexAbuse,
    MagicNumber,
    HardcodedThreshold,
}

impl fmt::Display for BullshitType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BullshitType::FakeComplexity => write!(f, "FakeComplexity"),
            BullshitType::CargoCult => write!(f, "CargoCult"),
            BullshitType::OverEngineering => write!(f, "OverEngineering"),
            BullshitType::ArcAbuse => write!(f, "ArcAbuse"),
            BullshitType::RwLockAbuse => write!(f, "RwLockAbuse"),
            BullshitType::SleepAbuse => write!(f, "SleepAbuse"),
            BullshitType::UnwrapAbuse => write!(f, "UnwrapAbuse"),
            BullshitType::DynTraitAbuse => write!(f, "DynTraitAbuse"),
            BullshitType::CloneAbuse => write!(f, "CloneAbuse"),
            BullshitType::MutexAbuse => write!(f, "MutexAbuse"),
            BullshitType::MagicNumber => write!(f, "MagicNumber"),
            BullshitType::HardcodedThreshold => write!(f, "HardcodedThreshold"),
        }
    }
}

/// Bullshit alert with confidence and suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BullshitAlert {
    pub issue_type: BullshitType,
    pub confidence: f32,
    pub location: (usize, usize), // (line, column)
    pub context_snippet: String,
    pub why_bs: String,
    pub sug: String,
    pub severity: f32,
}

/// Detection configuration
#[derive(Debug, Clone)]
pub struct DetectConfig {
    pub confidence_threshold: f32,
    pub max_snippet_length: usize,
    pub enable_regex_fallback: bool,
}

impl Default for DetectConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.618, // Golden ratio inverse
            max_snippet_length: 500,
            enable_regex_fallback: true,
        }
    }
}

/// Scan code for bullshit patterns using regex
pub fn scan_code(code: &str, config: &DetectConfig) -> anyhow::Result<Vec<BullshitAlert>> {
    use regex::Regex;
    use std::collections::HashMap;

    let mut alerts = Vec::new();
    let mut patterns = HashMap::new();

    // Pattern definitions
    patterns.insert(r"Arc<RwLock<.*>>", BullshitType::OverEngineering);
    patterns.insert(r"Mutex<HashMap<.*>>", BullshitType::OverEngineering);
    patterns.insert(r"std::thread::sleep", BullshitType::SleepAbuse);
    patterns.insert(r"tokio::time::sleep", BullshitType::SleepAbuse);
    patterns.insert(r"\.unwrap\(\)", BullshitType::UnwrapAbuse);
    patterns.insert(r"\.clone\(\)", BullshitType::CloneAbuse);
    
    // Magic number patterns
    patterns.insert(r"if\s+.*\s*[<>=]+\s*0\.[3-9][0-9]*", BullshitType::MagicNumber);
    patterns.insert(r"Duration::from_secs\(\d{2,}\)", BullshitType::HardcodedThreshold);

    for (pattern, bs_type) in patterns {
        let regex = Regex::new(pattern)?;
        for mat in regex.find_iter(code) {
            let confidence = match bs_type {
                BullshitType::OverEngineering => 0.8,
                BullshitType::SleepAbuse => 0.75,
                BullshitType::MagicNumber => 0.9,
                BullshitType::HardcodedThreshold => 0.85,
                _ => 0.7,
            };

            if confidence >= config.confidence_threshold {
                alerts.push(BullshitAlert {
                    issue_type: bs_type.clone(),
                    confidence,
                    location: find_line_column(code, mat.start()),
                    context_snippet: extract_snippet(code, mat.start(), mat.end(), config.max_snippet_length),
                    why_bs: format!("Pattern match: {}", pattern),
                    sug: generate_suggestion(&bs_type),
                    severity: confidence,
                });
            }
        }
    }

    Ok(alerts)
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
fn extract_snippet(code: &str, start: usize, end: usize, max_length: usize) -> String {
    let snippet_start = start.saturating_sub(50);
    let snippet_end = (end + 50).min(code.len());
    let snippet = &code[snippet_start..snippet_end];
    
    if snippet.len() > max_length {
        format!("{}...", &snippet[..max_length])
    } else {
        snippet.to_string()
    }
}

/// Generate suggestions based on bullshit type
fn generate_suggestion(bs_type: &BullshitType) -> String {
    match bs_type {
        BullshitType::OverEngineering => "Simplify with owned types or references".to_string(),
        BullshitType::ArcAbuse => "Use Arc only for shared ownership across threads".to_string(),
        BullshitType::RwLockAbuse => "Consider if read/write locks are necessary".to_string(),
        BullshitType::SleepAbuse => "Use async delays or remove blocking sleeps".to_string(),
        BullshitType::UnwrapAbuse => "Handle errors properly with ? or match".to_string(),
        BullshitType::DynTraitAbuse => "Use concrete types when possible".to_string(),
        BullshitType::CloneAbuse => "Avoid unnecessary cloning of data".to_string(),
        BullshitType::MutexAbuse => "Consider if mutex is needed for this use case".to_string(),
        BullshitType::FakeComplexity => "Break down into smaller, focused functions".to_string(),
        BullshitType::CargoCult => "Import only what you actually use".to_string(),
        BullshitType::MagicNumber => "Extract to constant or config".to_string(),
        BullshitType::HardcodedThreshold => "Move to configuration struct".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_number_detection() {
        let code = r#"
            if confidence > 0.85 {
                do_something();
            }
        "#;

        let config = DetectConfig::default();
        let alerts = scan_code(code, &config).unwrap();

        assert!(!alerts.is_empty());
        assert_eq!(alerts[0].issue_type, BullshitType::MagicNumber);
    }

    #[test]
    fn test_unwrap_detection() {
        let code = r#"
            let value = some_fn().unwrap();
        "#;

        let config = DetectConfig::default();
        let alerts = scan_code(code, &config).unwrap();

        assert!(alerts.iter().any(|a| a.issue_type == BullshitType::UnwrapAbuse));
    }

    #[test]
    fn test_sleep_abuse_detection() {
        let code = r#"
            std::thread::sleep(Duration::from_secs(5));
        "#;

        let config = DetectConfig::default();
        let alerts = scan_code(code, &config).unwrap();

        assert!(alerts.iter().any(|a| a.issue_type == BullshitType::SleepAbuse));
    }
}
