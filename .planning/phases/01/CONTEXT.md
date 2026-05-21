# Phase 1: HTTP Core & Basic Extraction

**Created:** 2026-05-21
**Phase Goal:** Fetch HTML pages and convert to clean markdown

## Requirements Coverage

Phase 1 implements the foundational HTTP client and HTML-to-markdown pipeline:

### HTTP Core
- **HTTP-01**: Fetch URLs via HTTP/HTTPS with curl crate
- **HTTP-02**: Follow redirects (max 5 default, configurable)
- **HTTP-03**: Handle timeouts (30s default, configurable)

### Content Extraction
- **EXTR-01**: Parse HTML with scraper crate
- **EXTR-02**: Article extraction (Readability-style)
- **EXTR-03**: Convert HTML to markdown (pulldown-cmark-based)
- **EXTR-04**: Extract page title from HTML

### Output Format
- **OUT-01**: YAML frontmatter (url, title, status, word_count, fetched_at)
- **OUT-05**: Error envelope with predictable structure

## Success Criteria

1. `mdget https://example.com` outputs YAML frontmatter + markdown body
2. HTTP errors produce error envelopes with predictable structure
3. Redirects followed and tracked in frontmatter
4. Article content extracted from HTML (Readability-style)

## Technical Context

### Existing Foundation (from template)
- ✓ CLI scaffolding with clap (commands, config, completions)
- ✓ Layered config system (defaults → user → env → CLI)
- ✓ Production logging with tracing (console + file)
- ✓ Graceful shutdown with tokio-graceful-shutdown
- ✓ XDG directory support

### What Needs Building
- HTTP client integration (curl crate)
- HTML parsing (scraper crate)
- Article extraction (Readability algorithm port)
- Markdown generation (custom renderer)
- YAML frontmatter serialization
- Error envelope structure
- `fetch` subcommand implementation

### Reference Implementation
- mdurl (TypeScript): Proven output format, Readability extraction
- Output structure: YAML frontmatter block + markdown body
- Error handling: Structured envelopes with status/message/url

## Constraints

- **Tech stack:** Rust 2024, edition 2024, MSRV 1.85+
- **Binary size:** Target <15MB release binary
- **Performance:** Must be faster than mdurl for typical webpage
- **Dependencies:** Minimize count, avoid unmaintained crates
- **License:** MIT

## Key Crates to Integrate

| Crate | Purpose | Notes |
|-------|---------|-------|
| `curl` | HTTP client | libcurl bindings, HTTP/2 support |
| `scraper` | HTML parsing | CSS selectors, html5ever based |
| `pulldown-cmark` | Markdown parsing | For custom renderer |
| `serde_yaml` | YAML serialization | For frontmatter |
| `chrono` | Timestamp handling | For `fetched_at` field |

## Integration Points with Existing Code

- Add new `fetch` subcommand to `src/commands/`
- Extend `Commands` enum in `src/cli.rs`
- Add HTTP/extraction config to `src/config.rs`
- Reuse logging infrastructure for HTTP trace logging
- Use existing error handling (miette) for error envelopes

## Out of Scope for Phase 1

- PDF/JSON/RSS handling (Phase 2)
- Caching (Phase 3)
- Authentication (Phase 4)
- Filtering/selectors (Phase 4)
- Browser rendering (v2)
- Search mode (v2)

---
*Context prepared for planning: 2026-05-21*
