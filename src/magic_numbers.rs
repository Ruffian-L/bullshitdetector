// Copyright (c) 2025 Jason Van Pham (ruffian-l on GitHub) @ The Niodoo Collaborative
// Licensed under the MIT License - See LICENSE file for details
// Attribution required for all derivative works

//! Magic Number Detection - Specialized hardcoded value scanner
//!
//! This module extends the bullshit detector to specifically hunt for
//! hardcoded numeric values that should be in configuration files.
//!
//! Aligned with NO_MAGIC_NUMBERS_PHASE1_PLAN.md

use crate::{BullshitAlert, BullshitType};
use anyhow::Result;
use regex::Regex;
use std::collections::HashSet;
use std::env;
use std::path::Path;

/// Configuration for magic number detection
#[derive(Debug, Clone)]
pub struct MagicNumberConfig {
    /// Files/paths to exclude from scanning (e.g., "src/config.rs", "tests/")
    pub whitelist_paths: Vec<String>,

    /// Numeric values to whitelist (e.g., 0, 1, 2 for array indexing)
    pub whitelist_values: HashSet<String>,

    /// Minimum confidence threshold
    pub confidence_threshold: f32,

    /// Whether to check inside config.rs (should be false for Phase 1)
    pub scan_config_files: bool,
}

impl Default for MagicNumberConfig {
    fn default() -> Self {
        let mut whitelist_values = HashSet::new();

        // Common non-magic numbers
        whitelist_values.insert("0".to_string());
        whitelist_values.insert("1".to_string());
        whitelist_values.insert("2".to_string());
        whitelist_values.insert("100".to_string()); // Common percentage base
        whitelist_values.insert("1000".to_string()); // Common millisecond base
        whitelist_values.insert("1e-10".to_string()); // Common epsilon

        Self {
            whitelist_paths: vec![
                "src/config.rs".to_string(),
                "tests/".to_string(),
                "benches/".to_string(),
            ],
            whitelist_values,
            confidence_threshold: 0.7,
            scan_config_files: false,
        }
    }
}

impl MagicNumberConfig {
    /// Build configuration from environment overrides while preserving sane defaults.
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(paths) = env::var("NIODOO_MAGIC_WHITELIST_PATHS") {
            config.whitelist_paths = paths
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();
        }

        if let Ok(values) = env::var("NIODOO_MAGIC_WHITELIST_VALUES") {
            config.whitelist_values = values
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<HashSet<_>>();
        }

        if let Ok(conf) = env::var("NIODOO_MAGIC_CONFIDENCE_THRESHOLD") {
            if let Ok(value) = conf.parse::<f32>() {
                config.confidence_threshold = value.clamp(0.0, 1.0);
            }
        }

        if let Ok(scan_config) = env::var("NIODOO_MAGIC_SCAN_CONFIG_FILES") {
            config.scan_config_files = matches!(
                scan_config.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            );
        }

        config
    }
}

/// Scan Rust code for magic numbers
pub fn scan_for_magic_numbers(
    code: &str,
    file_path: &str,
    config: &MagicNumberConfig,
) -> Result<Vec<BullshitAlert>> {
    let mut alerts = Vec::new();

    // Check if file is whitelisted
    if is_path_whitelisted(file_path, config) {
        return Ok(alerts);
    }

    // Scan for hardcoded thresholds in conditionals
    alerts.extend(scan_conditional_thresholds(code)?);

    // Scan for hardcoded constants in assignments
    alerts.extend(scan_assignment_literals(code, config)?);

    // Scan for hardcoded values in function arguments
    alerts.extend(scan_function_arg_literals(code, config)?);

    // Filter by confidence
    alerts.retain(|a| a.confidence >= config.confidence_threshold);

    Ok(alerts)
}

/// Check if a path matches whitelist patterns
fn is_path_whitelisted(file_path: &str, config: &MagicNumberConfig) -> bool {
    for pattern in &config.whitelist_paths {
        if config.scan_config_files
            && (pattern.contains("config.rs") || pattern.contains("config/"))
        {
            continue;
        }

        if file_path.contains(pattern) {
            return true;
        }
    }
    false
}

/// Scan for hardcoded thresholds in if/while/match conditions
/// Examples: `if entropy > 0.4`, `while knot_strength < 0.6`
fn scan_conditional_thresholds(code: &str) -> Result<Vec<BullshitAlert>> {
    let mut alerts = Vec::new();

    // Pattern: if/while/match with comparison to numeric literal
    let patterns = vec![
        r"(?m)^\s*(if|while)\s+.*?\s*([<>=!]+)\s*(\d+\.?\d*(?:[eE][+-]?\d+)?)",
        r"(?m)^\s*}\s*else\s+if\s+.*?\s*([<>=!]+)\s*(\d+\.?\d*(?:[eE][+-]?\d+)?)",
        r"(?m)^\s*\|\s*\w+\s+if\s+.*?\s*([<>=!]+)\s*(\d+\.?\d*(?:[eE][+-]?\d+)?)", // Guard clauses
    ];

    for pattern_str in patterns {
        let regex = Regex::new(pattern_str)?;

        for cap in regex.captures_iter(code) {
            if let Some(value_match) = cap.get(cap.len() - 1) {
                let value = value_match.as_str();
                let pos = value_match.start();
                let (line, col) = find_line_column(code, pos);

                // Extract context snippet
                let line_start = code[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_end = code[pos..]
                    .find('\n')
                    .map(|i| pos + i)
                    .unwrap_or(code.len());
                let snippet = code[line_start..line_end].trim().to_string();

                // Calculate confidence based on context
                let confidence = calculate_threshold_confidence(&snippet, value);

                if confidence > 0.5 {
                    alerts.push(BullshitAlert {
                        issue_type: BullshitType::HardcodedThreshold,
                        confidence,
                        location: (line, col),
                        context_snippet: snippet.clone(),
                        why_bs: format!(
                            "Hardcoded threshold {} in conditional - should be in RuntimeConfig",
                            value
                        ),
                        sug: format!(
                            "Move {} to config and use self.config.{}_threshold",
                            value,
                            infer_config_name(&snippet)
                        ),
                        severity: confidence,
                    });
                }
            }
        }
    }

    Ok(alerts)
}

/// Scan for hardcoded values in variable assignments
/// Examples: `let threshold = 0.4;`, `major_radius = 5.0f32;`
fn scan_assignment_literals(code: &str, config: &MagicNumberConfig) -> Result<Vec<BullshitAlert>> {
    let mut alerts = Vec::new();

    // Pattern: let bindings or assignments with numeric literals
    let patterns = vec![
        r"(?m)^\s*let\s+(\w+)\s*=\s*(\d+\.?\d*(?:[eE][+-]?\d+)?(?:f32|f64)?)\s*;",
        r"(?m)^\s*(\w+)\s*=\s*(\d+\.?\d*(?:[eE][+-]?\d+)?(?:f32|f64)?)\s*;",
    ];

    for pattern_str in patterns {
        let regex = Regex::new(pattern_str)?;

        for cap in regex.captures_iter(code) {
            if let (Some(var_match), Some(value_match)) = (cap.get(1), cap.get(2)) {
                let var_name = var_match.as_str();
                let value = value_match.as_str();

                // Skip whitelisted values
                if config.whitelist_values.contains(value) {
                    continue;
                }

                let pos = value_match.start();
                let (line, col) = find_line_column(code, pos);

                // Extract context
                let line_start = code[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_end = code[pos..]
                    .find('\n')
                    .map(|i| pos + i)
                    .unwrap_or(code.len());
                let snippet = code[line_start..line_end].trim().to_string();

                // Check if this looks like a config value based on variable name
                let confidence = calculate_assignment_confidence(var_name, value, &snippet);

                if confidence > 0.6 {
                    alerts.push(BullshitAlert {
                        issue_type: BullshitType::MagicNumber,
                        confidence,
                        location: (line, col),
                        context_snippet: snippet.clone(),
                        why_bs: format!(
                            "Magic number {} assigned to {} - should be in config",
                            value, var_name
                        ),
                        sug: format!(
                            "Add {} to RuntimeConfig and initialize from config",
                            var_name
                        ),
                        severity: confidence,
                    });
                }
            }
        }
    }

    Ok(alerts)
}

/// Scan for hardcoded values passed as function arguments
/// Example: `calculate_topology(0.5, 0.8)` instead of `calculate_topology(config.threshold1, config.threshold2)`
fn scan_function_arg_literals(
    code: &str,
    config: &MagicNumberConfig,
) -> Result<Vec<BullshitAlert>> {
    let mut alerts = Vec::new();

    // Pattern: function calls with numeric literal arguments
    let regex =
        Regex::new(r"(\w+)\s*\(\s*([^)]*?(\d+\.?\d*(?:[eE][+-]?\d+)?(?:f32|f64)?)[^)]*?)\s*\)")?;

    for cap in regex.captures_iter(code) {
        if let (Some(func_match), Some(args_match)) = (cap.get(1), cap.get(2)) {
            let func_name = func_match.as_str();
            let args = args_match.as_str();

            // Count numeric literals in arguments
            let literal_regex = Regex::new(r"\d+\.?\d*(?:[eE][+-]?\d+)?(?:f32|f64)?")?;
            let literals: Vec<&str> = literal_regex
                .find_iter(args)
                .map(|m| m.as_str())
                .filter(|v| !config.whitelist_values.contains(*v))
                .collect();

            if literals.len() >= 2 {
                let pos = args_match.start();
                let (line, col) = find_line_column(code, pos);

                // Extract context
                let line_start = code[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_end = code[pos..]
                    .find('\n')
                    .map(|i| pos + i)
                    .unwrap_or(code.len());
                let snippet = code[line_start..line_end].trim().to_string();

                let confidence = 0.75; // High confidence for multiple literals in function args

                alerts.push(BullshitAlert {
                    issue_type: BullshitType::MagicNumber,
                    confidence,
                    location: (line, col),
                    context_snippet: snippet.clone(),
                    why_bs: format!(
                        "Function {} called with {} hardcoded numeric arguments",
                        func_name,
                        literals.len()
                    ),
                    sug: "Pass config values instead of hardcoded literals".to_string(),
                    severity: confidence,
                });
            }
        }
    }

    Ok(alerts)
}

/// Calculate confidence that a threshold value is problematic
fn calculate_threshold_confidence(snippet: &str, value: &str) -> f32 {
    let mut confidence = 0.5;

    // Keywords that suggest this is a behavioral threshold
    let threshold_keywords = [
        "threshold",
        "limit",
        "bound",
        "min",
        "max",
        "tolerance",
        "entropy",
        "yawn",
        "healing",
        "spectral",
        "knot",
        "persistence",
        "quality",
        "gate",
        "circuit",
        "similarity",
        "cosine",
    ];

    for keyword in &threshold_keywords {
        if snippet.to_lowercase().contains(keyword) {
            confidence += 0.15;
        }
    }

    // Values between 0 and 1 are often thresholds
    if let Ok(val) = value.parse::<f64>() {
        if val > 0.0 && val < 1.0 {
            confidence += 0.2;
        }
    }

    confidence.min(0.95)
}

/// Calculate confidence that an assignment is a magic number
fn calculate_assignment_confidence(var_name: &str, value: &str, snippet: &str) -> f32 {
    let mut confidence = 0.4;

    // Variable name patterns suggesting config values
    let config_patterns = [
        "threshold",
        "limit",
        "bound",
        "weight",
        "ratio",
        "factor",
        "radius",
        "width",
        "height",
        "size",
        "count",
        "max",
        "min",
        "alpha",
        "beta",
        "gamma",
        "epsilon",
        "delta",
    ];

    for pattern in &config_patterns {
        if var_name.to_lowercase().contains(pattern) {
            confidence += 0.25;
        }
    }

    // Type suffixes suggest this is a raw value, not a constant
    if value.ends_with("f32") || value.ends_with("f64") {
        confidence += 0.15;
    }

    // Check if inside a function (not at module level)
    if snippet.starts_with("    ") || snippet.starts_with("\t") {
        confidence += 0.15; // Inside function scope
    }

    confidence.min(0.95)
}

/// Infer a config field name from the code snippet
fn infer_config_name(snippet: &str) -> String {
    let snippet_lower = snippet.to_lowercase();

    if snippet_lower.contains("entropy") {
        "entropy".to_string()
    } else if snippet_lower.contains("yawn") {
        "yawn".to_string()
    } else if snippet_lower.contains("healing") {
        "healing".to_string()
    } else if snippet_lower.contains("knot") {
        "knot".to_string()
    } else if snippet_lower.contains("spectral") {
        "spectral".to_string()
    } else if snippet_lower.contains("persistence") {
        "persistence".to_string()
    } else if snippet_lower.contains("quality") {
        "quality".to_string()
    } else if snippet_lower.contains("similarity") {
        "similarity".to_string()
    } else {
        "behavioral".to_string()
    }
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

/// Generate a comprehensive report of magic numbers in a codebase
pub fn generate_magic_number_report(file_alerts: Vec<(String, Vec<BullshitAlert>)>) -> String {
    let mut report = String::new();

    report.push_str("# Magic Number Detection Report\n\n");
    report.push_str(&format!("Generated: {}\n\n", chrono::Utc::now()));

    let total_files: usize = file_alerts.len();
    let total_alerts: usize = file_alerts.iter().map(|(_, alerts)| alerts.len()).sum();

    report.push_str(&format!("## Summary\n"));
    report.push_str(&format!("- Files scanned: {}\n", total_files));
    report.push_str(&format!(
        "- Total magic numbers found: {}\n\n",
        total_alerts
    ));

    report.push_str("## Files with Magic Numbers\n\n");

    for (file_path, alerts) in file_alerts {
        if alerts.is_empty() {
            continue;
        }

        report.push_str(&format!("### {}\n\n", file_path));
        report.push_str(&format!("Found {} magic numbers:\n\n", alerts.len()));

        for (i, alert) in alerts.iter().enumerate() {
            report.push_str(&format!(
                "{}. **{}** at line {}:{}\n",
                i + 1,
                alert.issue_type,
                alert.location.0,
                alert.location.1
            ));
            report.push_str(&format!("   - **Why**: {}\n", alert.why_bs));
            report.push_str(&format!("   - **Suggestion**: {}\n", alert.sug));
            report.push_str(&format!("   - **Confidence**: {:.2}\n", alert.confidence));
            report.push_str(&format!("   - **Code**: `{}`\n\n", alert.context_snippet));
        }
    }

    report.push_str("\n## Next Steps\n\n");
    report.push_str("1. Review each magic number and determine if it should be in config\n");
    report.push_str("2. Add appropriate fields to `RuntimeConfig` in `src/config.rs`\n");
    report.push_str("3. Replace hardcoded values with config reads\n");
    report.push_str("4. Add tests to verify config-driven behavior\n");
    report.push_str("5. Update NO_MAGIC_NUMBERS_PHASE1_PLAN.md checklist\n");

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_conditional_threshold() {
        let code = r#"
        if entropy > 0.4 {
            do_something();
        }
        "#;

        let config = MagicNumberConfig::default();
        let alerts = scan_for_magic_numbers(code, "test.rs", &config).unwrap();

        assert!(!alerts.is_empty());
        assert_eq!(alerts[0].issue_type, BullshitType::HardcodedThreshold);
    }

    #[test]
    fn test_detects_assignment_literal() {
        let code = r#"
        let healing_threshold = 0.6;
        "#;

        let config = MagicNumberConfig::default();
        let alerts = scan_for_magic_numbers(code, "test.rs", &config).unwrap();

        assert!(!alerts.is_empty());
        assert_eq!(alerts[0].issue_type, BullshitType::MagicNumber);
    }

    #[test]
    fn test_whitelist_common_values() {
        let code = r#"
        let index = 0;
        let count = 1;
        "#;

        let config = MagicNumberConfig::default();
        let alerts = scan_for_magic_numbers(code, "test.rs", &config).unwrap();

        assert!(alerts.is_empty(), "Common values should be whitelisted");
    }

    #[test]
    fn test_whitelist_config_file() {
        let code = r#"
        let healing_threshold = 0.6;
        "#;

        let config = MagicNumberConfig::default();
        let alerts = scan_for_magic_numbers(code, "src/config.rs", &config).unwrap();

        assert!(alerts.is_empty(), "config.rs should be whitelisted");
    }
}
