# 👻 Bullshitdetector

**Blazing-fast static analysis tool for detecting magic numbers, hardcoded values, and code smells in Rust (and other languages).**

[![Crates.io](https://img.shields.io/crates/v/bullshitdetector.svg)](https://crates.io/crates/bullshitdetector)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 🚀 Features

- ⚡ **Instant Scanning** - Pure regex-based detection, no ML overhead
- 🎯 **Magic Number Detection** - Finds hardcoded thresholds, timeouts, and constants
- 🔬 **Golden Ratio Math** - Confidence scoring using φ (phi) = 1.618
- 🌀 **Möbius Topology** - Advanced code structure analysis
- 📊 **PAD Valence Model** - Emotional analysis of code patterns
- 🔧 **Zero Config** - Works out of the box

## 📦 Installation

### As a CLI tool:
```bash
cargo install bullshitdetector
```

### As a library:
```toml
[dependencies]
bullshitdetector = "0.1"
```

## 🎯 Quick Start

### Scan for magic numbers:
```bash
bullshitdetector scan-magic ./src
```

### Scan for code smells:
```bash
bullshitdetector scan ./src --output report.json
```

### As a library:
```rust
use bullshitdetector::{DetectConfig, scan_code};

let code = r#"
    if confidence > 0.85 {  // ⚠️ Magic number!
        do_something();
    }
"#;

let config = DetectConfig::default();
let alerts = scan_code(code, &config)?;

for alert in alerts {
    println!("Found {} at line {}", alert.issue_type, alert.location.0);
}
```

## 🔍 What It Detects

| Pattern | Example | Severity |
|---------|---------|----------|
| **Magic Numbers** | `if x > 0.85` | 🔴 Critical |
| **Hardcoded Timeouts** | `Duration::from_secs(30)` | 🟠 High |
| **Arc/RwLock Abuse** | `Arc<RwLock<HashMap<...>>>` | 🟡 Medium |
| **Unwrap Abuse** | `.unwrap()` chains | 🟡 Medium |
| **Sleep Abuse** | `std::thread::sleep` in async | 🟡 Medium |

## 📊 Example Output

```
🔴 CRITICAL: Hardcoded Threshold
  File: src/pipeline/stages.rs:231
  Code: if knot > 0.4
  Suggestion: Move to PipelineConfig::knot_threshold
  
🟡 MEDIUM: Magic Number Assignment
  File: src/detection.rs:145
  Code: let base_top_p = 0.35;
  Suggestion: Extract to constant or config
```

## 🛠️ Configuration

Create a `.bullshitdetector.toml`:
```toml
[detect]
confidence_threshold = 0.618  # Golden ratio inverse
max_snippet_length = 500
enable_regex_fallback = true

[scan]
exclude_patterns = ["**/test/**", "**/tests/**"]
include_extensions = ["rs", "py", "js"]
```

## 🎓 How It Works

1. **Regex Pattern Matching** - Lightning-fast detection of common patterns
2. **AST Analysis** - (Optional) Tree-sitter based deep inspection
3. **Golden Ratio Scoring** - Confidence scoring using φ = 1.618034...
4. **Möbius Transforms** - Non-orientable topology for code structure
5. **PAD Valence** - Pleasure-Arousal-Dominance emotional model

## 🔧 Advanced Usage

### Custom Patterns:
```rust
use bullshitdetector::{DetectConfig, BullshitType};

let mut config = DetectConfig::default();
config.enable_tree_sitter = false;  // Regex only for speed
config.confidence_threshold = 0.7;   // Adjust sensitivity

let alerts = scan_code(code, &config)?;
```

### Shell Script Integration:
```bash
#!/bin/bash
# Pre-commit hook
./bullshitdetector scan-magic src | grep -q "CRITICAL" && exit 1
```

## 📚 API Documentation

Full documentation available at [docs.rs/bullshitdetector](https://docs.rs/bullshitdetector)

## 🤝 Contributing

Contributions welcome! The detector is designed to be extended with new patterns.

### Adding Custom Patterns:
1. Add to `BullshitType` enum in `src/lib.rs`
2. Implement detection in `src/detect.rs`
3. Add test cases
4. Submit PR!

## 📜 License

MIT License - Copyright (c) 2025 Jason Van Pham (ruffian-l on GitHub) @ The Niodoo Collaborative

## 🙏 Credits

- **Golden Ratio Math** - Based on φ = (1 + √5) / 2
- **Möbius Topology** - Non-orientable surface theory
- **PAD Model** - Pleasure-Arousal-Dominance emotional valence

## 🔗 Links

- [Repository](https://github.com/Ruffian-L/niodoo-tcs)
- [Issues](https://github.com/Ruffian-L/niodoo-tcs/issues)
- [Changelog](CHANGELOG.md)

---

**Who you gonna call?** 👻 **BULLSHITBUSTERS!**
