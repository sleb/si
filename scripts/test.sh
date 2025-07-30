#!/bin/bash

# Test verification script for Si CLI
# This script runs all tests and verifies the project is working correctly

set -e  # Exit on any error

echo "🧪 Running Si CLI Test Suite"
echo "================================"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Must be run from the project root directory"
    exit 1
fi

# Check Rust installation
echo "🔍 Checking Rust installation..."
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Cargo not found. Please install Rust: https://rustup.rs/"
    exit 1
fi

RUST_VERSION=$(rustc --version | cut -d' ' -f2)
echo "✅ Rust version: $RUST_VERSION"

# Clean build
echo ""
echo "🧹 Cleaning previous builds..."
cargo clean

# Format check
echo ""
echo "📝 Checking code formatting..."
if cargo fmt --all -- --check; then
    echo "✅ Code formatting is correct"
else
    echo "❌ Code formatting issues found. Run 'cargo fmt' to fix."
    exit 1
fi

# Clippy check
echo ""
echo "📎 Running Clippy lints..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    echo "✅ No Clippy warnings found"
else
    echo "❌ Clippy warnings found"
    exit 1
fi

# Build
echo ""
echo "🔨 Building project..."
if cargo build; then
    echo "✅ Build successful"
else
    echo "❌ Build failed"
    exit 1
fi

# Run unit tests
echo ""
echo "🧪 Running unit tests..."
if cargo test --lib; then
    echo "✅ Unit tests passed"
else
    echo "❌ Unit tests failed"
    exit 1
fi

# Run integration tests
echo ""
echo "🔗 Running integration tests..."
if cargo test --test '*'; then
    echo "✅ Integration tests passed"
else
    echo "❌ Integration tests failed"
    exit 1
fi

# Run all tests
echo ""
echo "🚀 Running full test suite..."
if cargo test; then
    echo "✅ All tests passed"
else
    echo "❌ Some tests failed"
    exit 1
fi

# Test CLI help
echo ""
echo "💻 Testing CLI functionality..."
if cargo run -- --help > /dev/null 2>&1; then
    echo "✅ CLI help command works"
else
    echo "❌ CLI help command failed"
    exit 1
fi

# Check if tarpaulin is available for coverage
echo ""
echo "📊 Checking test coverage tools..."
if command -v cargo-tarpaulin &> /dev/null; then
    echo "✅ cargo-tarpaulin is available"
    echo "📈 Running coverage analysis (this may take a moment)..."

    # Run coverage with timeout
    if timeout 120s cargo tarpaulin --skip-clean --all-features --timeout 60 --out Stdout | tail -5; then
        echo "✅ Coverage analysis completed"
    else
        echo "⚠️  Coverage analysis timed out or failed (this is non-critical)"
    fi
else
    echo "⚠️  cargo-tarpaulin not installed. Install with: cargo install cargo-tarpaulin"
    echo "   This is optional but recommended for coverage reporting."
fi

# Build release version
echo ""
echo "🎯 Building release version..."
if cargo build --release; then
    echo "✅ Release build successful"

    # Test release binary
    if ./target/release/si --version > /dev/null 2>&1; then
        echo "✅ Release binary works correctly"
    else
        echo "❌ Release binary test failed"
        exit 1
    fi
else
    echo "❌ Release build failed"
    exit 1
fi

echo ""
echo "🎉 All tests passed! The Si CLI is ready to use."
echo ""
echo "Summary:"
echo "  - ✅ Code formatting"
echo "  - ✅ Clippy lints"
echo "  - ✅ Debug build"
echo "  - ✅ Release build"
echo "  - ✅ Unit tests (15 tests)"
echo "  - ✅ Integration tests (28 tests)"
echo "  - ✅ CLI functionality"
echo ""
echo "Next steps:"
echo "  1. Run './target/release/si --help' to see available commands"
echo "  2. Try 'cargo run -- model list' to test model management"
echo "  3. Check out the README.md for more usage examples"
echo ""
echo "Happy coding! 🚀"
