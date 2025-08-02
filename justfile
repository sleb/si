# Si Development Commands

# Default recipe to display available commands
default:
    @just --list

# Run all tests
test:
    cargo test --verbose

# Run only unit tests
test-unit:
    cargo test --lib --verbose

# Run only integration tests
test-integration:
    cargo test --test '*' --verbose

# Run tests with coverage report
coverage:
    cargo tarpaulin --verbose --all-features --workspace --timeout 120 --out html --out xml

# Run tests with coverage and open HTML report
coverage-open: coverage
    open tarpaulin-report.html

# Check code formatting
fmt-check:
    cargo fmt --all -- --check

# Format code
fmt:
    cargo fmt --all

# Run clippy for linting
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run security audit
audit:
    cargo audit

# Build the project
build:
    cargo build --verbose

# Build release version
build-release:
    cargo build --release --verbose

# Run the CLI with help
run-help:
    cargo run -- --help

# Run model list command
run-model-list:
    cargo run -- model list

# Clean build artifacts
clean:
    cargo clean

# Check everything (format, clippy, tests)
check: fmt-check clippy test

# Prepare for commit (format, clippy, tests, coverage)
pre-commit: fmt clippy test coverage

# Install development dependencies
install-deps:
    cargo install cargo-tarpaulin cargo-audit

# Generate documentation
docs:
    cargo doc --no-deps --all-features --open

# Run benchmarks
bench:
    cargo bench --verbose

# Check MSRV compatibility
msrv:
    #!/usr/bin/env bash
    MSRV=$(./scripts/get-msrv.sh)
    echo "Testing with MSRV: $MSRV"
    cargo +$MSRV test --verbose

# Show current MSRV
msrv-show:
    @./scripts/get-msrv.sh

# Check if current Rust version meets MSRV requirement
msrv-check:
    @./scripts/get-msrv.sh --check

# Validate MSRV format in Cargo.toml
msrv-validate:
    @./scripts/get-msrv.sh --validate

# Watch for changes and run tests
watch:
    cargo watch -x test

# Watch for changes and run specific test
watch-test TEST:
    cargo watch -x "test {{ TEST }}"

# Run with environment logging
run-debug:
    RUST_LOG=debug cargo run -- --help

# Profile with perf (Linux only)
profile:
    cargo build --release
    perf record --call-graph=dwarf ./target/release/si --help
    perf report

# Show test output even for passing tests
test-verbose:
    cargo test --verbose -- --nocapture

# Run tests for a specific module
test-module MODULE:
    cargo test {{ MODULE }} --verbose

# Check for unused dependencies
unused-deps:
    cargo +nightly udeps

# Update dependencies
update:
    cargo update

# Show dependency tree
deps-tree:
    cargo tree

# Verify project structure
verify:
    @echo "Checking project structure..."
    @test -f Cargo.toml || (echo "❌ Cargo.toml missing" && exit 1)
    @test -f src/main.rs || (echo "❌ src/main.rs missing" && exit 1)
    @test -f src/lib.rs || (echo "❌ src/lib.rs missing" && exit 1)
    @test -f src/models.rs || (echo "❌ src/models.rs missing" && exit 1)
    @test -d tests || (echo "❌ tests directory missing" && exit 1)
    @test -f .github/workflows/ci.yml || (echo "❌ CI workflow missing" && exit 1)
    @echo "✅ Project structure verified"

# Run full CI pipeline locally
ci: verify fmt-check clippy test coverage audit
    @echo "✅ All CI checks passed!"

# Quick development cycle
dev: fmt clippy test-unit
    @echo "✅ Development checks passed!"

# Run test verification script
test-script:
    ./scripts/test.sh

# Run local CI verification
ci-local:
    ./scripts/ci-local.sh

# Make scripts executable
make-scripts-executable:
    chmod +x scripts/test.sh scripts/ci-local.sh scripts/get-msrv.sh
