#!/bin/bash

# Local CI verification script for Si CLI
# This script mimics the CI pipeline to catch issues before pushing

set -e  # Exit on any error

echo "🚀 Running Local CI Pipeline for Si CLI"
echo "========================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
success() {
    echo -e "${GREEN}✅ $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

error() {
    echo -e "${RED}❌ $1${NC}"
}

info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

step() {
    echo ""
    echo -e "${BLUE}🔄 $1${NC}"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    error "Must be run from the project root directory"
    exit 1
fi

# Check prerequisites
step "Checking prerequisites..."

if ! command -v cargo &> /dev/null; then
    error "Cargo not found. Please install Rust: https://rustup.rs/"
    exit 1
fi

if ! command -v rustc &> /dev/null; then
    error "rustc not found. Please install Rust: https://rustup.rs/"
    exit 1
fi

RUST_VERSION=$(rustc --version | cut -d' ' -f2)
info "Rust version: $RUST_VERSION"

# Check for minimum Rust version (1.70.0)
RUST_MAJOR=$(echo $RUST_VERSION | cut -d'.' -f1)
RUST_MINOR=$(echo $RUST_VERSION | cut -d'.' -f2)

if [ "$RUST_MAJOR" -lt 1 ] || ([ "$RUST_MAJOR" -eq 1 ] && [ "$RUST_MINOR" -lt 70 ]); then
    error "Rust 1.70.0 or later is required. Current version: $RUST_VERSION"
    exit 1
fi

success "Prerequisites check passed"

# Clean build artifacts
step "Cleaning build artifacts..."
cargo clean
success "Clean completed"

# Step 1: Check formatting
step "Step 1/8: Checking code formatting..."
if cargo fmt --all -- --check; then
    success "Code formatting is correct"
else
    error "Code formatting issues found"
    info "Run 'cargo fmt' to fix formatting issues"
    exit 1
fi

# Step 2: Run Clippy
step "Step 2/8: Running Clippy lints..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    success "No Clippy warnings found"
else
    error "Clippy warnings found"
    info "Fix the warnings shown above"
    exit 1
fi

# Step 3: Build project
step "Step 3/8: Building project..."
if cargo build --verbose; then
    success "Debug build successful"
else
    error "Debug build failed"
    exit 1
fi

# Step 4: Run unit tests
step "Step 4/8: Running unit tests..."
if cargo test --lib --verbose; then
    success "Unit tests passed"
else
    error "Unit tests failed"
    exit 1
fi

# Step 5: Run integration tests
step "Step 5/8: Running integration tests..."
if cargo test --test '*' --verbose; then
    success "Integration tests passed"
else
    error "Integration tests failed"
    exit 1
fi

# Step 6: Run all tests
step "Step 6/8: Running complete test suite..."
if cargo test --verbose; then
    success "All tests passed"
else
    error "Some tests failed"
    exit 1
fi

# Step 7: Check documentation
step "Step 7/8: Building documentation..."
if cargo doc --no-deps --all-features; then
    success "Documentation build successful"
else
    error "Documentation build failed"
    exit 1
fi

# Step 8: Build release
step "Step 8/8: Building release version..."
if cargo build --release --verbose; then
    success "Release build successful"
else
    error "Release build failed"
    exit 1
fi

# Optional: Security audit (if cargo-audit is installed)
if command -v cargo-audit &> /dev/null; then
    step "Running security audit..."
    if cargo audit; then
        success "Security audit passed"
    else
        warning "Security audit found issues (check output above)"
    fi
else
    warning "cargo-audit not installed. Install with: cargo install cargo-audit"
fi

# Optional: Coverage analysis (if cargo-tarpaulin is installed)
if command -v cargo-tarpaulin &> /dev/null; then
    step "Running coverage analysis..."
    info "This may take a few minutes..."

    if timeout 180s cargo tarpaulin --skip-clean --all-features --timeout 90 --fail-under 70 --out Stdout 2>/dev/null | tail -10; then
        success "Coverage analysis completed"
    else
        warning "Coverage analysis failed or timed out"
        info "This is non-critical for the CI pipeline"
    fi
else
    warning "cargo-tarpaulin not installed. Install with: cargo install cargo-tarpaulin"
    info "Coverage analysis will be skipped"
fi

# Test CLI functionality
step "Testing CLI functionality..."
if ./target/release/si --help > /dev/null 2>&1; then
    success "CLI help command works"
else
    error "CLI help command failed"
    exit 1
fi

if ./target/release/si --version > /dev/null 2>&1; then
    success "CLI version command works"
else
    error "CLI version command failed"
    exit 1
fi

# Summary
echo ""
echo "🎉 Local CI Pipeline Completed Successfully!"
echo "============================================"
echo ""
success "All checks passed:"
echo "  ✅ Code formatting"
echo "  ✅ Clippy lints"
echo "  ✅ Debug build"
echo "  ✅ Unit tests"
echo "  ✅ Integration tests"
echo "  ✅ Documentation"
echo "  ✅ Release build"
echo "  ✅ CLI functionality"

# Count total tests
TOTAL_TESTS=$(cargo test 2>/dev/null | grep -E "test result.*passed" | awk '{sum += $4} END {print sum}')
info "Total tests executed: $TOTAL_TESTS"

echo ""
echo "🚀 Your code is ready for CI/CD pipeline!"
echo "   You can safely push to trigger the GitHub Actions workflow."
echo ""
echo "Next steps:"
echo "  1. git add ."
echo "  2. git commit -m 'Your commit message'"
echo "  3. git push"
echo ""
info "The GitHub Actions workflow will run the same checks automatically."
