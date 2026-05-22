# Phase 2: Content Types & Output Modes - Research

**Researched:** 2026-05-21
**Phase:** 2
**Confidence:** MEDIUM-HIGH

## Executive Summary

Phase 2 should extend the existing HTML-first pipeline in `src/fetch/mod.rs` into a typed content router that preserves the current HTML path as-is and adds specialized handlers for JSON, plain text, feed/XML, and PDF. The lowest-risk architecture is to keep transport in `src/fetch/http.rs`, orchestration in `src/fetch/mod.rs`, and move per-content transformations into new handler modules under `src/fetch/handlers/`. [VERIFIED: codebase read of src/fetch/mod.rs, src/fetch/http.rs]

The current command and CLI surfaces already support timeout, redirects, user agent, compact mode, max-body-words, and output file writing. Phase 2 should add request customization (`-H`, `--cookie`, `--bearer`) and explicit output modes (`default frontmatter`, `--json`, `--no-frontmatter`) as typed options, avoiding stringly branching. [VERIFIED: codebase read of src/cli.rs, src/commands/fetch.rs]

For crate strategy, keep existing `ureq` transport and add minimal-purpose crates: `serde_json` for robust pretty JSON rendering, `mime` for content-type normalization/routing, `feed-rs` for RSS/Atom detection and parsing, and `pdf-extract` behind a feature gate for CONT-02. This keeps dependencies focused while preserving binary size control. [CITED: https://docs.rs/ureq/latest/ureq/] [CITED: https://docs.rs/serde_json/latest/serde_json/] [CITED: https://docs.rs/mime/latest/mime/] [CITED: https://docs.rs/feed-rs/latest/feed_rs/] [CITED: https://docs.rs/pdf-extract/latest/pdf_extract/] [VERIFIED: cargo search results]

## Requirement Coverage Strategy

| Requirement | Implementation approach in this codebase                                                                                                                                                                                                                                                                                            |
| ----------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| CONT-01     | Keep current HTML extraction path in `fetch_url` as baseline path (router case `text/html` + `application/xhtml+xml`) and avoid behavior changes to current markdown body + frontmatter output. [VERIFIED: src/fetch/mod.rs]                                                                                                        |
| CONT-02     | Add `pdf` handler: use `pdf_extract::extract_text_from_mem(&response.body)` for markdown output body text; map failures to `pdf_extraction_failed` error classification in envelope path. [CITED: https://docs.rs/pdf-extract/latest/pdf_extract/fn.extract_text_from_mem.html] [ASSUMED: exact extraction quality on complex PDFs] |
| CONT-03     | Add `json` handler: parse bytes with `serde_json::from_slice::<serde_json::Value>()` and output fenced pretty JSON (`to_string_pretty`) in markdown mode. [CITED: https://docs.rs/serde_json/latest/serde_json/fn.from_slice.html] [CITED: https://docs.rs/serde_json/latest/serde_json/fn.to_string_pretty.html]                   |
| CONT-04     | Add `text` handler: decode body using existing charset path; pass normalized text through with optional token reduction policy (see sequencing). [VERIFIED: src/fetch/mod.rs charset logic]                                                                                                                                         |
| CONT-05     | Add `feed` handler: parse body via `feed_rs::parser::parse`, render canonical markdown entry list with title/link/date summary; if parse fails for XML media type, fallback to fenced XML body. [CITED: https://docs.rs/feed-rs/latest/feed_rs/parser/fn.parse.html]                                                                |
| OUT-02      | Add `OutputMode::JsonEnvelope` and `SuccessEnvelope::to_json_output(body)` path using `serde_json` serialization; ensure `--json` returns stable envelope schema + body. [ASSUMED: envelope JSON schema final field names pending implementation choice]                                                                            |
| OUT-03      | Add `OutputMode::MarkdownOnly` (`--no-frontmatter`) returning body only; keep reducers enabled for markdown-like outputs and disabled for structured JSON output mode. [VERIFIED: current reducer in src/fetch/reduce.rs]                                                                                                           |
| OUT-04      | Keep write-to-file path in `src/commands/fetch.rs`; write serialized output string verbatim regardless of mode. [VERIFIED: src/commands/fetch.rs]                                                                                                                                                                                   |
| HTTP-04     | Extend CLI with repeatable `-H/--header` and parse strict `Name: Value`; pass typed headers to transport and apply with `RequestBuilder::header`. [CITED: https://docs.rs/ureq/latest/ureq/struct.RequestBuilder.html#method.header]                                                                                                |
| HTTP-05     | Add repeatable `--cookie`; combine values into deterministic `Cookie` header string (`; ` joined in declaration order), then set request header. [ASSUMED: deterministic merge format accepted by target services]                                                                                                                  |
| HTTP-06     | Add `--bearer` option and inject `Authorization: Bearer <token>` at transport layer; override any user-supplied Authorization header to satisfy D2-08. [VERIFIED: 02-CONTEXT.md decisions]                                                                                                                                          |
| EXTR-05     | Extend extraction model/envelope with optional `description` and `canonical_url`; derive from `<meta name="description">`, `<meta property="og:description">`, and `<link rel="canonical">`. [ASSUMED: selected precedence order]                                                                                                   |

## Recommended Crates

### Keep (already in repo)

1. `ureq = 3.1.x` (currently `3.1` in Cargo.toml) for HTTP transport and request header customization. [VERIFIED: Cargo.toml] [CITED: https://docs.rs/ureq/latest/ureq/]

### Add (Phase 2)

1. `serde_json = 1.0.150` for JSON parse/pretty-print and JSON envelope serialization. [VERIFIED: cargo search]
- Why: Standard serde-native JSON path, low integration risk with existing serde types.
- Tradeoff: Adds dependency weight but avoids hand-rolled formatting/parsing bugs.

2. `mime = 0.3.17` for robust MIME parsing/normalization and wildcard checks (`+json`, `text/*`, etc.). [VERIFIED: cargo search]
- Why: Safer than manual split/lowercase logic as content types diversify.
- Tradeoff: Small extra dependency; simplifies router logic significantly.

3. `feed-rs = 2.3.1` for RSS/Atom/feed parsing from raw bytes and unified feed model. [VERIFIED: cargo search]
- Why: Handles Atom + RSS under one model; avoids custom XML schema parsing.
- Tradeoff: Additional dependency graph; parsing leniency must be tested for malformed feeds.

4. `pdf-extract = 0.10.0` behind feature `pdf` (optional) for PDF text extraction from response bytes. [VERIFIED: cargo search]
- Why: Fastest path to CONT-02 without external binary runtime.
- Tradeoff: Extraction fidelity varies by PDF structure; should map failures predictably.

### Alternatives considered (not recommended as primary)

1. `atom_syndication = 0.12.8` only parses Atom, not RSS; would require extra crate for RSS. [VERIFIED: cargo search] [CITED: https://docs.rs/atom_syndication/latest/atom_syndication/]
2. Direct `quick-xml = 0.40.1` parsing gives maximal control but increases implementation complexity and schema handling burden for this phase. [VERIFIED: cargo search] [CITED: https://docs.rs/quick-xml/latest/quick_xml/]

### Package trust note

`slopcheck` is unavailable in this environment; all new package recommendations should be treated as `[ASSUMED]` for legitimacy and human-verified before install. [VERIFIED: terminal output]

## Proposed Module Changes

### Existing files to modify

1. `src/cli.rs`
- Add to `Commands::Fetch`:
  - `header: Vec<String>` (`-H`, `--header`, repeatable)
  - `cookie: Vec<String>` (`--cookie`, repeatable)
  - `bearer: Option<String>` (`--bearer`)
  - `json: bool` (`--json`)
  - `no_frontmatter: bool` (`--no-frontmatter`)
- Add clap conflict: `conflicts_with = "no_frontmatter"` on `--json` and inverse on `--no-frontmatter`.

2. `src/main.rs`
- Thread new fetch args into `commands::fetch::run` call.

3. `src/commands/fetch.rs`
- Expand `run(...)` signature with request options and output mode flags.
- Build typed `FetchOptions` with `RequestOptions` and `OutputMode`.

4. `src/fetch/mod.rs`
- Add content router and typed option enums/structs.
- Preserve existing HTML code path as router branch.
- Dispatch to content handlers and then envelope serializer selected by `OutputMode`.

5. `src/fetch/http.rs`
- Extend `HttpClient::fetch` to accept typed request options (headers, cookies, bearer).
- Apply header precedence rule: bearer overwrites Authorization from custom headers.

6. `src/fetch/extract.rs`
- Extend `ExtractedArticle` with optional `description`, `canonical_url`.
- Add metadata extraction helpers.

7. `src/fetch/envelope.rs`
- Add optional metadata fields to `SuccessEnvelope`.
- Add JSON serialization path (for OUT-02) while preserving YAML output for default.

8. `tests/fetch_envelope_integration.rs`
- Keep existing HTML regression tests and add coverage for new modes/content types.

### New modules recommended

1. `src/fetch/router.rs`
- `normalize_content_type(raw: Option<&str>) -> NormalizedContentType`
- `route_content(...) -> HandlerKind`

2. `src/fetch/request.rs`
- Header parsing and validation (`Name: Value`), cookie merge utility.

3. `src/fetch/output.rs`
- Output-mode serializer helpers (`frontmatter+body`, `markdown-only`, `json-envelope`).

4. `src/fetch/handlers/mod.rs`
- Trait or enum-based handler dispatch surface.

5. `src/fetch/handlers/html.rs`
- Thin wrapper around existing extract+convert+reduce path.

6. `src/fetch/handlers/json.rs`
- Pretty JSON fenced rendering and json-mode behavior.

7. `src/fetch/handlers/text.rs`
- Plain text normalization.

8. `src/fetch/handlers/feed.rs`
- `feed-rs` parse + markdown list rendering; XML fallback path signal.

9. `src/fetch/handlers/pdf.rs`
- Feature-gated PDF extraction path + failure classification.

## Data Model and CLI Surface

### Typed CLI and fetch model (recommended)

```rust
pub enum OutputMode {
    FrontmatterMarkdown, // default
    MarkdownOnly,        // --no-frontmatter
    JsonEnvelope,        // --json
}

pub struct RequestOptions {
    pub headers: Vec<(String, String)>,
    pub cookies: Vec<String>,
    pub bearer: Option<String>,
}

pub struct FetchOptions {
    pub timeout_secs: u64,
    pub max_redirects: u32,
    pub user_agent: String,
    pub compact: bool,
    pub max_body_words: Option<usize>,
    pub request: RequestOptions,
    pub output_mode: OutputMode,
}
```

### Envelope extension

```rust
pub struct SuccessEnvelope {
    pub success: bool,
    pub url: String,
    pub status: u16,
    pub title: Option<String>,
    pub description: Option<String>,
    pub canonical_url: Option<String>,
    pub word_count: usize,
    pub body_word_count: usize,
    pub render_mode: String,
    pub body_word_limit: Option<usize>,
    pub body_truncated: bool,
    pub content_type: Option<String>,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
    pub redirect_chain: Option<Vec<String>>,
}
```

### Output mode behavior contract

1. `FrontmatterMarkdown` (default): current YAML frontmatter + body behavior; HTML baseline unchanged.
2. `MarkdownOnly`: returns body only, no envelope delimiters.
3. `JsonEnvelope`: returns JSON object containing metadata and body; no markdown frontmatter.

## Test Strategy

### Unit test matrix

| Area                     | Test file target                             | Cases                                                                                                                                                                    |
| ------------------------ | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Header parsing           | `src/fetch/request.rs`                       | valid `Name: Value`, missing colon, empty name, whitespace handling                                                                                                      |
| Authorization precedence | `src/fetch/request.rs` / `src/fetch/http.rs` | `-H Authorization` + `--bearer` => bearer wins                                                                                                                           |
| Cookie merge             | `src/fetch/request.rs`                       | repeatable cookies preserve order and delimit correctly                                                                                                                  |
| MIME routing             | `src/fetch/router.rs`                        | `text/html`, `application/xhtml+xml`, `application/json`, `application/problem+json`, `text/plain`, `application/rss+xml`, `application/xml`, `application/pdf`, unknown |
| JSON rendering           | `src/fetch/handlers/json.rs`                 | valid JSON pretty output, invalid JSON fallback/error behavior                                                                                                           |
| Feed handler             | `src/fetch/handlers/feed.rs`                 | RSS parse success, Atom parse success, XML non-feed fallback                                                                                                             |
| PDF handler              | `src/fetch/handlers/pdf.rs`                  | extraction success fixture, extraction error classification                                                                                                              |
| Output modes             | `src/fetch/output.rs`                        | mutual exclusivity at CLI boundary, serialization invariants                                                                                                             |

### Integration test matrix (mockito)

| Requirement | Integration scenario                                                                                   |
| ----------- | ------------------------------------------------------------------------------------------------------ |
| CONT-01     | Existing HTML fixture tests remain green (regression guard).                                           |
| CONT-03     | `application/json` endpoint outputs fenced pretty JSON in markdown mode; JSON envelope in `--json`.    |
| CONT-04     | `text/plain; charset=...` endpoint decoded and emitted as expected.                                    |
| CONT-05     | RSS/Atom fixture endpoint renders markdown entries.                                                    |
| CONT-02     | `application/pdf` fixture bytes return extracted text or predictable `pdf_extraction_failed` envelope. |
| OUT-02      | `--json` output parses as JSON and contains metadata fields.                                           |
| OUT-03      | `--no-frontmatter` omits YAML delimiters and returns body only.                                        |
| OUT-04      | `-o` writes exact serialized output for each mode.                                                     |
| HTTP-04     | Echo server asserts custom headers transmitted.                                                        |
| HTTP-05     | Echo server asserts merged Cookie header.                                                              |
| HTTP-06     | Echo server asserts Authorization bearer transmitted and precedence over `-H Authorization`.           |
| EXTR-05     | HTML fixture with canonical/description populates envelope fields.                                     |

### Execution commands

- `cargo test --test fetch_envelope_integration`
- `cargo test fetch::`
- `cargo clippy -- -D warnings`

## Risks and Mitigations

1. HTML regressions while introducing router.
- Mitigation: Keep current HTML flow in dedicated handler and run existing integration tests unchanged first.

2. Binary size growth from new crates (especially PDF stack).
- Mitigation: Make PDF support optional via crate feature (e.g., `pdf`), default off unless Phase 2 requires default-on behavior. [ASSUMED: acceptable product tradeoff]

3. Invalid/malformed feed/XML edge cases.
- Mitigation: Feed parser first, deterministic XML fenced fallback path second, never panic.

4. Ambiguous reduction behavior across non-markdown outputs.
- Mitigation: Define policy: reduction applies to markdown/text render paths only, not JSON-envelope structured output unless explicitly required.

5. Header injection/unsafe parsing in `-H` values.
- Mitigation: Reject newline/control chars and invalid header names at parse stage; fail with user-facing miette error.

## Implementation Sequencing

1. Add typed option/data model scaffolding.
- CLI flags, `OutputMode`, `RequestOptions`, `FetchOptions` extension.

2. Implement request option plumbing in transport.
- Header parse utility, cookie merge, bearer precedence.

3. Add content router with HTML branch parity.
- Router + html handler wrapping existing extraction/conversion/reduction.

4. Add JSON and plain-text handlers.
- These are lowest complexity and validate routing/output mode flow.

5. Add feed/XML handler.
- Parse with `feed-rs`; fallback to fenced XML when not feed.

6. Add envelope output modes.
- YAML+body (existing), markdown-only, JSON envelope.

7. Extend metadata extraction (EXTR-05).
- Add canonical + description extraction and envelope fields.

8. Add PDF handler behind feature gate.
- Implement extraction and predictable failure envelope.

9. Expand tests in matrix order.
- Unit first (parsers/router), then integration for each requirement.

10. Final hardening.
- Run clippy/tests, confirm HTML baseline output did not change.

## Research Patch: Open Questions Resolved (2026-05-21)

This patch closes all previously open Phase 2 questions and locks implementation behavior for planning and execution.

- `--compact` and `--max-body-words` in `--json` mode:
  Decision: Do not apply reducers in `--json` mode.
  Rationale: Preserve envelope payload fidelity and avoid mode-dependent mutation of structured responses.

- PDF extraction enablement:
  Decision: Enable PDF extraction by default in Phase 2.
  Rationale: CONT-02 is a required Phase 2 deliverable, so default behavior must satisfy requirement coverage without feature gating.

- Non-feed XML fallback:
  Decision: Use fenced XML fallback for non-feed XML content.
  Rationale: Deterministic rendering with low regression risk and clear user-visible output.

- JSON-envelope payload shape:
  Decision: Keep a single `body` field in JSON-envelope mode for Phase 2.
  Rationale: Maintain contract parity across output modes and defer multi-field JSON payload expansion to a later phase.

## Sources

### Primary

- https://docs.rs/ureq/latest/ureq/
- https://docs.rs/serde_json/latest/serde_json/
- https://docs.rs/mime/latest/mime/
- https://docs.rs/feed-rs/latest/feed_rs/
- https://docs.rs/pdf-extract/latest/pdf_extract/

### Registry verification

- `cargo search --limit 1 feed-rs` -> `2.3.1`
- `cargo search --limit 1 serde_json` -> `1.0.150`
- `cargo search --limit 1 mime` -> `0.3.17`
- `cargo search --limit 1 pdf-extract` -> `0.10.0`
- `cargo search --limit 1 quick-xml` -> `0.40.1`
- `cargo search --limit 1 atom_syndication` -> `0.12.8`

### Codebase anchors

- `src/cli.rs`
- `src/main.rs`
- `src/commands/fetch.rs`
- `src/fetch/mod.rs`
- `src/fetch/http.rs`
- `src/fetch/extract.rs`
- `src/fetch/envelope.rs`
- `tests/fetch_envelope_integration.rs`
