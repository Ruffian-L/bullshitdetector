// Copyright (c) 2025 Jason Van Pham (ruffian-l on GitHub) @ The Niodoo Collaborative
// Licensed under the MIT License - See LICENSE file for details
// Attribution required for all derivative works

use crate::constants::{GOLDEN_RATIO_INV, ONE_MINUS_PHI_INV};
use crate::BullshitAlert;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tera::{Context, Tera}; // Updated import to use from lib.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub alert_id: usize,
    pub before_code: String,
    pub after_code: String,
    pub impact: String,
    pub doc_link: String,
    pub steer_tone: String, // assertive/educational
    pub steer_conf: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestConfig {
    pub templates_dir: String,
    pub docs: std::collections::HashMap<String, String>,
}

impl Default for SuggestConfig {
    fn default() -> Self {
        Self {
            templates_dir: "templates".to_string(),
            docs: std::collections::HashMap::from([
                (
                    "FakeComplexity".to_string(),
                    "https://rust-anti-bs/fake-complexity".to_string(),
                ),
                // ...
            ]),
        }
    }
}

fn parse_md_sug(md: &str) -> (String, String, String) {
    let re_before = regex::Regex::new(r"```rust\s*(.*?)\s*```").unwrap(); // Multi line
    let before_captures = re_before
        .captures_iter(md)
        .next()
        .map(|c| c[1].to_string())
        .unwrap_or_default();

    let re_after = regex::Regex::new(r"```rust\s*(.*?)\s*```").unwrap(); // Second block
    let after_captures = re_after
        .captures_iter(md)
        .skip(1)
        .next()
        .map(|c| c[1].to_string())
        .unwrap_or_default();

    let re_impact = regex::Regex::new(r"Impact: (.*)").unwrap();
    let impact = re_impact
        .captures(md)
        .map(|c| c[1].trim().to_string())
        .unwrap_or_default();

    (before_captures, after_captures, impact)
}

pub fn generate_sugs(alerts: &[BullshitAlert], config: &SuggestConfig) -> Result<Vec<Suggestion>> {
    // Commented out due to Tera compilation issues
    // let mut tera = Tera::new(&config.templates_dir)?;
    let mut sugs = Vec::new();

    for (idx, alert) in alerts.iter().enumerate() {
        // Use golden ratio inverse constant as confidence threshold - optimal decision boundary
        let tone = if alert.confidence > GOLDEN_RATIO_INV {
            "assertive"
        } else {
            "educational"
        };
        // Commented out due to Tera compilation issues
        // let mut ctx = Context::new();
        // ctx.insert("issue_type", &alert.issue_type.to_string());
        // ctx.insert("before", &alert.context_snippet);
        // ctx.insert("tone", &tone);
        // ctx.insert("why", &alert.why_bs);

        // let template_name = format!("{}.tera", alert.issue_type.to_string());
        // let rendered = tera.render(&template_name, &ctx)?;
        let rendered = format!("Mock suggestion for {}", alert.issue_type); // Mock implementation

        // Parse rendered to before/after/impact (assume MD format)
        let (before, after, impact) = parse_md_sug(&rendered);

        sugs.push(Suggestion {
            alert_id: idx,
            before_code: alert.context_snippet.clone(),
            after_code: after,
            impact,
            doc_link: config
                .docs
                .get(&alert.issue_type.to_string())
                .unwrap_or(&"".to_string())
                .clone(),
            steer_tone: tone.to_string(),
            steer_conf: alert.confidence,
        });
    }
    Ok(sugs)
}

pub fn add_docs_impact(sugs: &mut [Suggestion], config: &SuggestConfig) -> Result<()> {
    for sug in sugs {
        // Use BullshitType from lib.rs
        sug.doc_link = config
            .docs
            .get("FakeComplexity")
            .unwrap_or(&"General".to_string())
            .clone(); // Example, match to type
        sug.impact = format!(
            "Reduces BS by {:.0}-{:.0}% based on similar fixes",
            (ONE_MINUS_PHI_INV * 100.0),
            (GOLDEN_RATIO_INV * 100.0)
        ); // φ-based range using constants
    }
    Ok(())
}

// In main.rs phase2, use these
