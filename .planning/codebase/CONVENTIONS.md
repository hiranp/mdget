# Code Conventions

**Mapped:** 2026-05-21  
**Project:** mdget

## Language Standards

### Rust Edition
- **Edition:** 2024
- **MSRV:** 1.85+ (Minimum Supported Rust Version)
- **Toolchain:** Stable rust compiler

### Code Style
- **Formatter:** rustfmt (edition 2024)
- **Linter:** clippy with project-specific rules
- **Enforcement:** CI checks for both

## Naming Conventions

### Casing
- **Modules:** `snake_case` (`config`, `log`, `commands`)
- **Files:** `snake_case.rs` (`main.rs`, `command1.rs`)
- **Functions:** `snake_case` (`run`, `configure_log`, `countdown`)
- **Types:** `PascalCase` (`GlobalConfig`, `LogLevel`, `Commands`)
- **Constants:** `SCREAMING_SNAKE_CASE` (`QUALIFIER`, `ORGANIZATION`, `APPLICATION`)
- **Variables:** `snake_case` (`cli`, `global_config`, `log_dir`)

### Semantic Naming
- **Booleans:** Prefix with `is_`, `has_`, `should_`, `enabled`
  - Example: `is_fall_back`, `log_file_enable`
- **Functions:** Verb-first for actions
  - `run()`, `configure_log()`, `new()`
- **Modules:** Noun-based for domain
  - `config`, `log`, `commands`

## Module Organization

### File-Module Correspondence
- One module per file: `src/config.rs` → `mod config`
- Subdirectory modules: `src/commands/mod.rs` with re-exports

### Module Re-exports
Pattern in `src/commands/mod.rs`:
```rust
pub mod command1;
pub mod command2;
pub mod completion;
```

Public API exported via `pub` at module level, no glob re-exports.

### Visibility
- Default: Private unless explicitly `pub`
- Commands modules: Functions are `pub` for dispatch from main
- Config types: Public struct, fields public for pattern matching

## Error Handling

### Result Type
All fallible functions return `miette::Result<T>`:
```rust
use miette::Result;

pub async fn run(subsys: &mut SubsystemHandle) -> Result<()> {
    // ...
}
```

### Error Conversion
Standard library errors converted via `.into_diagnostic()`:
```rust
LogTracer::init().into_diagnostic()?;
fs::create_dir_all(log_dir).into_diagnostic()?;
```

### Error Context
Use `miette!` macro for custom errors with context:
```rust
return Err(miette!(
    "error getting configurations path following XDG base directory"
))
```

### No Unwrap/Expect
- ❌ Never use `.unwrap()` or `.expect()` in production paths
- ✅ Use `.unwrap_or_else()` with fallback values for non-critical paths:
  ```rust
  LevelFilter::from_str(&log_config.level).unwrap_or_else(|_| {
      is_fall_back = true;
      LevelFilter::INFO
  })
  ```

## Async Patterns

### Async Function Signatures
Commands are async with explicit lifetimes for subsystem handle:
```rust
pub async fn run(subsys: &mut SubsystemHandle) -> Result<()>
```

### Graceful Shutdown Pattern
All long-running commands follow this pattern:
```rust
tokio::select! {
    _ = subsys.on_shutdown_requested() => {
        // Cleanup on signal
    },
    result = command_logic() => {
        // Normal completion
    }
}
```

### Async Entry Point
Main uses `#[tokio::main]` macro:
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // ...
}
```

## Configuration Patterns

### Layered Config
Config sources in priority order (later overrides earlier):
1. Compile-time defaults (`.set_default()`)
2. System config file (`.add_source(File::with_name("/etc/mdget/config.toml").required(false))`)
3. User config file (`.add_source(File::from(user_config).required(false))`)
4. Environment variables (`.add_source(Environment::with_prefix("MDGET"))`)
5. CLI overrides (`.set_override()`)

### Optional Config Files
Config files are `.required(false)` — no error if missing:
```rust
.add_source(File::with_name(&system_config).required(false))
```

### CLI Overrides
CLI flags override config after loading:
```rust
config_builder = if let Some(log_level) = cli.log_level {
    config_builder
        .set_override("log.level", log_level.as_str())
        .into_diagnostic()?
} else {
    config_builder
};
```

## Logging Patterns

### Structured Logging
Use `tracing` macros, not `println!`:
```rust
use tracing::{info, warn, error};

info!("command1 started.");
warn!("invalid log level '{}', fall back to info level", &log_config.level);
```

### Log Levels
- `trace` — Detailed execution flow
- `debug` — Developer diagnostics
- `info` — User-facing progress
- `warn` — Recoverable issues
- `error` — Failures requiring attention

### File + Line Annotations
Logging configured with `.with_file(true).with_line_number(true)` for debugging.

## Build Script Conventions

### CLI Inclusion
`build.rs` includes CLI definition for completion generation:
```rust
include!("src/cli.rs");
```

This allows build script to access CLI structure at compile time.

### Output Directories
Generated artifacts go to `completions/` (git-ignored).

## Dependency Management

### Version Pinning
- Minor versions pinned in Cargo.toml: `"4.5.58"`, `"0.15.19"`
- Updates require explicit Cargo.toml edit + testing

### Feature Flags
Explicit feature selection:
```toml
tokio = { version = "1.49.0", features = ["full"] }
clap = { version = "4.5.58", features = ["derive"] }
```

### Dependency Audit
- `cargo-deny` checks licenses, security, duplicates
- CI enforces policy on every commit

## Code Quality Gates

### Pre-Commit
- ✅ `cargo fmt` — Formatting
- ✅ `cargo clippy` — Lints
- ✅ `cargo build` — Compilation
- ✅ `cargo test` — Tests (when added)

### CI Checks
- Format check (`cargo fmt -- --check`)
- Lint check (`cargo clippy -- -D warnings`)
- Security audit (`cargo audit`)
- License compliance (`cargo deny check`)

## Documentation

### Inline Comments
- Minimal inline comments (code should be self-explanatory)
- Comments for "why", not "what"
- Example: `// NOTE: see also https://github.com/...` for external references

### Doc Comments
- Standard Rust doc comments (`///`) for public APIs
- Currently sparse (template project)
- Future: Add module-level docs (`//!`)

### README
- High-level project description
- Installation instructions
- Usage examples
- Contributing guidelines

## Anti-Patterns to Avoid

### Global State
❌ No mutable global state
✅ Configuration passed through function parameters

### String-Based Errors
❌ Don't return `String` errors
✅ Use `miette::Result` with rich diagnostics

### Blocking Calls in Async
❌ Don't use blocking I/O in async functions
✅ Use `tokio::fs` or `spawn_blocking()` for sync I/O

### Panic in Production
❌ No `.unwrap()`, `.expect()`, or `panic!()`
✅ Return `Result` and handle errors gracefully

---
*Conventions mapped: 2026-05-21*
