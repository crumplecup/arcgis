# ArcGIS Rust SDK - Justfile
# Standard development recipes

# Default recipe - show available commands
default:
    @just --list

# Install development tools (cargo-dist, omnibor, cargo-audit, cargo-watch)
setup:
    #!/usr/bin/env bash
    echo "Installing development tools..."

    # Check and install cargo-dist
    if command -v cargo-dist &> /dev/null; then
        echo "✓ cargo-dist already installed"
    else
        echo "Installing cargo-dist..."
        cargo install cargo-dist
    fi

    # Check and install omnibor
    if command -v omnibor &> /dev/null; then
        echo "✓ omnibor already installed"
    else
        echo "Installing omnibor-cli..."
        cargo install omnibor-cli
    fi

    # Check and install cargo-audit
    if command -v cargo-audit &> /dev/null; then
        echo "✓ cargo-audit already installed"
    else
        echo "Installing cargo-audit..."
        cargo install cargo-audit
    fi

    # Check and install cargo-watch
    if command -v cargo-watch &> /dev/null; then
        echo "✓ cargo-watch already installed"
    else
        echo "Installing cargo-watch..."
        cargo install cargo-watch
    fi

    echo ""
    echo "✓ All development tools installed"
    echo ""
    echo "Next steps:"
    echo "  1. Run 'just build' to build the project"
    echo "  2. Run 'just test' to run tests"
    echo "  3. Run 'just check-all' to run all checks"

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
    set -o pipefail

    # Generate timestamped log file in /tmp
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    LOGFILE="/tmp/arcgis-check-all_${TIMESTAMP}.log"

    # Track overall status
    HAD_ERRORS=0

    echo "Logging to: $LOGFILE"
    echo "========================================" | tee "$LOGFILE"
    echo "check-all run started at $(date)" | tee -a "$LOGFILE"
    echo "========================================" | tee -a "$LOGFILE"
    echo "" | tee -a "$LOGFILE"

    # Run clippy
    echo "Running clippy..." | tee -a "$LOGFILE"
    if [ -z "{{package}}" ]; then
        if ! cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tee -a "$LOGFILE"; then
            HAD_ERRORS=1
        fi
    else
        if ! cargo clippy -p {{package}} --all-targets --all-features -- -D warnings 2>&1 | tee -a "$LOGFILE"; then
            HAD_ERRORS=1
        fi
    fi
    echo "" | tee -a "$LOGFILE"

    # Check formatting
    echo "Checking formatting..." | tee -a "$LOGFILE"
    if ! cargo fmt --all -- --check 2>&1 | tee -a "$LOGFILE"; then
        HAD_ERRORS=1
    fi
    echo "" | tee -a "$LOGFILE"

    # Run tests
    echo "Running tests..." | tee -a "$LOGFILE"
    if [ -z "{{package}}" ]; then
        if ! cargo test 2>&1 | tee -a "$LOGFILE"; then
            HAD_ERRORS=1
        fi
    else
        if ! cargo test -p {{package}} 2>&1 | tee -a "$LOGFILE"; then
            HAD_ERRORS=1
        fi
    fi
    echo "" | tee -a "$LOGFILE"

    # Final status
    echo "========================================" | tee -a "$LOGFILE"
    echo "check-all run completed at $(date)" | tee -a "$LOGFILE"
    echo "========================================" | tee -a "$LOGFILE"

    if [ $HAD_ERRORS -eq 1 ]; then
        echo ""
        echo "❌ Checks failed. Full output logged to:"
        echo "   $LOGFILE"
        echo ""
        exit 1
    else
        echo ""
        echo "✓ All checks passed"
        echo ""
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

# Run all pre-merge checks (comprehensive validation before merging to main)
pre-merge: check-all check-features audit
    @echo "Running public tests..."
    cargo test --features test-public
    @echo "✓ All pre-merge checks passed"

# Run all pre-publish checks (comprehensive validation before releasing)
pre-publish: pre-merge security dist-check publish-dry-run
    @echo "✓ All pre-publish checks passed"
    @echo ""
    @echo "Ready for release! Next steps:"
    @echo "  1. Review: just dist-plan"
    @echo "  2. Test build: just dist-build"
    @echo "  3. Publish: cargo publish"

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

# Check if cargo-dist is installed
_check-dist:
    #!/usr/bin/env bash
    if ! command -v cargo-dist &> /dev/null; then
        echo "❌ cargo-dist not found"
        echo "Install with: cargo install cargo-dist"
        exit 1
    fi

# Check if omnibor is installed
_check-omnibor:
    #!/usr/bin/env bash
    if ! command -v omnibor &> /dev/null; then
        echo "❌ omnibor not found"
        echo "Install with: cargo install omnibor-cli"
        exit 1
    fi

# Initialize cargo-dist (first-time setup)
dist-init: _check-dist
    cargo dist init

# Generate release artifacts
dist-build: _check-dist
    cargo dist build

# Check release configuration
dist-check: _check-dist
    cargo dist plan --output-format=json

# Plan release (show what would be released)
dist-plan: _check-dist
    cargo dist plan

# Generate CI workflow for releases
dist-generate: _check-dist
    cargo dist generate-ci

# Generate omnibor artifact tree (software bill of materials)
omnibor: _check-omnibor
    @echo "Generating OmniBOR artifact identifiers..."
    omnibor id create target/release/ 2>/dev/null || omnibor id create target/debug/ 2>/dev/null || echo "⚠️  No build artifacts found. Run 'just build' or 'just build-release' first."

# Run all security checks
security: audit omnibor
    @echo "✓ Security checks passed"
