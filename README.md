# si (see)

[![CI](https://github.com/YOUR_USERNAME/si/workflows/CI/badge.svg)](https://github.com/YOUR_USERNAME/si/actions)
[![Coverage](https://codecov.io/gh/YOUR_USERNAME/si/branch/main/graph/badge.svg)](https://codecov.io/gh/YOUR_USERNAME/si)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

A tool for generating AI images that runs entirely on your local device.

## Features

- Generate images using AI models
- Download and run models locally from Hugging Face
- No cloud dependencies - everything runs on your machine

## Getting Started

### Installation

```bash
git clone https://github.com/YOUR_USERNAME/si.git
cd si
cargo build --release
```

### Usage

```bash
# Show help
./target/release/si --help

# List available models
./target/release/si model list

# Download a model
./target/release/si model download openai/clip-vit-base-patch32

# Generate an image
./target/release/si image generate "A beautiful sunset" --model my-model --input input.jpg --output output.png
```

## Development

### Prerequisites

- Rust 1.70 or later
- Cargo

### Building

```bash
cargo build
```

### Testing

```bash
# Run all tests (68 tests total)
cargo test

# Run test verification script
./scripts/test.sh

# Run local CI pipeline
./scripts/ci-local.sh

# Run with coverage
cargo install cargo-tarpaulin
cargo tarpaulin --all-features --out Html

# Run development checks
just dev  # or manually:
cargo fmt --check
cargo clippy -- -D warnings
```

### Project Structure

- `src/lib.rs` - Library crate exposing core functionality
- `src/main.rs` - CLI binary entry point
- `src/models.rs` - Model management and Hugging Face integration
- `tests/` - Integration tests (28 tests)
- `scripts/` - Development and testing scripts
- `.github/workflows/ci.yml` - CI/CD pipeline
- `TESTING.md` - Comprehensive testing documentation

### Test Coverage

The project maintains ~80% test coverage with:

- **68 total tests** across unit and integration test suites
  - **40 unit tests** for core data structures and business logic
  - **28 integration tests** for CLI functionality and error handling
- **Comprehensive error path testing** for edge cases and data validation
- **Automated CI/CD pipeline** with GitHub Actions

For detailed testing information, see [TESTING.md](./TESTING.md).

## About

`si` allows you to generate images using state-of-the-art AI models while keeping everything local and private. Models are downloaded from Hugging Face and executed directly on your device.
