# Scripts Directory

This directory contains helper scripts for development and CI/CD workflows.

## MSRV Management

The project uses a single source of truth for the Minimum Supported Rust Version (MSRV) defined in `Cargo.toml`:

```toml
[package]
rust-version = "1.85.1"
```

### get-msrv.sh

A helper script that extracts and works with the MSRV from `Cargo.toml`.

**Usage:**
```bash
# Get the MSRV
./scripts/get-msrv.sh

# Check if current Rust version meets MSRV
./scripts/get-msrv.sh --check

# Validate MSRV format
./scripts/get-msrv.sh --validate

# Use in other scripts
MSRV=$(./scripts/get-msrv.sh)
```

**Justfile Commands:**
```bash
# Show current MSRV
just msrv-show

# Check current Rust version against MSRV
just msrv-check

# Validate MSRV format
just msrv-validate

# Test with MSRV (requires rustup toolchain)
just msrv
```

### How MSRV is Used

1. **Cargo.toml**: Single source of truth for MSRV
2. **CI Workflow**: Dynamically extracts MSRV from Cargo.toml for testing
3. **Local Scripts**: Use `get-msrv.sh` to ensure consistency
4. **Justfile**: Provides convenient commands for MSRV operations

### Updating MSRV

To update the MSRV:

1. Update `rust-version` in `Cargo.toml`
2. All other tools will automatically use the new version
3. No need to update CI workflows or scripts manually

### CI Integration

The GitHub Actions workflow automatically:
- Extracts MSRV from `Cargo.toml`
- Tests the codebase with that specific Rust version
- Uses the MSRV for cache keys to ensure consistency

## Other Scripts

### ci-local.sh

Runs the complete CI pipeline locally to catch issues before pushing.

**Features:**
- Automatically detects MSRV from `Cargo.toml`
- Runs all CI checks (formatting, linting, tests, etc.)
- Provides colored output and progress indicators
- Tests CLI functionality

**Usage:**
```bash
./scripts/ci-local.sh
```

### test.sh

Focused testing script for development workflows.

**Usage:**
```bash
./scripts/test.sh
```

## Making Scripts Executable

All scripts should be executable. Use the justfile command to ensure this:

```bash
just make-scripts-executable
```

## Dependencies

Some scripts may require additional tools:
- `cargo-audit` for security auditing
- `cargo-tarpaulin` for coverage analysis
- `rustup` for managing Rust toolchains

Install with:
```bash
just install-deps
```
