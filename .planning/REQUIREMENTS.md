# Requirements: mdget

**Defined:** 2026-05-21
**Core Value:** Agents can fetch any web resource with `mdget <url>` and receive clean, parseable markdown with predictable structure.

## v1 Requirements

### HTTP Core
- [ ] **HTTP-01**: Fetch URLs via HTTP/HTTPS with curl crate
- [ ] **HTTP-02**: Follow redirects (max 5 default, configurable)
- [ ] **HTTP-03**: Handle timeouts (30s default, configurable)
- [ ] **HTTP-04**: Custom headers via `-H` flag (repeatable)
- [ ] **HTTP-05**: Cookie support via `--cookie` flag
- [ ] **HTTP-06**: Bearer token via `--bearer` flag

### Content Extraction
- [ ] **EXTR-01**: Parse HTML with scraper crate
- [ ] **EXTR-02**: Article extraction (Readability-style)
- [ ] **EXTR-03**: Convert HTML to markdown (pulldown-cmark-based)
- [ ] **EXTR-04**: Extract page title from HTML
- [ ] **EXTR-05**: Extract metadata (description, canonical URL)

### Content Types
- [ ] **CONT-01**: HTML → markdown (primary)
- [ ] **CONT-02**: PDF → extracted text
- [ ] **CONT-03**: JSON → pretty-printed fenced block
- [ ] **CONT-04**: Plain text → text with metadata
- [ ] **CONT-05**: RSS/Atom → entry list

### Output Format
- [ ] **OUT-01**: YAML frontmatter (url, title, status, word_count, fetched_at)
- [ ] **OUT-02**: JSON envelope mode (`--json`)
- [ ] **OUT-03**: No-frontmatter mode (`--no-frontmatter`)
- [ ] **OUT-04**: Write to file (`-o <file>`)
- [ ] **OUT-05**: Error envelope with predictable structure

### Caching
- [ ] **CACHE-01**: On-disk cache with `--cache <dir>`
- [ ] **CACHE-02**: ETag/Last-Modified revalidation
- [ ] **CACHE-03**: Compressed storage (gzip)
- [ ] **CACHE-04**: Cache query: `mdget cache ls`
- [ ] **CACHE-05**: Cache prune: `mdget cache prune`

### Authentication
- [ ] **AUTH-01**: Store tokens: `mdget auth set <name> <token>`
- [ ] **AUTH-02**: Use stored token: `mdget <url> --auth <name>`
- [ ] **AUTH-03**: Session cookies: `--session <name>`
- [ ] **AUTH-04**: Keyring integration for secure storage

### Filtering
- [ ] **FILT-01**: CSS selector extraction: `--selector <css>`
- [ ] **FILT-02**: Section extraction: `--section <heading>`
- [ ] **FILT-03**: Max bytes: `--max-bytes <n>` with truncation marker

### Resources
- [ ] **RES-01**: Page resources table (links, images, forms)
- [ ] **RES-02**: Structured data extraction (JSON-LD)
- [ ] **RES-03**: Flag to disable resources: `--no-resources`

## v2 Requirements

### Browser Rendering
- **BROWSER-01**: Headless Chrome integration
- **BROWSER-02**: Auto-fallback for SPAs
- **BROWSER-03**: Wait strategies (`--wait-selector`)

### Search Mode
- **SEARCH-01**: Google search with clean results
- **SEARCH-02**: Bing/DuckDuckGo support
- **SEARCH-03**: Result unwrapping and formatting

### Advanced Features
- **ADV-01**: OAuth2 device flow
- **ADV-02**: JMESPath queries for JSON
- **ADV-03**: Regex extraction
- **ADV-04**: Concurrent fetching of multiple URLs

## Out of Scope

| Feature | Reason |
|---------|--------|
| Browser rendering (v1) | 100+MB Chromium dependency, defer to v2 |
| Search mode (v1) | Complex, focus on direct URL fetching first |
| WebSocket support | HTTP-only focus for v1 |
| Streaming responses | Full buffering simpler for v1 |
| GraphQL introspection | Future consideration |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| (To be filled by roadmapper) | | |

---
*Requirements defined: 2026-05-21*
