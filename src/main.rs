// Copyright (c) 2025 Jason Van Pham (ruffian-l on GitHub) @ The Niodoo Collaborative
// Licensed under the MIT License - See LICENSE file for details
// Attribution required for all derivative works

use anyhow::Result;
use bullshitdetector::{scan_code, DetectConfig, BullshitAlert};
use clap::{Parser, Subcommand};
use glob::glob;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "bullshitdetector")]
#[command(about = "Fast detector for magic numbers and code smells", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan code for magic numbers and hardcoded values
    ScanMagic {
        /// Directory or file to scan
        path: PathBuf,
        
        /// Output format (text or json)
        #[arg(short, long, default_value = "text")]
        output: String,
        
        /// Confidence threshold (0.0-1.0)
        #[arg(short, long, default_value = "0.618")]
        threshold: f32,
    },
    
    /// Scan code for all code smells
    Scan {
        /// Directory or file to scan
        path: PathBuf,
        
        /// Output format (text or json)
        #[arg(short, long, default_value = "text")]
        output: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::ScanMagic { path, output, threshold } => {
            scan_magic_numbers(path, &output, threshold)?;
        }
        Commands::Scan { path, output } => {
            scan_all(path, &output)?;
        }
    }

    Ok(())
}

fn scan_magic_numbers(path: PathBuf, output_format: &str, threshold: f32) -> Result<()> {
    let mut config = DetectConfig::default();
    config.confidence_threshold = threshold;

    let files = find_rust_files(&path)?;
    let mut total_alerts = Vec::new();

    for file_path in files {
        let code = fs::read_to_string(&file_path)?;
        let mut alerts = scan_code(&code, &config)?;
        
        // Filter for magic numbers only
        alerts.retain(|a| matches!(a.issue_type, bullshitdetector::BullshitType::MagicNumber | bullshitdetector::BullshitType::HardcodedThreshold));
        
        for alert in &mut alerts {
            alert.context_snippet = format!("{}:{}", file_path.display(), alert.context_snippet);
        }
        
        total_alerts.extend(alerts);
    }

    output_results(&total_alerts, output_format)?;

    Ok(())
}

fn scan_all(path: PathBuf, output_format: &str) -> Result<()> {
    let config = DetectConfig::default();
    let files = find_rust_files(&path)?;
    let mut total_alerts = Vec::new();

    for file_path in files {
        let code = fs::read_to_string(&file_path)?;
        let mut alerts = scan_code(&code, &config)?;
        
        for alert in &mut alerts {
            alert.context_snippet = format!("{}:{}", file_path.display(), alert.context_snippet);
        }
        
        total_alerts.extend(alerts);
    }

    output_results(&total_alerts, output_format)?;

    Ok(())
}

fn find_rust_files(path: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if path.is_file() {
        files.push(path.clone());
    } else if path.is_dir() {
        let pattern = format!("{}/**/*.rs", path.display());
        for entry in glob(&pattern)? {
            if let Ok(file_path) = entry {
                // Skip test files and target directory
                let path_str = file_path.to_string_lossy();
                if !path_str.contains("/target/") && !path_str.contains("/tests/") {
                    files.push(file_path);
                }
            }
        }
    }

    Ok(files)
}

fn output_results(alerts: &[BullshitAlert], format: &str) -> Result<()> {
    if format == "json" {
        let json = serde_json::to_string_pretty(alerts)?;
        println!("{}", json);
    } else {
        // Text output
        println!("\n🚨 Bullshitdetector Results\n");
        println!("Found {} issues:\n", alerts.len());

        // Group by severity
        let critical: Vec<_> = alerts.iter().filter(|a| a.severity >= 0.9).collect();
        let high: Vec<_> = alerts.iter().filter(|a| a.severity >= 0.75 && a.severity < 0.9).collect();
        let medium: Vec<_> = alerts.iter().filter(|a| a.severity < 0.75).collect();

        if !critical.is_empty() {
            println!("🔴 CRITICAL ({} issues):", critical.len());
            for alert in critical {
                print_alert(alert);
            }
            println!();
        }

        if !high.is_empty() {
            println!("🟠 HIGH ({} issues):", high.len());
            for alert in high {
                print_alert(alert);
            }
            println!();
        }

        if !medium.is_empty() {
            println!("🟡 MEDIUM ({} issues):", medium.len());
            for alert in medium {
                print_alert(alert);
            }
        }

        println!("\n✅ Scan complete!");
    }

    Ok(())
}

fn print_alert(alert: &BullshitAlert) {
    println!("  {} at line {}", alert.issue_type, alert.location.0);
    println!("    {}", alert.context_snippet.lines().next().unwrap_or(""));
    println!("    Why: {}", alert.why_bs);
    println!("    Fix: {}", alert.sug);
    println!("    Confidence: {:.0}%", alert.confidence * 100.0);
    println!();
}
