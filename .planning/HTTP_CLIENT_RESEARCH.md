# Rust HTTP Client Comparison for mdget

**Research Date:** May 2026
**Context:** Selecting a native Rust HTTP client for mdget Phase 1 (core HTTP fetch + markdown conversion)

**Constraints:**

- Binary size: <15MB (release, no debug symbols)
- Fast startup (agent-first, CLI-based)
- Good HTTP/2 support
- Clean redirect tracking/headers
- Native Rust (no libcurl bindings)

---

## Comparison Table

| Aspect                 | reqwest               | hyper          | ureq          | attohttpc     | isahc         | curl Rust  |
| ---------------------- | --------------------- | -------------- | ------------- | ------------- | ------------- | ---------- |
| Latest Version         | 0.12.x                | 1.x            | 3.x           | 0.30.x        | 1.x           | 0.4.x      |
| API Type               | Async (blocking mode) | Low-level HTTP | Blocking      | Blocking      | Async         | C bindings |
| Startup Speed          | Good                  | N/A            | Excellent     | Excellent     | Very good     | Slower     |
| HTTP/1.1               | ✅                     | ✅              | ✅             | ✅             | ✅             | ✅          |
| HTTP/2                 | ✅                     | ✅              | ❌             | ❌             | ✅             | ✅          |
| Redirect Handling      | ✅ Built-in            | ❌ Manual       | ✅ Built-in    | ✅ Built-in    | ✅ Built-in    | ✅ Built-in |
| Binary Impact          | ~2-3MB                | ~0.5MB         | ~0.4MB        | ~0.3MB        | ~1.5MB        | ~3-5MB     |
| Dependencies           | 20+                   | ~5             | ~5            | ~3-4          | ~10           | 1          |
| TLS Default            | rustls+system         | None           | rustls+system | rustls+system | rustls+system | libcurl    |
| Async Runtime Required | tokio/async-std       | Yes            | No            | No            | tokio         | No         |
| Maturity               | ⭐⭐⭐⭐⭐                 | ⭐⭐⭐⭐⭐          | ⭐⭐⭐⭐          | ⭐⭐⭐           | ⭐⭐⭐           | ⭐⭐⭐⭐       |
| Community              | Excellent             | Excellent      | Good          | Small         | Small         | Mature     |

---

## Detailed Library Profiles

### 1. reqwest

**Latest:** 0.12.x | **Benchmark Score:** 80.1/100 | **Code Snippets:** 9850+

#### Overview

Most popular Rust HTTP client. Available in async-first mode and blocking mode (`reqwest::blocking`). Excellent ecosystem and documentation.

#### Strengths

- Most popular Rust HTTP client for CLI/agent use
- Async-first with optional blocking API
- Excellent HTTP/2 support via h2 crate
- Clean, intuitive redirect handling: `response.url()` contains final URL
- Connection pooling built-in
- Rich middleware ecosystem
- Well-documented with extensive examples

#### Weaknesses

- Large dependency tree (~20+ crates) → heavier binary
- Async overhead for simple blocking use cases
- Startup time slightly slower than pure blocking clients
- Requires tokio runtime (even for blocking API in some contexts)

#### Binary Impact

Estimated 2-3MB added to release binary. Factors: HTTP/2 support (h2 crate), TLS (rustls), feature flags.

#### HTTP/2 & Redirects

- **HTTP/2:** ✅ Full support via h2 crate
- **Redirects:** ✅ Automatic, follows up to 20 by default, configurable
- **Redirect Tracking:**

```rust
let resp = client.get(url).send().await?;
let final_url = resp.url(); // Tracks through redirects
let status = resp.status();
```

#### Phase 1 Recommendation

⭐ **RECOMMENDED** if async is acceptable. Best for modern Rust practices and best community support. Accept slightly larger binary.

---

### 2. hyper

**Latest:** 1.x | **Code Snippets:** 5200+

#### Overview

Ultra-lightweight, low-level HTTP library. Powers reqwest internally. Designed as a building block for library authors, not end-user applications.

#### Strengths

- Ultra-lightweight, low-level HTTP library
- Powers reqwest internally
- Minimal dependencies (~5)
- Excellent HTTP/2 support
- Blazing fast compilation
- Very small binary impact

#### Weaknesses

- NOT a finished HTTP client — it's a building block
- No automatic redirect handling (manual implementation required)
- No cookie jar, proxy handling, middleware by default
- Requires writing significant boilerplate for a CLI tool
- Designed for library authors, not end-user applications
- Steeper learning curve

#### Binary Impact

Minimal at ~0.5MB. Trade-off: Must implement own redirect handling, cookie logic, etc.

#### HTTP/2 & Redirects

- **HTTP/2:** ✅ Full support
- **Redirects:** ❌ Manual (no built-in handling)

```rust
// Must manually handle Location header and retry
let resp = client.request(req).await?;
if resp.status().is_redirect() {
    let location = resp.headers().get("location")?;
    // Handle redirect manually...
}
```

#### Phase 1 Recommendation

⭐ **NOT RECOMMENDED** — Would require building significant infrastructure. Better for downstream libraries that wrap hyper.

---

### 3. ureq

**Latest:** 3.x | **Benchmark Score:** 87.33/100 | **Code Snippets:** 1953+

#### Overview

Minimal dependencies, blocking I/O only. Extremely lightweight and focused. Pure Rust, no C dependencies.

#### Strengths

- Minimal dependencies (~5 crates)
- Blocking I/O only (simpler model for CLI tools)
- Extremely lightweight and focused
- Excellent redirect handling with clean API
- Built-in cookie jar
- Built-in proxy support
- Very fast startup (no async runtime needed)
- Small binary footprint
- Pure Rust, no C dependencies

#### Weaknesses

- Blocking only — no concurrency support
- No HTTP/2 support (HTTP/1.1 only)
- Smaller community than reqwest
- Limited async ecosystem (not issue for Phase 1, but future scaling?)
- Less feature-rich for advanced use cases

#### Binary Impact

Minimal at ~0.4-0.5MB. Reason: Core functionality only, HTTP/1.1 only, minimal crypto overhead.

#### HTTP/2 & Redirects

- **HTTP/2:** ❌ Not supported (HTTP/1.1 only)
- **Redirects:** ✅ Automatic, follows up to 5 by default, configurable
- **Redirect Tracking:**

```rust
let resp = ureq::get(url)
    .set("User-Agent", "mdget")
    .call()?;
let final_url = resp.get_url(); // Tracks through redirects
```

#### Phase 1 Recommendation

⭐⭐ **GOOD FOR PHASE 1** if HTTP/2 is deferred to Phase 2+. Perfect binary size constraints, simple blocking model, fast startup.

**Risk:** HTTP/2 sites may be slower (HTTP/1.1 coalesces into fewer connections).

---

### 4. attohttpc

**Latest:** 0.30.x | **Code Snippets:** 856+

#### Overview

Ultra-minimal dependency footprint. Blocking I/O, very fast startup, extremely small binary impact.

#### Strengths

- Ultra-minimal dependency footprint
- Blocking I/O (simple for CLI)
- Very fast startup
- Extremely small binary impact
- JSON support built-in
- Multiple TLS implementations available
- Pure Rust

#### Weaknesses

- No HTTP/2 support
- Smaller community than ureq
- Less actively maintained (check last commit date)
- Minimal documentation
- Fewer features than ureq
- Not as battle-tested

#### Binary Impact

Minimal at ~0.3-0.4MB (smallest among tested options).

#### HTTP/2 & Redirects

- **HTTP/2:** ❌ Not supported
- **Redirects:** ✅ Automatic handling

#### Phase 1 Recommendation

⭐ **MINIMAL** — Interesting for minimalism, but smaller community means fewer examples/help. Fewer corner cases tested. Consider ureq instead.

---

### 5. isahc

**Latest:** 1.x | **Status:** Active

#### Overview

Async-first with HTTP/2 support. Smaller dependency tree than reqwest. Built on hyper but with higher-level API.

#### Strengths

- Async-first (like reqwest)
- HTTP/2 support via h2
- Smaller dependency tree than reqwest
- Built on hyper but with higher-level API
- Good redirect handling
- Moderate binary impact (lighter than reqwest)

#### Weaknesses

- Smaller community than reqwest
- Less documentation
- Less battle-tested in production
- Async runtime required (tokio)
- Not quite as ergonomic as reqwest

#### Binary Impact

Estimated 1.5-2MB (middle ground).

#### HTTP/2 & Redirects

- **HTTP/2:** ✅ Full support
- **Redirects:** ✅ Automatic handling

#### Phase 1 Recommendation

⭐ **POSSIBLE** — If you want async + smaller binary than reqwest. Good compromise but smaller community = less support for edge cases.

---

### 6. curl Rust Bindings

**Latest:** 0.4.x | **Status:** Mature

#### Overview

Bindings to C libcurl. NOT native Rust.

#### Why NOT for mdget

- **NOT native Rust** — Bindings to C libcurl via FFI
- Requires libcurl system dependency (violates "pure Rust" goal)
- Binary size: 3-5MB + system libcurl (~2MB minimum)
- Slower startup (C library initialization)
- Against mdget's "pure Rust agent" philosophy
- More complex cross-platform distribution
- Build complexity increases

---

## Recommendation Matrix

### For Phase 1 (v0.3)

| Scenario                            | Recommended        | Rationale                                                                                       |
| ----------------------------------- | ------------------ | ----------------------------------------------------------------------------------------------- |
| Prioritize HTTP/2 + Best DX         | reqwest (blocking) | Most documented, best redirect handling, HTTP/2. Accept larger binary.                          |
| Prioritize Binary Size + Simplicity | ureq               | Perfect for pure blocking use case, minimal deps, excellent redirects. Defer HTTP/2 to Phase 2. |
| Balance (HTTP/2 + Smaller Binary)   | isahc              | Async with smaller footprint than reqwest. Good middle ground.                                  |
| Absolutely Minimal                  | attohttpc          | If <1MB is critical. Risk: smaller community.                                                   |

---

## Final Recommendation for mdget Phase 1

### 🏆 Primary Recommendation: ureq (blocking)

**Why ureq is best for Phase 1:**

1. **Binary Size:** ~0.4MB vs. reqwest's ~2-3MB (massive difference for <15MB target)
2. **Startup Speed:** No async runtime overhead, instant startup (agent-friendly)
3. **Simplicity:** Blocking model matches CLI tool pattern perfectly
4. **Features:** All core HTTP features needed for Phase 1:
   - Automatic redirect handling ✅
   - Redirect URL tracking ✅
   - Cookies ✅
   - Proxies ✅
   - TLS/HTTPS ✅
5. **Maturity:** Well-maintained, good community, battle-tested
6. **Pure Rust:** No C dependencies, single-binary delivery
7. **Philosophy:** Aligns with "agent-first, minimal, fast" mdget vision

**Deferral Strategy:**

- Phase 1 (v0.3): ureq (HTTP/1.1, lightweight, fast)
- Phase 2+ (v0.4+): Evaluate HTTP/2 needs
  - If HTTP/2 traffic is significant: Migrate to reqwest or isahc
  - If HTTP/1.1 sufficient: Keep ureq, consider adding optional HTTP/2 via plugin

**Alternative Path (if async required early):**

Use **reqwest blocking mode** (`reqwest::blocking`). Requires tokio, but simplifies later migration to full async. Accept ~2MB binary cost.

---

## Implementation Notes

### ureq Setup (Recommended)

```toml
[dependencies]
ureq = "3"
```

```rust
let client = ureq::Agent::new();
let response = client
    .get("https://example.com")
    .set("User-Agent", "mdget/0.3.0")
    .call()?;

let final_url = response.get_url(); // After redirects
let body = response.into_string()?;
```

### reqwest::blocking Alternative

```toml
[dependencies]
reqwest = { version = "0.12", features = ["blocking"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

```rust
let client = reqwest::blocking::Client::new();
let response = client
    .get("https://example.com")
    .header("User-Agent", "mdget/0.3.0")
    .send()?;

let final_url = response.url().to_string(); // After redirects
let body = response.text()?;
```

---

## Decision Timeline

**By Phase 1 completion:**

- ✅ Verify ureq binary size after real implementation
- ✅ Benchmark redirect handling on 50+ test URLs
- ✅ Test proxy scenarios
- ⚠️ Flag HTTP/2 performance issues (if found)
- 📋 Plan Phase 2 HTTP/2 migration if needed

---

## References

- [ureq on crates.io](https://crates.io/crates/ureq)
- [reqwest on crates.io](https://crates.io/crates/reqwest)
- [hyper on crates.io](https://crates.io/crates/hyper)
- [isahc on crates.io](https://crates.io/crates/isahc)
- [attohttpc on crates.io](https://crates.io/crates/attohttpc)
