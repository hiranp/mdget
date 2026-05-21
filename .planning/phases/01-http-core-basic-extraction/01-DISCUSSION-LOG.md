# Phase 1: HTTP Core & Basic Extraction - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-21
**Phase:** 1-HTTP Core & Basic Extraction
**Areas discussed:** HTTP client integration, Article extraction approach, Markdown conversion pipeline, Error handling strategy, YAML frontmatter fields, Testing strategy, Performance targets, CLI command structure

---

## HTTP Client Integration

### Should HTTP fetching be a reusable module?

| Option | Description | Selected |
|--------|-------------|----------|
| Reusable module (Recommended) | Create src/http/client.rs with fetch functions that commands import. Better for testing, allows cache/auth layers to extend it later. Fits the existing modularity (cli, config, log are separate). | ✓ |
| Embedded in fetch command | Put curl logic directly in src/commands/fetch.rs. Simpler for now, but harder to reuse if other commands need HTTP (e.g., cache subcommands might need to re-fetch for validation). | |

**User's choice:** Reusable module (Recommended)

### How should HTTP requests handle cancellation?

| Option | Description | Selected |
|--------|-------------|----------|
| Immediate abort (Recommended) | Drop the curl handle immediately on shutdown signal. Fast exit, but partial responses are lost. Good for interactive use where users expect instant response to Ctrl+C. | ✓ |
| Graceful drain | Allow in-flight requests to complete during the 1-second shutdown window (current shutdown timeout). Better for batch operations, but users might wait up to 1 second. | |
| You decide | Let Claude choose the best approach based on typical curl client patterns. | |

**User's choice:** Immediate abort (Recommended)

### What should the default HTTP timeout be?

| Option | Description | Selected |
|--------|-------------|----------|
| 30 seconds (Recommended) | Standard curl default. Reasonable for most webpages, configurable via --timeout flag. Mentioned in REQUIREMENTS.md as the default. | ✓ |
| 60 seconds | More generous for slow servers or large pages. Better for PDF extraction or slow APIs. | |
| No default timeout | Let requests run indefinitely unless user sets --timeout. Riskier (could hang), but gives maximum flexibility. | |

**User's choice:** 30 seconds (Recommended)

### Should redirect chains be exposed in frontmatter?

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, full chain (Recommended) | Include all redirect URLs in frontmatter (e.g., redirect_chain: [url1, url2, final]). Useful for agents to detect URL changes, paywalls, or login redirects. ROADMAP.md mentions 'redirects tracked in frontmatter'. | ✓ |
| Only final URL | Just record the final URL after redirects. Simpler frontmatter, but loses redirect context. | |
| You decide | Let Claude choose based on agent-first design principles. | |

**User's choice:** Yes, full chain (Recommended)

---

## Article Extraction Approach

### Should we port mdurl logic or use readability crate?

| Option | Description | Selected |
|--------|-------------|----------|
| Port mdurl logic (Recommended) | Direct control over the extraction algorithm, can update and tune it. More work upfront but proven logic you already trust from mdurl. | ✓ |
| Use readability crate | Faster initial implementation, but unmaintained (2019). Risk of bugs, incompatibilities with modern HTML, no security updates. | |
| Search for alternatives | Look for other Rust extraction libraries or newer implementations before deciding. | |

**User's choice:** Port mdurl logic (Recommended)

### How should the article extraction module be organized?

| Option | Description | Selected |
|--------|-------------|----------|
| Single extractor module (Recommended) | Create src/extractor/ with main extraction logic. Follows the existing pattern (cli, config, log are all single modules). Can add scoring and filtering as sub-modules later. | |
| Split into scoring + extraction | Separate src/scoring/ (node scoring algorithm) and src/extractor/ (DOM manipulation). More granular but adds complexity upfront. | ✓ |
| You decide | Let Claude choose based on Rust module best practices. | |

**User's choice:** Split into scoring + extraction

### Should metadata extraction be part of extractor or separate?

| Option | Description | Selected |
|--------|-------------|----------|
| Part of extractor (Recommended) | Single pass through the DOM extracts both article content and metadata. More efficient, less complexity. Metadata is closely tied to content extraction. | ✓ |
| Separate metadata module | Independent src/metadata/ module. Allows metadata extraction without full article extraction (useful for --no-resources mode or quick info). More flexible but duplicates DOM traversal. | |

**User's choice:** Part of extractor (Recommended)

### What happens when extraction produces empty results?

| Option | Description | Selected |
|--------|-------------|----------|
| Return cleaned body (Recommended) | Fall back to the full cleaned HTML body (strip scripts/styles but keep all content). Better than nothing, users can still read the page. Matches curl behavior of always returning something. | ✓ |
| Return error | Treat empty extraction as a failure and return an error envelope. Clearer signal to agents that extraction failed, but less useful than partial content. | |
| Return raw HTML | Return the original HTML in the markdown output as a fenced code block. Preserves all content but defeats the purpose of markdown conversion. | |

**User's choice:** Return cleaned body (Recommended)

---

## Markdown Conversion Pipeline

### Should conversion be streaming or buffer-based?

| Option | Description | Selected |
|--------|-------------|----------|
| Buffer entire DOM (Recommended) | Simpler implementation, matches how Readability works (needs full DOM for scoring). For v1.0 with typical webpage sizes, buffering is fine. Can optimize later if needed. | ✓ |
| Streaming conversion | Process nodes as you traverse the DOM, write markdown incrementally. More complex, requires careful state management (nested lists, tables). Better for very large pages. | |
| You decide | Let Claude choose based on typical webpage sizes and Rust patterns. | |

**User's choice:** Buffer entire DOM (Recommended)

### How to handle unsupported HTML elements?

| Option | Description | Selected |
|--------|-------------|----------|
| Preserve as HTML (Recommended) | Keep unsupported elements as raw HTML in the markdown output. Markdown supports inline HTML, so nothing is lost. Agents can still see the structure. | ✓ |
| Convert to text | Extract just the text content, lose the semantic structure. Simpler output but loses information (e.g., <figure> captions become indistinguishable from body text). | |
| Skip entirely | Omit unsupported elements from output. Cleanest markdown but potentially loses important content. | |

**User's choice:** Preserve as HTML (Recommended)

### How should tables be handled?

| Option | Description | Selected |
|--------|-------------|----------|
| Flatten to simple markdown (Recommended) | Convert tables to markdown tables when possible, fall back to HTML for nested tables or tables with complex formatting. Pragmatic approach that handles most cases. | ✓ |
| Always preserve as HTML | Keep all tables as HTML. Preserves exact layout but makes output less readable for agents expecting markdown tables. | |
| Text representation | Convert tables to plain text with indentation/spacing. More readable than HTML but loses table structure entirely. | |

**User's choice:** Flatten to simple markdown (Recommended)

### How to identify code blocks?

| Option | Description | Selected |
|--------|-------------|----------|
| Detect by class/element (Recommended) | Use <code>, <pre>, and common code classes (language-*, hljs-*) to identify code. Standard approach that works with most syntax highlighters. Wrap in markdown fenced blocks with language hints when available. | |
| Heuristic detection | Use content patterns (indentation, punctuation, keywords) to guess if text is code. More flexible but error-prone (might misidentify structured data as code). | |
| You decide | Let Claude choose based on HTML parsing best practices. | ✓ |

**User's choice:** You decide

---

## Error Handling Strategy

### Should error types be unified or separate?

| Option | Description | Selected |
|--------|-------------|----------|
| Unified error envelope (Recommended) | Single error structure with an error_type field (http_error, parse_error, extraction_error). Consistent output format for agents, easier to handle. ROADMAP.md mentions 'error envelope with predictable structure'. | ✓ |
| Separate error types | Different enum variants for each error category with specific fields. More type-safe in Rust but agents need to handle multiple formats. | |
| HTTP status-based | Use HTTP status codes to indicate error type (4xx for client errors, 5xx for server/extraction errors). Familiar pattern but less expressive than structured errors. | |

**User's choice:** Unified error envelope (Recommended)

### What metadata in error envelopes?

| Option | Description | Selected |
|--------|-------------|----------|
| Essential fields (Recommended) | error_type, message, url, timestamp, status_code (for HTTP errors). Enough for agents to understand and log errors without overwhelming the output. | ✓ |
| Detailed diagnostics | Add request_headers, response_headers, redirect_chain, parsing_context. More debugging info but larger error payloads. | |
| Minimal | Just error_type and message. Simplest output but agents lose context about what failed. | |

**User's choice:** Essential fields (Recommended)

### Should errors respect --json flag?

| Option | Description | Selected |
|--------|-------------|----------|
| Always JSON (Recommended) | Errors are always JSON regardless of --json flag. Consistent, parseable format for agents even when normal output is markdown. Agents can detect errors by checking for error_type field. | ✓ |
| Respect --json flag | Errors are markdown with frontmatter by default, JSON only with --json. More consistent with normal output format but harder for agents to parse error cases. | |
| You decide | Let Claude choose based on agent-first design principles. | |

**User's choice:** Always JSON (Recommended)

### How to handle partial successes?

| Option | Description | Selected |
|--------|-------------|----------|
| Return success with warning (Recommended) | Return the cleaned HTML body as markdown with a warnings field in frontmatter noting extraction failed. Agents get something useful rather than an error. Matches the 'Return cleaned body' fallback decision. | ✓ |
| Return error | Treat extraction failure as a hard error even if HTTP succeeded. Clearer signal but agents get nothing back. | |
| Return both | Success response with both the partial markdown and an errors array. Most complete info but more complex structure. | |

**User's choice:** Return success with warning (Recommended)

---

## YAML Frontmatter Fields

### What additional frontmatter fields to include?

| Option | Description | Selected |
|--------|-------------|----------|
| Extended set (Recommended) | Add: final_url (after redirects), redirect_chain (array), content_type, extraction_method (readability/full_body), has_warnings (boolean). Rich context for agents without being overwhelming. | ✓ |
| Minimal core | Stick to just the 5 basic fields mentioned in REQUIREMENTS.md. Simplest output, but agents lose redirect and content-type context. | |
| Comprehensive | Include everything: basic + extended + response_time_ms, server_headers (selected), language_detected, author, publish_date (if available). Maximum info but larger payloads. | |

**User's choice:** Extended set (Recommended)

### snake_case or camelCase for field names?

| Option | Description | Selected |
|--------|-------------|----------|
| snake_case (Recommended) | Matches Rust naming conventions and mdurl's existing format. Consistent with word_count, fetched_at already mentioned in docs. | ✓ |
| camelCase | Common in JSON APIs, might be more familiar to TypeScript/JavaScript agents. | |
| You decide | Let Claude choose based on YAML conventions and existing patterns. | |

**User's choice:** snake_case (Recommended)

---

## Testing Strategy

### Primary testing approach?

| Option | Description | Selected |
|--------|-------------|----------|
| Fixture files (Recommended) | Store HTML snapshots as test fixtures, test extraction and conversion in isolation. Fast, reliable, no network dependency. Can add a few integration tests with real URLs for smoke testing. | ✓ |
| Mock server | Use a mock HTTP server (like wiremock-rs) to simulate responses. More realistic HTTP testing but more setup complexity. | |
| Real URLs | Test against real websites. Most realistic but fragile (sites change, network issues, slow tests). | |

**User's choice:** Fixture files (Recommended)

### What types of test fixtures?

| Option | Description | Selected |
|--------|-------------|----------|
| Representative corpus (Recommended) | 5-10 fixtures covering: news article, blog post, documentation page, complex nested structure, table-heavy page, edge cases (empty extraction, malformed HTML). Covers common patterns without excessive maintenance. | ✓ |
| Exhaustive corpus | 20+ fixtures covering many edge cases and website types. Thorough but high maintenance as HTML patterns evolve. | |
| Minimal set | 2-3 basic fixtures (simple article, nested structure). Fast to maintain but might miss edge cases. | |

**User's choice:** Representative corpus (Recommended)

---

## Performance Targets

### What performance target for v1.0?

| Option | Description | Selected |
|--------|-------------|----------|
| Measurably faster (Recommended) | Target 20-30% faster than mdurl for typical webpage (fetch + parse + convert). Meaningful improvement that justifies the rewrite, achievable with Rust's advantages. | ✓ |
| Comparable performance | Within 10% of mdurl performance. Good enough for v1, focus on feature parity over optimization. | |
| 2x faster | Aggressive target: twice as fast as mdurl. Impressive but might delay v1.0 while optimizing. | |

**User's choice:** Measurably faster (Recommended)

### How to measure performance?

| Option | Description | Selected |
|--------|-------------|----------|
| Benchmark suite (Recommended) | Use criterion.rs for micro-benchmarks (extraction, conversion) + hyperfine for end-to-end comparison against mdurl. Track performance over time, catch regressions in CI. | ✓ |
| Manual comparison | Periodic manual testing against mdurl with a set of URLs. Simpler but less rigorous, easier to miss regressions. | |
| You decide | Let Claude choose based on Rust testing best practices. | |

**User's choice:** Benchmark suite (Recommended)

---

## CLI Command Structure

### Top-level or subcommand?

| Option | Description | Selected |
|--------|-------------|----------|
| Top-level: mdget <url> (Recommended) | Simplest UX, matches curl. The URL becomes a positional argument at the top level. Future subcommands (cache, auth) coexist. Most ergonomic for the primary use case. | ✓ |
| Subcommand: mdget fetch <url> | More explicit, consistent with the existing Command1/Command2 pattern. Better for complex CLI tools but extra typing for the main use case. | |
| Both | Support both mdget <url> (shorthand) and mdget fetch <url> (explicit). Flexible but more code to maintain. | |

**User's choice:** Top-level: mdget <url> (Recommended)

### Flag style?

| Option | Description | Selected |
|--------|-------------|----------|
| Both short and long (Recommended) | Support both -H/--header, -o/--output patterns. Familiar to curl users (short forms) while being self-documenting (long forms). More work to define but better UX. | ✓ |
| Long-form only | Only --header, --output. More readable, less typing ambiguity, but less curl-like. | |
| You decide | Let Claude choose based on Rust CLI conventions and user expectations. | |

**User's choice:** Both short and long (Recommended)

---

## Claude's Discretion

- **Code block detection:** Claude has discretion to choose the best approach for identifying inline code and code blocks (likely using `<code>`, `<pre>`, and common code classes with markdown fenced blocks).

## Deferred Ideas

None — discussion stayed within phase scope.
