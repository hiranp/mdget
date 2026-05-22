# Roadmap: mdget

**Created:** 2026-05-21
**Last Updated:** 2026-05-22
**Granularity:** Coarse (3-5 phases)

## Overview

4 phases to deliver mdget v1.0: HTTP core → content handling → optimization → power features

---

### Phase 1: HTTP Core & Basic Extraction
**Goal:** Fetch HTML pages and convert to clean markdown
**Status:** Completed ✅

**Requirements:** HTTP-01, HTTP-02, HTTP-03, EXTR-01, EXTR-02, EXTR-03, EXTR-04, OUT-01, OUT-05

**Success Criteria:**
1. `mdget https://example.com` outputs YAML frontmatter + markdown body
2. HTTP errors produce error envelopes with predictable structure
3. Redirects followed and tracked in frontmatter
4. Article content extracted from HTML (Readability-style)

---

### Phase 2: Content Types & Output Modes
**Goal:** Handle multiple content types and output formats
**Status:** Active 🚀

**Requirements:** CONT-01, CONT-02, CONT-03, CONT-04, CONT-05, OUT-02, OUT-03, OUT-04, HTTP-04, HTTP-05, HTTP-06, EXTR-05

**Success Criteria:**
1. PDF files converted to extracted text
2. JSON/XML/RSS responses handled appropriately
3. `--json` envelope mode works
4. Custom headers, cookies, bearer tokens functional
5. Output to file with `-o` flag

---

### Phase 3: Caching & Resources
**Goal:** Add caching layer and resource extraction
**Status:** Planned 📋

**Requirements:** CACHE-01, CACHE-02, CACHE-03, CACHE-04, CACHE-05, RES-01, RES-02, RES-03

**Success Criteria:**
1. On-disk cache stores responses with ETag revalidation
2. Cached responses retrieved faster than fresh fetches
3. `mdget cache ls` shows cache entries
4. Page resources table appends to markdown output
5. JSON-LD structured data extracted when present

---

### Phase 4: Authentication & Filtering
**Goal:** Authentication helpers and response filtering
**Status:** Planned 📋

**Requirements:** AUTH-01, AUTH-02, AUTH-03, AUTH-04, FILT-01, FILT-02, FILT-03

**Success Criteria:**
1. `mdget auth set` stores tokens securely in keyring
2. `--auth <name>` injects stored token automatically
3. Session cookies persist across requests
4. `--selector` extracts specific HTML elements
5. `--section` extracts markdown sections by heading
6. `--max-bytes` truncates with marker

---

## Requirement Coverage

| Phase | Requirements | Count | Status |
|-------|--------------|-------|--------|
| 1 | HTTP-01 to HTTP-03, EXTR-01 to EXTR-04, OUT-01, OUT-05 | 9 | Completed |
| 2 | CONT-01 to CONT-05, OUT-02 to OUT-04, HTTP-04 to HTTP-06, EXTR-05 | 13 | Active |
| 3 | CACHE-01 to CACHE-05, RES-01 to RES-03 | 8 | Planned |
| 4 | AUTH-01 to AUTH-04, FILT-01 to FILT-03 | 7 | Planned |

**Total:** 37 v1 requirements across 4 phases

---
*Roadmap updated: 2026-05-22*
