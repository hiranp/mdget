# Applied Review Fixes

**Date:** 2026-05-21  
**Source:** Cycle 0 review recommendations  
**Status:** ✅ All fixes applied

---

## Summary

Applied all review recommendations from `cycle-0-review.md` to address MEDIUM and LOW concerns before execution.

**Changes made:**
1. ✅ Created test corpus (20 URLs) - `test-corpus.md`
2. ✅ Added binary size optimizations - `Cargo.toml`
3. ✅ Updated plan with review fixes - `01-PLAN.md`

---

## 1. Test Corpus Created (M2 Priority)

**File:** `.planning/phases/01/test-corpus.md`

**Contents:**
- 20 diverse test URLs (news, blogs, docs, forums, other)
- Extraction quality rubric (✅ Correct / ⚠️ Partial / ❌ Failed)
- Scoring log template for debugging
- Known edge cases documented (Medium, GitHub, news sites, Stack Overflow, Wikipedia)
- Success threshold: 15/20 URLs (75% accuracy)

**Usage:**
- Test extraction during Task 6 implementation
- Log top 3 scoring nodes per URL
- Validate extraction quality manually

---

## 2. Binary Size Optimizations (M4 Priority)

**File:** `Cargo.toml`

**Added profile:**
```toml
[profile.release]
strip = true         # Remove debug symbols (-3MB)
lto = true          # Link-time optimization (-1-2MB)
codegen-units = 1   # Better optimization (slower build, smaller binary)
opt-level = "z"     # Optimize for size
```

**Expected impact:** 30-40% binary size reduction (~5-8MB savings)

**Size checkpoints added to plan:**
- Task 1: Baseline measurement
- Task 3: After curl crate (~5-6MB)
- Task 5: After markdown rendering (~7-8MB)
- Task 6: After scraper/html5ever (~10-12MB)
- **Alarm threshold:** If >12MB before Task 10, investigate

---

## 3. Plan Updates (01-PLAN.md)

### Task 1: Add Dependencies
**Added:**
- Binary size baseline recording
- Size checkpoint alarm (>12MB)

### Task 3: HTTP Client Wrapper (M1)
**Added:**
- Async verification with `tokio::time::timeout`
- Callback pattern documentation requirement
- Binary size checkpoint (~5-6MB expected)

### Task 4: Error Envelope (L1)
**Added:**
- Error message UX examples:
  - `error`: machine-readable (e.g., "http_timeout")
  - `message`: human-readable (e.g., "Request to ... timed out after 30s")

### Task 5: Markdown Converter (M3)
**Added:**
- Explicit escaping rules:
  - In text nodes: escape markdown special chars
  - In code blocks: do NOT escape
  - HTML entities: decode BEFORE escaping
  - Link URLs: percent-encode spaces, parens, brackets
- Edge case tests: `&amp;`, `<code>*ptr</code>`, link with spaces
- Binary size checkpoint (~7-8MB expected)

### Task 6: Readability Extraction (M2 - CRITICAL)
**Added:**
- Reference to test-corpus.md
- Enhanced scoring heuristics:
  - Heavy penalty for `<nav>` tags (-20)
  - Positive signals for semantic class names
  - Tie-breaking: prefer deepest node when scores within 10%
- Scoring debug logs requirement (tracing::debug!)
- Validation requirements:
  - Test on 20 URLs from test-corpus.md
  - Success threshold: 15/20 (75%)
- Word count algorithm specified (L3)
- Binary size checkpoint (~10-12MB expected)

### Task 8: Fetch Orchestrator (L2)
**Added:**
- Charset detection fallback order:
  1. Content-Type header
  2. HTML meta charset
  3. UTF-8 (default)

---

## Review Concern Mapping

| Concern | Severity | Addressed By | Status |
|---------|----------|--------------|--------|
| M1: Async/blocking mismatch | MEDIUM | Task 3 updates | ✅ Fixed |
| M2: Readability complexity | MEDIUM | test-corpus.md + Task 6 updates | ✅ Fixed |
| M3: Markdown escaping | MEDIUM | Task 5 updates | ✅ Fixed |
| M4: Binary size validation | MEDIUM | Cargo.toml + size checkpoints | ✅ Fixed |
| L1: Error message UX | LOW | Task 4 updates | ✅ Fixed |
| L2: Charset detection | LOW | Task 8 updates | ✅ Fixed |
| L3: Word count algorithm | LOW | Task 6 updates | ✅ Fixed |

---

## Files Modified

```
Cargo.toml                              # Binary size optimizations added
.planning/phases/01/test-corpus.md      # Test URLs and validation criteria
.planning/phases/01/01-PLAN.md          # Updated Tasks 1, 3, 4, 5, 6, 8
.planning/phases/01/reviews/applied-fixes.md  # This file
```

---

## Next Steps

1. **Execute plan** with `/gsd-execute-phase 01`
2. **Monitor binary size** at each checkpoint (Tasks 1, 3, 5, 6)
3. **Test Readability** incrementally during Task 6 (don't wait until end)
4. **Log scoring decisions** for debugging extraction failures
5. **Verify 75% accuracy** on test corpus before marking Task 6 complete

---

## Validation Checklist

Before starting execution, verify:

- [x] test-corpus.md exists with 20 test URLs
- [x] Cargo.toml has [profile.release] optimizations
- [x] Task 1 includes binary size baseline requirement
- [x] Task 3 includes async verification and size checkpoint
- [x] Task 4 includes error message UX examples
- [x] Task 5 includes explicit escaping rules and size checkpoint
- [x] Task 6 includes test corpus reference, validation requirements, and size checkpoint
- [x] Task 8 includes charset detection fallback order

All items verified ✅ - Plan is ready for execution.

---

*Fixes applied: 2026-05-21*  
*Review cycle: 0*  
*Convergence status: APPROVED*
