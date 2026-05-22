# Technology Stack

**Mapped:** 2026-05-21  
**Project:** mdget

## Languages & Runtime

### Rust
- **Edition:** 2024
- **MSRV:** 1.85+
- **Compilation:** Standard Rust toolchain with cargo

## Core Framework & Libraries

### CLI Framework
- **clap** `4.5.58` — Command-line argument parsing
  - Using derive API for declarative CLI definition
  - Features: `derive` for proc macros
  - Subcommand pattern with `Commands` enum
  - Global options support (home, log-level, etc.)

- **clap_complete** `4.5.66` — Shell completion generation
  - Build-time completion generation (`build.rs`)
  - Supports: bash, zsh, fish, PowerShell
  - Generated to `completions/` directory

### Configuration Management
- **config** `0.15.19` — Layered configuration system
  - Hierarchy: default → system (`/etc/mdget/`) → user (`~/.config/mdget/`) → env vars (`MDGET_*`) → CLI args
  - File format: TOML
  - Environment variable prefix: `MDGET_`
  - XDG Base Directory compliance via `directories` crate

- **directories** `6.0.0` — XDG directory resolution
  - Config dir: `~/.config/mdget/`
  - Cache dir: For log files
  - Cross-platform path handling

### Logging & Tracing
- **tracing** `0.1.44` — Structured logging framework
  - File + line number tracking enabled
  - Level filtering: trace, debug, info, warn, error
  - Async-aware instrumentation

- **tracing-subscriber** `0.3.22` — Logging infrastructure
  - Registry-based layer composition
  - Dual output: stdout + rolling file
  - ANSI color support (disabled for file output)

- **tracing-appender** `0.2.4` — Log rotation
  - Daily rolling log files: `rolling.log`
  - Non-blocking writer for async compatibility

- **tracing-log** `0.2.0` — `log` → `tracing` bridge
  - Compatibility with `log` crate ecosystem

### Async Runtime
- **tokio** `1.49.0` — Async runtime
  - Features: `full` (all tokio features enabled)
  - `#[tokio::main]` macro for async entry point
  - Used for graceful shutdown coordination

- **tokio-graceful-shutdown** `0.19.2` — Shutdown orchestration
  - Signal handling (SIGINT, SIGTERM)
  - Subsystem coordination pattern
  - 1-second shutdown timeout

### Error Handling
- **miette** `7.6.0` — Rich diagnostic errors
  - `Result<T>` = `miette::Result<T>`
  - `.into_diagnostic()` for stdlib error conversion
  - Human-friendly error formatting

### Serialization
- **serde** `1.0.228` — Serialization framework
  - Features: `derive` for `Serialize`/`Deserialize` proc macros
  - Used for config deserialization

## Development Tools

### Build System
- Standard Cargo with custom `build.rs`
- Pre-compilation shell completion generation
- No build.rs dependencies beyond clap

### Code Quality
- **clippy** — Rust linter
  - Configuration: `clippy.toml`
  - Enforced warnings in CI

- **rustfmt** — Code formatter
  - Configuration: `rustfmt.toml`
  - Edition 2024 formatting rules

- **cargo-deny** — Dependency auditing
  - License compliance checks
  - Security vulnerability scanning
  - Duplicate dependency detection
  - Configuration: `deny.toml`

### CI/CD
- **GitHub Actions** workflows:
  - `ci.yml` — Build, test, clippy, fmt checks
  - `security-audit.yml` — cargo-audit for CVEs
  - `release.yml` — Binary releases
  - `cargo-deny.yml` — Dependency policy enforcement

### Editor Configuration
- **VS Code** settings (`.vscode/`)
- **EditorConfig** (`.editorconfig`) — Cross-editor consistency

### Task Automation
- **just** — Command runner
  - File: `justfile`
  - Tasks: build, test, lint, docs, publish

## Configuration Files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Package metadata, dependencies |
| `clippy.toml` | Linter rules |
| `rustfmt.toml` | Formatter settings |
| `deny.toml` | Dependency policy |
| `dist-workspace.toml` | Distribution config |
| `.editorconfig` | Editor consistency |
| `justfile` | Task automation |

## Package Metadata

- **Name:** mdget
- **Version:** 0.3.0
- **Description:** Agent-first curl alternative
- **License:** MIT
- **Repository:** https://github.com/hiranp/mdget
- **Binary:** `mdget` (from `src/main.rs`)

## Key Dependencies Summary

| Category | Primary Crates |
|----------|---------------|
| CLI | clap, clap_complete |
| Config | config, directories |
| Logging | tracing, tracing-subscriber, tracing-appender |
| Async | tokio, tokio-graceful-shutdown |
| Errors | miette |
| Serialization | serde |

---
*Stack mapped: 2026-05-21*
