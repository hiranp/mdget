# Phase 1 Research: HTTP Core & Basic Extraction

**Phase:** 01 - HTTP Core & Basic Extraction
**Researched:** 2026-05-21
**Requirements:** HTTP-01, HTTP-02, HTTP-03, EXTR-01, EXTR-02, EXTR-03, EXTR-04, OUT-01, OUT-05

## Research Question

"What do I need to know to PLAN this phase well?"

## Executive Summary

Phase 1 builds the foundation: HTTP client → HTML parsing → article extraction → markdown conversion → YAML frontmatter output. The existing CLI template provides solid scaffolding (clap, config, logging, graceful shutdown). We need to add:

1. HTTP client (`reqwest 0.12` — pure Rust, async-native)
2. HTML parsing (scraper crate)
3. Readability-style article extraction (custom implementation)
4. Markdown generation (custom renderer)
5. YAML frontmatter serialization (`serde_yml` — successor to deprecated `serde_yaml`)
6. New `fetch` subcommand

**Key decision (updated 2026-05-21):** Use `reqwest 0.12` (pure Rust async) instead of `curl` crate (libcurl FFI). See Section 1 and Appendix A for full rationale.

---

## 1. HTTP Client Selection

### Decision: `reqwest 0.12` (updated 2026-05-21)

After researching the full Rust HTTP client ecosystem (see Appendix A for the complete comparison), we are switching from the `curl` crate to `reqwest 0.12`.

**Rationale for switching:**

| Factor | `curl` crate | `reqwest 0.12` |
|--------|-------------|----------------|
| **Purity** | FFI wrapper around libcurl (C) | 100% pure Rust (hyper + rustls) |
| **Async** | Blocking; requires `spawn_blocking` | Native async/await — no wrapper needed |
| **Binary portability** | Links system libcurl (may not be present) | Self-contained; no system deps |
| **Binary size** | ~1.5MB FFI overhead + system libcurl | ~2-3MB with `default-features=false, rustls-tls` |
| **Redirect chain** | Manual via header callbacks | `response.url()` = final URL; built-in |
| **HTTP/2** | Yes (via libcurl) | Yes (enabled by default) |
| **MSRV** | N/A (system libcurl) | 1.63+ (well below our 1.85) |
| **Maintenance** | Mature but FFI-dependent | Actively maintained, largest Rust HTTP ecosystem |

**Why `reqwest` beats `curl` for this project:**
- We already use `tokio`. `ureq` (blocking) in a tokio runtime still requires `spawn_blocking` — same boilerplate as `curl`, but without reqwest's features. There is no binary size advantage to `ureq` over `reqwest` once `spawn_blocking` overhead is included.
- `reqwest 0.12` with `default-features = false, features = ["rustls-tls"]` produces binaries in the **4–6MB** range (measured community benchmarks), well under our 15MB target.
- Pure Rust means no system `libcurl` dependency — the binary works on any target without OS-level prerequisites.
- `response.url()` gives the final URL after redirects out of the box; building this with curl requires a manual header-capture callback.
- Rust 2024 edition and MSRV 1.85 — both fully compatible.

**Integration:**
```toml
[dependencies]
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "charset", "http2"] }
```

Feature flags:
- `default-features = false` — drops `native-tls` (OpenSSL), cookies, multipart, etc.
- `rustls-tls` — pure Rust TLS; no system OpenSSL dependency
- `charset` — automatic charset detection from Content-Type header (built-in; replaces our manual encoding_rs scan for the common case)
- `http2` — HTTP/2 enabled (on by default but explicit is clearer)

**Key patterns:**
```rust
// Build client once, reuse for connection pooling
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(timeout_secs))
    .redirect(reqwest::redirect::Policy::limited(max_redirects as usize))
    .user_agent(user_agent)
    .build()?;

// Fetch — native async, no spawn_blocking needed
let response = client.get(&url).send().await?;
let final_url = response.url().to_string();  // post-redirect URL
let status    = response.status().as_u16();
let body      = response.bytes().await?;     // Vec<u8> equivalent
```

**Redirect chain tracking:**
reqwest's `redirect::Policy::limited(n)` follows up to n redirects automatically. The final URL is available via `response.url()`. For full redirect history (all intermediate URLs), use a custom redirect policy that captures each hop — see Task 3 implementation.

**Alternatives fully considered (see Appendix A):**
- `curl` (0.4): Dropped — libcurl FFI, requires spawn_blocking, system libcurl dependency
- `ureq` (3.0): Not chosen — blocking-only; excellent for non-async tools, but we're already in tokio. MSRV 1.85 matches, but the blocking model is a mismatch.
- `hyper` (1.x): Low-level building block; reqwest is built on it — use reqwest instead
- `isahc`: libcurl-based like the `curl` crate; same FFI drawbacks

---

## 2. HTML Parsing & Scraping

### Recommended: `scraper` crate

**Rationale:**
- CSS selector-based API (familiar to web developers)
- Built on `html5ever` (correct HTML5 parsing)
- Clean API: `Html::parse_document`, `Selector::parse`
- Mature crate (widely used in Rust ecosystem)

**Integration:**
```toml
[dependencies]
scraper = "0.20"  # Latest stable
html5ever = "0.27"  # transitive, but explicit for clarity
```

**Key patterns:**
- Parse: `Html::parse_document(&html_string)`
- Extract title: `document.select(&Selector::parse("title").unwrap()).next()`
- Readability extraction: Custom traversal using `scraper::ElementRef` and `Node` APIs

---

## 3. Readability-Style Article Extraction

### Approach: Port core algorithm from mdurl (TypeScript)

**No suitable Rust crate:**
- Existing `readability` crate is unmaintained (last update 2019)
- Better to port proven logic from mdurl for control and maintainability

**Algorithm outline (from mdurl reference):**
1. **Score elements** by content density (text length / tag density ratio)
2. **Identify article node** (highest scoring `<article>`, `<div>`, `<main>`)
3. **Clean unwanted elements** (nav, footer, ads, social widgets)
4. **Extract text blocks** preserving semantic structure (headings, lists, blockquotes)
5. **Return cleaned HTML** for markdown conversion

**Key heuristics:**
- Prefer `<article>`, `<main>` tags (semantic HTML5)
- Penalize `<nav>`, `<footer>`, `<aside>`
- Positive signals: paragraph count, text length, class names like `content`, `article`, `post`
- Negative signals: class names like `comment`, `ad`, `sidebar`, `nav`

**Implementation:**
- Create `src/extract/readability.rs` module
- Struct: `ReadabilityExtractor` with scoring logic
- Public API: `extract_article(html: &Html) -> Option<String>`

---

## 4. Markdown Generation

### Approach: Custom renderer inspired by pulldown-cmark

**No turndown.rs equivalent:**
- `pulldown-cmark` is a markdown *parser* (markdown → HTML), not renderer
- We need the inverse (HTML → markdown)
- Implement custom traversal + markdown emission

**Strategy:**
1. Walk cleaned HTML tree (from Readability step)
2. Emit markdown for each node type:
   - `<h1>` → `# `
   - `<p>` → paragraph with blank line
   - `<a>` → `[text](url)`
   - `<strong>` → `**text**`
   - `<em>` → `*text*`
   - `<ul>`, `<ol>` → list items
   - `<pre>`, `<code>` → fenced code blocks
   - `<blockquote>` → `> ` prefix
3. Handle edge cases (nested lists, inline vs block elements)

**Implementation:**
- Create `src/convert/markdown.rs` module
- Struct: `MarkdownRenderer` with state (current indent level, list depth)
- Public API: `html_to_markdown(html: &str) -> String`

**Reference patterns from mdurl:**
- Turndown library (JavaScript) has clean rules-based approach
- Port the rule set rather than full library

---

## 5. YAML Frontmatter Serialization

### Recommended: `serde_yml` (NOT `serde_yaml`)

**Rationale:**
- `serde_yaml` 0.9 is officially deprecated — its author archived it and published `serde_yml` as the successor
- `serde_yml` receives security patches; `serde_yaml` does not
- API is nearly identical — drop-in replacement
- Same serde integration, handles String, i64, DateTime<Utc>, Option correctly

**Integration:**
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_yml  = "0.0.12"   # successor to deprecated serde_yaml 0.9
chrono = { version = "0.4", features = ["serde"] }
```

**Frontmatter structure:**
```rust
#[derive(Serialize)]
struct Frontmatter {
    url: String,
    title: Option<String>,
    status: u16,
    word_count: usize,
    fetched_at: DateTime<Utc>,
    redirect_chain: Option<Vec<String>>,
}
```

**Output format:**
```yaml
---
url: https://example.com
title: Example Domain
status: 200
word_count: 123
fetched_at: 2026-05-21T19:30:00Z
redirect_chain: null
---

# Example Domain

This domain is for use in illustrative examples...
```

---

## 6. Error Envelope Design

### Structure

**Success envelope:**
```yaml
---
success: true
url: https://example.com
status: 200
title: Example Domain
word_count: 123
fetched_at: 2026-05-21T19:30:00Z
---
[markdown body]
```

**Error envelope:**
```yaml
---
success: false
url: https://example.com
status: 404
error: Not Found
message: The requested URL was not found on this server
fetched_at: 2026-05-21T19:30:00Z
---
```

**Implementation:**
- Use `Result<SuccessEnvelope, ErrorEnvelope>` return types
- Convert both to markdown with YAML frontmatter
- Log errors via tracing for debugging

---

## 7. CLI Integration

### New Subcommand: `fetch`

**Add to `src/cli.rs`:**
```rust
#[derive(Subcommand)]
pub enum Commands {
    // ... existing commands ...
    
    #[command(about = "Fetch URL and convert to markdown")]
    Fetch {
        #[arg(value_name = "URL", help = "URL to fetch")]
        url: String,
        
        #[arg(short, long, help = "Output to file instead of stdout")]
        output: Option<PathBuf>,
        
        #[arg(long, help = "Output JSON envelope instead of YAML frontmatter")]
        json: bool,
        
        #[arg(long, default_value = "30", help = "Request timeout in seconds")]
        timeout: u64,
        
        #[arg(long, default_value = "5", help = "Maximum redirects to follow")]
        max_redirects: u32,
    },
}
```

**Implementation flow:**
```
1. Parse CLI args
2. Configure HTTP client (timeout, redirects, user-agent)
3. Fetch URL
4. Parse HTML
5. Extract article
6. Convert to markdown
7. Serialize frontmatter
8. Output (stdout or file)
```

---

## 8. Module Structure

### New files to create:

```
src/
├── fetch/
│   ├── mod.rs          # Public API
│   ├── http.rs         # HTTP client wrapper (curl)
│   ├── extract.rs      # Readability article extraction
│   ├── convert.rs      # HTML → markdown conversion
│   └── envelope.rs     # YAML frontmatter serialization
├── commands/
│   └── fetch.rs        # Fetch subcommand handler
```

### Dependency graph:
```
commands/fetch.rs
    ↓
fetch/mod.rs (orchestrator)
    ↓
fetch/http.rs → fetch/extract.rs → fetch/convert.rs → fetch/envelope.rs
```

---

## 9. Testing Strategy

### Unit tests (per module):
- `http.rs`: Mock HTTP responses, test redirect tracking
- `extract.rs`: Test article extraction on sample HTML
- `convert.rs`: Test markdown generation for each HTML element type
- `envelope.rs`: Test frontmatter serialization

### Integration tests:
- End-to-end: fetch real URL (example.com), verify output structure
- Error cases: 404, timeout, invalid HTML
- Redirect following: test redirect chain tracking

### Property-based tests (optional):
- Markdown round-trip: HTML → markdown → HTML (should preserve structure)

---

## 10. Dependencies to Add

```toml
[dependencies]
# HTTP client — pure Rust async (replaces curl crate decision)
reqwest   = { version = "0.12", default-features = false, features = ["rustls-tls", "charset", "http2"] }

# HTML parsing
scraper   = "0.20"         # CSS selectors; pulls html5ever transitively — do NOT add html5ever directly

# YAML serialization — serde_yml is the active successor to deprecated serde_yaml 0.9
serde_yml  = "0.0.12"
chrono    = { version = "0.4", features = ["serde"] }

# URL handling — absolute URL joining + validation (retained; serves base_url resolution)
url       = "2.5"

# Charset detection — fallback for pages where reqwest's built-in charset feature is insufficient
encoding_rs = "0.8"
```

**Removed:** `curl = "0.4"` and `html5ever = "0.27"` (see Section 1 for curl rationale; html5ever is a scraper transitive dep).

**Binary size impact:** reqwest with `rustls-tls` and stripped release build ≈ 4–7MB total. Well under 15MB target.

---

## 11. Known Pitfalls & Mitigations

### Pitfall 1 (RESOLVED): curl blocking on async
**Was:** curl crate is blocking, conflicts with tokio runtime — required `spawn_blocking`.
**Resolution:** Replaced with `reqwest 0.12` — native async, no `spawn_blocking` needed.

### Pitfall 2: HTML encoding issues
**Problem:** Non-UTF8 HTML or incorrect charset detection.
**Mitigation:** `reqwest`'s `charset` feature handles the common case (Content-Type header). For pages with wrong/missing charset headers, fall back to `encoding_rs` meta-charset scan of the first 1KB of body bytes.

### Pitfall 3: Readability false positives
**Problem:** Extracting wrong content (comments, sidebars).
**Mitigation:** Tune scoring heuristics; always run the DOM-cleaning pass (strip nav/footer/script/style) before markdown conversion.

### Pitfall 4: Markdown escaping
**Problem:** Special characters in HTML text breaking markdown syntax.
**Mitigation:** Escape `*`, `_`, `[`, `]`, `<`, `>`, `#`, `` ` ``, `\` in text nodes only (not inside `<code>`/`<pre>`).

### Pitfall 5: Redirect loops
**Problem:** Infinite redirects causing hangs.
**Mitigation:** `reqwest::redirect::Policy::limited(n)` enforces max redirects. Automatic loop detection built into reqwest's redirect policy.

### Pitfall 6 (NEW): reqwest TLS root certificates
**Problem:** On minimal Linux environments, system root certificates may be absent.
**Mitigation:** Use `rustls-tls-webpki-roots` instead of `rustls-tls` if targeting minimal environments (bundles Mozilla's root certs). Default (`rustls-tls`) uses system roots.

---

## 12. Implementation Order (for Planning)

1. **HTTP client wrapper** (`fetch/http.rs`) — foundation
2. **Error envelope** (`fetch/envelope.rs`) — needed for error handling
3. **HTML parsing** (`fetch/extract.rs` skeleton) — parse structure
4. **Markdown conversion** (`fetch/convert.rs`) — basic transformation
5. **Readability extraction** (`fetch/extract.rs` full) — content cleaning
6. **Frontmatter serialization** (enhance `envelope.rs`) — output format
7. **Fetch subcommand** (`commands/fetch.rs`) — CLI integration
8. **Tests** (unit + integration) — verification

**Why this order:**
- HTTP first (can test with raw HTML output)
- Error handling early (needed for HTTP errors)
- Markdown conversion before Readability (simpler to test)
- Readability last (most complex, depends on other pieces)

---

## 13. Validation Architecture

### Build-time validation:
- `cargo build` succeeds
- `cargo clippy -- -D warnings` passes
- `cargo fmt -- --check` passes

### Runtime validation:
- `mdget fetch https://example.com` produces output
- Output has valid YAML frontmatter (parseable)
- Output has markdown body
- HTTP errors produce error envelope

### Behavioral validation:
- Redirects followed and tracked
- Timeouts enforced
- Article extraction removes nav/footer
- Markdown generation handles nested lists, code blocks

---

## 14. Integration with Existing Code

### Reuse existing:
- `src/config.rs` — add fetch config section (default timeout, max redirects, user-agent)
- `src/log.rs` — HTTP trace logging
- `src/main.rs` — register `fetch` command
- Error handling with `miette` — structured diagnostics

### Add new:
- `src/fetch/` module tree
- `src/commands/fetch.rs`

### Modify:
- `src/cli.rs` — add `Fetch` variant to `Commands` enum
- `Cargo.toml` — add dependencies

---

## 15. Performance Considerations

### Expected performance (vs mdurl):
- **Cold start:** ~5-15ms (vs ~100ms Node.js cold start)
- **Fetch + convert:** ~200-500ms for typical webpage (network-bound)
- **Memory:** ~8-15MB peak (vs ~50-100MB Node.js)
- **Binary size:** ~4-7MB release build with `opt-level='z', strip=true` (vs ~50+MB node_modules)
  - reqwest + rustls contributes ~2-3MB
  - scraper + html5ever contributes ~500KB-1MB
  - serde_yml, chrono, url, encoding_rs contribute ~500KB combined

### Optimization opportunities (future):
- Connection pooling — reqwest's `Client` is already connection-pooling by design (reuse `Client` instance)
- HTTP/2 multiplexing — enabled by default in reqwest 0.12
- Streaming markdown output (lower memory for large pages)

---

## Research Completeness Checklist

- [x] HTTP client crate selection and rationale
- [x] HTML parsing approach
- [x] Readability algorithm strategy
- [x] Markdown generation approach
- [x] YAML frontmatter design
- [x] Error envelope structure
- [x] CLI integration plan
- [x] Module structure and dependency graph
- [x] Testing strategy
- [x] Dependency list with version rationale
- [x] Known pitfalls and mitigations
- [x] Implementation order
- [x] Validation architecture
- [x] Integration points with existing code
- [x] Performance baseline expectations

---

## References

- mdurl source (TypeScript): https://github.com/hiranp/mdurl-cli (reference implementation)
- curl crate docs: https://docs.rs/curl
- scraper crate docs: https://docs.rs/scraper
- Readability.js algorithm: https://github.com/mozilla/readability (canonical reference)
- YAML spec: https://yaml.org/spec/1.2.2/ (for frontmatter format)

---

## Appendix A: Rust HTTP Client Ecosystem Research (2026-05-21)

Full comparison of all viable Rust HTTP clients evaluated for Phase 1:

### Candidates Evaluated

#### 1. `reqwest 0.12` ✅ CHOSEN
- **Type:** Pure Rust, async-native (built on hyper + tokio + rustls)
- **MSRV:** 1.63 (well below our 1.85 requirement)
- **HTTP/2:** Yes (default, via hyper)
- **HTTP/3:** Experimental feature flag
- **TLS:** rustls (pure Rust) or native-tls (OpenSSL); we use `rustls-tls`
- **Redirect:** `Policy::limited(n)`; `response.url()` = final URL; custom policy for full chain
- **Binary size:** ~4-7MB for minimal feature set with `default-features=false, rustls-tls`
- **Async:** Native — integrates directly with `#[tokio::main]`; no `spawn_blocking` needed
- **Strengths:** Ergonomic API, production-proven, largest ecosystem, pure Rust portability
- **Weaknesses:** Larger than ureq when tokio not already present; here tokio is already a dep

#### 2. `ureq 3.0` — CONSIDERED, NOT CHOSEN
- **Type:** Pure Rust, **blocking/synchronous only** (Sans-IO architecture)
- **MSRV:** 1.85 (matches our target exactly)
- **HTTP/2:** No (synchronous model; HTTP/2 multiplexing requires async)
- **TLS:** rustls by default (pure Rust)
- **Redirect:** `ResponseExt::get_uri()` for final URL; `get_redirect_history()` with `save_redirect_history(true)` config
- **Binary size:** Smaller than reqwest when tokio is absent; **equal or larger** when tokio already present (as in our project)
- **Async:** Not supported — requires `spawn_blocking` inside a tokio runtime
- **Why not chosen:** We already depend on tokio (graceful shutdown, CLI structure). Using ureq means adding `spawn_blocking` boilerplate — the same situation as the old `curl` crate, with no binary size benefit since tokio is already compiled in. No HTTP/2 support is a further disadvantage.
- **When to use instead:** Single-threaded CLI tools with no existing async runtime; embedded targets.

#### 3. `curl` crate (0.4) — PREVIOUSLY PLANNED, DROPPED
- **Type:** FFI wrapper around system libcurl (C library)
- **Async:** Blocking; requires `spawn_blocking` in tokio context
- **Portability:** Requires system libcurl — breaks on minimal Docker images, musl targets without libcurl
- **Binary:** Adds FFI overhead; binary depends on libcurl.so at runtime
- **Why dropped:** FFI fragility, spawn_blocking boilerplate, system dependency — reqwest is strictly better for this use case

#### 4. `hyper` (1.x) — NOT CHOSEN
- **Type:** Low-level async HTTP — the foundation reqwest is built on
- **Verdict:** Too low-level for this use case; use reqwest which wraps it

#### 5. `isahc` — NOT CHOSEN
- **Type:** libcurl-based (same FFI concerns as `curl` crate)
- **Verdict:** Same system dependency problem as the `curl` crate; less active than reqwest

### Decision Matrix

| Criterion | Weight | reqwest | ureq | curl crate |
|-----------|--------|---------|------|------------|
| Pure Rust (no FFI) | High | ✅ | ✅ | ❌ |
| Native async (tokio) | High | ✅ | ❌ | ❌ |
| Binary size (tokio present) | Medium | ✅ Good | ≈ Same | ❌ Worse |
| HTTP/2 | Medium | ✅ | ❌ | ✅ |
| Redirect chain API | Medium | ✅ | ✅ | ⚠️ Manual |
| No spawn_blocking | High | ✅ | ❌ | ❌ |
| Production ecosystem | Medium | ✅ | ✅ | ⚠️ |
| **TOTAL** | | **Best** | **Good** | **Poor** |

---

*Research completed: 2026-05-21*
*Updated: 2026-05-21 — HTTP client switched from curl to reqwest 0.12 based on Rust-native client research*
*Ready for planning.*
