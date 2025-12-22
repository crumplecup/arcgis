# ArcGIS Rust SDK - Justfile
# Standard development recipes

# Default recipe - show available commands
default:
    @just --list

# Run basic compilation check
check package="":
    #!/usr/bin/env bash
    if [ -z "{{package}}" ]; then
        cargo check
    else
        cargo check -p {{package}}
    fi

# Run tests for a specific package
test-package package="":
    #!/usr/bin/env bash
    if [ -z "{{package}}" ]; then
        cargo test
    else
        cargo test -p {{package}}
    fi

# Run all checks: clippy, fmt, and test
check-all package="":
    #!/usr/bin/env bash
    echo "Running clippy..."
    if [ -z "{{package}}" ]; then
        cargo clippy --all-targets --all-features -- -D warnings
    else
        cargo clippy -p {{package}} --all-targets --all-features -- -D warnings
    fi
    echo "Checking formatting..."
    cargo fmt --all -- --check
    echo "Running tests..."
    if [ -z "{{package}}" ]; then
        cargo test
    else
        cargo test -p {{package}}
    fi

# Run clippy
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Check code formatting
fmt-check:
    cargo fmt --all -- --check

# Format code
fmt:
    cargo fmt --all

# Build the project
build:
    cargo build

# Build with release optimizations
build-release:
    cargo build --release

# Run all tests (unit tests only, not integration)
test:
    cargo test --lib

# Run all tests including doc tests
test-all:
    cargo test

# Run integration tests (requires .env with credentials)
test-integration:
    cargo test --test integration_basic --features api

# Run API tests (rate-limited, use sparingly)
test-api:
    @echo "⚠️  API tests hit live ArcGIS services - use sparingly!"
    cargo test --features api

# Build documentation
doc:
    cargo doc --no-deps --all-features

# Build and open documentation
doc-open:
    cargo doc --no-deps --all-features --open

# Check all feature combinations
check-features:
    #!/usr/bin/env bash
    echo "Checking with no features..."
    cargo check --no-default-features
    echo "Checking with all features..."
    cargo check --all-features
    echo "Checking with api feature..."
    cargo check --features api

# Run security audit
audit:
    cargo audit

# Update dependencies
update-deps:
    cargo update

# Clean build artifacts
clean:
    cargo clean

# Run all pre-commit checks
pre-commit: check-all
    @echo "✓ All pre-commit checks passed"

# Run all pre-merge checks
pre-merge: check-all check-features audit
    @echo "✓ All pre-merge checks passed"

# Watch for changes and run tests
watch:
    cargo watch -x test

# Check MSRV (Minimum Supported Rust Version)
msrv:
    cargo +1.75 check

# Run benchmarks (when they exist)
bench:
    cargo bench

# Verify the crate can be published
publish-dry-run:
    cargo publish --dry-run

# Generate release artifacts (placeholder for cargo-dist)
dist-build:
    @echo "cargo-dist not configured yet"

# Check release configuration
dist-check:
    @echo "cargo-dist not configured yet"

# Plan release
dist-plan:
    @echo "cargo-dist not configured yet"

# Generate CI workflow for releases
dist-generate:
    @echo "cargo-dist not configured yet"

# Run all security checks
security: audit
    @echo "✓ Security checks passed"

# Generate omnibor artifact tree (placeholder)
omnibor:
    @echo "omnibor not configured yet"
