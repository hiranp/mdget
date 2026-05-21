---
wave: 1
depends_on: []
files_modified:
  - Cargo.toml
  - src/cli.rs
  - src/fetch/mod.rs
  - src/fetch/http.rs
  - src/fetch/extract.rs
  - src/fetch/convert.rs
  - src/fetch/envelope.rs
  - src/commands/fetch.rs
  - src/commands/mod.rs
  - src/main.rs
  - tests/fixtures/README.md
  - tests/fixtures/simple_article.html
  - tests/fixtures/github_readme.html
  - tests/fixtures/news_article.html
autonomous: true
requirements_addressed: [HTTP-01, HTTP-02, HTTP-03, EXTR-01, EXTR-02, EXTR-03, EXTR-04, OUT-01, OUT-05]
reviews_applied: true
reviews_source: 01-REVIEWS.md
http_client: reqwest-0.12
http_client_rationale: pure-Rust-async-native-no-spawn_blocking
---

# Plan: HTTP Core & Basic Extraction (Post-Review)

## Objective

Implement the core `fetch` command that fetches URLs via HTTP, extracts article content using Readability-style algorithm, converts to clean markdown with absolute URLs, and outputs with YAML frontmatter.

## Review Changes Applied

This plan incorporates feedback from `01-REVIEWS.md` (reviewers: Gemini, Antigravity):

| Issue                                    | Severity | Fix Applied                                                         |
| ---------------------------------------- | -------- | ------------------------------------------------------------------- |
| `html5ever` redundant direct dep         | HIGH     | Removed — scraper manages it transitively                           |
| `--json` flag silently ignored           | HIGH     | Removed from Phase 1 scope entirely                                 |
| Relative URLs not resolved to absolute   | HIGH     | `base_url: &str` added to `html_to_markdown()`                      |
| `serde_yaml 0.9` deprecated              | MEDIUM   | Replaced with `serde_yml = "0.0.12"`                                |
| Table support missing                    | MEDIUM   | GFM pipe-table rendering added to Task 5                            |
| MarkdownRenderer borrow checker friction | MEDIUM   | Explicit node-collection traversal pattern documented               |
| Task 6 acceptance requires live network  | MEDIUM   | HTML fixtures added; live test demoted to manual                    |
| DOM cleaning before conversion           | MEDIUM   | Task 6 clean step made explicit pre-serialization                   |
| libcurl static vs dynamic                | RESOLVED | Dropped `curl` crate entirely — replaced with `reqwest 0.12` (pure Rust, no libcurl) |
| `curl` crate FFI & spawn_blocking        | RESOLVED | Replaced with `reqwest 0.12` — native async, no system dependency   |

## Context

Phase 1 of mdget — building the HTTP-fetch-to-markdown pipeline. The existing CLI template provides scaffolding (clap, config, logging, graceful shutdown). We're adding HTTP fetching, HTML parsing, article extraction, and markdown conversion.

**Key design decisions:**

- Use `reqwest 0.12` (pure Rust, async-native) — replaces `curl` crate. No FFI, no system libcurl dependency, no `spawn_blocking` needed. See `01-RESEARCH.md` Appendix A for full decision matrix.
- `reqwest` features: `default-features = false, features = ["rustls-tls", "charset", "http2"]` — minimal footprint with pure-Rust TLS.
- `serde_yml` (not `serde_yaml`) — active successor crate, receives security patches.
- `url` crate retained — serves double duty: base-URL joining for absolute link resolution AND percent-encoding.
- Charset detection: `reqwest`'s `charset` feature handles Content-Type header automatically; `encoding_rs` is fallback for meta-charset scan only.

## Tasks

### Task 1: Add Dependencies

<read_first>

- Cargo.toml (current dependencies)
- .planning/phases/01/01-RESEARCH.md (dependency rationale)
</read_first>

<action>
Add these dependencies to `Cargo.toml [dependencies]`:

```toml
# HTTP client — pure Rust, async-native, no libcurl FFI
reqwest    = { version = "0.12", default-features = false, features = ["rustls-tls", "charset", "http2"] }

# HTML parsing — do NOT add html5ever directly; scraper pulls it transitively
scraper    = "0.20"

# YAML — serde_yml is the active successor to deprecated serde_yaml 0.9
serde_yml  = "0.0.12"
chrono     = { version = "0.4", features = ["serde"] }
url        = "2.5"           # base URL joining + percent-encoding for absolute link resolution
encoding_rs = "0.8"          # charset detection fallback (meta-charset scan)
```

**Removed from original plan:**
- `curl = "0.4"` — dropped; replaced by `reqwest`. No libcurl, no spawn_blocking.
- `html5ever = "0.27"` — never add directly; scraper manages its version internally.

**Why these reqwest features:**
- `default-features = false` — excludes native-tls (OpenSSL), cookies, multipart; keeps binary lean
- `rustls-tls` — pure Rust TLS; no system OpenSSL or libcurl dependency
- `charset` — reqwest decodes response body charset from Content-Type automatically (reduces manual `encoding_rs` use to edge cases)
- `http2` — HTTP/2 support (hyper-based; on by default but explicit is self-documenting)

Preserve all existing dependencies (clap, config, serde, tracing, tokio, miette, etc.).

**Add release profile to Cargo.toml** (size optimization):
```toml
[profile.release]
opt-level = "z"       # optimize for size over speed
lto = true            # link-time optimization
codegen-units = 1     # better dead-code elimination
strip = true          # strip debug symbols
```
</action>

<acceptance_criteria>

- Cargo.toml contains `reqwest` with `default-features = false` and `["rustls-tls", "charset", "http2"]`
- `curl` crate is **absent** from Cargo.toml
- `html5ever` is **absent** from Cargo.toml direct deps
- `cargo check` exits 0 (all deps resolve without version conflicts)
- `cargo tree | grep html5ever` shows html5ever as a transitive dep of scraper (not direct)
- `cargo tree | grep libcurl` returns empty (no native libcurl dependency)
- `cargo tree | grep serde-yaml` or `serde_yaml` returns empty (deprecated crate absent)
- **Binary size baseline (M4):** `cargo build --release && ls -lh target/release/mdget` — document size
- Binary size with deps-only is expected ~4-7MB (stripped); alarm if >12MB at any task
- Release profile (`opt-level="z"`, `lto`, `strip`) added to Cargo.toml
</acceptance_criteria>

---

### Task 2: Create Fetch Module Structure

<read_first>

- src/main.rs (module declarations)
- src/commands/mod.rs (existing pattern)
</read_first>

<action>
Create module tree at `src/fetch/`:

1. `src/fetch/mod.rs` — public API (`fetch_url`) and orchestrator
2. `src/fetch/http.rs` — HTTP client wrapper (reqwest crate)
3. `src/fetch/extract.rs` — Readability-style article extraction
4. `src/fetch/convert.rs` — HTML to markdown conversion (with absolute URL support)
5. `src/fetch/envelope.rs` — YAML frontmatter structures and serialization

In `src/main.rs`, add `mod fetch;` after existing `mod` declarations.

Also create the test fixtures directory:

- `tests/fixtures/README.md` — documents what each fixture is for
- `tests/fixtures/simple_article.html` — minimal `<article>` page (stub, filled in Task 5)
- `tests/fixtures/github_readme.html` — GitHub-style README page (stub, filled in Task 6)
- `tests/fixtures/news_article.html` — ad-heavy news page (stub, filled in Task 6)
</action>

<acceptance_criteria>

- Five files exist: `src/fetch/{mod.rs, http.rs, extract.rs, convert.rs, envelope.rs}`
- `src/main.rs` contains `mod fetch;`
- `tests/fixtures/` directory exists with README.md
- `cargo check` exits 0
- No warnings about unused code (stub implementations are fine with `#[allow(dead_code)]`)
</acceptance_criteria>

---

### Task 3: Implement HTTP Client Wrapper

<read_first>
- .planning/phases/01/01-RESEARCH.md (Section 1: HTTP Client Selection, Appendix A: Rust HTTP Client Ecosystem)
- src/fetch/http.rs (stub from Task 2)
</read_first>

<action>
Implement `src/fetch/http.rs` using `reqwest 0.12`.

**Design:** reqwest is async-native — no `spawn_blocking` required. Build a single `reqwest::Client` configured with timeout, redirect policy, and user-agent. For redirect chain tracking, use a custom `redirect::Policy` that captures each hop into a shared `Arc<Mutex<Vec<String>>>`.

```rust
use miette::{miette, IntoDiagnostic, Result};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const DEFAULT_USER_AGENT: &str =
    "mdget/0.2.0 (https://github.com/hiranp/mdget; bot)";

pub struct HttpClient {
    client: reqwest::Client,
}

pub struct HttpResponse {
    pub status:         u16,
    pub body:           Vec<u8>,      // raw bytes; charset decode happens in orchestrator
    pub final_url:      String,       // URL after all redirects
    pub redirect_chain: Vec<String>,  // intermediate URLs (not including final)
    pub content_type:   Option<String>,
}

impl HttpClient {
    /// Build a configured reqwest Client.
    ///
    /// # Notes
    /// - `reqwest::Client` is `Clone + Send + Sync`; share one instance across calls.
    /// - Redirect tracking uses a custom policy that records each `Location` hop.
    pub fn new(timeout_secs: u64, max_redirects: u32, user_agent: &str) -> Result<Self> {
        // Shared state for redirect chain; populated by the custom policy closure.
        // Note: the closure is called once per redirect hop by reqwest.
        let chain: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let chain_clone = Arc::clone(&chain);

        let policy = reqwest::redirect::Policy::custom(move |attempt| {
            if attempt.previous().len() >= max_redirects as usize {
                attempt.error(format!(
                    "redirect_limit: Exceeded {} redirects",
                    max_redirects
                ))
            } else {
                // Record each intermediate URL
                if let Some(prev) = attempt.previous().last() {
                    if let Ok(mut v) = chain_clone.lock() {
                        v.push(prev.to_string());
                    }
                }
                attempt.follow()
            }
        });

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .redirect(policy)
            .user_agent(user_agent)
            // reqwest 0.12 with rustls-tls: no system OpenSSL needed
            .build()
            .into_diagnostic()?;

        Ok(Self { client })
    }

    /// Fetch a URL asynchronously.
    ///
    /// Returns `HttpResponse` on success. Network errors and invalid URLs are
    /// returned as `Err(miette::Report)` — the orchestrator converts these to
    /// `ErrorEnvelope`.
    pub async fn fetch(
        &self,
        url: &str,
        max_redirects: u32,
    ) -> Result<HttpResponse> {
        // Build a fresh client per-fetch to get a clean redirect chain.
        // (The redirect policy closure captures the Arc; we need a fresh one each call.)
        let chain: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let chain_for_policy = Arc::clone(&chain);

        let policy = reqwest::redirect::Policy::custom(move |attempt| {
            if attempt.previous().len() >= max_redirects as usize {
                attempt.error(format!(
                    "redirect_limit: Exceeded {max_redirects} redirects"
                ))
            } else {
                if let Some(prev) = attempt.previous().last() {
                    if let Ok(mut v) = chain_for_policy.lock() {
                        v.push(prev.to_string());
                    }
                }
                attempt.follow()
            }
        });

        let configured_client = reqwest::Client::builder()
            .timeout(self.client.timeout().unwrap_or(Duration::from_secs(30)))
            .redirect(policy)
            .user_agent(DEFAULT_USER_AGENT)
            .build()
            .into_diagnostic()
            .map_err(|e| miette!("Failed to build HTTP client: {e}"))?;

        let response = configured_client
            .get(url)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    miette!("http_timeout: Request timed out for {url}: {e}")
                } else if e.is_connect() {
                    miette!("dns_failure: Could not connect to {url}: {e}")
                } else if e.is_redirect() {
                    miette!("redirect_limit: Too many redirects for {url}: {e}")
                } else {
                    miette!("http_error: {e}")
                }
            })?;

        let status      = response.status().as_u16();
        let final_url   = response.url().to_string();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Read raw bytes — charset decode is handled separately in the orchestrator.
        let body = response.bytes().await.into_diagnostic()?.to_vec();

        let redirect_chain = chain
            .lock()
            .map(|v| v.clone())
            .unwrap_or_default();

        Ok(HttpResponse {
            status,
            body,
            final_url,
            redirect_chain,
            content_type,
        })
    }
}

/// Human-readable machine slug for HTTP status codes — used in ErrorEnvelope.error field.
pub fn http_status_error_slug(status: u16) -> &'static str {
    match status {
        400 => "http_bad_request",
        401 => "http_unauthorized",
        403 => "http_forbidden",
        404 => "http_not_found",
        429 => "http_rate_limited",
        500 => "http_server_error",
        502 => "http_bad_gateway",
        503 => "http_service_unavailable",
        _   => "http_error",
    }
}
```
</action>

<acceptance_criteria>

- `HttpClient` and `HttpResponse` defined in `src/fetch/http.rs`
- `HttpClient::fetch` is `async fn` — no `spawn_blocking`, no blocking calls
- `fetch()` returns `Err` for: timeout (`http_timeout`), connection failure (`dns_failure`), redirect limit (`redirect_limit`), other errors (`http_error`)
- `final_url` is always the post-redirect URL (from `response.url()`)
- `redirect_chain` contains intermediate URLs (populated by custom policy)
- `content_type` populated from `Content-Type` response header
- `body` is raw `Vec<u8>` (charset decode deferred to orchestrator)
- `http_status_error_slug()` exported, covers 400/401/403/404/429/500/502/503
- No `curl`, `spawn_blocking`, or FFI in the file
- `cargo check` exits 0
- **(M1) Async verification:** Call `HttpClient::fetch()` directly from a `#[tokio::test]` to confirm it awaits cleanly
- **(M4) Binary size checkpoint:** ~5-7MB after this task (stripped release)
</acceptance_criteria>

---

### Task 4: Implement Error Envelope

<read_first>

- .planning/phases/01/01-RESEARCH.md (Section 6: Error Envelope Design)
- src/fetch/envelope.rs (stub from Task 2)
</read_first>

<action>
Implement `src/fetch/envelope.rs` using `serde_yml` (not `serde_yaml`):

```rust
use miette::Result;
use serde::Serialize;

/// Machine-readable error envelope. Printed to stdout on fetch failures.
#[derive(Serialize)]
pub struct ErrorEnvelope {
    pub success:    bool,           // always false
    pub url:        String,
    pub status:     Option<u16>,
    pub error:      String,         // machine-readable slug, e.g. "http_not_found"
    pub message:    String,         // human-readable description
    pub fetched_at: chrono::DateTime<chrono::Utc>,
}

/// Successful fetch envelope — used for YAML frontmatter.
#[derive(Serialize)]
pub struct SuccessEnvelope {
    pub success:        bool,       // always true
    pub url:            String,
    pub status:         u16,
    pub title:          Option<String>,
    pub word_count:     usize,
    pub fetched_at:     chrono::DateTime<chrono::Utc>,
    pub redirect_chain: Option<Vec<String>>,
}

/// Serialise any Serialize value as a YAML frontmatter block:
///   ---
///   key: value
///   ---
///
/// Uses `serde_yml` (the active successor to the deprecated `serde_yaml 0.9`).
pub fn yaml_frontmatter(data: &impl Serialize) -> Result<String> {
    let yaml = serde_yml::to_string(data)
        .map_err(|e| miette::miette!("YAML serialisation failed: {e}"))?;
    Ok(format!("---\n{}---\n", yaml))   // serde_yml adds trailing \n to yaml
}

impl ErrorEnvelope {
    pub fn new(url: &str, status: Option<u16>, error: &str, message: String) -> Self {
        Self {
            success: false,
            url: url.to_string(),
            status,
            error: error.to_string(),
            message,
            fetched_at: chrono::Utc::now(),
        }
    }

    pub fn to_yaml(&self) -> Result<String> {
        yaml_frontmatter(self)
    }
}

impl SuccessEnvelope {
    pub fn new(
        url: String,
        status: u16,
        title: Option<String>,
        word_count: usize,
        redirect_chain: Option<Vec<String>>,
    ) -> Self {
        Self {
            success: true,
            url,
            status,
            title,
            word_count,
            fetched_at: chrono::Utc::now(),
            redirect_chain,
        }
    }

    /// Produce the full output string: YAML frontmatter + blank line + markdown body.
    pub fn to_output(&self, markdown_body: &str) -> Result<String> {
        let frontmatter = yaml_frontmatter(self)?;
        Ok(format!("{}\n{}", frontmatter, markdown_body))
    }
}
```

</action>

<acceptance_criteria>

- `yaml_frontmatter()` uses `serde_yml`, NOT `serde_yaml` — verify with `cargo tree | grep serde_yaml` (should be absent or only transitive if something else pulls it)
- `ErrorEnvelope::to_yaml()` returns `"---\n...\n---\n"` (opens and closes with `---`)
- `SuccessEnvelope::to_output()` returns `"---\n...\n---\n\n...markdown..."`
- `success: false` in ErrorEnvelope, `success: true` in SuccessEnvelope
- `fetched_at` contains current UTC timestamp (ISO 8601)
- `Option<Vec<String>>` redirect_chain serialises as `null` when None, as YAML list when Some
- `cargo test --test '*envelope*'` (if unit tests added) exits 0
</acceptance_criteria>

---

### Task 5: Implement Markdown Converter (with Absolute URLs & Tables)

<read_first>

- .planning/phases/01/01-RESEARCH.md (Section 4: Markdown Generation)
- src/fetch/convert.rs (stub from Task 2)
- tests/fixtures/simple_article.html (fill during this task)
</read_first>

<action>
Implement `src/fetch/convert.rs`.

#### Traversal Pattern (Borrow Checker Safety)

The naive `visit_nodes(&mut self, node: ElementRef)` pattern hits the borrow checker because `ElementRef` borrows from the `Html` document, and holding that reference while also mutably borrowing `self.output` is disallowed by Rust's aliasing rules.

**Use the two-phase collect-then-render pattern:**

```rust
/// Collect the element tree into an owned IR (Intermediate Representation)
/// that has no lifetime ties to the Html document. Then render the IR
/// into markdown without any borrow conflicts.
enum Node {
    Heading { level: u8, children: Vec<Node> },
    Paragraph(Vec<Node>),
    Text(String),
    Link { href: String, children: Vec<Node> },
    Image { src: String, alt: String },
    Bold(Vec<Node>),
    Italic(Vec<Node>),
    Code(String),          // inline code
    CodeBlock { lang: Option<String>, content: String },
    Blockquote(Vec<Node>),
    List { ordered: bool, items: Vec<Vec<Node>> },
    Table { headers: Vec<String>, rows: Vec<Vec<String>> },
    LineBreak,
    HorizontalRule,
}
```

**Collect phase** (`fn collect(elem: ElementRef, base_url: &Url) -> Vec<Node>`):

- Walks the scraper tree, producing owned `Node` values.
- Resolves all `href`/`src` attributes to absolute URLs using `base_url.join(attr)`.

**Render phase** (`fn render(nodes: &[Node]) -> String`):

- Pure function, no borrow issues (works on the owned IR).
- Produces final markdown string.

#### Full Implementation

```rust
use miette::Result;
use scraper::{ElementRef, Html, Node as ScraperNode};
use url::Url;

// --- IR types ---
// (enum Node as above)

pub struct MarkdownConverter {
    base_url: Url,
}

impl MarkdownConverter {
    pub fn new(base_url: &str) -> Result<Self> {
        let base_url = Url::parse(base_url)
            .map_err(|e| miette::miette!("Invalid base URL '{base_url}': {e}"))?;
        Ok(Self { base_url })
    }

    pub fn convert(&self, html: &str) -> Result<String> {
        let document = Html::parse_fragment(html);
        let nodes = self.collect_element(document.root_element());
        Ok(render_nodes(&nodes))
    }

    fn collect_element(&self, elem: ElementRef) -> Vec<Node> {
        let mut result = Vec::new();
        for child in elem.children() {
            match child.value() {
                ScraperNode::Text(t) => {
                    let text = escape_markdown(t.trim());
                    if !text.is_empty() {
                        result.push(Node::Text(text));
                    }
                }
                ScraperNode::Element(_) => {
                    if let Some(child_elem) = ElementRef::wrap(child) {
                        let tag = child_elem.value().name();
                        let node = self.collect_tag(tag, child_elem);
                        if let Some(n) = node {
                            result.push(n);
                        }
                    }
                }
                _ => {}
            }
        }
        result
    }

    fn collect_tag(&self, tag: &str, elem: ElementRef) -> Option<Node> {
        match tag {
            "h1"|"h2"|"h3"|"h4"|"h5"|"h6" => {
                let level = tag[1..].parse::<u8>().unwrap_or(1);
                Some(Node::Heading { level, children: self.collect_element(elem) })
            }
            "p" => Some(Node::Paragraph(self.collect_element(elem))),
            "a" => {
                let href = elem.value().attr("href").unwrap_or("#");
                let abs = self.resolve_url(href);
                Some(Node::Link { href: abs, children: self.collect_element(elem) })
            }
            "img" => {
                let src = elem.value().attr("src").unwrap_or("");
                let alt = elem.value().attr("alt").unwrap_or("");
                Some(Node::Image { src: self.resolve_url(src), alt: alt.to_string() })
            }
            "strong" | "b" => Some(Node::Bold(self.collect_element(elem))),
            "em" | "i"     => Some(Node::Italic(self.collect_element(elem))),
            "code"         => Some(Node::Code(elem.text().collect::<String>())),
            "pre"          => {
                let lang = elem.select(&scraper::Selector::parse("code").unwrap())
                    .next()
                    .and_then(|c| c.value().attr("class"))
                    .and_then(|cls| cls.strip_prefix("language-"))
                    .map(|l| l.to_string());
                let content = elem.text().collect::<String>();
                Some(Node::CodeBlock { lang, content })
            }
            "blockquote" => Some(Node::Blockquote(self.collect_element(elem))),
            "ul" => Some(self.collect_list(elem, false)),
            "ol" => Some(self.collect_list(elem, true)),
            "table" => Some(self.collect_table(elem)),
            "br" => Some(Node::LineBreak),
            "hr" => Some(Node::HorizontalRule),
            // Skip script, style, nav, footer (should be cleaned before conversion)
            "script" | "style" | "nav" | "footer" | "aside" => None,
            // Transparent container elements — descend into children
            "div" | "section" | "article" | "main" | "span" |
            "header" | "figure" | "figcaption" | "details" | "summary" => {
                let children = self.collect_element(elem);
                if children.is_empty() { None } else { Some(Node::Paragraph(children)) }
            }
            _ => None,
        }
    }

    fn collect_list(&self, elem: ElementRef, ordered: bool) -> Node {
        let li_sel = scraper::Selector::parse("li").unwrap();
        let items = elem.select(&li_sel)
            .map(|li| self.collect_element(li))
            .collect();
        Node::List { ordered, items }
    }

    fn collect_table(&self, elem: ElementRef) -> Node {
        let th_sel = scraper::Selector::parse("th").unwrap();
        let tr_sel = scraper::Selector::parse("tr").unwrap();
        let td_sel = scraper::Selector::parse("td").unwrap();

        let headers: Vec<String> = elem.select(&th_sel)
            .map(|th| th.text().collect::<String>().trim().to_string())
            .collect();

        let rows: Vec<Vec<String>> = elem.select(&tr_sel)
            .map(|tr| {
                tr.select(&td_sel)
                    .map(|td| td.text().collect::<String>().trim().to_string())
                    .collect()
            })
            .filter(|row: &Vec<String>| !row.is_empty())
            .collect();

        Node::Table { headers, rows }
    }

    fn resolve_url(&self, href: &str) -> String {
        if href.is_empty() || href.starts_with('#') {
            return href.to_string();
        }
        self.base_url
            .join(href)
            .map(|u| u.to_string())
            .unwrap_or_else(|_| href.to_string())
    }
}

// --- Render phase (no borrows from Html document) ---

fn render_nodes(nodes: &[Node]) -> String {
    nodes.iter().map(render_node).collect::<Vec<_>>().join("")
}

fn render_node(node: &Node) -> String {
    match node {
        Node::Heading { level, children } => {
            format!("{} {}\n\n", "#".repeat(*level as usize), render_nodes(children))
        }
        Node::Paragraph(children) => format!("{}\n\n", render_nodes(children)),
        Node::Text(t) => t.clone(),
        Node::Link { href, children } => {
            format!("[{}]({})", render_nodes(children), href)
        }
        Node::Image { src, alt } => format!("![{}]({})", alt, src),
        Node::Bold(children) => format!("**{}**", render_nodes(children)),
        Node::Italic(children) => format!("*{}*", render_nodes(children)),
        Node::Code(c) => format!("`{c}`"),
        Node::CodeBlock { lang, content } => {
            let fence_lang = lang.as_deref().unwrap_or("");
            format!("```{fence_lang}\n{content}\n```\n\n")
        }
        Node::Blockquote(children) => {
            render_nodes(children)
                .lines()
                .map(|l| format!("> {l}"))
                .collect::<Vec<_>>()
                .join("\n") + "\n\n"
        }
        Node::List { ordered, items } => {
            items.iter().enumerate().map(|(i, item)| {
                let prefix = if *ordered { format!("{}. ", i + 1) } else { "- ".to_string() };
                format!("{}{}", prefix, render_nodes(item))
            }).collect::<Vec<_>>().join("\n") + "\n\n"
        }
        Node::Table { headers, rows } => render_gfm_table(headers, rows),
        Node::LineBreak    => "  \n".to_string(),
        Node::HorizontalRule => "---\n\n".to_string(),
    }
}

/// GFM pipe-table renderer.
///
/// Output example:
/// | Name | Value |
/// |------|-------|
/// | foo  | bar   |
fn render_gfm_table(headers: &[String], rows: &[Vec<String>]) -> String {
    if headers.is_empty() && rows.is_empty() {
        return String::new();
    }
    let cols = headers.len().max(rows.iter().map(|r| r.len()).max().unwrap_or(0));
    let pad = |s: &str, width: usize| format!("{:<width$}", s);

    // Column widths
    let mut widths: Vec<usize> = (0..cols).map(|c| {
        let h = headers.get(c).map(|s| s.len()).unwrap_or(0);
        let r = rows.iter().map(|row| row.get(c).map(|s| s.len()).unwrap_or(0)).max().unwrap_or(0);
        h.max(r).max(3)
    }).collect();

    let mut out = String::new();
    // Header row
    out.push('|');
    for (c, w) in widths.iter().enumerate() {
        let h = headers.get(c).map(|s| s.as_str()).unwrap_or("");
        out.push_str(&format!(" {} |", pad(h, *w)));
    }
    out.push('\n');
    // Separator
    out.push('|');
    for w in &widths {
        out.push_str(&format!("-{}-|", "-".repeat(*w)));
    }
    out.push('\n');
    // Data rows
    for row in rows {
        out.push('|');
        for (c, w) in widths.iter().enumerate() {
            let cell = row.get(c).map(|s| s.as_str()).unwrap_or("");
            out.push_str(&format!(" {} |", pad(cell, *w)));
        }
        out.push('\n');
    }
    out.push('\n');
    out
}

/// Escape markdown special characters in plain text nodes.
///
/// Rules:
/// - Escape in text nodes: * _ [ ] < > # ` \
/// - Do NOT escape inside code/pre (handled by Node::Code/CodeBlock)
/// - HTML entities are decoded by scraper before we receive the text
fn escape_markdown(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '*' | '_' | '[' | ']' | '<' | '>' | '#' | '`' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

/// Public API — used by the orchestrator in fetch/mod.rs
pub fn html_to_markdown(html: &str, base_url: &str) -> Result<String> {
    let converter = MarkdownConverter::new(base_url)?;
    converter.convert(html)
}
```

#### Fill test fixture (tests/fixtures/simple_article.html)

Write a minimal HTML page:

```html
<!DOCTYPE html>
<html>
<head><title>Simple Article</title></head>
<body>
  <nav><a href="/home">Home</a></nav>
  <article>
    <h1>Test Article</h1>
    <p>This is a <strong>test</strong> article with a <a href="/relative">relative link</a>.</p>
    <table>
      <tr><th>Name</th><th>Value</th></tr>
      <tr><td>foo</td><td>bar</td></tr>
    </table>
  </article>
  <footer>Footer content</footer>
</body>
</html>
```

</action>

<acceptance_criteria>

- `html_to_markdown("<h1>Title</h1>", "https://example.com")` returns `"# Title\n\n"`
- `html_to_markdown("<p>Text</p>", "https://example.com")` returns `"Text\n\n"`
- `html_to_markdown("<a href='/page'>link</a>", "https://example.com/blog/")` returns `"[link](https://example.com/page)"` — relative URL resolved to absolute
- `html_to_markdown("<img src='logo.png'>", "https://example.com/")` returns `"![](https://example.com/logo.png)"`
- `html_to_markdown("<table>...(simple table)</table>", base)` returns GFM pipe-table format with `|` separators
- `html_to_markdown("<code>*ptr</code>", base)` returns `` `*ptr` `` (no escaping inside code)
- Special chars escaped in text: `*` → `\*`, `_` → `\_`, `[` → `\[`
- `scraper::Selector` compiled at call site (no `unwrap()` in production path where possible)
- `tests/fixtures/simple_article.html` exists and is valid HTML
- `cargo test` on unit tests covering the above cases exits 0
- **(M4) Binary size checkpoint:** ~7-9MB after this task
</acceptance_criteria>

---

### Task 6: Implement Readability Article Extraction

<read_first>

- .planning/phases/01/01-RESEARCH.md (Section 3: Readability-Style Article Extraction)
- .planning/phases/01/test-corpus.md (M2: scoring heuristics and test URL list)
- tests/fixtures/simple_article.html, github_readme.html, news_article.html (fill during this task)
- src/fetch/extract.rs (stub from Task 2)
</read_first>

<action>
Implement `src/fetch/extract.rs`.

#### Two-phase approach: Score → Clean → Serialise

The key insight from the reviews: **cleaning must happen before** serialising to HTML for the converter. The extractor:

1. Scores all candidate nodes
2. Selects the best candidate
3. **Explicitly removes unwanted tags** (script, style, nav, footer, aside, [role=navigation]) from the selected subtree
4. Serialises the cleaned subtree back to an HTML string
5. Returns the HTML string to the converter

```rust
use miette::{miette, Result};
use scraper::{ElementRef, Html, Selector};

pub struct ExtractedArticle {
    pub title:        Option<String>,
    pub content_html: String,   // cleaned HTML ready for html_to_markdown()
    pub word_count:   usize,
}

pub struct ReadabilityExtractor {
    pub min_score: f64,
}

impl ReadabilityExtractor {
    pub fn extract(&self, html: &str) -> Result<ExtractedArticle> {
        let document = Html::parse_document(html);

        let title = self.extract_title(&document);
        let article_html = self.find_and_clean_article(&document)?;
        let word_count = count_words(&article_html);

        Ok(ExtractedArticle { title, content_html: article_html, word_count })
    }

    fn extract_title(&self, doc: &Html) -> Option<String> {
        // Preference order: <title> → first <h1> → None
        let title_sel = Selector::parse("title").unwrap();
        let h1_sel    = Selector::parse("h1").unwrap();

        doc.select(&title_sel).next()
            .map(|t| t.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                doc.select(&h1_sel).next()
                    .map(|h| h.text().collect::<String>().trim().to_string())
                    .filter(|s| !s.is_empty())
            })
    }

    fn find_and_clean_article(&self, doc: &Html) -> Result<String> {
        // Candidate selectors: elements that can contain article content
        let candidates_sel = Selector::parse(
            "article, main, [role='main'], div, section"
        ).unwrap();

        let mut best_score = self.min_score - 1.0;
        let mut best_html  = String::new();

        for elem in doc.select(&candidates_sel) {
            let score = self.score_element(elem);
            tracing::debug!(
                tag = elem.value().name(),
                id  = elem.value().attr("id").unwrap_or(""),
                class = elem.value().attr("class").unwrap_or(""),
                score,
                "Readability candidate"
            );
            if score > best_score {
                best_score = score;
                // Serialise candidate to HTML string immediately (avoids lifetime issues)
                best_html  = elem.html();
            }
        }

        if best_html.is_empty() {
            return Err(miette!("No article content found (all candidates scored below {:.1})", self.min_score));
        }

        // Clean the selected subtree:
        // Re-parse the candidate HTML, remove noise tags, re-serialise.
        let cleaned = self.clean_html(&best_html);
        Ok(cleaned)
    }

    fn score_element(&self, elem: ElementRef) -> f64 {
        let tag   = elem.value().name();
        let id    = elem.value().attr("id").unwrap_or("").to_lowercase();
        let class = elem.value().attr("class").unwrap_or("").to_lowercase();

        let mut score = 0.0_f64;

        // Positive signals
        if matches!(tag, "article" | "main") { score += 10.0; }
        for kw in &["content", "article", "post", "entry", "markdown-body"] {
            if class.contains(kw) { score += 5.0; }
        }
        for kw in &["content", "article", "main"] {
            if id.contains(kw) { score += 5.0; }
        }

        // Paragraph density
        let p_sel = Selector::parse("p").unwrap();
        let p_count = elem.select(&p_sel).count() as f64;
        score += p_count;

        // Text length
        let text_len = elem.text().collect::<String>().len() as f64;
        score += (text_len / 10.0) * 0.1;

        // Negative signals
        if matches!(tag, "nav" | "header") { score -= 20.0; }
        if matches!(tag, "footer" | "aside") { score -= 5.0; }
        for kw in &["comment", " ad ", "sidebar", "nav", "footer", "promo", "sponsor", "banner"] {
            if class.contains(kw) { score -= 5.0; }
        }

        // Low text density penalty
        let all_tags = elem.descendants().filter(|n| n.value().is_element()).count() as f64;
        if all_tags > 0.0 && text_len / all_tags < 10.0 {
            score -= all_tags * 0.5;
        }

        score
    }

    /// Re-parse a candidate HTML string and strip noise elements.
    /// Returns cleaned HTML suitable for the markdown converter.
    ///
    /// Removes: nav, footer, aside, script, style, [role=navigation],
    ///          .ad, .sidebar, .comment, .promo, .sponsor, .banner
    fn clean_html(&self, html: &str) -> String {
        let doc = Html::parse_fragment(html);
        let mut out = String::new();
        self.clean_element(doc.root_element(), &mut out);
        out
    }

    fn clean_element(&self, elem: ElementRef, out: &mut String) {
        let tag   = elem.value().name();
        let class = elem.value().attr("class").unwrap_or("").to_lowercase();
        let role  = elem.value().attr("role").unwrap_or("").to_lowercase();

        // Tags to discard entirely (with all children)
        if matches!(tag, "script" | "style" | "nav" | "footer" | "aside" | "noscript") {
            return;
        }
        if role == "navigation" || role == "banner" { return; }
        for kw in &["sidebar", " ad ", "comment", "promo", "sponsor", "banner"] {
            if class.contains(kw) { return; }
        }

        // Emit opening tag with href/src/class/id attributes only (strip event handlers etc.)
        out.push('<');
        out.push_str(tag);
        for attr_name in &["href", "src", "alt", "class", "id", "lang", "type"] {
            if let Some(val) = elem.value().attr(attr_name) {
                out.push_str(&format!(" {attr_name}=\"{}\"", val.replace('"', "&quot;")));
            }
        }
        out.push('>');

        // Recurse children
        for child in elem.children() {
            match child.value() {
                scraper::node::Node::Text(t) => out.push_str(t),
                scraper::node::Node::Element(_) => {
                    if let Some(child_elem) = ElementRef::wrap(child) {
                        self.clean_element(child_elem, out);
                    }
                }
                _ => {}
            }
        }

        // Self-closing check (void elements in HTML5)
        if !matches!(tag, "br" | "hr" | "img" | "input" | "meta" | "link") {
            out.push_str(&format!("</{tag}>"));
        }
    }
}

/// Word count: split on whitespace, filter tokens <2 chars, count remaining.
pub fn count_words(html: &str) -> usize {
    let doc = Html::parse_fragment(html);
    doc.root_element()
        .text()
        .collect::<String>()
        .split_whitespace()
        .filter(|w| w.len() >= 2)
        .count()
}

/// Public API
pub fn extract_article(html: &str) -> Result<ExtractedArticle> {
    ReadabilityExtractor { min_score: 20.0 }.extract(html)
}
```

#### Fill test fixtures (during this task)

Write `tests/fixtures/github_readme.html` — a GitHub-style README with `<article class="markdown-body">` and navigation sidebar.

Write `tests/fixtures/news_article.html` — ad-heavy news page with `<div class="ad">`, `<nav>`, `<footer>`, and an `<article>` containing the real content.

#### Unit tests (add to `tests/extract_test.rs` or inline with `#[cfg(test)]`)

```rust
#[test]
fn test_extract_simple_article() {
    let html = std::fs::read_to_string("tests/fixtures/simple_article.html").unwrap();
    let result = extract_article(&html).unwrap();
    assert!(result.content_html.contains("Test Article") || result.title.as_deref() == Some("Simple Article"));
    assert!(result.word_count > 0);
}

#[test]
fn test_extract_removes_nav_and_footer() {
    let html = std::fs::read_to_string("tests/fixtures/news_article.html").unwrap();
    let result = extract_article(&html).unwrap();
    // Cleaned HTML should not contain nav or footer content
    assert!(!result.content_html.to_lowercase().contains("<nav"));
    assert!(!result.content_html.to_lowercase().contains("<footer"));
}

#[test]
fn test_word_count_algorithm() {
    assert_eq!(count_words("<p>hello world</p>"), 2);
    assert_eq!(count_words("<p>a bb ccc</p>"), 2);  // "a" filtered (<2 chars)
}
```

</action>

<acceptance_criteria>

- `extract_article()` returns `ExtractedArticle` with `title`, `content_html`, `word_count`
- `content_html` does NOT contain `<nav>`, `<footer>`, `<script>`, `<style>`, `<aside>` or their content
- Scoring log: `tracing::debug!` emits candidate info (tag, id, class, score) for top candidates
- Word count: split on whitespace, filter tokens <2 chars, count remaining
- `extract_article()` returns `Err` if no candidate scores above `min_score` (20.0)
- **Offline unit tests** (no network required): `cargo test --test extract_test` exits 0 using fixtures
- **(M2 - MANUAL integration test, NOT in CI):** Against 20 URLs from test-corpus.md, ≥15/20 extract correctly. Document results in `tests/fixtures/README.md` after manual run.
- **(M4) Binary size checkpoint:** ~10-12MB after this task
</acceptance_criteria>

---

### Task 7: Complete SuccessEnvelope Implementation

<read_first>

- src/fetch/envelope.rs (from Task 4)
</read_first>

<action>
`SuccessEnvelope` is already complete as specified in Task 4. Verify `to_output()` produces the exact format:

```
---
success: true
url: https://example.com
status: 200
title: Example Domain
word_count: 42
fetched_at: "2026-05-21T23:00:00Z"
redirect_chain: null
---

# Example Domain

Article content here...
```

Add a unit test:

```rust
#[test]
fn test_success_envelope_format() {
    let env = SuccessEnvelope::new(
        "https://example.com".to_string(),
        200,
        Some("Example".to_string()),
        42,
        None,
    );
    let out = env.to_output("# Heading\n\nBody text.").unwrap();
    assert!(out.starts_with("---\n"));
    assert!(out.contains("success: true"));
    assert!(out.contains("word_count: 42"));
    assert!(out.contains("---\n\n# Heading"));
    assert!(out.contains("redirect_chain: null") || out.contains("redirect_chain:"));
}
```

</action>

<acceptance_criteria>

- Output starts with `---\n`, contains `---\n\n` separator before markdown body
- `success: true` present
- `None` fields serialise as `null` in YAML (not omitted)
- Unit test passes: `cargo test test_success_envelope_format`
</acceptance_criteria>

---

### Task 8: Implement Fetch Orchestrator

<read_first>

- src/fetch/mod.rs (stub from Task 2)
- src/fetch/http.rs (Task 3)
- src/fetch/extract.rs (Task 6)
- src/fetch/convert.rs (Task 5)
- src/fetch/envelope.rs (Tasks 4, 7)
</read_first>

<action>
Implement `src/fetch/mod.rs`:

```rust
mod http;
mod extract;
mod convert;
mod envelope;

pub use http::{HttpClient, HttpResponse};
pub use extract::ExtractedArticle;
pub use envelope::{SuccessEnvelope, ErrorEnvelope};

pub struct FetchOptions {
    pub timeout_secs:   u64,
    pub max_redirects:  u32,
    pub user_agent:     String,
}

impl Default for FetchOptions {
    fn default() -> Self {
        Self {
            timeout_secs:  30,
            max_redirects: 5,
            user_agent:    "mdget/0.2.0 (https://github.com/hiranp/mdget; bot)".to_string(),
        }
    }
}

/// Main entry point: fetch a URL and return markdown output (with YAML frontmatter).
///
/// On any failure (network, HTTP error, extraction error), returns a YAML
/// ErrorEnvelope string — never returns Err to the caller. The caller always
/// gets valid output suitable for stdout.
pub async fn fetch_url(url: &str, options: FetchOptions) -> miette::Result<String> {
    let client = HttpClient::new(options.timeout_secs, options.max_redirects, &options.user_agent)?;

    // 1. HTTP fetch
    let response = match client.fetch(url, options.max_redirects).await {
        Ok(r) => r,
        Err(e) => {
            let env = ErrorEnvelope::new(url, None, "network_error", e.to_string());
            return Ok(env.to_yaml()?);
        }
    };

    // 2. HTTP error status (4xx / 5xx)
    if response.status >= 400 {
        let slug = http::http_status_error_slug(response.status);
        let msg  = format!("Server returned HTTP {} for {url}", response.status);
        let env  = ErrorEnvelope::new(url, Some(response.status), slug, msg);
        return Ok(env.to_yaml()?);
    }

    // 3. Charset detection.
    //    reqwest's `charset` feature decodes body; we use encoding_rs as a fallback
    //    for HTML meta-charset scanning if the Content-Type header is missing/incomplete.
    let html = detect_and_decode_charset(&response.body, response.content_type.as_deref());

    // 4. Extract article
    let article = match extract::extract_article(&html) {
        Ok(a) => a,
        Err(e) => {
            let env = ErrorEnvelope::new(
                url, Some(response.status),
                "extraction_failed",
                format!("Article extraction failed: {e}"),
            );
            return Ok(env.to_yaml()?);
        }
    };

    // 5. Convert to markdown (pass final_url as base for relative URL resolution)
    let markdown = match convert::html_to_markdown(&article.content_html, &response.final_url) {
        Ok(md) => md,
        Err(e) => {
            let env = ErrorEnvelope::new(
                url, Some(response.status),
                "conversion_failed",
                format!("Markdown conversion failed: {e}"),
            );
            return Ok(env.to_yaml()?);
        }
    };

    // 6. Build SuccessEnvelope
    let redirect_chain = if response.redirect_chain.is_empty() {
        None
    } else {
        Some(response.redirect_chain)
    };

    let envelope = SuccessEnvelope::new(
        response.final_url,
        response.status,
        article.title,
        article.word_count,
        redirect_chain,
    );

    Ok(envelope.to_output(&markdown)?)
}

/// Decode raw HTTP body bytes to UTF-8 string.
///
/// Detection order:
/// 1. Content-Type header charset parameter (already handled by reqwest if present)
/// 2. HTML <meta charset="..."> or <meta http-equiv="Content-Type"> in first 1024 bytes
/// 3. Fall back to UTF-8
fn detect_and_decode_charset(body: &[u8], content_type: Option<&str>) -> String {
    // 1. If reqwest provided content_type with charset, it's done. 
    //    We scan for meta-tags just in case headers are missing or charset is omitted.
    let prefix = std::str::from_utf8(&body[..body.len().min(1024)]).unwrap_or("");
    
    if let Some(cs) = extract_meta_charset(prefix) {
        if let Some(encoding) = encoding_rs::Encoding::for_label(cs.as_bytes()) {
            let (cow, _, _) = encoding.decode(body);
            return cow.into_owned();
        }
    }

    // Fallback: assume UTF-8 (reqwest handles basic conversion if charset was header-defined)
    String::from_utf8_lossy(body).into_owned()
}

/// Extract charset from <meta> tags in a partial HTML string.
fn extract_meta_charset(prefix: &str) -> Option<String> {
    let lower = prefix.to_lowercase();
    if let Some(pos) = lower.find("charset=") {
        let rest = &prefix[pos + 8..];
        let end = rest.find(|c: char| matches!(c, '"' | '\'' | ';' | '>' | ' '))
            .unwrap_or(rest.len());
        let cs = rest[..end].trim().to_string();
        if !cs.is_empty() { return Some(cs.to_lowercase()); }
    }
    None
}
```
</action>

<acceptance_criteria>

- `fetch_url` is async and returns `Result<String>` (the String is always valid output)
- Network errors → ErrorEnvelope with `error: "network_error"`
- HTTP 4xx/5xx → ErrorEnvelope with appropriate slug
- Extraction failure → ErrorEnvelope with `error: "extraction_failed"`
- Conversion failure → ErrorEnvelope with `error: "conversion_failed"`
- Success → YAML frontmatter (`---\n...\n---\n\n`) + markdown body
- `final_url` passed as `base_url` to `html_to_markdown()`
- Charset detection handles meta-charset correctly
- No `spawn_blocking` anywhere in the orchestration path
- `fetch_url` compiles and passes `cargo check`
</acceptance_criteria>

---

### Task 9: Add Fetch Subcommand

<read_first>

- src/cli.rs (Commands enum)
- src/commands/mod.rs
- .planning/phases/01/01-RESEARCH.md (Section 7: CLI Integration)
</read_first>

<action>
1. **Add `Fetch` variant to `Commands` enum in `src/cli.rs`:**

```rust
#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Fetch URL and convert to markdown with YAML frontmatter")]
    Fetch {
        #[arg(value_name = "URL", help = "URL to fetch")]
        url: String,

        #[arg(short = 'o', long, value_name = "FILE",
              help = "Write output to FILE instead of stdout (overwrites existing)")]
        output: Option<std::path::PathBuf>,

        #[arg(long, default_value = "30",
              help = "Request timeout in seconds")]
        timeout: u64,

        #[arg(long, default_value = "5",
              help = "Maximum number of redirects to follow")]
        max_redirects: u32,

        #[arg(long, default_value = "mdget/0.2.0 (https://github.com/hiranp/mdget; bot)",
              help = "User-Agent header value")]
        user_agent: String,
    },
}
```

1. **Create `src/commands/fetch.rs`:**

```rust
use crate::fetch::{fetch_url, FetchOptions};
use miette::Result;
use std::path::PathBuf;
use tokio_graceful_shutdown::SubsystemHandle;

pub async fn run(
    _subsys: SubsystemHandle,
    url: String,
    output: Option<PathBuf>,
    timeout: u64,
    max_redirects: u32,
    user_agent: String,
) -> Result<()> {
    tracing::info!(url = %url, timeout, max_redirects, "Fetching URL");

    let options = FetchOptions { timeout_secs: timeout, max_redirects, user_agent };
    let result = fetch_url(&url, options).await?;

    match output {
        Some(ref path) => {
            std::fs::write(path, &result)
                .map_err(|e| miette::miette!("Failed to write to {}: {e}", path.display()))?;
            tracing::info!(path = %path.display(), "Output written");
        }
        None => {
            print!("{result}");
        }
    }

    Ok(())
}
```

1. **Update `src/commands/mod.rs`** — add `pub mod fetch;`
</action>

<acceptance_criteria>

- `Commands::Fetch` exists in `src/cli.rs`
- `src/commands/fetch.rs` exists
- `run()` is `async`, logs correctly
- Output logic follows requirements
- `cargo check` exits 0
</acceptance_criteria>

---

### Task 10: Wire Fetch Command to Main

<read_first>

- src/main.rs (existing command dispatch)
- src/commands/fetch.rs (Task 9)
</read_first>

<action>
Add `Commands::Fetch` arm to the `match` statement in `src/main.rs`:

```rust
Commands::Fetch {
    url,
    output,
    timeout,
    max_redirects,
    user_agent,
} => {
    subsys
        .start(SubsystemBuilder::new("fetch", move |s| {
            crate::commands::fetch::run(
                s,
                url.clone(),
                output.clone(),
                timeout,
                max_redirects,
                user_agent.clone(),
            )
        }))
        .handle()
        .await?;
}
```
</action>

<acceptance_criteria>

- `Commands::Fetch` arm exists in `src/main.rs` match block
- `cargo build` exits 0
- `cargo run -- fetch https://example.com` works
- Final release binary size < 15MB
</acceptance_criteria>

---

## Verification Criteria

### Build Verification

- [ ] `cargo build --release` succeeds
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo fmt -- --check` passes
- [ ] Release binary size is under 15MB (`ls -lh target/release/mdget`); expected 4-8MB
- [ ] `cargo tree | grep html5ever` — html5ever is a **transitive dep** of scraper, NOT a direct dep
- [ ] `cargo tree | grep serde.yaml` — deprecated `serde_yaml` is **absent**; only `serde_yml` used
- [ ] `cargo tree | grep libcurl` — **empty** (no system libcurl dependency)
- [ ] `cargo tree | grep '\bcurl\b'` — **empty** (curl crate absent)

### Functional Verification

- [ ] `mdget fetch https://example.com` outputs YAML frontmatter + markdown body
- [ ] Frontmatter contains: `success: true`, `url`, `title`, `status: 200`, `word_count`, `fetched_at`
- [ ] All links/image URLs in markdown output are absolute (not relative)
- [ ] HTML tables render as GFM pipe tables in markdown output
- [ ] `mdget fetch https://httpstat.us/404` produces error envelope with `error: "http_not_found"`, `status: 404`
- [ ] `mdget fetch http://example.com` (HTTP→HTTPS redirect): `redirect_chain` populated in frontmatter
- [ ] `mdget fetch https://httpstat.us/200?sleep=60000 --timeout 5` times out with error envelope `error: "http_timeout"`
- [ ] `mdget fetch https://example.com -o output.md` creates file (check with `cat output.md`)
- [ ] `mdget fetch --help` does NOT show a `--json` flag

### Unit Tests (Offline / CI-Safe)

- [ ] `cargo test` passes (all unit tests using fixtures in `tests/fixtures/`)
- [ ] `test_extract_simple_article` passes
- [ ] `test_extract_removes_nav_and_footer` passes
- [ ] `test_word_count_algorithm` passes
- [ ] `test_success_envelope_format` passes
- [ ] Link absolute URL resolution test passes

### Edge Cases

- [ ] Invalid URL: `mdget fetch not-a-url` → error envelope `error: "http_error"` or `"invalid_url"`
- [ ] Non-UTF-8 content (charset=iso-8859-1 page): decoded to UTF-8 without panic
- [ ] Empty `<body>` or no scoreable content: error envelope `error: "extraction_failed"`

### Phase Goal Alignment

**Goal:** Fetch HTML pages and convert to clean markdown

**ROADMAP Success Criteria:**

1. ✓ `mdget https://example.com` outputs YAML frontmatter + markdown body
2. ✓ HTTP errors produce error envelopes with predictable structure
3. ✓ Redirects followed and tracked in frontmatter
4. ✓ Article content extracted from HTML (Readability-style)

## Must-Haves (Goal-Backward Verification)

1. **Working fetch command** — `mdget fetch <url>` executes without panic
2. **YAML frontmatter output** — every successful fetch outputs valid YAML frontmatter
3. **Absolute URLs in markdown** — all links and images resolved to absolute URLs
4. **Markdown body** — HTML converted to readable markdown (headings, paragraphs, links, tables)
5. **Error handling** — HTTP errors and network errors produce structured envelopes (never panics)
6. **Redirect tracking** — redirects followed; final URL and chain recorded
7. **Article extraction** — Readability algorithm removes nav/footer/script/style/ads
8. **Timeout enforcement** — requests respect `--timeout` flag
9. **Dependency correctness** — `reqwest 0.12` used (not `curl`), `serde_yml` (not `serde_yaml`), no direct `html5ever` dep
10. **Pure Rust** — no FFI, no system libcurl, no `spawn_blocking` in the HTTP path

## Truths (Constraints That Must Hold)

1. **HTTP client is async-native** — `reqwest::Client::fetch()` is a native `async fn`; no `spawn_blocking`, no FFI
2. **YAML frontmatter format is stable** — `---\n...yaml...\n---\n\n` exactly
3. **Error envelopes are structured** — `success`, `error`, `message`, `status` always present
4. **Markdown escaping is safe** — special chars in text nodes escaped (not in code blocks)
5. **Binary size under 15MB** — release build stays under target; expected 4-8MB stripped
6. **No `html5ever` direct dep** — scraper manages it transitively; direct dep causes version conflicts
7. **`serde_yml`** — not `serde_yaml`; deprecated crate must not appear in the dependency tree
8. **No `curl` crate** — replaced by `reqwest 0.12`; no system libcurl dependency

## Rollback Plan

1. **Build failures:** Revert `Cargo.toml` changes; fix one module at a time
2. **Functional failures:** Stub the failing module, commit working subset, open follow-up task
3. **Binary size >15MB:** Profile with `cargo bloat`, disable unused features (`curl` static-ssl if enabled), consider lighter alternatives
4. **Readability accuracy <75%:** Increase `min_score` threshold adjustments, add site-specific heuristics, run more fixture iterations

---

*Plan created: 2026-05-21*
*Revised: 2026-05-21 — post-review replan incorporating 01-REVIEWS.md (reviewers: Gemini, Antigravity)*
*Revised: 2026-05-21 — HTTP client switched from `curl` crate (libcurl FFI) to `reqwest 0.12` (pure Rust async-native) after Rust-native HTTP crate research*
*Ready for execution.*
