set shell := ["bash", "-cu"]

default:
  @just --list

# Build all targets in debug mode.
build:
  cargo build --all-targets

# Build optimized binaries.
release:
  cargo build --release --all-targets

# Run the CLI; pass args with `just run -- --help`.
run *args:
  cargo run -- 

# Run all tests.
test:
  cargo test --all-features

# Check formatting without changing files.
fmt-check:
  cargo fmt --all -- --check

# Apply canonical formatting.
fmt:
  cargo fmt --all

# Run clippy with warnings denied.
lint:
  cargo clippy --all-targets --all-features -- -D warnings

# Run full local CI sequence.
check: fmt-check lint test

# Build docs and fail on warnings.
doc:
  RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

# Security audit (requires cargo-audit).
audit:
  cargo audit

# Policy checks for dependencies and licenses (requires cargo-deny).
deny:
  cargo deny check

# Install current crate locally.
install:
  cargo install --path .

# Publish to crates.io after local checks.
publish: check
  cargo publish

# Build cross-platform release artifacts locally with cargo-dist.
dist:
  cargo dist build

# Create and announce release artifacts with cargo-dist.
dist-host:
  cargo dist host
