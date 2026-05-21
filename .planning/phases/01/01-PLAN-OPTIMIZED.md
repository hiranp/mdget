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
---

# Plan: HTTP Core & Basic Extraction (OPTIMIZED POST-REVIEW)

## Objective

Implement the core `fetch` command that fetches URLs via HTTP, extracts article content using Readability-style algorithm, converts to clean markdown with absolute URLs and table support, and outputs with YAML frontmatter.

## Review Optimizations Applied

This plan incorporates feedback from `01-REVIEWS.md` (reviewers: Gemini, Antigravity) with these optimizations:

| Issue                                    | Severity        | Fix Applied                                                                |
| ---------------------------------------- | --------------- | -------------------------------------------------------------------------- |
| HTTP client: curl vs pure Rust           | HIGH (Research) | **Switch to ureq**: pure Rust, 0.4MB binary impact, excellent redirect API |
| `html5ever` redundant direct dep         | HIGH            | Removed — scraper manages it transitively                                  |
| `--json` flag silently ignored           | HIGH            | **Removed from Phase 1 scope entirely** (defer to Phase 2)                 |
| Relative URLs not resolved to absolute   | HIGH            | `base_url: &str` parameter added to `html_to_markdown()`                   |
| `serde_yaml 0.9` deprecated              | MEDIUM          | Replaced with `serde_yml = "0.0.12"` (active successor)                    |
| Table support missing                    | MEDIUM          | GFM pipe-table rendering added to Task 5                                   |
| DOM cleaning not explicit                | MEDIUM          | Task 6 includes explicit removal step pre-output                           |
| MarkdownRenderer borrow checker friction | MEDIUM          | Documented explicit node-collection traversal pattern                      |
| Task 6 live network dependency           | MEDIUM          | HTML fixtures added; live tests demoted to optional manual validation      |
| `url` crate overpowered                  | MEDIUM          | Documented double-duty: base-URL joining AND percent-encoding              |
| `--output` silently truncates            | LOW             | Documented in Task 9 acceptance criteria                                   |

### Key Design Decisions (OPTIMIZED)

- **HTTP Client:** Use `ureq` (pure Rust, blocking) instead of curl bindings:
  - ✅ No system libcurl dependency
  - ✅ ~0.4MB binary impact vs 1.5MB for curl
  - ✅ Excellent redirect tracking: `response.get_url()` and `response.history()` API
  - ✅ Faster startup (no C FFI overhead)
  - ✅ HTTP/1.1 sufficient for Phase 1; HTTP/2 deferred to Phase 2

- **Markdown with Absolute URLs:** Converter accepts `base_url` parameter to resolve all relative URLs/images to absolute URLs using `url::Url::join()`

- **Table Support:** Basic GFM table conversion (pipe-delimited rows with header separator)

- **Encoding:** Use `encoding_rs` with pre-scan for `<meta charset>` fallback

- **Dependencies:** No direct `html5ever`; no deprecated `serde_yaml`

---

## Context

Phase 1 of mdget — building the HTTP-fetch-to-markdown pipeline. The existing CLI template provides scaffolding (clap, config, logging, graceful shutdown). We're adding HTTP fetching, HTML parsing, article extraction, and markdown conversion.

**Implementation order (from research, optimized):**
1. Dependencies (Task 1)
2. Module structure (Task 2)
3. HTTP client with ureq (Task 3)
4. Error envelope (Task 4)
5. Markdown converter with absolute URLs + tables (Task 5)
6. Readability extraction with DOM cleaning (Task 6)
7. SuccessEnvelope (Task 7)
8. Orchestrator (Task 8)
9. Fetch subcommand (Task 9)
10. Main dispatch (Task 10)

---

## Tasks

### Task 1: Add Dependencies (OPTIMIZED)

**Read first:**
- `Cargo.toml` (current dependencies)
- `.planning/phases/01/01-RESEARCH.md` (dependency rationale)

**Action:**

Add these dependencies to `Cargo.toml [dependencies]`:

```toml
ureq       = "3.1"           # Pure Rust HTTP client (no system libcurl dependency)
scraper    = "0.20"
serde_yml  = "0.0.12"        # Replaces deprecated serde_yaml 0.9
chrono     = { version = "0.4", features = ["serde"] }
url        = "2.5"           # Absolute URL joining + percent-encoding
encoding_rs = "0.8"          # Charset detection
```

**HTTP client rationale (ureq over curl):**
- Pure Rust, no system libcurl dependency
- ~0.4MB binary impact (vs ~1.5MB for curl bindings)
- Excellent redirect tracking: `response.get_url()` returns final URL, `response.history()` returns redirect chain
- Faster startup (no C FFI overhead)
- HTTP/1.1 sufficient for Phase 1; HTTP/2 added in Phase 2 if performance warrants

**Do NOT add `html5ever` directly.** `scraper` pulls it as a transitive dependency and manages version pinning internally. Explicit direct dependency creates version-conflict risk on `cargo update`.

Preserve all existing dependencies (clap, config, serde, tracing, tokio, miette, etc.).

**Acceptance criteria:**
- `Cargo.toml` contains exactly the 6 new deps listed (NOT `html5ever`)
- `cargo check` exits 0
- `cargo tree | grep html5ever` shows html5ever as transitive dep of scraper (not direct)
- **Binary size baseline (M4):** `cargo build --release && ls -lh target/release/mdget` — record baseline before implementation
- Binary size increase after deps-only is <1MB
- **Size checkpoint:** If binary exceeds 11MB at any task, investigate immediately

---

### Task 2: Create Fetch Module Structure

**Read first:**
- `src/main.rs` (module declarations)
- `src/commands/mod.rs` (existing pattern)

**Action:**

Create module tree at `src/fetch/`:

1. `src/fetch/mod.rs` — public API (`fetch_url`) and orchestrator
2. `src/fetch/http.rs` — HTTP client wrapper (ureq-based)
3. `src/fetch/extract.rs` — Readability-style article extraction with DOM cleaning
4. `src/fetch/convert.rs` — HTML to markdown conversion with absolute URL support
5. `src/fetch/envelope.rs` — YAML frontmatter structures and serialization

In `src/main.rs`, add `mod fetch;` after existing `mod` declarations.

Create test fixtures directory:

- `tests/fixtures/README.md` — documents fixture purposes
- `tests/fixtures/simple_article.html` — minimal `<article>` with one paragraph
- `tests/fixtures/github_readme.html` — GitHub-style README (semantic HTML)
- `tests/fixtures/news_article.html` — ad-heavy news page with nav/footer

**Acceptance criteria:**
- Five files exist: `src/fetch/{mod.rs, http.rs, extract.rs, convert.rs, envelope.rs}`
- `src/main.rs` contains `mod fetch;`
- `tests/fixtures/` directory exists with README.md and 3 HTML files
- `cargo check` exits 0
- No warnings about unused code

---

### Task 3: Implement HTTP Client Wrapper (UREQ)

**Read first:**
- `.planning/phases/01/01-RESEARCH.md` (HTTP Client Selection)
- `src/fetch/http.rs` (stub from Task 2)

**Action:**

Implement `src/fetch/http.rs` using ureq (pure Rust, blocking HTTP client):

```rust
use miette::{miette, Result};
use std::time::Duration;

pub struct HttpClient {
    timeout_secs: u64,
    max_redirects: u32,
    user_agent: String,
}

pub struct HttpResponse {
    pub status:         u16,
    pub body:           Vec<u8>,
    pub final_url:      String,
    pub redirect_chain: Vec<String>,
    pub content_type:   Option<String>,
}

impl HttpClient {
    pub fn new(timeout_secs: u64, max_redirects: u32, user_agent: &str) -> Self {
        Self {
            timeout_secs,
            max_redirects,
            user_agent: user_agent.to_string(),
        }
    }

    /// Fetch a URL. Runs ureq in `spawn_blocking` — never blocks the tokio runtime.
    ///
    /// # Thread-boundary note
    /// ureq is blocking, so we must run it inside `spawn_blocking`. ureq is `Send` and
    /// thread-safe, so no issues with move semantics. Redirect history is captured via
    /// `response.history()` which returns an iterator over each redirect step.
    pub async fn fetch(&self, url: &str) -> Result<HttpResponse> {
        let url = url.to_string();
        let timeout = self.timeout_secs;
        let max_redirects = self.max_redirects;
        let user_agent = self.user_agent.clone();

        tokio::task::spawn_blocking(move || {
            Self::fetch_blocking(&url, timeout, max_redirects, &user_agent)
        })
        .await
        .map_err(|e| miette!("HTTP fetch task panicked: {e}"))?
    }

    fn fetch_blocking(url: &str, timeout_secs: u64, max_redirects: u32, user_agent: &str) -> Result<HttpResponse> {
        // Create ureq agent with timeout and max redirects
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(timeout_secs))
            .redirects(max_redirects)
            .build();

        // Send request
        let response = agent
            .get(url)
            .set("User-Agent", user_agent)
            .call()
            .map_err(|e| match e {
                ureq::Error::Status(code, _resp) => {
                    miette!("HTTP error {}: {}", code, url)
                }
                ureq::Error::Transport(t) => {
                    miette!("HTTP transport error: {}", t)
                }
            })?;

        // Collect redirect history
        let redirect_chain: Vec<String> = response
            .history()
            .iter()
            .map(|r| r.get_url().to_string())
            .collect();

        // Get final URL and status
        let final_url = response.get_url().to_string();
        let status = response.status();
        let content_type = response.header("content-type").map(|s| s.to_string());

        // Read body as bytes
        let body = response.into_reader()
            .bytes()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| miette!("Failed to read response body: {}", e))?;

        Ok(HttpResponse {
            status,
            body,
            final_url,
            redirect_chain,
            content_type,
        })
    }
}
```

**Error handling:**
- Connection timeout → `miette!("Request timed out: {url}")`
- DNS resolution failure → `miette!("Could not resolve host: {url}")`
- HTTP 4xx/5xx → included in response.status (not an error—let orchestrator decide)
- Invalid URL → `miette!("Invalid URL format: {url}")`

**Acceptance criteria:**
- `HttpClient::fetch()` compiles without errors
- Returns `Result<HttpResponse>` via async/await
- Uses `spawn_blocking` for ureq operations (non-blocking for tokio)
- Timeout enforced (configurable)
- Max redirects enforced
- Redirect chain tracked and returned
- User-agent header set (default: "mdget/0.2.0")
- **Binary size checkpoint (M4):** Measure after this task—should be ~5MB (small ureq overhead)

---

### Task 4: Implement Error Envelope

**Read first:**
- `.planning/phases/01/01-RESEARCH.md` (Section 6: Error Envelope Design)
- `src/fetch/envelope.rs` (stub from Task 2)

**Action:**

Implement `src/fetch/envelope.rs`:

```rust
use serde::Serialize;
use miette::Result;

#[derive(Serialize)]
pub struct ErrorEnvelope {
    pub success: bool,                          // always false
    pub url: String,
    pub status: Option<u16>,
    pub error: String,                          // machine-readable: "http_timeout", "http_not_found", etc.
    pub message: String,                        // human-readable: "Request timed out after 30s"
    pub fetched_at: chrono::DateTime<chrono::Utc>,
}

impl ErrorEnvelope {
    pub fn to_yaml(&self) -> Result<String> {
        let yaml = serde_yml::to_string(self)?;
        Ok(format!("---\n{}\n---\n", yaml.trim()))
    }
}

#[derive(Serialize)]
pub struct SuccessEnvelope {
    pub success: bool,                          // always true
    pub url: String,
    pub status: u16,
    pub title: Option<String>,
    pub word_count: usize,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
    pub redirect_chain: Option<Vec<String>>,
}

impl SuccessEnvelope {
    pub fn to_output(&self, markdown_body: &str) -> Result<String> {
        let frontmatter = yaml_frontmatter(self)?;
        Ok(format!("{}\n{}", frontmatter, markdown_body))
    }
}

pub fn yaml_frontmatter<T: Serialize>(data: &T) -> Result<String> {
    let yaml = serde_yml::to_string(data)?;
    Ok(format!("---\n{}\n---\n", yaml.trim()))
}
```

**Acceptance criteria:**
- `ErrorEnvelope::to_yaml()` returns valid YAML frontmatter (opens with `---\n`, closes with `---\n`)
- `success` field is `false` in `ErrorEnvelope`
- `fetched_at` contains current UTC timestamp
- Error message UX: `error` field is machine-readable (e.g., `http_timeout`), `message` field is human-readable
- Example: `error: "http_not_found"`, `message: "HTTP 404: Page not found at https://example.com"`
- `SuccessEnvelope` struct exists with all required fields
- `to_output()` produces frontmatter + markdown body
- `yaml_frontmatter()` works for both envelope types

---

### Task 5: Implement Markdown Converter (with Absolute URLs & Tables)

**Read first:**
- `.planning/phases/01/01-RESEARCH.md` (Section 4: Markdown Generation)
- `src/fetch/convert.rs` (stub from Task 2)

**Action:**

Implement `src/fetch/convert.rs` with HTML to markdown conversion including absolute URL resolution and table support:

```rust
use miette::Result;
use url::Url;

pub struct MarkdownRenderer {
    output: String,
    base_url: String,
    list_depth: usize,
}

impl MarkdownRenderer {
    pub fn new(base_url: String) -> Self {
        Self {
            output: String::new(),
            base_url,
            list_depth: 0,
        }
    }

    pub fn render(&mut self, html: &str) -> Result<String> {
        let document = scraper::Html::parse_fragment(html);
        // Collect all nodes first to avoid borrow checker issues
        let nodes: Vec<_> = document.root_element().children().collect();
        for node in nodes {
            self.visit_node(node)?;
        }
        Ok(self.output.clone())
    }

    fn visit_node(&mut self, node: scraper::ElementRef) -> Result<()> {
        let tag_name = node.value().name();

        match tag_name {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level = tag_name[1..].parse::<usize>().unwrap_or(1);
                let text = self.extract_text(node);
                self.output.push_str(&format!("{} {}\n\n", "#".repeat(level), escape_markdown(&text)));
            }
            "p" => {
                let text = self.extract_text(node);
                self.output.push_str(&format!("{}\n\n", text));
            }
            "a" => {
                let text = self.extract_text(node);
                let href = node.value().attr("href").unwrap_or("");
                let absolute_url = self.resolve_url(href);
                self.output.push_str(&format!("[{}]({})", text, absolute_url));
            }
            "strong" | "b" => {
                let text = self.extract_text(node);
                self.output.push_str(&format!("**{}**", text));
            }
            "em" | "i" => {
                let text = self.extract_text(node);
                self.output.push_str(&format!("*{}*", text));
            }
            "code" => {
                let text = self.extract_text(node);
                self.output.push_str(&format!("`{}`", text));  // Do NOT escape in code
            }
            "pre" => {
                let text = self.extract_text(node);
                self.output.push_str(&format!("```\n{}\n```\n\n", text));
            }
            "blockquote" => {
                let text = self.extract_text(node);
                for line in text.lines() {
                    self.output.push_str(&format!("> {}\n", line));
                }
                self.output.push('\n');
            }
            "ul" | "ol" => {
                self.render_list(node, tag_name == "ol")?;
            }
            "img" => {
                let alt = node.value().attr("alt").unwrap_or("");
                let src = node.value().attr("src").unwrap_or("");
                let absolute_url = self.resolve_url(src);
                self.output.push_str(&format!("![{}]({})\n\n", alt, absolute_url));
            }
            "table" => {
                self.render_table(node)?;
            }
            _ => {
                // Recursively process children
                for child in node.children() {
                    self.visit_node(scraper::ElementRef::wrap(child))?;
                }
            }
        }
        Ok(())
    }

    fn resolve_url(&self, url: &str) -> String {
        if url.is_empty() {
            return String::new();
        }

        // If already absolute, return as-is
        if url.starts_with("http://") || url.starts_with("https://") {
            return url.to_string();
        }

        // Try to join with base URL
        match Url::parse(&self.base_url) {
            Ok(base) => {
                match base.join(url) {
                    Ok(absolute) => absolute.to_string(),
                    Err(_) => url.to_string(),
                }
            }
            Err(_) => url.to_string(),
        }
    }

    fn render_table(&mut self, table_node: scraper::ElementRef) -> Result<()> {
        let selector = scraper::Selector::parse("tr").map_err(|_| miette::miette!("Invalid CSS selector"))?;
        let rows: Vec<_> = table_node.select(&selector).collect();

        for (row_idx, row) in rows.iter().enumerate() {
            let cell_selector = scraper::Selector::parse("th,td").map_err(|_| miette::miette!("Invalid CSS selector"))?;
            let cells: Vec<String> = row
                .select(&cell_selector)
                .map(|cell| self.extract_text(cell))
                .collect();

            self.output.push('|');
            for cell in cells {
                self.output.push_str(&format!(" {} |", cell));
            }
            self.output.push('\n');

            // Add separator after header row
            if row_idx == 0 {
                self.output.push_str("| --- ");
                for _ in 1..cells.len() {
                    self.output.push_str("| --- ");
                }
                self.output.push_str("|\n");
            }
        }
        self.output.push('\n');

        Ok(())
    }

    fn render_list(&mut self, list_node: scraper::ElementRef, is_ordered: bool) -> Result<()> {
        let li_selector = scraper::Selector::parse("li").map_err(|_| miette::miette!("Invalid CSS selector"))?;

        for (idx, li) in list_node.select(&li_selector).enumerate() {
            let prefix = if is_ordered {
                format!("{}. ", idx + 1)
            } else {
                "- ".to_string()
            };
            let text = self.extract_text(li);
            self.output.push_str(&format!("{}{}\n", prefix, text));
        }
        self.output.push('\n');
        Ok(())
    }

    fn extract_text(&self, elem: scraper::ElementRef) -> String {
        elem.text().collect::<String>()
    }
}

fn escape_markdown(text: &str) -> String {
    text
        .replace('\\', "\\\\")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('<', "\\<")
        .replace('>', "\\>")
        .replace('#', "\\#")
        .replace('`', "\\`")
}

pub fn html_to_markdown(html: &str, base_url: &str) -> Result<String> {
    let mut renderer = MarkdownRenderer::new(base_url.to_string());
    renderer.render(html)
}
```

**Supported HTML elements:**
- Headings: `<h1>` → `# `, `<h2>` → `## `, etc.
- Paragraphs: `<p>` → text + blank line
- Links: `<a href="...">text</a>` → `[text](absolute_url)` (base_url resolves relative URLs)
- Emphasis: `<strong>` → `**text**`, `<em>` → `*text*`
- Lists: `<ul>`, `<ol>`, `<li>` → markdown lists with `-` or `1.` prefixes
- Code: `<code>` → `` `text` ``, `<pre>` → fenced code block
- Blockquotes: `<blockquote>` → `> ` prefix
- Images: `<img src="..." alt="...">` → `![alt](absolute_url)`
- **Tables (NEW):** `<table>` → GFM pipe-delimited table (`| col1 | col2 |`)

**Acceptance criteria:**
- `html_to_markdown("<h1>Title</h1>", "http://example.com")` returns `# Title\n\n`
- `html_to_markdown("<p>Text</p>", "http://example.com")` returns `Text\n\n`
- `html_to_markdown("<strong>bold</strong>", "...")` returns `**bold**`
- Relative links resolved: `html_to_markdown("<a href='/page2'>link</a>", "http://example.com")` returns `[link](http://example.com/page2)`
- Relative images resolved: `html_to_markdown("<img src='image.jpg'>", "http://example.com/blog/")` returns `![](http://example.com/blog/image.jpg)`
- Tables render with pipes: `html_to_markdown("<table><tr><td>A</td><td>B</td></tr></table>", "...")` returns GFM table
- Special characters escaped: `*`, `_`, `[`, `]`, `<`, `>`, `#`, `` ` ``
- Code blocks NOT escaped (preserve literal `*ptr`)
- Nested elements work
- **Binary size checkpoint (M4):** ~6-7MB (smaller with ureq)

---

### Task 6: Implement Readability Article Extraction (with DOM Cleaning)

**Read first:**
- `.planning/phases/01/01-RESEARCH.md` (Section 3: Readability Algorithm)
- `tests/fixtures/` (HTML samples to test against)
- `src/fetch/extract.rs` (stub from Task 2)

**Action:**

Implement `src/fetch/extract.rs` with article extraction and explicit DOM cleaning:

```rust
use miette::Result;
use scraper::{Html, ElementRef, Selector};

pub struct ReadabilityExtractor {
    min_score: f64,
}

pub struct ExtractedArticle {
    pub title: Option<String>,
    pub content_html: String,
    pub word_count: usize,
}

impl ReadabilityExtractor {
    pub fn new(min_score: f64) -> Self {
        Self { min_score }
    }

    pub fn extract(&self, html: &str) -> Result<ExtractedArticle> {
        let document = Html::parse_document(html);

        // 1. Extract title
        let title = self.extract_title(&document);

        // 2. Find article node (highest scoring candidate)
        let article_node = self.find_article_node(&document)?;

        // 3. Clean unwanted elements BEFORE serialization
        let cleaned_html = self.clean_and_serialize(article_node);

        // 4. Count words
        let word_count = self.count_words(&cleaned_html);

        Ok(ExtractedArticle {
            title,
            content_html: cleaned_html,
            word_count,
        })
    }

    fn extract_title(&self, doc: &Html) -> Option<String> {
        // Try <title> first
        if let Ok(selector) = Selector::parse("title") {
            if let Some(title_elem) = doc.select(&selector).next() {
                let text = title_elem.text().collect::<String>().trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }

        // Fall back to first <h1>
        if let Ok(selector) = Selector::parse("h1") {
            if let Some(h1_elem) = doc.select(&selector).next() {
                let text = h1_elem.text().collect::<String>().trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }

        None
    }

    fn find_article_node(&self, doc: &Html) -> Result<ElementRef> {
        let candidates = ["article", "main", "section", "div"];
        let mut best_candidate: Option<(ElementRef, f64)> = None;

        for tag in &candidates {
            if let Ok(selector) = Selector::parse(tag) {
                for elem in doc.select(&selector) {
                    let score = self.score_element(elem);

                    if score >= self.min_score {
                        if best_candidate.is_none() || score > best_candidate.as_ref().unwrap().1 {
                            best_candidate = Some((elem, score));
                        }
                    }
                }
            }
        }

        best_candidate
            .map(|(elem, _)| elem)
            .ok_or_else(|| miette::miette!("No suitable article node found (min_score: {})", self.min_score))
    }

    fn score_element(&self, elem: ElementRef) -> f64 {
        let mut score = 0.0;

        // Positive signals
        if elem.value().name() == "article" {
            score += 10.0;
        }
        if elem.value().name() == "main" {
            score += 10.0;
        }

        // Check class and ID
        if let Some(class) = elem.value().attr("class") {
            if class.contains("content") || class.contains("article") ||
               class.contains("post") || class.contains("entry") {
                score += 5.0;
            }
        }

        if let Some(id) = elem.value().attr("id") {
            if id.contains("content") || id.contains("article") || id.contains("main") {
                score += 5.0;
            }
        }

        // Count paragraphs and text
        if let Ok(p_selector) = Selector::parse("p") {
            let p_count = elem.select(&p_selector).count();
            score += p_count as f64;
        }

        let text_length: usize = elem.text().map(|t| t.len()).sum();
        score += (text_length as f64) / 100.0;

        // Negative signals
        if elem.value().name() == "nav" {
            score -= 20.0;  // Heavy penalty
        }
        if elem.value().name() == "footer" {
            score -= 5.0;
        }
        if elem.value().name() == "aside" {
            score -= 5.0;
        }

        if let Some(class) = elem.value().attr("class") {
            if class.contains("comment") || class.contains("ad") ||
               class.contains("sidebar") || class.contains("nav") {
                score -= 5.0;
            }
        }

        tracing::debug!("Scored {} element: {}", elem.value().name(), score);
        score
    }

    fn clean_and_serialize(&self, elem: ElementRef) -> String {
        let bad_tags = ["nav", "footer", "aside", "script", "style"];
        let bad_classes = ["ad", "comment", "sidebar", "nav"];

        // Clone the element and remove bad nodes
        let html = elem.html();
        let fragment = Html::parse_fragment(&html);

        // Simple approach: serialize with bad tags removed
        let mut output = String::new();
        self.serialize_clean(&fragment.root_element(), &bad_tags, &bad_classes, &mut output);
        output
    }

    fn serialize_clean(&self, node: ElementRef, bad_tags: &[&str], bad_classes: &[&str], output: &mut String) {
        if bad_tags.contains(&node.value().name()) {
            return; // Skip this element entirely
        }

        if let Some(class) = node.value().attr("class") {
            if bad_classes.iter().any(|c| class.contains(c)) {
                return;  // Skip this element
            }
        }

        // Serialize this node
        output.push_str(&format!("<{}", node.value().name()));
        for attr in node.value().attr_iter() {
            output.push_str(&format!(" {}=\"{}\"", attr.0, attr.1));
        }
        output.push('>');

        // Recurse into children
        for child in node.children() {
            if let Some(elem) = ElementRef::wrap(child) {
                self.serialize_clean(elem, bad_tags, bad_classes, output);
            } else if let Some(text_node) = child.value().as_text() {
                output.push_str(&text_node.text);
            }
        }

        output.push_str(&format!("</{}>", node.value().name()));
    }

    fn count_words(&self, html: &str) -> usize {
        html
            .split_whitespace()
            .filter(|word| word.len() >= 2)
            .count()
    }
}

pub fn extract_article(html: &str) -> Result<ExtractedArticle> {
    let extractor = ReadabilityExtractor::new(20.0);
    extractor.extract(html)
}
```

**Scoring heuristics:**
- **Positive:** `+10` for `<article>` or `<main>`; `+5` for classes (`content`, `article`, `post`, `entry`) or IDs; `+1` per paragraph; `+0.01` per character of text
- **Negative:** `-20` for `<nav>` (heavy penalty); `-5` for `<footer>`, `<aside>`; `-5` for classes (`comment`, `ad`, `sidebar`)
- **Tie-breaking:** When scores within 10%, prefer deepest node

**Acceptance criteria:**
- `extract_article()` returns `ExtractedArticle` with `title`, `content_html`, `word_count`
- Article node scoring prefers semantic HTML (`<article>`, `<main>`)
- Class name heuristics work (positive for `content`, negative for `nav`)
- Unwanted elements removed before output (nav, footer, script, style)
- Word count algorithm: split on whitespace, filter tokens < 2 chars
- **Readability validation (M2 - OPTIMIZED):** Test extraction on 3 HTML fixtures from `tests/fixtures/` (not live network)
  - Unit tests: `cargo test extract_article` — load fixtures from disk
  - No network dependency (fixtures are committed to repo)
  - Manual validation optional: `mdget fetch https://example.com` on live URLs
- **Scoring debug logs (M2):** Add `tracing::debug!` for each element scored
- **Success criterion (M2):** All 3 fixtures extract the correct article node
- **Binary size checkpoint (M4):** ~6.5-7MB

---

### Task 7: Complete SuccessEnvelope Implementation

**Read first:**
- `src/fetch/envelope.rs` (ErrorEnvelope from Task 4)
- `.planning/phases/01/01-RESEARCH.md` (Section 5: YAML Frontmatter)

**Action:**

(Already done in Task 4 — SuccessEnvelope fully implemented with `to_output()` method)

Verify:
- `SuccessEnvelope::new()` sets `success=true` and `fetched_at=now()`
- `to_output()` produces frontmatter + markdown body
- Optional fields serialize as `null` when `None`

**Acceptance criteria:**
- `SuccessEnvelope::to_output()` returns `---\n...yaml...\n---\n\n...markdown...`
- Non-null `redirect_chain` serializes as YAML list
- Output format matches spec exactly

---

### Task 8: Implement Fetch Orchestrator

**Read first:**
- `src/fetch/mod.rs` (stub from Task 2)
- All previous task modules

**Action:**

Implement `src/fetch/mod.rs` as the orchestrator:

```rust
mod http;
mod extract;
mod convert;
mod envelope;

pub use http::{HttpClient, HttpResponse};
pub use extract::ExtractedArticle;
pub use envelope::{SuccessEnvelope, ErrorEnvelope};

use miette::Result;

pub struct FetchOptions {
    pub timeout_secs: u64,
    pub max_redirects: u32,
    pub user_agent: String,
}

impl Default for FetchOptions {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            max_redirects: 5,
            user_agent: "mdget/0.2.0".to_string(),
        }
    }
}

pub async fn fetch_url(url: &str, options: FetchOptions) -> Result<String> {
    // 1. Create HTTP client
    let client = HttpClient::new(options.timeout_secs, options.max_redirects, &options.user_agent);

    // 2. Fetch URL
    let response = match client.fetch(url).await {
        Ok(resp) => resp,
        Err(e) => {
            let envelope = ErrorEnvelope {
                success: false,
                url: url.to_string(),
                status: None,
                error: "http_error".to_string(),
                message: format!("HTTP request failed: {}", e),
                fetched_at: chrono::Utc::now(),
            };
            return Ok(envelope.to_yaml()?);
        }
    };

    // 3. Check for HTTP errors
    if response.status >= 400 {
        let error_msg = match response.status {
            404 => "Not Found",
            401 | 403 => "Forbidden",
            500..=599 => "Server Error",
            _ => "HTTP Error",
        };

        let envelope = ErrorEnvelope {
            success: false,
            url: response.final_url.clone(),
            status: Some(response.status),
            error: format!("http_{}", response.status),
            message: format!("HTTP {}: {}", response.status, error_msg),
            fetched_at: chrono::Utc::now(),
        };
        return Ok(envelope.to_yaml()?);
    }

    // 4. Detect charset and convert to UTF-8
    let charset_label = extract_charset_from_headers(&response.content_type)
        .or_else(|| extract_charset_from_html(&response.body))
        .unwrap_or("utf-8");

    let encoding = encoding_rs::Encoding::for_label(charset_label.as_bytes())
        .unwrap_or(encoding_rs::UTF_8);
    let html = encoding.decode(&response.body).0.into_owned();

    // 5. Extract article
    let article = match extract::extract_article(&html) {
        Ok(a) => a,
        Err(e) => {
            let envelope = ErrorEnvelope {
                success: false,
                url: response.final_url.clone(),
                status: Some(response.status),
                error: "extraction_failed".to_string(),
                message: format!("Failed to extract article: {}", e),
                fetched_at: chrono::Utc::now(),
            };
            return Ok(envelope.to_yaml()?);
        }
    };

    // 6. Convert to markdown
    let markdown = match convert::html_to_markdown(&article.content_html, &response.final_url) {
        Ok(md) => md,
        Err(e) => {
            let envelope = ErrorEnvelope {
                success: false,
                url: response.final_url.clone(),
                status: Some(response.status),
                error: "conversion_failed".to_string(),
                message: format!("Failed to convert to markdown: {}", e),
                fetched_at: chrono::Utc::now(),
            };
            return Ok(envelope.to_yaml()?);
        }
    };

    // 7. Build SuccessEnvelope
    let envelope = SuccessEnvelope {
        success: true,
        url: response.final_url,
        status: response.status,
        title: article.title,
        word_count: article.word_count,
        fetched_at: chrono::Utc::now(),
        redirect_chain: if response.redirect_chain.is_empty() {
            None
        } else {
            Some(response.redirect_chain)
        },
    };

    // 8. Return frontmatter + markdown
    Ok(envelope.to_output(&markdown)?)
}

fn extract_charset_from_headers(content_type: &Option<String>) -> Option<String> {
    content_type.as_ref().and_then(|ct| {
        // Look for "charset=..." in Content-Type header
        ct.split(';')
            .find_map(|param| {
                let param = param.trim();
                if param.starts_with("charset=") {
                    Some(param[8..].trim_matches('"').to_string())
                } else {
                    None
                }
            })
    })
}

fn extract_charset_from_html(body: &[u8]) -> Option<String> {
    // Pre-scan first 1KB for <meta charset="...">
    let scan_limit = std::cmp::min(1024, body.len());
    let html_start = String::from_utf8_lossy(&body[..scan_limit]);

    // Look for <meta charset="..."> or <meta charset='...'>
    if let Some(pos) = html_start.find("charset") {
        let snippet = &html_start[pos..std::cmp::min(pos + 50, html_start.len())];
        // Extract charset value
        if snippet.contains("=") {
            let after_eq = snippet.split('=').nth(1)?;
            let charset = after_eq
                .trim_matches(|c| c == '"' || c == '\'' || c == '>')
                .split(|c| c == '"' || c == '\'' || c == '>')
                .next()?
                .to_string();
            return if charset.is_empty() { None } else { Some(charset) };
        }
    }

    None
}
```

**Acceptance criteria:**
- `fetch_url()` compiles and returns `Result<String>`
- Success case returns YAML frontmatter + markdown body
- HTTP errors produce `ErrorEnvelope` (4xx, 5xx)
- Network errors produce `ErrorEnvelope`
- Redirect chain tracked in `SuccessEnvelope`
- Non-UTF-8 HTML converted to UTF-8 using `encoding_rs`
- **Charset detection (L2):** Fallback order: Content-Type header → pre-scan first 1KB for `<meta charset>` → UTF-8
- Function is async (can be awaited)

---

### Task 9: Add Fetch Subcommand

**Read first:**
- `src/cli.rs` (Commands enum)
- `src/commands/fetch.rs` (to be created)

**Action:**

1. **Add Fetch variant to Commands enum in src/cli.rs:**

```rust
#[derive(Subcommand)]
pub enum Commands {
    // ... existing variants ...

    #[command(about = "Fetch URL and convert to markdown")]
    Fetch {
        #[arg(value_name = "URL", help = "URL to fetch")]
        url: String,

        #[arg(short, long, help = "Output to file instead of stdout")]
        output: Option<std::path::PathBuf>,

        #[arg(long, default_value = "30", help = "Request timeout in seconds")]
        timeout: u64,

        #[arg(long, default_value = "5", help = "Maximum redirects to follow")]
        max_redirects: u32,

        #[arg(long, default_value = "mdget/0.2.0", help = "User-Agent header")]
        user_agent: String,
    },
}
```

Note: **`--json` flag removed** per review feedback. JSON output deferred to Phase 2 (OUT-02). Phase 1 outputs YAML frontmatter only.

2. **Create `src/commands/fetch.rs`:**

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
    tracing::info!("Fetching URL: {}", url);

    let options = FetchOptions {
        timeout_secs: timeout,
        max_redirects,
        user_agent,
    };

    let result = fetch_url(&url, options).await?;

    match output {
        Some(path) => {
            std::fs::write(&path, result)?;
            tracing::info!("Output written to: {}", path.display());
        }
        None => {
            println!("{}", result);
        }
    }

    Ok(())
}
```

3. **Update `src/commands/mod.rs`:**

```rust
pub mod command1;
pub mod command2;
pub mod completion;
pub mod fetch;  // Add this
```

**Acceptance criteria:**
- `Commands::Fetch` variant exists with all fields
- `src/commands/fetch.rs` compiles
- `fetch::run()` is async and accepts `SubsystemHandle`
- Logs to tracing before fetching
- Output goes to stdout when no `-o` flag
- Output written to file with `-o` flag
- Timeout and max_redirects passed to `FetchOptions`
- User-agent passed to `FetchOptions`
- **File write behavior documented:** `std::fs::write()` truncates/overwrites existing files
- `cargo check` exits 0

---

### Task 10: Wire Fetch Command to Main

**Read first:**
- `src/main.rs` (existing command dispatch)
- `src/commands/fetch.rs` (from Task 9)

**Action:**

Update `src/main.rs` to handle `Commands::Fetch`:

In the match statement where Commands are dispatched (inside the async tokio runtime), add BEFORE existing Command1/Command2 arms:

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

**Acceptance criteria:**
- `Commands::Fetch` arm exists in main match statement
- Fetch command runs in a subsystem (graceful shutdown compatible)
- All parameters passed to `fetch::run()`
- `cargo build` exits 0
- `cargo run -- fetch --help` shows help text
- `cargo run -- fetch https://example.com` runs without panicking

---

## Verification Criteria

**Build verification:**
- [ ] `cargo build --release` succeeds
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo fmt -- --check` passes
- [ ] Release binary size <11MB

**Functional verification:**
- [ ] `mdget fetch https://example.com` outputs YAML frontmatter + markdown
- [ ] Frontmatter contains: url, title, status, word_count, fetched_at
- [ ] Markdown body is readable (headings, paragraphs, links converted)
- [ ] `mdget fetch https://httpstat.us/404` produces error envelope with status=404
- [ ] Redirects work: `mdget fetch http://example.com` tracks redirect_chain
- [ ] Timeout enforced: `mdget fetch [slow URL] --timeout 5` times out
- [ ] Output to file works: `mdget fetch https://example.com -o output.md` creates file
- [ ] Tables rendered: HTML `<table>` produces GFM pipe-table in markdown

**Edge cases:**
- [ ] Invalid URL: `mdget fetch not-a-url` produces error envelope
- [ ] Relative URLs resolved: `mdget fetch https://github.com/user/repo` converts `../issues` → `https://github.com/user/issues`
- [ ] Non-UTF-8 HTML handled: page with different charset converts correctly
- [ ] Empty/invalid HTML: produces error envelope or minimal markdown

---

## Summary of Optimizations vs Original Plan

| Item          | Original                   | Optimized                 | Benefit                                      |
| ------------- | -------------------------- | ------------------------- | -------------------------------------------- |
| HTTP client   | curl (libcurl binding)     | ureq (pure Rust)          | ~0.4MB vs 1.5MB binary, no system dependency |
| Markdown URLs | Relative (useless)         | Absolute (base_url param) | Agents can follow links reliably             |
| Table support | Not included               | GFM pipe-tables           | Better structured data preservation          |
| serde_yaml    | 0.9 (deprecated)           | serde_yml 0.0.12 (active) | Future-proof, security patches               |
| html5ever     | Direct dep (conflict risk) | Transitive via scraper    | Cleaner version management                   |
| JSON flag     | Included (broken)          | Removed (defer Phase 2)   | Clearer scope, honest scope-gating           |
| Tests         | Live network (flaky)       | HTML fixtures (stable)    | Reliable CI, offline development             |
| DOM cleaning  | Implicit in extraction     | Explicit pre-output       | Clearer code, easier debugging               |

**Expected result:** Smaller binary (~5-7MB), faster startup, more reliable extraction, better agent experience (absolute URLs + tables), faster CI (offline fixtures).

