# Wave 2 Summary: Content-Type Router and Handlers

Completed: 2026-05-22

## Objective
Introduce normalized content-type routing and specialized handlers (JSON, plain text, RSS/Atom feeds, XML fallback, PDF text extraction) into the fetch pipeline, while keeping HTML as the baseline compatibility lock.

## Proposed Changes Implemented
1. **Modules Registered**: Registered `handlers` and `router` in [mod.rs](file:///Users/hp/MyCode/rust/mdget/src/fetch/mod.rs).
2. **Content-Type Routing**: Integrated [router.rs](file:///Users/hp/MyCode/rust/mdget/src/fetch/router.rs) into the core orchestrator flow. Content types are normalized and mapped to their corresponding `HandlerKind`. HTML/Unknown MIME types fallback gracefully to the original HTML conversion pipeline, ensuring zero regression for HTML processing.
3. **Structured & Plain-Text Handlers**:
   - **JSON**: Formats JSON body using a fenced code block (` ```json `), with lossy fallback for malformed JSON.
   - **Plain Text**: Serves raw text content directly, supporting `max_body_words` limits.
   - **Feeds**: Parses feeds via `feed-rs` to format RSS/Atom posts in clean Markdown. Falls back to a fenced XML block (` ```xml `) if feed parsing fails.
   - **PDF**: Extracts text via `pdf-extract`. Map extraction failures to a predictable `pdf_extraction_failed` error classification.

## Verification
- Run unit tests: `cargo test` (31 unit tests succeeded).
- Added comprehensive integration tests in [fetch_envelope_integration.rs](file:///Users/hp/MyCode/rust/mdget/tests/fetch_envelope_integration.rs) covering:
  - JSON fetching and pretty format wrapping.
  - Plain text fetching with and without `max_body_words` limits.
  - RSS/Atom feed rendering in markdown.
  - XML non-feed fallback formatting.
  - PDF extraction failure classification.
- All integration tests passed cleanly.
- Clippy checks compile successfully with zero warnings/errors.
