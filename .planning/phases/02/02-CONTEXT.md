# Phase 2: Content Types & Output Modes - Context

**Gathered:** 2026-05-21
**Status:** Ready for planning
**Source:** Architect critique + codebase analysis

## Phase Boundary

Expand mdget from HTML-first output to a multi-content fetch pipeline with explicit output modes and request customization, while preserving Phase 1 HTML behavior and performance characteristics.

In scope for this phase:

- Multiple content-type handling (HTML, JSON, plain text, RSS/Atom/XML, PDF text extraction)
- Output modes (`--json`, `--no-frontmatter`, file output continuity)
- Request customization (`-H`, `--cookie`, `--bearer`)
- Metadata extraction additions (description, canonical URL)
- Token reduction continuity (`--compact`, `--max-body-words`) with deterministic behavior per content type

Out of scope for this phase:

- Browser rendering (BROWSER-* requirements)
- Search mode
- Cache layer
- Keyring-backed auth profiles

## Implementation Decisions

## Architecture and Contracts

- **D2-01:** Introduce a content-type router in the fetch orchestrator to select handlers by normalized MIME type. HTML remains the current primary path.
- **D2-02:** Preserve existing HTML behavior as the compatibility baseline; new handlers must not regress current envelope/body output for HTML pages.
- **D2-03:** Add typed fetch request options and output mode enums to avoid command-layer stringly-typed branching.
- **D2-04:** Keep I/O orchestration in command layer (`src/commands/fetch.rs`) and transformation/serialization concerns in fetch modules.

## Request Surface (HTTP-04/05/06)

- **D2-05:** Add repeatable custom headers via CLI (`-H/--header`) using strict `Name: Value` parsing with user-facing validation errors.
- **D2-06:** Add repeatable cookies via CLI (`--cookie`) and merge into Cookie header deterministically (preserve order).
- **D2-07:** Add bearer auth via CLI (`--bearer`) and inject `Authorization: Bearer <token>` in transport.
- **D2-08:** If users set Authorization in `-H` and also use `--bearer`, `--bearer` wins to keep behavior explicit.

## Content-Type Handling (CONT-01..05)

- **D2-09:** HTML (`text/html`, `application/xhtml+xml`) uses existing extraction + markdown conversion path.
- **D2-10:** JSON (`application/json`, `*+json`) renders as pretty-printed fenced code block when output mode is markdown; envelope JSON mode returns structured body as JSON string payload plus metadata.
- **D2-11:** Plain text (`text/plain`) is passed through with normalization and metadata envelope.
- **D2-12:** RSS/Atom/XML (`application/rss+xml`, `application/atom+xml`, `application/xml`, `text/xml`) routes to feed parser first; on non-feed XML fallback, render as fenced XML block with warning metadata.
- **D2-13:** PDF (`application/pdf`) uses text extraction path; failures return predictable error envelope with `pdf_extraction_failed` classification.

## Output Modes (OUT-02/03/04)

- **D2-14:** Define output modes: `frontmatter_markdown` (default), `markdown_only` (`--no-frontmatter`), and `json_envelope` (`--json`).
- **D2-15:** `--json` and `--no-frontmatter` are mutually exclusive at CLI parse layer.
- **D2-16:** Keep `-o/--output` behavior in command layer unchanged; serialization output is written verbatim to file.

## Metadata (EXTR-05)

- **D2-17:** Extend success envelope with `description` and `canonical_url` as optional fields.
- **D2-18:** HTML metadata extraction occurs in extractor pass; non-HTML handlers may leave these null.

## Testing and Quality

- **D2-19:** Build a content-matrix integration test suite with mock endpoints for html/json/plain/rss/xml/pdf-like failure paths.
- **D2-20:** Add unit tests for header/cookie/bearer option parsing and output mode serialization invariants.
- **D2-21:** Enforce requirement traceability in plan tasks (each task lists addressed requirement IDs).

## Risk and Delivery Controls

- **D2-22:** Land router scaffold first, then add handlers incrementally to reduce regression blast radius.
- **D2-23:** Defer advanced PDF fidelity optimization; phase goal is robust extraction path with predictable failures.
- **D2-24:** Align planning/state docs once plan is accepted to remove active-vs-pending ambiguity.

## the agent's Discretion

- Exact crate selection for RSS/Atom and PDF extraction based on maintenance, size impact, and MSRV compatibility.
- Internal module split for content handlers (single router file vs per-handler modules), provided public behavior and tests remain stable.

## Canonical References

Downstream agents MUST read these before planning or implementing.

## Product and Scope

- `.planning/PROJECT.md` - Product mission, constraints, and v1 boundaries.
- `.planning/REQUIREMENTS.md` - Canonical requirement IDs for Phase 2.
- `.planning/ROADMAP.md` - Phase 2 goal and success criteria.
- `.planning/STATE.md` - Current phase tracking and progress consistency.

## Existing Implementation

- `src/cli.rs` - Current fetch flags and command surface.
- `src/commands/fetch.rs` - Command orchestration and output file behavior.
- `src/fetch/mod.rs` - Current HTML-first fetch pipeline.
- `src/fetch/http.rs` - Transport behavior and current header handling.
- `src/fetch/envelope.rs` - Envelope schema and YAML output path.
- `tests/fetch_envelope_integration.rs` - Existing integration test pattern.

## Prior Planning Pattern

- `.planning/phases/01/01-PLAN-OPTIMIZED.md` - Plan structure and acceptance criteria style.

## Specific Ideas

- Add a `content_type` helper that strips parameters (e.g., `; charset=utf-8`) and normalizes case before routing.
- Keep `FetchOptions` focused on user intent; translate to HTTP request details in transport layer.
- Preserve existing markdown body post-processing (`--compact`, `--max-body-words`) where semantically appropriate; avoid applying compact mode to non-markdown body formats unless explicitly defined.

## Deferred Ideas

- Browser-backed rendering and wait strategies (v2 roadmap).
- Search provider integration (v2 roadmap).
- Auth profile/keyring workflows (Phase 4).

---

*Phase: 02-Content Types & Output Modes*
*Context gathered: 2026-05-21*
