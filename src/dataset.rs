// Copyright (c) 2025 Jason Van Pham (ruffian-l on GitHub) @ The Niodoo Collaborative
// Licensed under the MIT License - See LICENSE file for details
// Attribution required for all derivative works

use crate::{BullshitType, DetectConfig};
use anyhow::{anyhow, Result};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

/// Configuration for dataset generation
#[derive(Debug, Clone)]
pub struct DatasetConfig {
    pub total_snippets: usize,
    pub bs_ratio: f64, // 0.0 to 1.0 (percentage of bullshit examples)
    pub augmentation_factor: usize,
    pub max_snippet_length: usize,
    pub min_snippet_length: usize,
    pub output_file: String,
}

impl Default for DatasetConfig {
    fn default() -> Self {
        Self {
            total_snippets: 24000,
            bs_ratio: 0.7, // 70% bullshit examples
            augmentation_factor: 5,
            max_snippet_length: 200,
            min_snippet_length: 50,
            output_file: "synthetic_bs_dataset.json".to_string(),
        }
    }
}

/// Dataset entry with code snippet and bullshit label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetEntry {
    pub code_snippet: String,
    pub is_bullshit: bool,
    pub bullshit_types: Vec<BullshitType>,
    pub confidence_score: f32,
    pub augmentations: Vec<String>,
}

/// Dataset generator for training bullshit detector
pub struct DatasetGenerator {
    config: DatasetConfig,
    rng: ChaCha8Rng,
}

impl DatasetGenerator {
    pub fn new(config: DatasetConfig) -> Self {
        Self {
            config,
            rng: ChaCha8Rng::from_seed(rand::thread_rng().gen::<[u8; 32]>()),
        }
    }

    /// Generate synthetic dataset with bullshit and clean code examples
    pub fn generate_dataset(&mut self) -> Result<Vec<DatasetEntry>> {
        let mut dataset = Vec::new();

        // Generate base snippets
        let bs_count = (self.config.total_snippets as f64 * self.config.bs_ratio) as usize;
        let clean_count = self.config.total_snippets - bs_count;

        // Generate bullshit examples
        for _ in 0..bs_count {
            let entry = self.generate_bullshit_example()?;
            dataset.push(entry);
        }

        // Generate clean examples
        for _ in 0..clean_count {
            let entry = self.generate_clean_example()?;
            dataset.push(entry);
        }

        // Apply augmentations
        dataset = self.augment_dataset(dataset)?;

        Ok(dataset)
    }

    /// Generate a single bullshit code example
    fn generate_bullshit_example(&mut self) -> Result<DatasetEntry> {
        let bullshit_type = self.select_random_bullshit_type();
        let code_snippet = self.generate_bullshit_code(&bullshit_type)?;

        Ok(DatasetEntry {
            code_snippet,
            is_bullshit: true,
            bullshit_types: vec![bullshit_type],
            confidence_score: 0.8, // High confidence for generated bullshit
            augmentations: Vec::new(),
        })
    }

    /// Generate a single clean code example
    fn generate_clean_example(&mut self) -> Result<DatasetEntry> {
        let code_snippet = self.generate_clean_code()?;

        Ok(DatasetEntry {
            code_snippet,
            is_bullshit: false,
            bullshit_types: Vec::new(),
            confidence_score: 0.2, // Low confidence for clean code
            augmentations: Vec::new(),
        })
    }

    /// Select random bullshit type with weighted probability
    fn select_random_bullshit_type(&mut self) -> BullshitType {
        let types = vec![
            (BullshitType::OverEngineering, 0.25),
            (BullshitType::ArcAbuse, 0.20),
            (BullshitType::RwLockAbuse, 0.15),
            (BullshitType::SleepAbuse, 0.10),
            (BullshitType::UnwrapAbuse, 0.10),
            (BullshitType::DynTraitAbuse, 0.10),
            (BullshitType::FakeComplexity, 0.10),
        ];

        let mut cumulative = 0.0;
        let random_value = self.rng.gen::<f32>();

        for (bs_type, weight) in types {
            cumulative += weight;
            if random_value <= cumulative {
                return bs_type;
            }
        }

        BullshitType::OverEngineering // Fallback
    }

    /// Generate bullshit code based on type
    fn generate_bullshit_code(&mut self, bs_type: &BullshitType) -> Result<String> {
        match bs_type {
            BullshitType::OverEngineering => self.generate_over_engineered_code(),
            BullshitType::ArcAbuse => self.generate_arc_abuse_code(),
            BullshitType::RwLockAbuse => self.generate_rwlock_abuse_code(),
            BullshitType::SleepAbuse => self.generate_sleep_abuse_code(),
            BullshitType::UnwrapAbuse => self.generate_unwrap_abuse_code(),
            BullshitType::DynTraitAbuse => self.generate_dyn_trait_abuse_code(),
            BullshitType::FakeComplexity => self.generate_fake_complexity_code(),
            BullshitType::CargoCult => self.generate_cargo_cult_code(),
            BullshitType::CloneAbuse => self.generate_clone_abuse_code(),
            BullshitType::MutexAbuse => self.generate_mutex_abuse_code(),
        }
    }

    /// Generate over-engineered struct/function
    fn generate_over_engineered_code(&mut self) -> Result<String> {
        let struct_name = self.generate_random_identifier();
        let field_count = self.rng.gen_range(5..10);

        let mut fields = Vec::new();
        for i in 0..field_count {
            let field_name = format!("field_{}", i);
            let field_type = match self.rng.gen_range(0..4) {
                0 => format!("Arc<RwLock<{}>>", self.generate_random_type()),
                1 => format!("Mutex<HashMap<String, {}>>", self.generate_random_type()),
                2 => format!("Box<dyn {}>", self.generate_trait_name()),
                3 => format!("Option<{}>", self.generate_random_type()),
                _ => self.generate_random_type(),
            };
            fields.push(format!("    {}: {}", field_name, field_type));
        }

        let code = format!(
            "struct {} {{\n{}\n}}\n\nimpl {} {{\n    fn over_engineered_method(&self) -> Result<(), Box<dyn std::error::Error>> {{\n        // Complex logic that could be simplified\n        Ok(())\n    }}\n}}",
            struct_name,
            fields.join(",\n"),
            struct_name
        );

        Ok(code)
    }

    /// Generate Arc abuse example
    fn generate_arc_abuse_code(&mut self) -> Result<String> {
        let type_name = self.generate_random_type();
        let function_name = self.generate_random_identifier();

        Ok(format!(
            "fn {}(data: Vec<{}>) -> Arc<{}> {{\n    Arc::new(data.into_iter().collect())\n}}",
            function_name, type_name, type_name
        ))
    }

    /// Generate RwLock abuse example
    fn generate_rwlock_abuse_code(&mut self) -> Result<String> {
        let type_name = self.generate_random_type();

        Ok(format!(
            "struct DataWrapper {{\n    data: RwLock<Vec<{}>>,\n}}\n\nimpl DataWrapper {{\n    fn get_data(&self) -> Vec<{}> {{\n        self.data.read().unwrap().clone()\n    }}\n}}",
            type_name, type_name
        ))
    }

    /// Generate sleep abuse example
    fn generate_sleep_abuse_code(&mut self) -> Result<String> {
        Ok("async fn delayed_operation() {\n    tokio::time::sleep(Duration::from_secs(1)).await;\n    tracing::info!(\"Operation completed after unnecessary delay\");\n}".to_string())
    }

    /// Generate unwrap abuse example
    fn generate_unwrap_abuse_code(&mut self) -> Result<String> {
        Ok("fn parse_input(input: &str) -> i32 {\n    input.parse().unwrap()\n}".to_string())
    }

    /// Generate dyn trait abuse example
    fn generate_dyn_trait_abuse_code(&mut self) -> Result<String> {
        let trait_name = self.generate_trait_name();

        Ok(format!(
            "fn process_items(items: Vec<Box<dyn {}>>) {{\n    for item in items {{\n        // Complex trait object usage\n    }}\n}}",
            trait_name
        ))
    }

    /// Generate fake complexity example
    fn generate_fake_complexity_code(&mut self) -> Result<String> {
        let mut nested_blocks = String::new();
        let depth = self.rng.gen_range(3..6);

        for i in 0..depth {
            nested_blocks.push_str(&format!(
                "{}{}if condition_{} {{\n{}    if sub_condition_{} {{\n{}        // Deep nesting\n{}    }}\n{}}}\n",
                "    ".repeat(i),
                "    ".repeat(i),
                i,
                "    ".repeat(i + 1),
                i,
                "    ".repeat(i + 2),
                "    ".repeat(i + 1),
                "    ".repeat(i)
            ));
        }

        Ok(format!("fn complex_function() {{\n{}}}\n", nested_blocks))
    }

    /// Generate cargo cult example
    fn generate_cargo_cult_code(&mut self) -> Result<String> {
        Ok("use std::collections::HashMap;\nuse std::sync::{Arc, Mutex};\n\nfn main() {\n    // Unused imports above\n    tracing::info!(\"Hello, world!\");\n}".to_string())
    }

    /// Generate clone abuse example
    fn generate_clone_abuse_code(&mut self) -> Result<String> {
        Ok("fn process_data(data: &Vec<i32>) {\n    let cloned_data = data.clone();\n    // Process cloned_data instead of borrowing\n    for item in &cloned_data {\n        tracing::info!(\"{}\", item);\n    }\n}".to_string())
    }

    /// Generate mutex abuse example
    fn generate_mutex_abuse_code(&mut self) -> Result<String> {
        Ok("fn simple_counter() -> Mutex<i32> {\n    Mutex::new(0)\n}\n\nfn increment_counter(counter: &Mutex<i32>) {\n    let mut count = counter.lock().unwrap();\n    *count += 1;\n}".to_string())
    }

    /// Generate clean code example
    fn generate_clean_code(&mut self) -> Result<String> {
        let code_type = self.rng.gen_range(0..5);

        match code_type {
            0 => Ok(format!("fn add(x: i32, y: i32) -> i32 {{\n    x + y\n}}")),
            1 => Ok(format!("struct Point {{\n    x: f64,\n    y: f64,\n}}")),
            2 => Ok("fn fibonacci(n: u32) -> u64 {\n    match n {\n        0 => 0,\n        1 => 1,\n        _ => fibonacci(n - 1) + fibonacci(n - 2),\n    }\n}".to_string()),
            3 => Ok("fn greet(name: &str) {\n    tracing::info!(\"Hello, {}!\", name);\n}".to_string()),
            _ => Ok("fn is_even(n: i32) -> bool {\n    n % 2 == 0\n}".to_string()),
        }
    }

    /// Generate random identifier
    fn generate_random_identifier(&mut self) -> String {
        let adjectives = [
            "quick", "lazy", "smart", "dumb", "fast", "slow", "big", "small",
        ];
        let nouns = ["dog", "cat", "bird", "fish", "tree", "car", "house", "book"];

        let adj = adjectives[self.rng.gen_range(0..adjectives.len())];
        let noun = nouns[self.rng.gen_range(0..nouns.len())];

        format!("{}{}", adj, noun).to_lowercase()
    }

    /// Generate random type
    fn generate_random_type(&mut self) -> String {
        let types = [
            "i32",
            "String",
            "Vec<i32>",
            "HashMap<String, i32>",
            "Option<String>",
        ];
        types[self.rng.gen_range(0..types.len())].to_string()
    }

    /// Generate trait name
    fn generate_trait_name(&mut self) -> String {
        let traits = ["Clone", "Debug", "Display", "Iterator", "Future"];
        traits[self.rng.gen_range(0..traits.len())].to_string()
    }

    /// Augment dataset with variations
    fn augment_dataset(&mut self, mut dataset: Vec<DatasetEntry>) -> Result<Vec<DatasetEntry>> {
        let mut augmented = Vec::new();

        for entry in dataset {
            // Add original
            augmented.push(entry.clone());

            // Generate augmentations
            for _ in 0..self.config.augmentation_factor {
                let mut augmented_entry = entry.clone();

                // Apply random augmentations
                self.apply_augmentations(&mut augmented_entry)?;

                augmented.push(augmented_entry);
            }
        }

        Ok(augmented)
    }

    /// Apply random code augmentations
    fn apply_augmentations(&mut self, entry: &mut DatasetEntry) -> Result<()> {
        let mut code = entry.code_snippet.clone();

        // Randomly apply different augmentation types
        let augmentation_type = self.rng.gen_range(0..5);

        match augmentation_type {
            0 => {
                // Add comments
                code = self.add_random_comments(code);
            }
            1 => {
                // Modify variable names
                code = self.modify_variable_names(code);
            }
            2 => {
                // Add unnecessary complexity
                code = self.add_unnecessary_complexity(code);
            }
            3 => {
                // Change formatting
                code = self.change_formatting(code);
            }
            _ => {
                // Add type annotations
                code = self.add_type_annotations(code);
            }
        }

        entry.code_snippet = code;
        entry
            .augmentations
            .push(format!("augmentation_{}", augmentation_type));

        Ok(())
    }

    /// Add random comments to code
    fn add_random_comments(&mut self, code: String) -> String {
        let comments = [
            "// TODO: Implement this properly",
            "// FIXME: This is a hack",
            "// NOTE: This could be optimized",
            "// WARNING: Don't touch this",
        ];

        let lines: Vec<&str> = code.lines().collect();
        let mut result = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            result.push(line.to_string());

            // Add comment with 30% probability
            if self.rng.random_bool(0.3) && i < lines.len() - 1 {
                let comment = comments[self.rng.gen_range(0..comments.len())];
                result.push(format!("    {}", comment));
            }
        }

        result.join("\n")
    }

    /// Modify variable names in code
    fn modify_variable_names(&mut self, code: String) -> String {
        // Simple variable name replacement (could be more sophisticated)
        code.replace("data", "information")
            .replace("item", "element")
            .replace("count", "number")
    }

    /// Add unnecessary complexity
    fn add_unnecessary_complexity(&mut self, code: String) -> String {
        if code.contains("fn ") {
            code.replace("fn ", "fn _unnecessarily_complex_wrapper() {\n    ")
                .replace("{", "{\n    // Unnecessary wrapper\n    ")
                + "\n}"
        } else {
            code
        }
    }

    /// Change code formatting
    fn change_formatting(&mut self, code: String) -> String {
        // Add extra spaces and line breaks randomly
        let lines: Vec<&str> = code.lines().collect();
        let mut result = Vec::new();

        for line in lines {
            let mut modified_line = line.to_string();

            // Add extra spaces with 20% probability
            if self.rng.random_bool(0.2) {
                modified_line = modified_line.replace(" ", "  ");
            }

            // Add extra newlines with 10% probability
            if self.rng.random_bool(0.1) && !result.is_empty() {
                result.push(String::new());
            }

            result.push(modified_line);
        }

        result.join("\n")
    }

    /// Add type annotations
    fn add_type_annotations(&mut self, code: String) -> String {
        // Add explicit type annotations where possible
        code.replace("let mut ", "let mut _: ")
            .replace("let ", "let _: ")
    }

    /// Save dataset to JSON file
    pub fn save_dataset(&self, dataset: &[DatasetEntry]) -> Result<()> {
        let json_data = serde_json::to_string_pretty(dataset)
            .map_err(|e| anyhow!("Failed to serialize dataset: {}", e))?;

        let mut file = File::create(&self.config.output_file)
            .map_err(|e| anyhow!("Failed to create output file: {}", e))?;

        file.write_all(json_data.as_bytes())
            .map_err(|e| anyhow!("Failed to write dataset: {}", e))?;

        tracing::info!(
            "Generated dataset with {} entries saved to {}",
            dataset.len(),
            self.config.output_file
        );

        Ok(())
    }

    /// Load dataset from JSON file
    pub fn load_dataset(&self, file_path: &str) -> Result<Vec<DatasetEntry>> {
        let file_content = std::fs::read_to_string(file_path)
            .map_err(|e| anyhow!("Failed to read dataset file: {}", e))?;

        let dataset: Vec<DatasetEntry> = serde_json::from_str(&file_content)
            .map_err(|e| anyhow!("Failed to deserialize dataset: {}", e))?;

        Ok(dataset)
    }

    /// Generate statistics about the dataset
    pub fn generate_statistics(&self, dataset: &[DatasetEntry]) -> DatasetStatistics {
        let total = dataset.len();
        let bullshit_count = dataset.iter().filter(|e| e.is_bullshit).count();
        let clean_count = total - bullshit_count;

        let mut type_counts = HashMap::new();
        for entry in dataset {
            if entry.is_bullshit {
                for bs_type in &entry.bullshit_types {
                    *type_counts.entry(format!("{:?}", bs_type)).or_insert(0) += 1;
                }
            }
        }

        DatasetStatistics {
            total_entries: total,
            bullshit_entries: bullshit_count,
            clean_entries: clean_count,
            bullshit_ratio: bullshit_count as f32 / total as f32,
            type_distribution: type_counts,
        }
    }
}

/// Dataset statistics
#[derive(Debug, Clone)]
pub struct DatasetStatistics {
    pub total_entries: usize,
    pub bullshit_entries: usize,
    pub clean_entries: usize,
    pub bullshit_ratio: f32,
    pub type_distribution: HashMap<String, usize>,
}

impl DatasetStatistics {
    pub fn print(&self) {
        tracing::info!("Dataset Statistics:");
        tracing::info!("  Total entries: {}", self.total_entries);
        tracing::info!(
            "  Bullshit entries: {} ({:.1}%)",
            self.bullshit_entries,
            self.bullshit_ratio * 100.0
        );
        tracing::info!(
            "  Clean entries: {} ({:.1}%)",
            self.clean_entries,
            100.0 - self.bullshit_ratio * 100.0
        );
        tracing::info!("  Bullshit type distribution:");

        for (bs_type, count) in &self.type_distribution {
            tracing::info!("    {}: {}", bs_type, count);
        }
    }
}

/// Command-line interface for dataset generation
pub fn run_dataset_generation(config: DatasetConfig) -> Result<()> {
    let mut generator = DatasetGenerator::new(config);

    tracing::info!("Generating synthetic bullshit detection dataset...");
    let dataset = generator.generate_dataset()?;

    let stats = generator.generate_statistics(&dataset);
    stats.print();

    generator.save_dataset(&dataset)?;

    tracing::info!("Dataset generation complete!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_generation() {
        let config = DatasetConfig {
            total_snippets: 100,
            bs_ratio: 0.7,
            augmentation_factor: 2,
            ..Default::default()
        };

        let mut generator = DatasetGenerator::new(config);
        let dataset = generator.generate_dataset().unwrap();

        assert_eq!(dataset.len(), 300); // 100 * (1 + 2 augmentations)

        let stats = generator.generate_statistics(&dataset);
        assert_eq!(stats.total_entries, 300);
        assert!(stats.bullshit_ratio > 0.6 && stats.bullshit_ratio < 0.8);
    }

    #[test]
    fn test_bullshit_generation() {
        let config = DatasetConfig::default();
        let mut generator = DatasetGenerator::new(config);

        let bs_entry = generator.generate_bullshit_example().unwrap();
        assert!(bs_entry.is_bullshit);
        assert!(!bs_entry.bullshit_types.is_empty());
        assert!(bs_entry.confidence_score > 0.7);

        let clean_entry = generator.generate_clean_example().unwrap();
        assert!(!clean_entry.is_bullshit);
        assert!(clean_entry.confidence_score < 0.3);
    }

    #[test]
    fn test_augmentation() {
        let config = DatasetConfig {
            total_snippets: 10,
            augmentation_factor: 3,
            ..Default::default()
        };

        let mut generator = DatasetGenerator::new(config);
        let dataset = generator.generate_dataset().unwrap();

        // Should have original + augmentations
        assert_eq!(dataset.len(), 40); // 10 * 4 (1 original + 3 augmentations)

        // Check that augmentations are different
        let original_snippets: std::collections::HashSet<String> = dataset
            .iter()
            .take(10)
            .map(|e| e.code_snippet.clone())
            .collect();

        assert!(original_snippets.len() > 1); // Should have variety
    }
}
