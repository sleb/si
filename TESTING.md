# Testing Documentation for Si CLI

This document provides comprehensive information about the testing strategy, coverage, and procedures for the Si CLI project.

## Overview

The Si CLI project maintains high-quality code through comprehensive testing with:

- **68 total tests** across unit and integration test suites
- **~80% code coverage** target
- **Multiple test types** including unit, integration, and CLI tests
- **Automated CI/CD** pipeline with GitHub Actions

## Test Structure

### Unit Tests (40 tests)

Located in `src/` files with `#[cfg(test)]` modules:

#### `src/models.rs` Tests (15 tests)
- `ModelInfo` creation and serialization
- `ModelFile` data structures and edge cases
- `ModelIndex` persistence and loading
- `ModelManager` builder pattern and validation
- Error handling for invalid data and missing files

#### `src/main.rs` Tests (25 tests)
- CLI command parsing and validation
- Command handler functions for all subcommands
- Error cases for missing arguments
- CLI structure validation

### Integration Tests (28 tests)

Located in `tests/` directory:

#### `tests/integration_tests.rs` (18 tests)
- End-to-end CLI functionality testing
- Help and version commands
- All subcommand help text validation
- Error handling for invalid commands and arguments
- File I/O operations for image generation commands

#### `tests/model_manager_tests.rs` (10 tests)
- ModelManager lifecycle testing
- Model persistence and retrieval
- Concurrent access patterns
- Edge cases with special characters and large files
- Index file creation and validation

## Test Categories

### 1. Data Structure Tests
- Serialization/deserialization of all data types
- Clone implementations
- Builder pattern validation
- Error path coverage

### 2. Business Logic Tests
- Model management operations
- File system interactions
- Configuration management
- API integration points

### 3. CLI Interface Tests
- Command parsing with clap
- Help text generation
- Error message formatting
- Argument validation

### 4. Error Handling Tests
- Invalid file paths
- Malformed JSON data
- Missing dependencies
- Network failure scenarios

### 5. Edge Case Tests
- Empty data structures
- Maximum size values
- Special characters in paths
- Concurrent operations

## Coverage Analysis

### Target Coverage: ~80%

The project aims for 80% code coverage across:

- **Core business logic**: 90%+ coverage
- **Error handling paths**: 85%+ coverage
- **CLI interfaces**: 75%+ coverage
- **Integration points**: 70%+ coverage

### Coverage Tools

```bash
# Install coverage tool
cargo install cargo-tarpaulin

# Run coverage analysis
cargo tarpaulin --all-features --out Html --out Xml

# View HTML report
open tarpaulin-report.html
```

### Current Coverage Areas

#### Well Covered (90%+)
- Model data structures and serialization
- Builder pattern implementations
- Basic CLI command parsing
- File I/O operations

#### Moderately Covered (70-89%)
- Error handling paths
- Edge case scenarios
- Integration with external APIs
- Complex business logic flows

#### Areas for Improvement (<70%)
- Network error scenarios
- Complex async operations
- Performance edge cases

## Running Tests

### Quick Test Commands

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test '*'

# Run with output capture
cargo test -- --nocapture

# Run specific test
cargo test test_model_info_new
```

### Test Scripts

#### Basic Test Verification
```bash
./scripts/test.sh
```

#### Full CI Pipeline Locally
```bash
./scripts/ci-local.sh
```

#### Using Just Commands
```bash
# Quick development cycle
just dev

# Full CI checks
just ci

# Coverage analysis
just coverage
```

## Test Development Guidelines

### Writing Unit Tests

1. **Test one thing at a time**
   ```rust
   #[test]
   fn test_model_info_creation() {
       let files = vec![ModelFile { size: 1024, path: PathBuf::from("test.bin") }];
       let model = ModelInfo::new("test-model", files);
       assert_eq!(model.model_id, "test-model");
   }
   ```

2. **Use descriptive test names**
   - `test_model_info_serialization_with_empty_files`
   - `test_model_manager_handles_missing_index_gracefully`

3. **Test error conditions**
   ```rust
   #[test]
   fn test_model_info_from_invalid_json() {
       let result = ModelInfo::try_from(Path::new("invalid.json"));
       assert!(result.is_err());
   }
   ```

### Writing Integration Tests

1. **Use temporary directories**
   ```rust
   let temp_dir = tempdir()?;
   let models_dir = temp_dir.path().join("models");
   ```

2. **Test realistic scenarios**
   ```rust
   #[test]
   fn test_cli_model_download_workflow() {
       // Test the complete model download workflow
   }
   ```

3. **Clean up resources**
   ```rust
   // tempdir automatically cleans up when dropped
   ```

### Test Data Management

#### Fixtures
- Use `tempfile` crate for temporary test data
- Create minimal, focused test data
- Avoid dependency on external services in unit tests

#### Mocking
- Use `mockall` crate for complex dependencies
- Mock external API calls
- Test both success and failure scenarios

## Continuous Integration

### GitHub Actions Workflow

The CI pipeline runs on every push and pull request:

1. **Format Check**: `cargo fmt --check`
2. **Linting**: `cargo clippy -- -D warnings`
3. **Build**: `cargo build`
4. **Unit Tests**: `cargo test --lib`
5. **Integration Tests**: `cargo test --test '*'`
6. **Documentation**: `cargo doc`
7. **Coverage**: `cargo tarpaulin`
8. **Security Audit**: `cargo audit`

### Multiple Environments

Tests run on:
- **Ubuntu Latest** (Primary)
- **Windows Latest**
- **macOS Latest**
- **Rust Stable & Beta**

### Minimum Rust Version

The project supports Rust 1.70.0+ with MSRV testing.

## Performance Testing

### Benchmarks
```bash
cargo bench
```

### Memory Usage
- Integration tests monitor memory usage
- Large file handling tests
- Concurrent operation stress tests

## Debugging Tests

### Failed Test Investigation

1. **Run with verbose output**
   ```bash
   cargo test -- --nocapture
   ```

2. **Run single test**
   ```bash
   cargo test test_name -- --exact
   ```

3. **Enable debug logging**
   ```bash
   RUST_LOG=debug cargo test
   ```

### Common Issues

#### Test Isolation
- Each test uses separate temporary directories
- No shared state between tests
- Database/file cleanup after each test

#### Timing Issues
- Use `tokio-test` for async test utilities
- Avoid hardcoded timeouts
- Use deterministic test data

## Quality Metrics

### Success Criteria
- All tests pass on CI
- No clippy warnings
- Code coverage ≥ 80%
- Documentation builds successfully
- Security audit passes

### Failure Handling
- Tests fail fast on first error
- Clear error messages with context
- Actionable feedback for developers

## Contributing Guidelines

### Before Submitting
1. Run `./scripts/ci-local.sh`
2. Ensure all tests pass
3. Add tests for new functionality
4. Update documentation if needed

### Test Review Checklist
- [ ] Tests cover happy path
- [ ] Tests cover error conditions
- [ ] Tests are isolated and repeatable
- [ ] Test names are descriptive
- [ ] No hardcoded values or timeouts
- [ ] Cleanup is properly handled

## Future Improvements

### Planned Enhancements
- Property-based testing with `quickcheck`
- Performance regression testing
- End-to-end workflow testing
- Cross-platform compatibility testing
- Load testing for concurrent operations

### Tooling Improvements
- Test result caching
- Parallel test execution optimization
- Better coverage visualization
- Automated test case generation

---

## Quick Reference

```bash
# Essential commands
cargo test                           # Run all tests
cargo test --lib                     # Unit tests only
cargo test --test '*'                # Integration tests only
cargo tarpaulin --out Html           # Coverage report
./scripts/test.sh                    # Full test verification
./scripts/ci-local.sh                # Local CI pipeline
just ci                              # Complete CI checks
```

For more information, see:
- [Cargo Test Documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Project README](./README.md)
