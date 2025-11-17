# Publishing Bullshitdetector to crates.io

## Prerequisites

1. **Create crates.io account**: https://crates.io/
2. **Get API token**: https://crates.io/settings/tokens
3. **Login to cargo**:
   ```bash
   cargo login <your-token>
   ```

## Option 1: Publish Minimal Version (RECOMMENDED for first release)

Create a stripped-down version with only the working parts:

### Features to include:
- ✅ Regex-based magic number detection
- ✅ Pattern matching for code smells
- ✅ Golden ratio confidence scoring
- ✅ CLI tool

### Features to exclude (for now):
- ❌ BERT/ML models (fake implementations)
- ❌ Qwen RAG (mock responses)
- ❌ Tree-sitter (commented out)
- ❌ Heavy dependencies (candle, etc.)

### Minimal Cargo.toml:
```toml
[package]
name = "bullshitdetector"
version = "0.1.0"
edition = "2021"
authors = ["Jason Van Pham <your-email@example.com>"]
description = "Fast regex-based detector for magic numbers and code smells"
license = "MIT"
repository = "https://github.com/Ruffian-L/niodoo-tcs"
readme = "README.md"
keywords = ["linter", "code-quality", "static-analysis", "magic-numbers"]
categories = ["development-tools", "command-line-utilities"]

[[bin]]
name = "bullshitdetector"
path = "src/main.rs"

[dependencies]
# Core dependencies only
regex = "1.10"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
clap = { version = "4.0", features = ["derive"] }
glob = "0.3"

# Optional heavy features
sled = { version = "0.34", optional = true }
tokio = { version = "1.0", optional = true, features = ["full"] }
axum = { version = "0.7", optional = true }

[features]
default = []
full = ["sled", "tokio", "axum"]  # For users who want everything
api-server = ["tokio", "axum"]     # For API service
```

## Option 2: Publish Full Version (with workspace deps resolved)

If you want to publish everything, you need to:

1. **Replace all `workspace = true`** with actual versions
2. **Inline the `tcs-core` dependency** or publish it separately first
3. **Fix all the fake ML code** to either work or be feature-gated

## Publishing Commands

### 1. Test the package locally:
```bash
cd bullshitdetector
cargo package --allow-dirty  # Create tarball
cargo package --list          # See what will be published
```

### 2. Check for issues:
```bash
cargo publish --dry-run
```

### 3. Actually publish:
```bash
cargo publish
```

### 4. Verify it worked:
```bash
cargo search bullshitdetector
cargo install bullshitdetector
```

## Post-Publishing Checklist

- [ ] Add badges to README (crates.io, docs.rs, CI)
- [ ] Create GitHub release with same version tag
- [ ] Update documentation
- [ ] Announce on Reddit r/rust, Twitter, etc.
- [ ] Add to awesome-rust list

## Version Bumping (for future releases)

```bash
# Bug fixes
cargo set-version --bump patch  # 0.1.0 -> 0.1.1

# New features (backward compatible)
cargo set-version --bump minor  # 0.1.1 -> 0.2.0

# Breaking changes
cargo set-version --bump major  # 0.2.0 -> 1.0.0
```

## Common Issues & Fixes

### Issue: "workspace dependencies not allowed"
**Fix**: Replace all `workspace = true` with explicit versions

### Issue: "README.md not found"
**Fix**: Create README.md in the crate root

### Issue: "path dependencies not allowed"
**Fix**: Either:
- Publish dependencies first (e.g., `tcs-core`)
- Or make them optional features
- Or inline the code

### Issue: "binary has no examples"
**Fix**: Add example usage to README.md

## Recommended First Release Strategy

**Publish a minimal `v0.1.0` with ONLY:**
- `detect.rs` (regex patterns)
- `magic_numbers.rs` (working scanner)
- `constants.rs` (golden ratio)
- `lib.rs` (core types)
- `main.rs` (CLI)

**This gives you:**
- ✅ Working, testable crate
- ✅ Fast iteration (no heavy deps)
- ✅ Easy to maintain
- ✅ Users can install and use immediately

**Later releases can add:**
- `v0.2.0`: Add memory system
- `v0.3.0`: Add ML features (when fixed)
- `v1.0.0`: Full production-ready release

## Quick Start for Minimal Release

```bash
# 1. Create clean branch
git checkout -b crates-io-minimal

# 2. Strip down to essentials
# (I can help with this)

# 3. Test locally
cargo build --release
./target/release/bullshitdetector scan-magic src

# 4. Dry run
cargo publish --dry-run

# 5. Ship it!
cargo publish
```

Want me to create the minimal publishable version for you? I can strip out all the broken ML stuff and give you a clean v0.1.0 ready to publish.
