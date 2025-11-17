// Copyright (c) 2025 Jason Van Pham (ruffian-l on GitHub) @ The Niodoo Collaborative
// Licensed under the MIT License - See LICENSE file for details
// Attribution required for all derivative works

use anyhow::Result;
use candle_transformers::models::qwen::{Qwen2ForCausalLM, Config};
use candle_core::{Device, Tensor, safetensors::SafeTensors};
use candle_nn::VarBuilder;
use tokenizers::Tokenizer;

pub fn prepare_finetune_dataset(code_reviews: Vec<(String, String, f32)>) -> Result<Vec<(String, String)>> {
    let device = Device::Cpu;
    let model_path = "models/qwen2.5-7b-awq";
    let vb = VarBuilder::from_mmap(model_path.join("model.safetensors").to_str().unwrap(), DType::F16, &device)?;
    let config = Config::from_file(model_path.join("config.json"))?;
    let model = Qwen2ForCausalLM::load(&vb, &config)?;
    let tokenizer = Tokenizer::from_file(model_path.join("tokenizer.json"))?;

    let mut dataset = Vec::new();
    for (code, review, severity) in code_reviews {
        let prompt = format!("Review code: {}\nSeverity: {:.2}\nProvide detailed review:", code, severity);
        let enc = tokenizer.encode(&prompt, true)?;
        let input_ids = Tensor::new(enc.get_ids(), &device)?.unsqueeze(0)?;
        let target = format!("{} - Detailed analysis of code issues, suggestions, and coherence.", review);
        dataset.push((prompt, target));
    }
    Ok(dataset)
}

fn finetune_prompt() -> String {
    "You are a code review AI specializing in Rust. Review the following code snippet with severity (0-1, high BS), detailed issues, actionable suggestions, and coherence (0-1, high consensus). Use a snarky, gamified tone to engage developers. Example:\n\nCode: `struct OverEng { data: Arc<RwLock<Vec<i32>>> }`\nSeverity: 0.75\nIssues: Overkill Arc/RwLock for tiny state—level 3 BS unlocked!\nSuggestions: Strip to `Vec<i32>`, test concurrency—earn 50 XP!\nCoherence: 0.9\n\nNow review: [CODE_SNIPPET]\nSeverity: [0-1]\nIssues:\nSuggestions:\nCoherence:"
}

fn synthetic_reviews() -> Vec<(String, String, f32)> {
    let bs_heavy: Vec<_> = vec![
        ("struct OverEng { data: Arc<RwLock<Vec<i32>>> }".to_string(), "Overkill locks for simple state".to_string(), 0.8),
        ("async fn sleep_abuse() { tokio::time::sleep(Duration::from_secs(10)); }".to_string(), "Unnecessary sleep in async".to_string(), 0.75),
        // Add 300 more BS examples...
    ];
    let clean: Vec<_> = vec![
        ("fn simple_add(x: i32, y: i32) -> i32 { x + y }".to_string(), "Clean simple function".to_string(), 0.2),
        ("struct Point { x: f64, y: f64 }".to_string(), "Basic struct no BS".to_string(), 0.1),
        // Add 200 clean...
    ];
    let mut dataset = Vec::new();
    dataset.extend(bs_heavy.iter().cloned().cycle().take(900)); // 60% BS, augment 3x to 900
    dataset.extend(clean.iter().cloned().cycle().take(600)); // 40% clean, augment 3x to 600
    dataset // Total ~1500
}

// Usage in main.rs or train.rs
fn main() -> Result<()> {
    let dataset = vec![
        ("struct OverEng { data: Arc<RwLock<Vec<i32>>> }".to_string(), "Over-engineered lock fest".to_string(), 0.75),
        ("async fn sleep_abuse() { tokio::time::sleep(Duration::from_secs(10)); }".to_string(), "Sleepzilla strikes again".to_string(), 0.68),
    ];
    let finetune_data = prepare_finetune_dataset(dataset)?;
    for (prompt, target) in finetune_data {
        tracing::info!("Prompt: {}\nTarget: {}", prompt, target);
    }
    Ok(())
}
