#!/bin/bash

# Helper script to extract the Minimum Supported Rust Version (MSRV) from Cargo.toml
# This script can be sourced or executed directly to get the MSRV value

set -e

# Function to extract MSRV from Cargo.toml
get_msrv() {
    local cargo_toml="${1:-Cargo.toml}"

    if [ ! -f "$cargo_toml" ]; then
        echo "Error: $cargo_toml not found" >&2
        return 1
    fi

    local msrv=$(grep '^rust-version' "$cargo_toml" | sed 's/rust-version = "\(.*\)"/\1/')

    if [ -z "$msrv" ]; then
        echo "Error: rust-version not found in $cargo_toml" >&2
        return 1
    fi

    echo "$msrv"
}

# Function to validate MSRV format
validate_msrv() {
    local msrv="$1"

    if [[ ! "$msrv" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo "Error: Invalid MSRV format: $msrv (expected X.Y.Z)" >&2
        return 1
    fi

    return 0
}

# Function to compare versions (returns 0 if current >= required, 1 otherwise)
compare_rust_version() {
    local current="$1"
    local required="$2"

    # Remove any pre-release or build metadata
    current=$(echo "$current" | sed 's/-.*//')
    required=$(echo "$required" | sed 's/-.*//')

    local current_major=$(echo "$current" | cut -d'.' -f1)
    local current_minor=$(echo "$current" | cut -d'.' -f2)
    local current_patch=$(echo "$current" | cut -d'.' -f3)

    local required_major=$(echo "$required" | cut -d'.' -f1)
    local required_minor=$(echo "$required" | cut -d'.' -f2)
    local required_patch=$(echo "$required" | cut -d'.' -f3)

    if [ "$current_major" -gt "$required_major" ]; then
        return 0
    elif [ "$current_major" -lt "$required_major" ]; then
        return 1
    fi

    if [ "$current_minor" -gt "$required_minor" ]; then
        return 0
    elif [ "$current_minor" -lt "$required_minor" ]; then
        return 1
    fi

    if [ "$current_patch" -ge "$required_patch" ]; then
        return 0
    else
        return 1
    fi
}

# Function to check if current Rust version meets MSRV requirement
check_rust_version() {
    local msrv="$1"
    local current_rust_version

    if ! command -v rustc &> /dev/null; then
        echo "Error: rustc not found. Please install Rust: https://rustup.rs/" >&2
        return 1
    fi

    current_rust_version=$(rustc --version | cut -d' ' -f2)

    if compare_rust_version "$current_rust_version" "$msrv"; then
        echo "✅ Rust version $current_rust_version meets MSRV requirement ($msrv)"
        return 0
    else
        echo "❌ Rust version $current_rust_version does not meet MSRV requirement ($msrv)" >&2
        return 1
    fi
}

# Main execution when script is run directly (not sourced)
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    # Parse command line arguments
    ACTION="get"
    CARGO_TOML="Cargo.toml"

    while [[ $# -gt 0 ]]; do
        case $1 in
            --check)
                ACTION="check"
                shift
                ;;
            --validate)
                ACTION="validate"
                shift
                ;;
            --help|-h)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Extract and work with MSRV from Cargo.toml"
                echo ""
                echo "OPTIONS:"
                echo "  --check      Check if current Rust version meets MSRV"
                echo "  --validate   Validate MSRV format"
                echo "  --help, -h   Show this help message"
                echo ""
                echo "Examples:"
                echo "  $0                    # Print MSRV"
                echo "  $0 --check           # Check current Rust version against MSRV"
                echo "  $0 --validate        # Validate MSRV format"
                echo "  export MSRV=\$($0)   # Store MSRV in environment variable"
                exit 0
                ;;
            *)
                echo "Unknown option: $1" >&2
                echo "Use --help for usage information" >&2
                exit 1
                ;;
        esac
    done

    # Execute requested action
    case $ACTION in
        get)
            get_msrv "$CARGO_TOML"
            ;;
        check)
            msrv=$(get_msrv "$CARGO_TOML")
            if [ $? -ne 0 ]; then
                exit 1
            fi
            check_rust_version "$msrv"
            ;;
        validate)
            msrv=$(get_msrv "$CARGO_TOML")
            if [ $? -ne 0 ]; then
                exit 1
            fi
            validate_msrv "$msrv"
            if [ $? -eq 0 ]; then
                echo "✅ MSRV format is valid: $msrv"
            fi
            ;;
    esac
fi
