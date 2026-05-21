# Directory Structure

**Mapped:** 2026-05-21  
**Project:** mdget

## Root Layout

```
mdget/
├── src/                    # Source code
├── config/                 # Default configuration
├── .github/                # CI/CD workflows
├── .vscode/                # VS Code settings
├── completions/            # Generated shell completions (build artifact)
├── target/                 # Cargo build output
├── IGNORE/                 # User scratch/notes (ignored by git)
├── Cargo.toml              # Package manifest
├── build.rs                # Build script
├── justfile                # Task automation
├── clippy.toml             # Linter config
├── rustfmt.toml            # Formatter config
├── deny.toml               # Dependency policy
├── dist-workspace.toml     # Distribution config
├── .editorconfig           # Editor consistency
├── .gitignore              # Git ignore rules
├── LICENSE                 # MIT license
├── README.md               # Project documentation
├── CODE_OF_CONDUCT.md      # Community guidelines
└── CONTRIBUTING.md         # Contribution guide
```

## Source Directory (`src/`)

```
src/
├── main.rs                 # Application entry point
├── cli.rs                  # CLI definition (clap)
├── config.rs               # Configuration management
├── log.rs                  # Logging setup
└── commands/               # Command implementations
    ├── mod.rs              # Commands module re-exports
    ├── command1.rs         # Example command 1
    ├── command2.rs         # Example command 2
    └── completion.rs       # Shell completion generator
```

### File Purposes

| File | Lines | Purpose |
|------|-------|---------|
| `main.rs` | 49 | Entry point, command dispatch, shutdown coordination |
| `cli.rs` | 74 | CLI structure (Cli, Commands, LogLevel) |
| `config.rs` | 116 | Layered config loading, XDG paths |
| `log.rs` | 77 | Tracing setup, dual output (stdout + file) |
| `commands/mod.rs` | 3 | Module re-exports |
| `commands/command1.rs` | 27 | Example: countdown with graceful cancel |
| `commands/command2.rs` | ~20 | Example: argument handling |
| `commands/completion.rs` | ~15 | Shell completion output |

## Configuration Directory (`config/`)

```
config/
└── config.toml             # Default configuration template
```

**Runtime config locations:**
- System: `/etc/mdget/config.toml`
- User: `~/.config/mdget/config.toml` (XDG)
- Override: `$MDGET_HOME/config.toml`

## CI/CD Directory (`.github/workflows/`)

```
.github/workflows/
├── ci.yml                  # Build, test, clippy, fmt
├── security-audit.yml      # cargo-audit for CVEs
├── release.yml             # Binary releases
└── cargo-deny.yml          # Dependency policy checks
```

## Key Files

### Package Manifest (`Cargo.toml`)
- Package metadata (name, version, description, license)
- Dependencies with version pinning
- Binary target configuration
- Rust edition 2024, MSRV 1.85

### Build Script (`build.rs`)
- Includes `src/cli.rs` to access CLI structure
- Generates shell completions at compile time
- Output: `completions/` directory (git-ignored)

### Task Automation (`justfile`)
Common development tasks:
- `just check` — Local CI checks
- `just release` — Build release binaries
- `just publish` — Publish after checks

### Linter Config (`clippy.toml`)
Project-specific clippy rules and warnings.

### Formatter Config (`rustfmt.toml`)
Rust formatting preferences (edition 2024).

### Dependency Policy (`deny.toml`)
- License allowlist
- Security vulnerability checks
- Duplicate dependency detection
- Unmaintained crate warnings

## Build Artifacts

### Ignored by Git (`.gitignore`)
```
target/                     # Cargo build output
debug                       # Debug builds
**/*.rs.bk                  # rustfmt backups
*.pdb                       # MSVC debug symbols
ACTIVE_CONTEXT.md           # Local scratch file
**/mutants.out*/            # Mutation testing data
```

### Generated at Build Time
```
completions/                # Shell completions
    ├── mdget.bash
    ├── mdget.fish
    ├── _mdget              # zsh
    └── _mdget.ps1          # PowerShell
```

## Naming Conventions

### Files
- Rust source: `snake_case.rs`
- Config files: `kebab-case.toml`
- Documentation: `UPPERCASE.md` or `README.md`

### Modules
- Module names: `snake_case`
- Re-export pattern: `mod.rs` in subdirectories

### Directories
- Source code: `src/`
- Submodules: `src/commands/`, `src/utils/` (future)
- Config: `config/`
- CI: `.github/workflows/`

## Key Locations

### Source Code Entry Points
- Main binary: `src/main.rs`
- CLI definition: `src/cli.rs` (also used by `build.rs`)
- Command dispatch: `src/main.rs` match statement (lines 23-46)

### Configuration
- Default config: `config/config.toml`
- User config: `~/.config/mdget/config.toml` (runtime)
- Env prefix: `MDGET_*`

### Logging
- Default log location: `~/.cache/mdget/rolling.log` (XDG cache dir)
- Configurable via `log.file.path` in config or `--log-file-path` flag

### Build System
- Compilation: `cargo build`
- Completion generation: `build.rs` → `completions/`
- Task runner: `just <task>`

## Extensibility Structure

### Adding New Commands
1. Create `src/commands/new_command.rs`
2. Add `pub mod new_command;` to `src/commands/mod.rs`
3. Import in `src/main.rs`
4. Add variant to `Commands` enum in `src/cli.rs`
5. Add dispatch logic in `src/main.rs` match

### Adding New Config Sections
1. Add struct/fields to `src/config.rs`
2. Set defaults in `GlobalConfig::new()`
3. Document in `config/config.toml`

### Adding New Modules
- Functional modules: `src/<name>.rs`
- Module groups: `src/<name>/mod.rs` + `src/<name>/<submodule>.rs`

## IGNORE Directory

**Purpose:** Local scratch space for development notes, experiments, prototypes.

**Current contents:**
- `IGNORE/mdurl-main/` — Appears to be markdown URL parsing library (external project reference)

**Convention:** Add personal notes, context files, or experimental code here. Git-ignored to prevent accidental commits.

---
*Structure mapped: 2026-05-21*
