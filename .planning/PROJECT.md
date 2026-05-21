# mdget — Agent-First HTTP Client in Rust

## What This Is

mdget is a high-performance Rust rewrite of mdurl (TypeScript), designed as a curl alternative optimized for AI agents and LLM tools. It fetches webpages and outputs clean markdown with structured metadata, enabling agents to read web content without HTML parsing complexity. Key improvements over mdurl: faster execution, smaller binary (~5-10MB vs 50+MB node_modules), instant startup, advanced caching, authentication helpers, and response filtering.

## Core Value

**The ONE thing:** Agents can fetch any web resource with `mdget <url>` and receive clean, parseable markdown with predictable structure — no HTML noise, no ad clutter, just content.

## Requirements

### Validated

- ✓ CLI scaffolding (clap-based commands, config system, logging) — existing
- ✓ Graceful shutdown infrastructure — existing
- ✓ XDG directory support for cache and config — existing

### Active

**HTTP Core (v1.0 baseline):**
- [ ] HTTP client with curl crate (libcurl bindings for performance)
- [ ] Plain HTTP fetch → HTML extraction → markdown conversion pipeline
- [ ] Readability-style article extraction (port mdurl's logic or use Rust equivalent)
- [ ] Markdown conversion (pulldown-cmark for parsing, custom renderer)
- [ ] YAML frontmatter output (url, title, status, word_count, fetched_at, etc.)
- [ ] JSON envelope mode (`--json` flag)
- [ ] Error envelope with predictable structure

**Content Type Handling:**
- [ ] HTML → markdown (core flow)
- [ ] PDF → extracted text (pdf-extract or pdfium-render crate)
- [ ] RSS/Atom feeds → markdown list of entries
- [ ] Sitemap XML → URL list with dates
- [ ] JSON responses → pretty-printed fenced code block
- [ ] XML responses → fenced code block
- [ ] Plain text → text body with metadata
- [ ] Binary/media → markdown stub with metadata

**HTTP Features (curl crate advantages):**
- [ ] Redirect following with chain tracking (max redirects configurable)
- [ ] Custom headers (`-H` flag, repeatable)
- [ ] Cookie support (`--cookie` flag)
- [ ] User-Agent override (`--user-agent`)
- [ ] Bearer token (`--bearer`)
- [ ] Referer header (`--referer`)
- [ ] Timeout configuration (`--timeout <ms>`)
- [ ] HTTP/2 and HTTP/3 support (curl crate provides this)

**Advanced Caching (improvement over mdurl):**
- [ ] On-disk cache with ETag/Last-Modified tracking (`--cache <dir>`)
- [ ] Compressed cache storage (gzip/zstd) to save disk space
- [ ] Cache query commands: `mdget cache ls`, `mdget cache prune`, `mdget cache export`
- [ ] TTL policies: configurable expiry per domain or content-type
- [ ] Smart invalidation: respect Cache-Control headers, Vary
- [ ] Stale-while-revalidate pattern for faster responses

**Authentication Helpers (new feature):**
- [ ] Token management: `mdget auth set <name> <token>` stores tokens securely
- [ ] Auto-inject Bearer tokens: `mdget <url> --auth <name>`
- [ ] OAuth2 device flow: `mdget auth oauth <provider>` for interactive login
- [ ] Session cookie persistence: `--session <name>` maintains cookies across requests
- [ ] Credential store integration: keyring crate for platform keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- [ ] Token expiry tracking with auto-refresh hints

**Response Filtering (new feature):**
- [ ] CSS selector extraction: `--selector <css>` extracts element before markdown conversion
- [ ] JMESPath queries for JSON: `--query <jmespath>` filters JSON responses (jmespath.rs crate)
- [ ] Regex extraction: `--extract <pattern>` pulls matching content
- [ ] Content pruning: `--no-ads`, `--no-trackers` strips known ad/analytics patterns
- [ ] Section extraction: `--section <heading>` emits only matching markdown section (like mdurl)

**Output & UX:**
- [ ] Frontmatter mode (default): YAML block + markdown body
- [ ] JSON envelope mode (`--json`): structured output with markdown field
- [ ] No-frontmatter mode (`--no-frontmatter`): pure markdown
- [ ] Output to file (`-o <file>`)
- [ ] Quiet mode (`--quiet`): suppress progress to stderr
- [ ] Concurrent fetching: `mdget <url1> <url2> <url3>` with `--concurrency <n>`
- [ ] Archive fallback: `--archive-fallback` tries Wayback Machine on 4xx

**Resource Tables (from mdurl):**
- [ ] Page Resources section: TOC, navigation, images, forms, iframes
- [ ] Structured Data section: JSON-LD extraction (recipes, products, events, articles)
- [ ] Include-links table: `--include-links` appends link inventory
- [ ] Transcript extraction: YouTube captions when available

**Extraction Options:**
- [ ] Full-page mode: `--full` skips Readability, keeps cleaned full body
- [ ] Max bytes: `--max-bytes <n>` truncates with `[truncated]` marker
- [ ] No-resources flag: `--no-resources` omits resource tables
- [ ] No-structured-data flag: `--no-structured-data` skips JSON-LD

### Out of Scope

- Browser rendering (headless Chrome) — deferred to v2.0. mdurl's `--js` flag not in v1.
- Search mode (Google/Bing/DuckDuckGo) — v2.0 feature. Focus on direct URL fetching first.
- Wayback Machine fallback for v1 (too complex) — simple archive fallback only, full integration in v2.
- GraphQL introspection — future consideration, not v1.
- WebSocket support — HTTP-only for v1.
- Streaming responses to stdout — full buffering in v1, streaming in v2.

## Context

### Existing Codebase (Template Foundation)
- Solid Rust 2024 CLI template with clap (derive API, subcommands, shell completions)
- Layered config system: defaults → system → user → env vars → CLI flags
- Production-grade logging: tracing with dual output (stdout + rolling files)
- Graceful shutdown with tokio-graceful-shutdown
- No HTTP functionality yet — commands are placeholders (countdown timers)

### Reference Implementation (mdurl - TypeScript)
- Proven design from mdurl-cli (TypeScript + Node.js)
- Features: HTTP/browser fallback, Readability extraction, search mode, 12+ content types
- Distributed via npm (`npx mdurl-cli <url>`)
- Battle-tested output format: YAML frontmatter + markdown body
- Agent-friendly: predictable structure, error envelopes, resource tables

### Why Rust Rewrite
1. **Performance:** Faster than Node.js for CPU-bound extraction/conversion
2. **Binary size:** 5-10MB executable vs 50+MB node_modules (including Chromium)
3. **Startup time:** Instant vs Node.js cold-start overhead
4. **Memory:** Lower memory footprint for batch operations
5. **New features:** Better caching, auth helpers, filtering (easier in Rust than TypeScript)

### Domain Knowledge
- HTTP client: Use `curl` crate (libcurl bindings) for robust HTTP with HTTP/2, redirects, cookies
- HTML parsing: `scraper` crate (CSS selectors), `html5ever` for parsing
- Article extraction: Port mdurl's Readability logic or use Rust alternatives (readability crate exists but unmaintained — may need custom)
- Markdown generation: `pulldown-cmark` for parsing, custom renderer for Turndown-like output
- PDF: `pdf-extract` or `pdfium-render` for text extraction
- Cache: `sled` embedded database or simple filesystem with SQLite index
- Auth: `keyring` crate for platform credential storage, `oauth2` crate for OAuth flows
- Filtering: `jmespath` crate for JSON queries, `regex` for pattern extraction

## Constraints

- **Tech stack:** Rust 2024, edition 2024, MSRV 1.85+
- **Timeline:** v1.0 in 2-3 months (incremental releases: 0.3, 0.4, ... 1.0)
- **Binary size:** Target <15MB release binary (exclude debug symbols)
- **Compatibility:** macOS, Linux, Windows (tier 1 platforms)
- **Performance:** Must be faster than mdurl for typical webpage fetch+convert
- **Dependencies:** Minimize dependency count, avoid unmaintained crates
- **License:** MIT (matching mdurl and template)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Use `curl` crate (libcurl) | Proven, fast, supports HTTP/2+HTTP/3, handles redirects/cookies/TLS well | — Pending |
| Defer browser rendering to v2 | Browser adds 100+MB Chromium dependency, complex async coordination. HTTP-only v1 delivers 80% value. | ✓ Good |
| Port mdurl's Readability logic | Existing Rust readability crate is unmaintained (last update 2019). Port proven TypeScript logic for control + updates. | — Pending |
| Use `sled` for cache | Embedded key-value store, fast, supports TTL, compression-friendly | — Pending |
| `keyring` crate for auth | Cross-platform credential storage, integrates with OS keychains | — Pending |
| `jmespath` for JSON filtering | Standard query language for JSON, widely known, Rust impl available | — Pending |

---

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-05-21 after initialization*
