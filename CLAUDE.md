<!-- GSD:project-start source:PROJECT.md -->
## Project

**mdget — Agent-First HTTP Client in Rust**

mdget is a high-performance Rust rewrite of mdurl (TypeScript), designed as a curl alternative optimized for AI agents and LLM tools. It fetches webpages and outputs clean markdown with structured metadata, enabling agents to read web content without HTML parsing complexity. Key improvements over mdurl: faster execution, smaller binary (~5-10MB vs 50+MB node_modules), instant startup, advanced caching, authentication helpers, and response filtering.

**Core Value:** **The ONE thing:** Agents can fetch any web resource with `mdget <url>` and receive clean, parseable markdown with predictable structure — no HTML noise, no ad clutter, just content.

### Constraints

- **Tech stack:** Rust 2024, edition 2024, MSRV 1.85+
- **Timeline:** v1.0 in 2-3 months (incremental releases: 0.3, 0.4, ... 1.0)
- **Binary size:** Target <15MB release binary (exclude debug symbols)
- **Compatibility:** macOS, Linux, Windows (tier 1 platforms)
- **Performance:** Must be faster than mdurl for typical webpage fetch+convert
- **Dependencies:** Minimize dependency count, avoid unmaintained crates
- **License:** MIT (matching mdurl and template)
<!-- GSD:project-end -->

<!-- GSD:stack-start source:codebase/STACK.md -->
## Technology Stack

## Languages & Runtime
### Rust
- **Edition:** 2024
- **MSRV:** 1.85+
- **Compilation:** Standard Rust toolchain with cargo
## Core Framework & Libraries
### CLI Framework
- **clap** `4.5.58` — Command-line argument parsing
- **clap_complete** `4.5.66` — Shell completion generation
### Configuration Management
- **config** `0.15.19` — Layered configuration system
- **directories** `6.0.0` — XDG directory resolution
### Logging & Tracing
- **tracing** `0.1.44` — Structured logging framework
- **tracing-subscriber** `0.3.22` — Logging infrastructure
- **tracing-appender** `0.2.4` — Log rotation
- **tracing-log** `0.2.0` — `log` → `tracing` bridge
### Async Runtime
- **tokio** `1.49.0` — Async runtime
- **tokio-graceful-shutdown** `0.19.2` — Shutdown orchestration
### Error Handling
- **miette** `7.6.0` — Rich diagnostic errors
### Serialization
- **serde** `1.0.228` — Serialization framework
## Development Tools
### Build System
- Standard Cargo with custom `build.rs`
- Pre-compilation shell completion generation
- No build.rs dependencies beyond clap
### Code Quality
- **clippy** — Rust linter
- **rustfmt** — Code formatter
- **cargo-deny** — Dependency auditing
### CI/CD
- **GitHub Actions** workflows:
### Editor Configuration
- **VS Code** settings (`.vscode/`)
- **EditorConfig** (`.editorconfig`) — Cross-editor consistency
### Task Automation
- **just** — Command runner
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
- **Version:** 0.2.0
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
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

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
- **Functions:** Verb-first for actions
- **Modules:** Noun-based for domain
## Module Organization
### File-Module Correspondence
- One module per file: `src/config.rs` → `mod config`
- Subdirectory modules: `src/commands/mod.rs` with re-exports
### Module Re-exports
### Visibility
- Default: Private unless explicitly `pub`
- Commands modules: Functions are `pub` for dispatch from main
- Config types: Public struct, fields public for pattern matching
## Error Handling
### Result Type
### Error Conversion
### Error Context
### No Unwrap/Expect
- ❌ Never use `.unwrap()` or `.expect()` in production paths
- ✅ Use `.unwrap_or_else()` with fallback values for non-critical paths:
## Async Patterns
### Async Function Signatures
### Graceful Shutdown Pattern
### Async Entry Point
#[tokio::main]
## Configuration Patterns
### Layered Config
### Optional Config Files
### CLI Overrides
## Logging Patterns
### Structured Logging
### Log Levels
- `trace` — Detailed execution flow
- `debug` — Developer diagnostics
- `info` — User-facing progress
- `warn` — Recoverable issues
- `error` — Failures requiring attention
### File + Line Annotations
## Build Script Conventions
### CLI Inclusion
### Output Directories
## Dependency Management
### Version Pinning
- Minor versions pinned in Cargo.toml: `"4.5.58"`, `"0.15.19"`
- Updates require explicit Cargo.toml edit + testing
### Feature Flags
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
### String-Based Errors
### Blocking Calls in Async
### Panic in Production
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

## Architectural Pattern
## System Layers
### 1. Entry Point (`src/main.rs`)
```
```
### 2. CLI Layer (`src/cli.rs`)
- `Cli` struct — top-level parser with global options
- `Commands` enum — subcommand variants
- `LogLevel` enum — structured log level values
- `--home` / `-H` — config directory override
- `--log-level` / `-l` — console log level
- `--log-file-enable` — toggle file logging
- `--log-file-path` — log file location
- `--log-file-level` — file log level
- `Command1` — Example async command with countdown
- `Command2 --arg <ARG>` — Example command with arguments
- `Completion --shell <SHELL>` — Shell completion generator
### 3. Configuration Layer (`src/config.rs`)
```
```
```rust
```
- XDG Base Directory for user paths
- `MDGET_HOME` environment variable override
- Optional file sources (no error if missing)
- CLI flag overrides applied last
### 4. Logging Layer (`src/log.rs`)
- Independent level filters per output
- Fallback to `INFO` on invalid level strings
- Non-blocking writer for file output (async-safe)
- `WorkerGuard` return for cleanup coordination
### 5. Command Layer (`src/commands/`)
```rust
```
- `command1.rs` — Countdown demo with cancellation
- `command2.rs` — Argument handling demo
- `completion.rs` — Shell completion output
## Data Flow
### Startup Flow
```
```
### Shutdown Flow
```
```
### Configuration Flow
```
```
## Key Abstractions
### SubsystemHandle
- Signal propagation to async tasks
- Subsystem lifecycle management
- Coordinated cleanup on exit
```rust
```
### Result<T> (miette)
- Consistent error handling
- Rich diagnostic context
- `.into_diagnostic()` conversion from stdlib errors
### Config Hierarchy
## Modularity
### Module Boundaries
| Module | Responsibility | Exports |
|--------|---------------|---------|
| `cli` | CLI parsing | `Cli`, `Commands`, `LogLevel` |
| `config` | Config loading | `GlobalConfig` |
| `log` | Logging setup | `configure_log()` |
| `commands/*` | Command implementations | `run()` functions |
- `main` imports all modules
- `config` depends on `cli` (reads CLI struct)
- `log` depends on `config` (reads log config)
- Commands depend on `miette::Result` and `SubsystemHandle`
## Extensibility Points
### Adding New Commands
### Adding Config Sections
### Adding Logging Sinks
## Current State vs Vision
<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->
## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, `.github/skills/`, or `.codex/skills/` with a `SKILL.md` index file.
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
