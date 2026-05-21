# Architecture

**Mapped:** 2026-05-21  
**Project:** mdget

## Architectural Pattern

**CLI Application with Graceful Shutdown** — Command-based architecture with subsystem coordination for async task management and signal handling.

## System Layers

### 1. Entry Point (`src/main.rs`)
```
CLI Parsing → Config Loading → Logging Setup → Command Dispatch → Shutdown Coordination
```

**Flow:**
1. Parse CLI arguments (`Cli::parse()`)
2. Load layered configuration (`GlobalConfig::new()`)
3. Configure logging infrastructure
4. Match command and dispatch to subsystem
5. Handle signals (SIGINT, SIGTERM) with 1-second grace period

**Key abstraction:** Each command runs as an isolated subsystem managed by `tokio-graceful-shutdown`

### 2. CLI Layer (`src/cli.rs`)
Declarative command-line interface using clap derive macros.

**Structure:**
- `Cli` struct — top-level parser with global options
- `Commands` enum — subcommand variants
- `LogLevel` enum — structured log level values

**Global options:**
- `--home` / `-H` — config directory override
- `--log-level` / `-l` — console log level
- `--log-file-enable` — toggle file logging
- `--log-file-path` — log file location
- `--log-file-level` — file log level

**Commands:**
- `Command1` — Example async command with countdown
- `Command2 --arg <ARG>` — Example command with arguments
- `Completion --shell <SHELL>` — Shell completion generator

### 3. Configuration Layer (`src/config.rs`)
Hierarchical configuration management with serde deserialization.

**Config hierarchy (lowest to highest priority):**
```
Compile defaults → /etc/mdget/config.toml → ~/.config/mdget/config.toml → MDGET_* env vars → CLI flags
```

**Config structure:**
```rust
GlobalConfig {
    log: Log {
        level: String,
        file: LogFile {
            enabled: bool,
            path: String,
            level: String
        }
    }
}
```

**Key patterns:**
- XDG Base Directory for user paths
- `MDGET_HOME` environment variable override
- Optional file sources (no error if missing)
- CLI flag overrides applied last

### 4. Logging Layer (`src/log.rs`)
Dual-output structured logging with tracing subscriber composition.

**Outputs:**
1. **Stdout layer:** Colored, file+line annotations
2. **File layer:** Plain text, daily rolling (`rolling.log`)

**Configuration:**
- Independent level filters per output
- Fallback to `INFO` on invalid level strings
- Non-blocking writer for file output (async-safe)
- `WorkerGuard` return for cleanup coordination

### 5. Command Layer (`src/commands/`)
Command implementation modules with subsystem lifecycle.

**Pattern:**
```rust
pub async fn run(subsys: &mut SubsystemHandle) -> Result<()> {
    // Command logic with graceful shutdown support
    tokio::select! {
        _ = subsys.on_shutdown_requested() => { /* cleanup */ },
        _ = command_logic() => { /* normal completion */ }
    }
    Ok(())
}
```

**Current commands:**
- `command1.rs` — Countdown demo with cancellation
- `command2.rs` — Argument handling demo
- `completion.rs` — Shell completion output

## Data Flow

### Startup Flow
```
main()
  └─> Parse CLI (clap)
      └─> Load config (config-rs hierarchy)
          └─> Setup logging (tracing-subscriber)
              └─> Dispatch command
                  └─> Toplevel::new() creates shutdown coordinator
                      └─> SubsystemBuilder::start() launches command
                          └─> Command runs until completion or signal
```

### Shutdown Flow
```
Signal (SIGINT/SIGTERM)
  └─> Toplevel::catch_signals() intercepts
      └─> Subsystem notified via SubsystemHandle
          └─> Command receives on_shutdown_requested()
              └─> Command cleanup + exit
                  └─> 1-second timeout enforced
```

### Configuration Flow
```
CLI flags → Config builder
             ↓
Environment vars (MDGET_*) → Config builder
                              ↓
User config file (~/.config/mdget/config.toml) → Config builder
                                                  ↓
System config (/etc/mdget/config.toml) → Config builder
                                          ↓
Compile-time defaults → Config builder
                        ↓
                   GlobalConfig struct
```

## Key Abstractions

### SubsystemHandle
Graceful shutdown coordination primitive from `tokio-graceful-shutdown`.

**Responsibilities:**
- Signal propagation to async tasks
- Subsystem lifecycle management
- Coordinated cleanup on exit

**Usage pattern:**
```rust
tokio::select! {
    _ = subsys.on_shutdown_requested() => { /* cancel work */ },
    result = do_work() => { /* normal completion */ }
}
```

### Result<T> (miette)
All fallible operations return `miette::Result<T>` for:
- Consistent error handling
- Rich diagnostic context
- `.into_diagnostic()` conversion from stdlib errors

### Config Hierarchy
Explicit priority chain for configuration values. Later sources override earlier ones:
1. Defaults (compile-time)
2. System config (`/etc/mdget/`)
3. User config (`~/.config/mdget/`)
4. Environment variables (`MDGET_*`)
5. CLI flags

## Modularity

### Module Boundaries

| Module | Responsibility | Exports |
|--------|---------------|---------|
| `cli` | CLI parsing | `Cli`, `Commands`, `LogLevel` |
| `config` | Config loading | `GlobalConfig` |
| `log` | Logging setup | `configure_log()` |
| `commands/*` | Command implementations | `run()` functions |

**Coupling:**
- `main` imports all modules
- `config` depends on `cli` (reads CLI struct)
- `log` depends on `config` (reads log config)
- Commands depend on `miette::Result` and `SubsystemHandle`

**No circular dependencies.** Clean top-down flow from main.

## Extensibility Points

### Adding New Commands
1. Create `src/commands/new_command.rs` with `pub async fn run(subsys: &mut SubsystemHandle) -> Result<()>`
2. Add variant to `Commands` enum in `cli.rs`
3. Add match arm in `main.rs` to dispatch

### Adding Config Sections
1. Add field to `GlobalConfig` struct
2. Set defaults in `Config::builder()`
3. Access via `global_config.new_section`

### Adding Logging Sinks
1. Create new layer in `log.rs`
2. Compose with `registry().with(new_layer)`
3. Add config options for new sink

## Current State vs Vision

**What exists:** CLI scaffolding, config system, logging, graceful shutdown
**What's missing:** HTTP client functionality (the actual "curl alternative" features)

The architecture is **prepared for** but does not yet **implement** HTTP request handling. Commands are placeholders.

---
*Architecture mapped: 2026-05-21*
