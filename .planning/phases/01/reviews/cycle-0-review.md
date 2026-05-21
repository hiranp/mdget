# Phase 1 Plan Review - Cycle 0

**Reviewer:** Claude Sonnet 4.5 (Main Session)
**Date:** 2026-05-21
**Plan Version:** 01-PLAN.md (initial)
**Phase:** 01 - HTTP Core & Basic Extraction

---

## Executive Summary

**Overall Assessment:** MEDIUM concerns found - Plan is fundamentally sound but has implementation risks that should be addressed.

**Concern Counts:**
- 🔴 HIGH: 0
- 🟡 MEDIUM: 4
- 🟢 LOW: 3

**Recommendation:** Proceed with modifications. Address MEDIUM concerns before execution to reduce implementation risk.

---

## HIGH Severity Concerns

None found. Plan structure is solid, dependencies are appropriate, and task breakdown is logical.

---

## MEDIUM Severity Concerns

### M1: Async/Blocking Mismatch Risk

**Location:** Task 3 - HTTP Client Wrapper

**Issue:** The plan specifies using `spawn_blocking` for curl operations, but doesn't detail how to handle the blocking-to-async boundary cleanly. The curl crate's API is entirely synchronous, and the redirect tracking callback pattern may not compose well with spawn_blocking.

**Why it matters:** Improper async wrapping can cause:
- Runtime blocking if spawn_blocking isn't used correctly
- Callback lifetime issues when moving data across thread boundaries
- Performance degradation from excessive context switching

**Suggested fix:**
- In Task 3 acceptance criteria, add: "Verify no blocking calls occur on tokio runtime (use tokio-console or tracing to confirm)"
- Consider adding a test that uses `tokio::time::timeout` to ensure spawn_blocking is working
- Document the callback→thread boundary pattern in the implementation

**Risk if not addressed:** Silent runtime blocking, hard to debug performance issues in production.

---

### M2: Readability Algorithm Complexity Underestimated

**Location:** Task 6 - Implement Readability Article Extraction

**Issue:** The plan presents Readability extraction as straightforward scoring + cleaning, but the algorithm is notoriously finicky. Real-world pages have:
- Nested article containers (Reddit comments in article context)
- Ad content disguised as article text
- Multiple "main" candidates with similar scores
- Edge cases where the best node is *inside* a high-scoring parent

**Why it matters:** A naive scoring implementation will:
- Extract nav/sidebar content on ~30% of tested pages
- Fail to extract content from medium.com, substack, github READMEs
- Produce inconsistent results across different site layouts

**Suggested fix:**
- Add a research sub-task: "Test scoring algorithm against 20 diverse URLs (news, blogs, docs, github)"
- Include tie-breaking logic in `find_article_node` (prefer deepest node when scores are close)
- Add logging of top 3 scoring nodes with their scores (helps debug extraction failures)
- In acceptance criteria, add: "Manually verify extraction quality on 5 test URLs from different domains"

**Risk if not addressed:** Users get nav/footer/comments instead of article content, breaking the core value prop.

---

### M3: Markdown Escaping Edge Cases Not Specified

**Location:** Task 5 - Implement Markdown Converter

**Issue:** The plan says "Escape special characters" but doesn't specify:
- When to escape (only in text nodes? what about `<code>` content?)
- How to handle pre-escaped content (HTML entities like `&amp;`)
- Edge cases like `**already bold**` in source HTML
- URL escaping in link targets (spaces, parens, brackets)

**Why it matters:** Broken markdown escaping causes:
- Broken markdown rendering (unmatched `**`, stray `#`)
- Lost content when special chars are double-escaped
- Security issues if user-controlled HTML injects markdown syntax

**Suggested fix:**
- Add explicit escaping rules to Task 5 action:
  - "Escape markdown special chars ONLY in text nodes (not in code blocks)"
  - "Decode HTML entities BEFORE escaping (so `&lt;` becomes `<`, not `\&lt;`)"
  - "Escape spaces in URLs as `%20`, parens/brackets as `(` → `%28`"
- Add test cases for edge scenarios in acceptance criteria

**Risk if not addressed:** Broken markdown output, security vulnerabilities, content loss.

---

### M4: Binary Size Constraint Not Validated Early

**Location:** Task 1 - Add Dependencies + Overall Plan

**Issue:** The plan sets a 15MB binary size target but doesn't validate it until the end. By Task 10, if the binary is 20MB, rolling back is expensive (7 new dependencies, 10 tasks).

**Why it matters:**
- `curl` crate pulls in libcurl (1-2MB)
- `scraper` pulls in `html5ever` + `selectors` (~1MB)
- Debug symbols and unused features can bloat binaries
- Hitting the limit late means rework or scope cut

**Suggested fix:**
- Move binary size check to Task 1 acceptance criteria: "`cargo build --release && ls -lh target/release/mdget` shows baseline size"
- Add incremental size checks: After Task 3 (HTTP client), Task 5 (markdown), Task 6 (extraction)
- Set alarm threshold: "If binary exceeds 12MB before Task 10, investigate before proceeding"
- Document how to strip symbols and optimize: `[profile.release] strip = true, lto = true`

**Risk if not addressed:** Late-stage scope cut or architectural rework to meet binary size constraint.

---

## LOW Severity Concerns

### L1: Error Message UX Not Specified

**Location:** Task 3, Task 8 (Error handling)

**Issue:** Error envelopes have `error` and `message` fields, but the plan doesn't specify what goes in each or how user-friendly they should be.

**Suggested improvement:** Add examples to Task 4 (ErrorEnvelope):
- `error: "http_timeout"` (machine-readable)
- `message: "Request to https://example.com timed out after 30s"` (human-readable)

**Risk:** Minor - Users get errors, but they might not be as helpful as they could be.

---

### L2: Charset Detection Fallback Not Robust

**Location:** Task 8, step 4 (charset detection)

**Issue:** The plan uses `encoding_rs::Encoding::for_label(...)` but doesn't specify:
- Where the label comes from (Content-Type header? HTML meta tag?)
- What happens if both are missing or conflicting

**Suggested improvement:**
- Add fallback order: Content-Type header → HTML meta charset → UTF-8
- Handle conflicting declarations (header wins)

**Risk:** Minor - Most modern sites use UTF-8, but legacy sites may have mojibake.

---

### L3: Word Count Algorithm Not Specified

**Location:** Task 6 (word_count in ExtractedArticle)

**Issue:** "Count whitespace-separated tokens" is simple but language-specific. Chinese/Japanese have no spaces; URLs/code inflate counts.

**Suggested improvement:**
- Specify: "Split on whitespace, filter tokens <2 chars, count remaining"
- Or: Use `unicode-segmentation` crate for proper word boundaries

**Risk:** Minor - Word count is metadata, not critical for core function.

---

## Positive Observations

✅ **Excellent task granularity** - 10 tasks with clear dependencies, each ~1 hour of work
✅ **Strong acceptance criteria** - Measurable, testable outcomes for each task
✅ **Robust verification section** - Build, functional, edge case tests all specified
✅ **Dependency choices justified** - Research document backs up curl, scraper, serde_yaml
✅ **Graceful shutdown integration** - Fetch command properly uses SubsystemHandle
✅ **Error handling philosophy** - Errors are envelopes, not panics (good for agents)
✅ **Rollback plan included** - Clear strategy if verification fails
✅ **Goal-backward verification** - Must-haves and truths explicitly stated

---

## Requirements Coverage Audit

| Requirement | Task(s) | Coverage | Notes |
|-------------|---------|----------|-------|
| HTTP-01 | 3, 8 | ✅ Full | curl crate with async wrapper |
| HTTP-02 | 3, 8 | ✅ Full | max_redirects configurable, redirect_chain tracked |
| HTTP-03 | 3, 8 | ✅ Full | timeout_secs configurable, enforced via CURLOPT_TIMEOUT |
| EXTR-01 | 2, 6 | ✅ Full | scraper crate for HTML parsing |
| EXTR-02 | 6 | ⚠️ Partial | Readability extraction implemented but untested on diverse sites |
| EXTR-03 | 5 | ✅ Full | Custom HTML→markdown renderer |
| EXTR-04 | 6 | ✅ Full | Title extraction from `<title>` or `<h1>` |
| OUT-01 | 4, 7, 8 | ✅ Full | YAML frontmatter with all specified fields |
| OUT-05 | 4, 8 | ✅ Full | ErrorEnvelope with predictable structure |

**Coverage:** 8/9 full, 1/9 partial (EXTR-02 needs validation)

---

## Architecture Review

### Module Structure
✅ Clean separation: fetch/{http, extract, convert, envelope} + commands/fetch
✅ Follows existing template pattern (commands as subsystems)
✅ Re-exports at mod.rs level for clean external API

### Async Strategy
⚠️ spawn_blocking pattern is correct but implementation risk (see M1)
✅ Fetch command runs in tokio subsystem (graceful shutdown compatible)

### Error Handling
✅ miette::Result everywhere (consistent error type)
✅ Error envelopes prevent panics (agent-friendly)
⚠️ Error message UX not detailed (see L1)

### Dependency Tree
✅ 7 new dependencies, all widely used and maintained
✅ No transitive dependency conflicts identified
⚠️ Binary size needs early validation (see M4)

---

## Recommendations

### Before Execution
1. **Address M2 first** - Readability extraction is the highest implementation risk
   - Add test URL corpus (20 diverse sites)
   - Implement logging for top-scoring nodes
   - Add manual verification step

2. **Address M4 early** - Measure binary size after Task 1
   - Set alarm threshold at 12MB
   - Document optimization flags in Cargo.toml

3. **Clarify M3** - Add explicit markdown escaping rules to Task 5
   - Include edge case test examples

### During Execution
- Monitor binary size after Tasks 1, 3, 5, 6
- Test Readability extraction incrementally (don't wait for Task 6 completion)
- Use tracing to verify spawn_blocking usage in Task 3

### After Execution
- Run verification checklist in plan (build, functional, edge cases)
- Test on 10+ diverse real-world URLs
- Profile binary with `cargo bloat` if size is close to limit

---

## Convergence Status

**Status:** ✅ **APPROVED FOR EXECUTION** (with modifications)

**Rationale:** No HIGH severity blockers. MEDIUM concerns are addressable with targeted improvements before/during execution. Plan structure is solid, requirements coverage is strong, and verification criteria are comprehensive.

**Next Steps:**
1. Planner should review M2 (Readability testing) and M4 (binary size) recommendations
2. Consider adding test URL corpus to research artifacts
3. Execute with monitoring for binary size and extraction quality

---

*Review completed: 2026-05-21*
*Reviewer: Claude Sonnet 4.5 (Main Session)*
