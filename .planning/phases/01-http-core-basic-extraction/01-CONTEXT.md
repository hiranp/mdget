# Phase 1: HTTP Core & Basic Extraction - Context

**Gathered:** 2026-05-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Fetch HTML pages via HTTP/HTTPS and convert them to clean markdown with YAML frontmatter. Implement the core pipeline: HTTP client → HTML extraction → markdown conversion → structured output. This establishes the foundation that later phases will extend with caching, authentication, and advanced features.

</domain>

<decisions>
## Implementation Decisions

### HTTP Client Integration
- **D-01:** HTTP fetching will be a reusable module (`src/http/client.rs`) that multiple commands can import, rather than embedded directly in a fetch command. This supports testing and allows cache/auth layers to extend it in later phases.
- **D-02:** HTTP requests will immediately abort on shutdown signal (Ctrl+C). Fast exit for interactive use where users expect instant response to cancellation.
- **D-03:** Default HTTP timeout is 30 seconds (configurable via `--timeout` flag). Standard curl default, reasonable for most webpages.
- **D-04:** Redirect chains will be fully exposed in frontmatter output (`redirect_chain: [url1, url2, final]`). Useful for agents to detect URL changes, paywalls, or login redirects.

### Article Extraction Approach
- **D-05:** Port mdurl's TypeScript Readability logic to Rust rather than using the unmaintained `readability` crate. Direct control over the extraction algorithm, can update and tune it.
- **D-06:** Article extraction module will be split into two sub-modules: `src/scoring/` (node scoring algorithm) and `src/extractor/` (DOM manipulation). More granular organization for complex extraction logic.
- **D-07:** Metadata extraction (title, description, canonical URL) will be part of the extractor module, not separate. Single pass through the DOM extracts both article content and metadata efficiently.
- **D-08:** When Readability extraction produces empty or very short results, fall back to the cleaned HTML body (strip scripts/styles but keep all content). Better than nothing, users can still read the page.

### Markdown Conversion Pipeline
- **D-09:** HTML-to-markdown conversion will buffer the entire DOM rather than streaming. Simpler implementation, matches how Readability works (needs full DOM for scoring). For v1.0 with typical webpage sizes, buffering is fine.
- **D-10:** HTML elements without clear markdown equivalents (`<details>`, `<figure>`, `<aside>`) will be preserved as raw HTML in the markdown output. Markdown supports inline HTML, so structure is not lost.
- **D-11:** Nested or complex tables will be flattened to simple markdown tables when possible, falling back to HTML for truly complex cases. Pragmatic approach that handles most tables readably.

### Error Handling Strategy
- **D-12:** All error types (HTTP error, parse error, extraction failure) will use a unified error envelope structure with an `error_type` field. Consistent output format for agents, easier to handle.
- **D-13:** Error envelopes will include essential fields: `error_type`, `message`, `url`, `timestamp`, `status_code` (for HTTP errors). Enough for agents to understand and log errors without overwhelming the output.
- **D-14:** Error envelopes will always be JSON regardless of the `--json` flag. Consistent, parseable format for agents even when normal output is markdown.
- **D-15:** Partial successes (e.g., HTTP fetch succeeded but extraction failed) will return success with a `warnings` field in frontmatter noting the issue. Agents get useful content rather than an error.

### YAML Frontmatter Fields
- **D-16:** Beyond basic fields (`url`, `title`, `status`, `word_count`, `fetched_at`), include: `final_url` (after redirects), `redirect_chain` (array), `content_type`, `extraction_method` (readability/full_body), `has_warnings` (boolean). Rich context without being overwhelming.
- **D-17:** Frontmatter field names will use `snake_case` (e.g., `word_count`, `fetched_at`). Matches Rust naming conventions and mdurl's existing format.

### Testing Strategy
- **D-18:** Primary testing approach will use HTML fixture files (test snapshots) to test extraction and conversion in isolation. Fast, reliable, no network dependency. Add a few integration tests with real URLs for smoke testing.
- **D-19:** Test corpus will include 5-10 representative fixtures: news article, blog post, documentation page, complex nested structure, table-heavy page, edge cases (empty extraction, malformed HTML). Covers common patterns without excessive maintenance.

### Performance Targets
- **D-20:** Target 20-30% faster than mdurl for typical webpage (fetch + parse + convert). Meaningful improvement that justifies the Rust rewrite, achievable with Rust's performance advantages.
- **D-21:** Performance will be measured using criterion.rs for micro-benchmarks (extraction, conversion) and hyperfine for end-to-end comparison against mdurl. Track performance over time, catch regressions in CI.

### CLI Command Structure
- **D-22:** Main fetch operation will be top-level: `mdget <url>` (not `mdget fetch <url>`). Simplest UX, matches curl. Future subcommands (cache, auth) coexist.
- **D-23:** Flags will support both short and long forms (e.g., `-H`/`--header`, `-o`/`--output`). Familiar to curl users while being self-documenting.

### Claude's Discretion
- **Code detection:** Claude has discretion to choose the best approach for identifying inline code and code blocks (likely using `<code>`, `<pre>`, and common code classes like `language-*`, `hljs-*` with markdown fenced blocks).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Documentation
- `.planning/PROJECT.md` — Core value proposition, requirements overview, existing codebase context, domain knowledge (HTTP client libraries, extraction approaches, Rust crates)
- `.planning/REQUIREMENTS.md` — v1 requirements (HTTP-01 to HTTP-06, EXTR-01 to EXTR-05, OUT-01 to OUT-05)
- `.planning/ROADMAP.md` — Phase 1 goal and success criteria

### Codebase Maps
- `.planning/codebase/STACK.md` — Technology stack (Rust 2024, clap, tokio, miette, tracing)
- `.planning/codebase/ARCHITECTURE.md` — Existing architecture patterns (subsystem pattern, config hierarchy, logging setup, module boundaries)
- `.planning/codebase/INTEGRATIONS.md` — File system integration (XDG directories), future integration points

No external specs — requirements fully captured in decisions above.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Subsystem pattern** (`tokio-graceful-shutdown`): Commands run as subsystems with `SubsystemHandle` for cancellation. HTTP client should integrate with this pattern using `tokio::select!` to monitor `on_shutdown_requested()`.
- **Config hierarchy** (`config` crate): Layered config system (defaults → system → user → env vars → CLI flags). HTTP timeouts, redirect limits, and other HTTP options should be configurable through this system.
- **Structured logging** (`tracing`): File + line tracking, dual output (stdout + file). HTTP requests should be instrumented with spans for debugging.
- **Error handling** (`miette::Result`): All fallible operations return `miette::Result<T>`. HTTP errors, parse errors, extraction failures should use `.into_diagnostic()` for rich error context.

### Established Patterns
- **Module organization**: Separate modules for distinct concerns (cli, config, log). HTTP client and extraction logic should follow this pattern with `src/http/`, `src/scoring/`, `src/extractor/`, `src/markdown/` modules.
- **CLI derive API** (`clap`): Declarative command-line interface. The top-level URL positional argument and flags like `--timeout`, `--json`, `--no-frontmatter` will use clap's derive macros.
- **Async runtime** (`tokio`): Full tokio features enabled. HTTP client (curl crate) async calls integrate naturally with the existing tokio runtime.

### Integration Points
- **Command dispatch in main.rs**: Add URL positional argument pattern matching before subcommand dispatch. Something like: `if let Some(url) = cli.url { /* handle fetch */ } else { match cli.command { ... } }`
- **Config struct**: Extend `GlobalConfig` with an `http` section for timeout, redirect limits, user-agent, etc.
- **Completion generation**: Shell completions already set up in `build.rs`. Flags for the fetch command will auto-generate.

</code_context>

<specifics>
## Specific Ideas

- **mdurl reference**: The TypeScript implementation in mdurl serves as the proven reference for Readability extraction logic and output format (YAML frontmatter + markdown body). Port the core scoring and extraction algorithms.
- **curl crate**: PROJECT.md explicitly recommends using the `curl` crate (libcurl bindings) for HTTP. It provides HTTP/2, redirects, cookies, and TLS handling out of the box.
- **scraper crate**: For HTML parsing, use the `scraper` crate (CSS selectors) with `html5ever` under the hood. Standard choice in Rust for HTML DOM manipulation.
- **pulldown-cmark**: While it's a markdown *parser*, understanding its output structure helps design the reverse (HTML → markdown) renderer.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope. No scope creep detected during the discussion.

</deferred>

---

*Phase: 1-HTTP Core & Basic Extraction*
*Context gathered: 2026-05-21*
