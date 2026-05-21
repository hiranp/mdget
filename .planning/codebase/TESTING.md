# Testing Infrastructure

**Mapped:** 2026-05-21  
**Project:** mdget

## Current State

**No tests implemented yet.** This is a template/scaffolding project. Test infrastructure is **prepared but unused**.

## Test Framework

### Built-in Rust Testing
- Standard Rust test framework (no external test harness)
- Test runner: `cargo test`
- Attributes: `#[test]`, `#[tokio::test]` for async tests

### Expected Dependencies (Not Yet Added)
When tests are written, likely additions:
- **tokio** `test` feature — `#[tokio::test]` macro for async tests
- **assert_matches** or **similar** — Pattern matching assertions

## Test Structure

### Convention (When Implemented)
Typical Rust project test organization:

```
src/
├── main.rs
├── lib.rs (if extracted)
├── config.rs
│   └── #[cfg(test)] mod tests { ... }
├── log.rs
│   └── #[cfg(test)] mod tests { ... }
└── commands/
    ├── command1.rs
    │   └── #[cfg(test)] mod tests { ... }
    └── command2.rs
        └── #[cfg(test)] mod tests { ... }
```

**OR**

```
tests/
├── integration_test.rs
├── config_test.rs
└── cli_test.rs
```

## Testing Recommendations

### Unit Tests
Test individual functions in isolation.

**Config loading:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_log_level() {
        // Verify default is "info"
    }

    #[test]
    fn test_config_layering() {
        // Verify CLI flags override env vars
    }
}
```

**Log setup:**
```rust
#[test]
fn test_log_fallback_on_invalid_level() {
    // Verify fallback to INFO on invalid level string
}
```

### Integration Tests
Test command execution end-to-end.

**CLI parsing:**
```rust
#[test]
fn test_cli_parsing_with_global_flags() {
    // Parse sample CLI args, verify structure
}
```

**Graceful shutdown:**
```rust
#[tokio::test]
async fn test_command_cancellation_on_signal() {
    // Simulate signal, verify command stops within timeout
}
```

### Property-Based Testing (Optional)
For config validation, consider `proptest` or `quickcheck`:
- Generate random config combinations
- Verify invariants hold (e.g., CLI always wins)

## Mocking Strategy

### File System Mocking
For config loading tests:
- Use `tempfile` crate for temporary test directories
- Create test config files, verify loading

### Environment Variable Mocking
For env var tests:
- Use `serial_test` crate to avoid parallel test conflicts
- Or use subprocess approach with `std::process::Command`

### Async Subsystem Mocking
For command tests:
- Use `tokio::time::pause()` for time-based tests
- Mock `SubsystemHandle` behavior (may need test double)

## Coverage

### Current Coverage
**0%** — No tests written.

### Coverage Tools
- **cargo-tarpaulin** — Code coverage for Rust
  - Install: `cargo install cargo-tarpaulin`
  - Run: `cargo tarpaulin --out Html`
  - Output: `tarpaulin-report.html`

- **cargo-llvm-cov** — LLVM-based coverage
  - Install: `cargo install cargo-llvm-cov`
  - Run: `cargo llvm-cov --html`

### Coverage Goals
Standard Rust project targets:
- **80%+ line coverage** for core logic (config, logging)
- **100% coverage** for error paths (miette conversions)
- **Lower coverage acceptable** for command scaffolding (placeholder logic)

## Test Execution

### Running Tests
```bash
# All tests
cargo test

# Specific module
cargo test config::tests

# Integration tests only
cargo test --test integration_test

# With output
cargo test -- --nocapture

# Parallel execution (default)
cargo test

# Sequential execution
cargo test -- --test-threads=1
```

### CI Integration
Add to `.github/workflows/ci.yml`:
```yaml
- name: Run tests
  run: cargo test --all-features
```

## Test Data

### Fixtures
When tests are added, create:
```
tests/fixtures/
├── valid_config.toml
├── invalid_config.toml
└── minimal_config.toml
```

### Example Data
For CLI parsing tests:
```rust
const SAMPLE_ARGS: &[&str] = &[
    "mdget",
    "--log-level", "debug",
    "command1"
];
```

## Debugging Tests

### Print Debugging
```rust
#[test]
fn test_something() {
    println!("Debug output"); // Visible with --nocapture
    assert_eq!(1, 1);
}
```

### Conditional Compilation
```rust
#[cfg(test)]
use tracing_subscriber; // Only in test builds
```

### Test-Only Code
Mark test utilities with `#[cfg(test)]`:
```rust
#[cfg(test)]
pub fn create_test_config() -> GlobalConfig {
    // Helper for tests
}
```

## Mutation Testing

### cargo-mutants
- Install: `cargo install cargo-mutants`
- Run: `cargo mutants`
- Output: `mutants.out/` (git-ignored)

Mutation testing verifies that tests actually catch bugs by introducing deliberate code changes and checking if tests fail.

## Test-Driven Development

### Recommended Flow
1. Write test for new feature (RED)
2. Implement minimal code to pass (GREEN)
3. Refactor (IMPROVE)
4. Repeat

### Example: Adding HTTP Client
```rust
#[tokio::test]
async fn test_http_get_request() {
    let url = "https://httpbin.org/get";
    let response = http_client::get(url).await.unwrap();
    assert_eq!(response.status(), 200);
}
```

Write test first, then implement `http_client::get()`.

## Async Testing

### tokio::test Macro
For async tests:
```rust
#[tokio::test]
async fn test_async_command() {
    let result = some_async_function().await;
    assert!(result.is_ok());
}
```

### Timeout Testing
```rust
#[tokio::test]
#[should_panic(expected = "timeout")]
async fn test_command_timeout() {
    tokio::time::timeout(
        Duration::from_secs(1),
        never_completes()
    ).await.unwrap();
}
```

## Performance Testing

### Benchmarking
- **criterion.rs** — Standard Rust benchmarking
  - Add to `Cargo.toml`: `[dev-dependencies] criterion = "0.5"`
  - Create `benches/` directory
  - Run: `cargo bench`

### Profiling
- **cargo-flamegraph** — CPU profiling
  - Install: `cargo install flamegraph`
  - Run: `cargo flamegraph`

## Test Anti-Patterns

### Don't Test Private Implementation
❌ Don't test internal helper functions
✅ Test public API behavior

### Don't Over-Mock
❌ Don't mock everything
✅ Use real dependencies when fast enough

### Don't Write Brittle Tests
❌ Don't assert on exact log messages
✅ Assert on behavior and outcomes

### Don't Skip Error Tests
❌ Don't only test happy path
✅ Test error conditions explicitly

## Next Steps for Testing

1. Add `#[tokio::test]` to dev-dependencies
2. Write unit tests for `config.rs` (layering logic)
3. Write unit tests for `log.rs` (level fallback)
4. Add integration tests for CLI parsing
5. Add integration test for graceful shutdown
6. Set up coverage reporting in CI
7. Achieve 80%+ coverage before v1.0

---
*Testing infrastructure mapped: 2026-05-21*
*Status: Framework ready, tests not yet written*
