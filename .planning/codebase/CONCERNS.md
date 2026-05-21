# Technical Concerns

**Mapped:** 2026-05-21  
**Project:** mdget

## Summary

This is a **template project** transitioning to a production tool. The codebase is clean scaffolding but has **significant functionality gaps** and **template remnants** that need addressing before the project delivers on its "agent-first curl alternative" promise.

## Critical Concerns

### 1. Missing Core Functionality (CRITICAL)
**Severity:** CRITICAL  
**Impact:** Project does not yet implement its stated purpose

**Issue:**
- Cargo.toml describes mdget as "Agent-first curl alternative"
- No HTTP client implementation exists
- No request/response handling
- No agent/LLM integration logic
- Commands are placeholder examples (countdown timers, not HTTP operations)

**Evidence:**
- `src/commands/command1.rs` — 9-second countdown, not HTTP request
- `src/commands/command2.rs` — Same countdown with optional arg
- No `reqwest`, `hyper`, or other HTTP client in dependencies

**Recommendation:**
1. Design HTTP client interface (sync vs async, streaming, etc.)
2. Add HTTP client dependency (`reqwest` recommended for ease)
3. Implement core request commands (GET, POST, etc.)
4. Define "agent-first" functionality (LLM interpretation? Context extraction?)
5. Replace placeholder commands with real HTTP operations

**Priority:** Must fix before v0.2.0 can be used

---

### 2. Template Placeholder Commands (MAJOR)
**Severity:** MAJOR  
**Impact:** User confusion, no production value

**Issue:**
Current commands are scaffolding from rust-cli-template:
- `command1` — Countdown timer with graceful cancellation
- `command2` — Countdown timer with optional string argument
- Neither command relates to HTTP requests or curl functionality

**Evidence:**
```rust
// src/commands/command1.rs
async fn countdown() {
    for i in (1..10).rev() {
        info!("command1 countdown: {}", i);
        sleep(Duration::from_millis(1000)).await;
    }
}
```

This is demo code from the template, not curl-alternative logic.

**Recommendation:**
1. Delete `command1.rs` and `command2.rs` entirely
2. Create HTTP operation commands:
   - `get.rs` — HTTP GET
   - `post.rs` — HTTP POST
   - `put.rs`, `delete.rs`, etc.
3. Update `Commands` enum in `cli.rs` to reflect HTTP verbs
4. Update README with actual usage examples

**Priority:** High — confusing for users trying to understand the project

---

### 3. No Tests (MAJOR)
**Severity:** MAJOR  
**Impact:** No quality assurance, refactoring risk

**Issue:**
- Zero tests written (`cargo test` would show 0 tests)
- No test coverage reporting
- No CI enforcement of test coverage
- Async shutdown logic untested (complex, easy to break)
- Config layering untested (subtle precedence bugs possible)

**Recommendation:**
1. Add unit tests for `config.rs` (layering hierarchy)
2. Add unit tests for `log.rs` (level fallback logic)
3. Add integration tests for graceful shutdown
4. Set CI target: 80% coverage minimum
5. Add `cargo-tarpaulin` or `cargo-llvm-cov` to CI

**Priority:** High — technical debt accumulates without tests

---

### 4. IGNORE/ Directory Contents (MINOR)
**Severity:** MINOR  
**Impact:** Cleanup, organization

**Issue:**
- `IGNORE/mdurl-main/` contains what appears to be a markdown URL parsing library
- Unclear purpose (research? vendored dependency? abandoned experiment?)
- Not documented in README

**Evidence:**
```
IGNORE/
└── mdurl-main/  # External project or scratchpad?
```

**Recommendation:**
1. If `mdurl-main` is needed: Document its purpose in README
2. If it's research: Move to docs or delete after extracting insights
3. If it's a dependency: Vendor properly or add to Cargo.toml
4. If it's abandoned: Delete

**Priority:** Low — doesn't affect functionality, but confusing for contributors

---

### 5. SSL Certificate Issue (ENVIRONMENT)
**Severity:** ENVIRONMENT  
**Impact:** Cannot build on this machine without cert fix

**Issue:**
`cargo tree` and other cargo commands fail with SSL certificate error:
```
SSL certificate problem: unable to get local issuer certificate
```

This is an **environment issue**, not a code issue. The local machine's SSL certificates are not configured for HTTPS connections to crates.io.

**Recommendation:**
1. Update system CA certificates
2. Or set `CARGO_HTTP_CAINFO` environment variable to point to valid cert bundle
3. Or use HTTP fallback (not recommended for security)

**Priority:** Low for codebase, HIGH for this developer's workflow

---

## Quality Concerns

### 1. Dependency Version Pinning (INFO)
**Severity:** INFO  
**Impact:** Update friction

**Observation:**
Dependencies use exact minor version pinning:
```toml
clap = "4.5.58"
tokio = "1.49.0"
```

**Tradeoff:**
- ✅ Reproducible builds
- ✅ No surprise breakage
- ❌ Manual updates required for patches
- ❌ Security fixes require explicit version bump

**Recommendation:**
- Keep current approach for stability
- Add Dependabot config to auto-PR security updates
- Or switch to caret requirements (`^4.5`) for minor/patch updates

**Priority:** Low — acceptable for small project

---

### 2. No Documentation Comments (INFO)
**Severity:** INFO  
**Impact:** Onboarding friction

**Observation:**
- Public functions lack doc comments (`///`)
- Modules lack module-level docs (`//!`)
- No examples in docs

**Current state:**
```rust
pub async fn run(subsys: &mut SubsystemHandle) -> Result<()> {
    // No doc comment explaining what this command does
}
```

**Recommendation:**
1. Add module-level docs to `src/cli.rs`, `src/config.rs`, `src/log.rs`
2. Add doc comments to public functions
3. Run `cargo doc --open` to verify rendered docs
4. Add examples in doc comments

**Priority:** Low for now, MEDIUM before v1.0

---

### 3. Config File Locations Not Documented (INFO)
**Severity:** INFO  
**Impact:** User confusion

**Observation:**
Config system supports multiple locations:
- `/etc/mdget/config.toml`
- `~/.config/mdget/config.toml`
- `$MDGET_HOME/config.toml`

But README doesn't document:
- Which location takes precedence
- What keys are available
- Example config file contents

**Recommendation:**
1. Add "Configuration" section to README
2. Document precedence chain
3. Include example `config.toml` with all options commented
4. Document `MDGET_*` environment variables

**Priority:** Low — template project, not production yet

---

## Performance Concerns

### 1. Tokio "full" Feature Set (INFO)
**Severity:** INFO  
**Impact:** Binary size, compile time

**Observation:**
```toml
tokio = { version = "1.49.0", features = ["full"] }
```

`features = ["full"]` includes all tokio components, even if unused. This increases:
- Binary size (~500KB+ larger)
- Compile time (all features compiled)
- Attack surface (more code linked)

**Current usage:**
- `#[tokio::main]` — needs `rt-multi-thread`
- `tokio::select!` — needs `macros`
- `tokio::time::sleep` — needs `time`
- Graceful shutdown — needs `signal` and `sync`

**Recommendation:**
1. Audit actual tokio usage
2. Replace `full` with explicit features:
   ```toml
   tokio = { version = "1.49.0", features = ["rt-multi-thread", "macros", "time", "signal", "sync"] }
   ```
3. Benchmark binary size before/after

**Priority:** Low — optimize later if binary size becomes concern

---

## Security Concerns

### 1. No Input Validation Yet (WATCH)
**Severity:** WATCH  
**Impact:** Depends on future HTTP implementation

**Observation:**
Once HTTP client is added, the project will handle:
- URLs (potential injection vectors)
- Headers (potential CRLF injection)
- Request bodies (potential overflow)
- Redirects (potential SSRF)

Current code has no validation because it's still template.

**Recommendation:**
When implementing HTTP client:
1. Validate URLs before requests (reject `file://`, `data://` schemes?)
2. Sanitize header values (reject CRLF: `\r\n`)
3. Limit request body size
4. Control redirect behavior (max redirects, schemes allowed)
5. Handle authentication secrets securely (no logging)

**Priority:** CRITICAL for future HTTP implementation

---

### 2. Log File Permissions (INFO)
**Severity:** INFO  
**Impact:** Potential information disclosure

**Observation:**
Log files created in `~/.cache/mdget/rolling.log` with default umask permissions. If mdget logs sensitive data (auth tokens, URLs with secrets, etc.), logs may be world-readable.

**Recommendation:**
1. Set restrictive permissions on log directory:
   ```rust
   fs::create_dir_all(log_dir).into_diagnostic()?;
   #[cfg(unix)]
   std::os::unix::fs::PermissionsExt::from_mode(0o700);
   ```
2. Document that logs may contain sensitive data
3. Add `--no-log-secrets` flag to redact sensitive fields

**Priority:** Low for now (no sensitive data logged yet), MEDIUM for v1.0

---

## Maintenance Concerns

### 1. Dependabot Not Configured (INFO)
**Severity:** INFO  
**Impact:** Security update friction

**Observation:**
`.github/workflows/security-audit.yml` runs `cargo-audit` but doesn't auto-PR fixes. Manual work required to update vulnerable dependencies.

**Recommendation:**
Add `.github/dependabot.yml`:
```yaml
version: 2
updates:
  - package-ecosystem: cargo
    directory: "/"
    schedule:
      interval: weekly
```

**Priority:** Low — project is young, few dependencies

---

### 2. No CHANGELOG (INFO)
**Severity:** INFO  
**Impact:** Release notes gap

**Observation:**
No `CHANGELOG.md` to track version history. Current approach seems to be GitHub releases only.

**Recommendation:**
1. Add `CHANGELOG.md` following Keep a Changelog format
2. Update on each version bump
3. Link from README

**Priority:** Low — start before v0.3.0

---

## Tech Debt Summary

| Concern | Severity | Effort | Priority |
|---------|----------|--------|----------|
| Missing HTTP functionality | CRITICAL | High | **1. Must fix** |
| Template placeholder commands | MAJOR | Low | **2. Cleanup** |
| No tests | MAJOR | Medium | **3. Quality** |
| IGNORE/ directory | MINOR | Low | 4. Cleanup |
| SSL environment issue | ENV | N/A | 5. Local fix |
| No input validation (future) | WATCH | Medium | 6. Design ahead |
| Documentation gaps | INFO | Low | 7. Polish |

---
*Concerns mapped: 2026-05-21*
*Next review: After HTTP client implementation*
