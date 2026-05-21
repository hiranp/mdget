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
autonomous: true
requirements_addressed: [HTTP-01, HTTP-02, HTTP-03, EXTR-01, EXTR-02, EXTR-03, EXTR-04, OUT-01, OUT-05]
---

# Plan: HTTP Core & Basic Extraction

## Objective

Implement the core `fetch` command that fetches URLs via HTTP, extracts article content using Readability-style algorithm, converts to clean markdown, and outputs with YAML frontmatter.

## Context

This is Phase 1 of the mdget project — building the foundation for an agent-first HTTP client. The existing CLI template provides scaffolding (clap, config, logging, graceful shutdown). We're adding HTTP fetching, HTML parsing, article extraction, and markdown conversion capabilities.

**Key design decision:** Use `curl` crate (libcurl bindings) for HTTP instead of `reqwest` because:
- Better low-level control for redirect tracking
- Smaller binary footprint
- HTTP/2+ support
- Familiar to curl users

## Tasks

### Task 1: Add Dependencies

<read_first>
- Cargo.toml (current dependencies)
- .planning/phases/01/01-RESEARCH.md (dependency rationale)
</read_first>

<action>
Add these dependencies to Cargo.toml [dependencies] section:
- curl = "0.4"
- scraper = "0.20"
- html5ever = "0.27"
- serde_yaml = "0.9"
- chrono = { version = "0.4", features = ["serde"] }
- url = "2.5"
- encoding_rs = "0.8" (for charset detection)

Preserve existing dependencies (clap, config, serde, tracing, tokio, miette, etc.)
</action>

<acceptance_criteria>
- Cargo.toml contains all 7 new dependencies with exact versions
- `cargo check` exits 0 (dependencies resolve)
- Binary size increase is <3MB (measured via `cargo build --release && ls -lh target/release/mdget`)
</acceptance_criteria>

---

### Task 2: Create Fetch Module Structure

<read_first>
- src/main.rs (module structure)
- src/commands/mod.rs (existing pattern)
</read_first>

<action>
Create module tree at src/fetch/:
1. src/fetch/mod.rs — public API and orchestrator
2. src/fetch/http.rs — HTTP client wrapper using curl crate
3. src/fetch/extract.rs — Readability-style article extraction
4. src/fetch/convert.rs — HTML to markdown conversion
5. src/fetch/envelope.rs — YAML frontmatter structures and serialization

In src/main.rs, add `mod fetch;` declaration after existing `mod` declarations.

Each module should start with standard headers:
```rust
use miette::Result;
// module-specific imports
```
</action>

<acceptance_criteria>
- Five files exist: src/fetch/{mod.rs, http.rs, extract.rs, convert.rs, envelope.rs}
- src/main.rs contains `mod fetch;`
- `cargo check` exits 0 (all modules compile)
- No warnings about unused code (stub implementations are fine)
</acceptance_criteria>

---

### Task 3: Implement HTTP Client Wrapper

<read_first>
- .planning/phases/01/01-RESEARCH.md (Section 1: HTTP Client Selection, Section 11: Pitfall 1)
- src/fetch/http.rs (stub from Task 2)
</read_first>

<action>
Implement src/fetch/http.rs with these components:

1. **HttpClient struct:**
```rust
pub struct HttpClient {
    timeout_secs: u64,
    max_redirects: u32,
    user_agent: String,
}
```

2. **HttpResponse struct:**
```rust
pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
    pub final_url: String,
    pub redirect_chain: Vec<String>,
    pub content_type: Option<String>,
}
```

3. **fetch method:**
```rust
impl HttpClient {
    pub async fn fetch(&self, url: &str) -> Result<HttpResponse> {
        // Use tokio::task::spawn_blocking to run curl (blocking API) in async context
        // Configure curl Easy2 with:
        // - CURLOPT_TIMEOUT
        // - CURLOPT_MAXREDIRS
        // - CURLOPT_USERAGENT
        // - CURLOPT_FOLLOWLOCATION
        // Track redirects via custom callback
        // Return HttpResponse with status, body, final_url, redirect_chain
    }
}
```

Handle these error cases:
- Connection timeout → miette error with "Request timed out"
- DNS resolution failure → "Could not resolve host"
- HTTP 4xx/5xx → include status code in error
- Invalid URL → "Invalid URL format"
</action>

<acceptance_criteria>
- HttpClient::fetch compiles without errors
- Method returns Result<HttpResponse>
- Uses spawn_blocking for curl operations (no blocking in async context)
- Timeout is enforced (configurable via HttpClient constructor)
- Max redirects enforced (configurable)
- Redirect chain tracked (stored in HttpResponse)
- HTTP errors produce miette::Result::Err with status code
- User-agent header set (default: "mdget/0.2.0")
</acceptance_criteria>

---

### Task 4: Implement Error Envelope

<read_first>
- .planning/phases/01/01-RESEARCH.md (Section 6: Error Envelope Design)
- src/fetch/envelope.rs (stub from Task 2)
</read_first>

<action>
Implement src/fetch/envelope.rs with these structures:

1. **ErrorEnvelope struct:**
```rust
#[derive(Serialize)]
pub struct ErrorEnvelope {
    pub success: bool,  // always false
    pub url: String,
    pub status: Option<u16>,
    pub error: String,
    pub message: String,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
}

impl ErrorEnvelope {
    pub fn to_yaml(&self) -> Result<String> {
        // Serialize to YAML frontmatter format:
        // ---
        // success: false
        // url: ...
        // ---
    }
}
```

2. **SuccessEnvelope struct (stub for now, full implementation in Task 7):**
```rust
#[derive(Serialize)]
pub struct SuccessEnvelope {
    pub success: bool,  // always true
    pub url: String,
    pub status: u16,
    pub title: Option<String>,
    pub word_count: usize,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
    pub redirect_chain: Option<Vec<String>>,
}
```

3. **Helper function:**
```rust
pub fn yaml_frontmatter(data: &impl Serialize) -> Result<String> {
    let yaml = serde_yaml::to_string(data)?;
    Ok(format!("---\n{}\n---\n", yaml.trim()))
}
```
</action>

<acceptance_criteria>
- ErrorEnvelope::to_yaml() returns valid YAML frontmatter (opens with `---\n`, closes with `---\n`)
- success field is false in ErrorEnvelope
- fetched_at contains current UTC timestamp
- SuccessEnvelope struct exists (full implementation deferred to Task 7)
- yaml_frontmatter helper compiles and works for both envelope types
- `cargo test envelope` (if tests added) exits 0
</acceptance_criteria>

---

### Task 5: Implement Markdown Converter (Basic)

<read_first>
- .planning/phases/01/01-RESEARCH.md (Section 4: Markdown Generation)
- src/fetch/convert.rs (stub from Task 2)
</read_first>

<action>
Implement src/fetch/convert.rs with HTML to markdown conversion:

1. **MarkdownRenderer struct:**
```rust
pub struct MarkdownRenderer {
    output: String,
    list_depth: usize,
}

impl MarkdownRenderer {
    pub fn new() -> Self { ... }
    
    pub fn render(&mut self, html: &str) -> Result<String> {
        let document = scraper::Html::parse_fragment(html);
        self.visit_nodes(document.root_element());
        Ok(self.output.clone())
    }
    
    fn visit_nodes(&mut self, node: scraper::ElementRef) {
        // Recursive traversal
        // Match on node name and emit markdown
    }
}
```

2. **Support these HTML elements (basic set):**
- Headings: `<h1>` → `# `, `<h2>` → `## `, etc.
- Paragraphs: `<p>` → text + blank line
- Links: `<a href="...">text</a>` → `[text](...)`
- Emphasis: `<strong>` → `**text**`, `<em>` → `*text*`
- Lists: `<ul>`, `<ol>`, `<li>` → markdown lists
- Code: `<code>` → `` `text` ``, `<pre>` → fenced code block
- Blockquotes: `<blockquote>` → `> ` prefix
- Images: `<img src="..." alt="...">` → `![alt](src)`

3. **Escape special characters** in text nodes:
- `*`, `_`, `[`, `]`, `<`, `>`, `#` → prepend `\`

4. **Public API:**
```rust
pub fn html_to_markdown(html: &str) -> Result<String> {
    let mut renderer = MarkdownRenderer::new();
    renderer.render(html)
}
```
</action>

<acceptance_criteria>
- html_to_markdown("&lt;h1&gt;Title&lt;/h1&gt;") returns "# Title\n\n"
- html_to_markdown("&lt;p&gt;Text&lt;/p&gt;") returns "Text\n\n"
- html_to_markdown("&lt;strong&gt;bold&lt;/strong&gt;") returns "**bold**"
- html_to_markdown("&lt;a href='...'&gt;link&lt;/a&gt;") returns "[link](...)"
- Lists render with `-` or `1.` prefixes
- Code blocks use triple backticks
- Special characters in text are escaped
- Nested elements work (e.g., `<p><strong>text</strong></p>`)
</acceptance_criteria>

---

### Task 6: Implement Readability Article Extraction

<read_first>
- .planning/phases/01/01-RESEARCH.md (Section 3: Readability-Style Article Extraction)
- src/fetch/extract.rs (stub from Task 2)
</read_first>

<action>
Implement src/fetch/extract.rs with article extraction logic:

1. **ReadabilityExtractor struct:**
```rust
pub struct ReadabilityExtractor {
    min_score: f64,
}

impl ReadabilityExtractor {
    pub fn extract(&self, html: &str) -> Result<ExtractedArticle> {
        let document = scraper::Html::parse_document(html);
        
        // 1. Extract title from <title> or first <h1>
        let title = self.extract_title(&document);
        
        // 2. Find article node (score elements)
        let article_node = self.find_article_node(&document)?;
        
        // 3. Clean unwanted elements
        let cleaned_html = self.clean_content(article_node);
        
        Ok(ExtractedArticle {
            title,
            content_html: cleaned_html,
            word_count: self.count_words(&cleaned_html),
        })
    }
    
    fn score_element(&self, elem: scraper::ElementRef) -> f64 {
        // Scoring heuristics:
        // +10 for <article>, <main>
        // +5 for class names: content, article, post, entry
        // -5 for class names: comment, ad, sidebar, nav, footer
        // +1 per paragraph, +0.1 per text length
        // -1 per <nav>, <footer>, <aside>
    }
    
    fn find_article_node(&self, doc: &scraper::Html) -> Result<scraper::ElementRef> {
        // Score all candidate elements (div, article, main, section)
        // Return highest scoring element above min_score threshold
    }
    
    fn clean_content(&self, node: scraper::ElementRef) -> String {
        // Remove: nav, footer, aside, [role=navigation], .ad, .comment
        // Keep: p, h1-h6, ul, ol, li, blockquote, pre, code, a, strong, em
        // Return cleaned HTML string
    }
}
```

2. **ExtractedArticle struct:**
```rust
pub struct ExtractedArticle {
    pub title: Option<String>,
    pub content_html: String,
    pub word_count: usize,
}
```

3. **Public API:**
```rust
pub fn extract_article(html: &str) -> Result<ExtractedArticle> {
    let extractor = ReadabilityExtractor { min_score: 20.0 };
    extractor.extract(html)
}
```
</action>

<acceptance_criteria>
- extract_article returns ExtractedArticle with title, content_html, word_count
- Article node scoring prefers `<article>` and `<main>` tags
- Class name heuristics work (positive for "content", negative for "nav")
- Unwanted elements removed (nav, footer, ads)
- Word count is accurate (count whitespace-separated tokens in text)
- Function returns Err if no suitable article node found (score too low)
- Cleaned HTML is valid (parseable by scraper)
</acceptance_criteria>

---

### Task 7: Complete SuccessEnvelope Implementation

<read_first>
- src/fetch/envelope.rs (ErrorEnvelope from Task 4)
- .planning/phases/01/01-RESEARCH.md (Section 5: YAML Frontmatter Serialization)
</read_first>

<action>
Complete SuccessEnvelope implementation in src/fetch/envelope.rs:

1. **Add methods to SuccessEnvelope:**
```rust
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
    
    pub fn to_output(&self, markdown_body: &str) -> Result<String> {
        let frontmatter = yaml_frontmatter(self)?;
        Ok(format!("{}\n{}", frontmatter, markdown_body))
    }
}
```

2. **Update yaml_frontmatter to handle Option fields:**
Ensure null values are serialized as `null` in YAML (not omitted).
</action>

<acceptance_criteria>
- SuccessEnvelope::new sets success=true and fetched_at=now()
- SuccessEnvelope::to_output returns "---\n...yaml...\n---\n\n...markdown..."
- Optional fields (title, redirect_chain) serialize as null when None
- Non-null redirect_chain serializes as YAML list
- Output format matches research spec exactly
</acceptance_criteria>

---

### Task 8: Implement Fetch Orchestrator

<read_first>
- src/fetch/mod.rs (stub from Task 2)
- src/fetch/http.rs (HttpClient from Task 3)
- src/fetch/extract.rs (extract_article from Task 6)
- src/fetch/convert.rs (html_to_markdown from Task 5)
- src/fetch/envelope.rs (envelopes from Tasks 4, 7)
</read_first>

<action>
Implement src/fetch/mod.rs as the orchestrator:

1. **Re-exports:**
```rust
mod http;
mod extract;
mod convert;
mod envelope;

pub use http::{HttpClient, HttpResponse};
pub use extract::ExtractedArticle;
pub use envelope::{SuccessEnvelope, ErrorEnvelope};
```

2. **FetchOptions struct:**
```rust
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
```

3. **fetch_url function:**
```rust
pub async fn fetch_url(url: &str, options: FetchOptions) -> Result<String> {
    // 1. Create HTTP client
    let client = HttpClient::new(options.timeout_secs, options.max_redirects, &options.user_agent);
    
    // 2. Fetch URL (returns HttpResponse or error)
    let response = match client.fetch(url).await {
        Ok(resp) => resp,
        Err(e) => {
            // Convert to ErrorEnvelope and return
            let envelope = ErrorEnvelope { ... };
            return Ok(envelope.to_yaml()?);
        }
    };
    
    // 3. Check for HTTP errors
    if response.status >= 400 {
        let envelope = ErrorEnvelope { status: Some(response.status), ... };
        return Ok(envelope.to_yaml()?);
    }
    
    // 4. Detect charset and convert to UTF-8
    let html = encoding_rs::Encoding::for_label(...)
        .unwrap_or(encoding_rs::UTF_8)
        .decode(&response.body)
        .0
        .into_owned();
    
    // 5. Extract article
    let article = extract::extract_article(&html)?;
    
    // 6. Convert to markdown
    let markdown = convert::html_to_markdown(&article.content_html)?;
    
    // 7. Build SuccessEnvelope
    let envelope = SuccessEnvelope::new(
        response.final_url,
        response.status,
        article.title,
        article.word_count,
        if response.redirect_chain.is_empty() { None } else { Some(response.redirect_chain) },
    );
    
    // 8. Return frontmatter + markdown
    Ok(envelope.to_output(&markdown)?)
}
```
</action>

<acceptance_criteria>
- fetch_url compiles and returns Result<String>
- Success case returns YAML frontmatter + markdown body
- HTTP errors produce ErrorEnvelope (4xx, 5xx)
- Network errors produce ErrorEnvelope (timeout, DNS failure)
- Redirect chain tracked in SuccessEnvelope when redirects occur
- Non-UTF-8 HTML converted to UTF-8 (using encoding_rs)
- Function is async (can be awaited)
</acceptance_criteria>

---

### Task 9: Add Fetch Subcommand

<read_first>
- src/cli.rs (Commands enum)
- src/commands/fetch.rs (to be created)
- .planning/phases/01/01-RESEARCH.md (Section 7: CLI Integration)
</read_first>

<action>
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
        output: Option<PathBuf>,
        
        #[arg(long, help = "Output JSON envelope instead of YAML frontmatter")]
        json: bool,
        
        #[arg(long, default_value = "30", help = "Request timeout in seconds")]
        timeout: u64,
        
        #[arg(long, default_value = "5", help = "Maximum redirects to follow")]
        max_redirects: u32,
        
        #[arg(long, default_value = "mdget/0.2.0", help = "User-Agent header")]
        user_agent: String,
    },
}
```

2. **Create src/commands/fetch.rs:**
```rust
use crate::fetch::{fetch_url, FetchOptions};
use miette::Result;
use std::path::PathBuf;
use tokio_graceful_shutdown::SubsystemHandle;

pub async fn run(
    subsys: SubsystemHandle,
    url: String,
    output: Option<PathBuf>,
    json: bool,
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
    
    // TODO: Implement JSON mode in later tasks (OUT-02)
    // For now, ignore `json` flag (Phase 1 only outputs YAML frontmatter)
    
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

3. **Update src/commands/mod.rs:**
```rust
pub mod command1;
pub mod command2;
pub mod completion;
pub mod fetch;  // Add this line
```
</action>

<acceptance_criteria>
- Commands::Fetch variant exists in src/cli.rs with all fields
- src/commands/fetch.rs exists and compiles
- fetch::run is async and accepts SubsystemHandle
- fetch::run logs to tracing (info level) before fetching
- Output goes to stdout when no -o flag
- Output written to file when -o flag provided
- Timeout and max_redirects passed to FetchOptions
- User-agent passed to FetchOptions
- `cargo check` exits 0
</acceptance_criteria>

---

### Task 10: Wire Fetch Command to Main

<read_first>
- src/main.rs (existing command dispatch)
- src/commands/fetch.rs (from Task 9)
</read_first>

<action>
Update src/main.rs to handle Commands::Fetch:

In the match statement where Commands are dispatched (inside the async tokio runtime), add:

```rust
Commands::Fetch {
    url,
    output,
    json,
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
                json,
                timeout,
                max_redirects,
                user_agent.clone(),
            )
        }))
        .handle()
        .await?;
}
```

Place this BEFORE the existing Command1, Command2, Completion match arms.
</action>

<acceptance_criteria>
- Commands::Fetch arm exists in src/main.rs match statement
- Fetch command runs in a subsystem (graceful shutdown compatible)
- All parameters passed through to fetch::run
- `cargo build` exits 0
- `cargo run -- fetch --help` shows help text for fetch command
- `cargo run -- fetch https://example.com` runs without panicking (may fail with network error if offline, that's OK)
</acceptance_criteria>

---

## Verification Criteria

### Build Verification
- [ ] `cargo build --release` succeeds
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo fmt -- --check` passes
- [ ] Release binary size is under 15MB

### Functional Verification
- [ ] `mdget fetch https://example.com` outputs YAML frontmatter + markdown
- [ ] Frontmatter contains: url, title, status, word_count, fetched_at
- [ ] Markdown body is readable (headings, paragraphs, links converted)
- [ ] `mdget fetch https://httpstat.us/404` produces error envelope with status=404
- [ ] Redirects work: `mdget fetch http://example.com` (redirects to https) tracks redirect_chain
- [ ] Timeout enforced: `mdget fetch https://httpstat.us/200?sleep=60000 --timeout 5` times out
- [ ] Output to file works: `mdget fetch https://example.com -o output.md` creates file

### Edge Cases
- [ ] Invalid URL: `mdget fetch not-a-url` produces error envelope
- [ ] Network error (simulated): produces error envelope
- [ ] Empty/invalid HTML: produces error envelope or minimal markdown
- [ ] Non-UTF-8 content: converted to UTF-8 without panic

### Phase Goal Alignment

**Goal:** Fetch HTML pages and convert to clean markdown

**Success Criteria (from ROADMAP):**
1. ✓ `mdget https://example.com` outputs YAML frontmatter + markdown body
2. ✓ HTTP errors produce error envelopes with predictable structure
3. ✓ Redirects followed and tracked in frontmatter
4. ✓ Article content extracted from HTML (Readability-style)

All success criteria are verified by the functional tests above.

## Must-Haves (Goal-Backward Verification)

These are non-negotiable outcomes for this phase to be considered complete:

1. **Working fetch command** — `mdget fetch <url>` must execute without panic
2. **YAML frontmatter output** — Every successful fetch outputs valid YAML frontmatter
3. **Markdown body** — HTML converted to readable markdown (headings, paragraphs, links)
4. **Error handling** — HTTP errors and network errors produce error envelopes (not panics)
5. **Redirect tracking** — Redirects followed and final URL + chain recorded
6. **Article extraction** — Readability algorithm removes nav/footer/ads
7. **Timeout enforcement** — Requests respect --timeout flag
8. **Dependency integration** — curl, scraper, serde_yaml working together

If any must-have is missing, the phase is incomplete.

## Truths (Constraints That Must Hold)

1. **HTTP client is async-compatible** — curl runs in spawn_blocking, never blocks tokio runtime
2. **YAML frontmatter format is stable** — Matches spec exactly (---\n...yaml...\n---\n\n)
3. **Error envelopes are structured** — success: false, error, message, status fields always present
4. **Markdown escaping is safe** — Special chars in text nodes escaped (no broken markdown syntax)
5. **Binary size under 15MB** — Release build stays under target (critical constraint)
6. **Dependencies are minimal** — Only 7 new deps added (curl, scraper, html5ever, serde_yaml, chrono, url, encoding_rs)

These truths are architectural decisions from CONTEXT.md and RESEARCH.md.

## Rollback Plan

If verification fails:

1. **Build failures:** Revert dependency changes in Cargo.toml, fix compilation errors one module at a time
2. **Functional failures:** Disable failing module (stub it), commit working subset, open follow-up task
3. **Performance failures (binary size >15MB):** Profile with `cargo bloat`, remove unused features, consider alternative crates

Rollback commits:
- `git revert <commit-hash>` for each failed task
- Preserve RESEARCH.md and PLAN.md (they inform retry)

---

*Plan created: 2026-05-21*
*Ready for execution.*
