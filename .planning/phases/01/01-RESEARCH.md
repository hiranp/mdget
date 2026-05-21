# Phase 1 Research: HTTP Core & Basic Extraction

**Phase:** 01 - HTTP Core & Basic Extraction
**Researched:** 2026-05-21
**Requirements:** HTTP-01, HTTP-02, HTTP-03, EXTR-01, EXTR-02, EXTR-03, EXTR-04, OUT-01, OUT-05

## Research Question

"What do I need to know to PLAN this phase well?"

## Executive Summary

Phase 1 builds the foundation: HTTP client → HTML parsing → article extraction → markdown conversion → YAML frontmatter output. The existing CLI template provides solid scaffolding (clap, config, logging, graceful shutdown). We need to add:

1. HTTP client (curl crate recommended)
2. HTML parsing (scraper crate)
3. Readability-style article extraction (custom implementation)
4. Markdown generation (custom renderer based on pulldown-cmark patterns)
5. YAML frontmatter serialization (serde_yaml)
6. New `fetch` subcommand

**Key decision:** Use `curl` crate (libcurl bindings) over `reqwest` for better HTTP/2+ support, lower-level control, and curl compatibility.

---

## 1. HTTP Client Selection

### Recommended: `curl` crate

**Rationale:**
- Wraps libcurl (battle-tested, widely deployed)
- Excellent HTTP/2 and HTTP/3 support
- Low-level control for redirect tracking, header manipulation
- Smaller binary footprint than reqwest + native-tls
- Familiar to curl users (one of the stated goals)

**Alternatives considered:**
- `reqwest`: Higher-level, more ergonomic, but adds significant binary size and abstracts away details we need (redirect chain tracking)
- `ureq`: Simpler blocking API, but no async support (incompatible with tokio-based template)

**Integration:**
```toml
[dependencies]
curl = "0.4"  # Latest stable
```

**Key patterns:**
- Use `Easy2` API for custom write callback (accumulate response body)
- Track redirect chain via `CURLINFO_REDIRECT_URL`
- Configure timeouts, max redirects, user-agent
- Handle HTTP errors (4xx, 5xx) → structured error envelope

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

### Recommended: `serde_yaml`

**Rationale:**
- Standard serde integration
- Mature, widely used
- Handles common types (String, i64, DateTime, Option)

**Integration:**
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
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
curl = "0.4"              # HTTP client
scraper = "0.20"          # HTML parsing
html5ever = "0.27"        # HTML parser (transitive)
serde_yaml = "0.9"        # YAML serialization
chrono = { version = "0.4", features = ["serde"] }  # Timestamps
url = "2.5"               # URL parsing/validation
```

**Binary size impact:** ~2-3MB (curl adds ~1.5MB, scraper + html5ever ~500KB, rest minimal)

---

## 11. Known Pitfalls & Mitigations

### Pitfall 1: curl blocking on async
**Problem:** curl crate is blocking, conflicts with tokio async runtime
**Mitigation:** Use `tokio::task::spawn_blocking` to run curl operations

### Pitfall 2: HTML encoding issues
**Problem:** Non-UTF8 HTML or incorrect charset detection
**Mitigation:** Use `encoding_rs` crate to detect/convert to UTF-8

### Pitfall 3: Readability false positives
**Problem:** Extracting wrong content (comments, sidebars)
**Mitigation:** Tune scoring heuristics, add domain-specific overrides

### Pitfall 4: Markdown escaping
**Problem:** Special characters in HTML text breaking markdown syntax
**Mitigation:** Escape `*`, `_`, `[`, `]`, `<`, `>` in text nodes

### Pitfall 5: Redirect loops
**Problem:** Infinite redirects causing hangs
**Mitigation:** Enforce max_redirects, detect loops (track seen URLs)

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
- **Cold start:** ~10ms (vs ~100ms Node.js cold start)
- **Fetch + convert:** ~200-500ms for typical webpage (network-bound)
- **Memory:** ~10-20MB peak (vs ~50-100MB Node.js)
- **Binary size:** ~8-10MB release build (vs ~50+MB node_modules)

### Optimization opportunities (future):
- Connection pooling (reuse TCP connections)
- HTTP/2 pipelining (parallel requests)
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

*Research completed: 2026-05-21*
*Ready for planning.*
