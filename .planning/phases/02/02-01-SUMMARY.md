# Plan 02-01 Summary: Request Surface and Typed Output Selection

## Changes Implemented

### CLI and Command Wiring
- **[cli.rs](file:///Users/hp/MyCode/rust/mdget/src/cli.rs)**: Removed invalid `pub` visibility qualifiers from variant struct fields in `Commands::Fetch` to fix compilation issues.
- **[commands/fetch.rs](file:///Users/hp/MyCode/rust/mdget/src/commands/fetch.rs)**: Updated `run` signature to accept new CLI flags (`headers`, `cookies`, `bearer`, `json`, `no_frontmatter`), parsed custom headers using `parse_header`, mapped output modes, and constructed `FetchOptions`. Added a Clippy suppression for `too_many_arguments` to keep argument propagation simple.

### Fetch Mod & Request Options
- **[fetch/mod.rs](file:///Users/hp/MyCode/rust/mdget/src/fetch/mod.rs)**: Registered `request` module, re-exported `RequestOptions` and `OutputMode`, updated `FetchOptions` struct and its `Default` implementation, added `#[allow(dead_code)]` to the `output_mode` field (which will be fully utilized in Plan 02-03), and threaded `options.request` to the HTTP client's `fetch` method.
- **[fetch/request.rs](file:///Users/hp/MyCode/rust/mdget/src/fetch/request.rs)**: Confirmed implementation of `parse_header`, `merge_cookies`, and options struct models.
- **[fetch/http.rs](file:///Users/hp/MyCode/rust/mdget/src/fetch/http.rs)**: Updated `HttpClient::fetch` and `fetch_blocking` signatures to accept `RequestOptions`. Applied custom headers, cookies (merged), and bearer tokens onto the `ureq::Request` builder. Ensured `--bearer` token presence overrides any manual `Authorization` header supplied via `-H`.

### Testing and Integration
- **[tests/fetch_request_options_integration.rs](file:///Users/hp/MyCode/rust/mdget/tests/fetch_request_options_integration.rs)**: Created 7 test cases covering:
  - Custom header injection and verification.
  - Cookie serialization and deterministic merge ordering.
  - Bearer token precedence.
  - Strict header format validation (`parse_header`).
  - Cookie merging utility (`merge_cookies`).
  - CLI argument conflict validation (asserting `--json` and `--no-frontmatter` mutual exclusivity).
  - CLI flag override validation (asserting header `-H` does not conflict with the global `--home` flag).

## Verification Results

- All 58 unit and integration tests passed:
  ```bash
  cargo test
  ```
- Checked compile cleanly with Clippy:
  ```bash
  cargo clippy -- -D warnings
  ```
- verified that `--json` and `--no-frontmatter` correctly conflict at parse time with an exit code of `101`.
