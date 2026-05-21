# Readability Extraction Test Corpus

**Purpose:** Validate the Readability algorithm (Task 6) against diverse real-world sites  
**Created:** 2026-05-21 (from M2 review recommendation)  
**Usage:** Test extraction during Task 6 implementation, log top 3 scoring nodes per URL

---

## Test URL List (20 sites)

### News Sites (5)
1. https://www.bbc.com/news/technology - Clean layout, minimal ads
2. https://www.cnn.com/business/tech - Ad-heavy, multiple columns
3. https://www.theguardian.com/technology - Sidebar content, related links
4. https://apnews.com/hub/technology - Minimal design, focus on text
5. https://www.reuters.com/technology - Breaking news layout

### Tech Blogs (5)
6. https://blog.rust-lang.org/ - Clean markdown-based blog
7. https://medium.com/@_samyak/article-slug - Medium's nested divs
8. https://dev.to/t/rust - Developer community blog
9. https://overreacted.io/ - Gatsby-based personal blog
10. https://jvns.ca/ - Simple Hugo blog, minimal CSS

### Documentation (5)
11. https://github.com/rust-lang/rust - GitHub README (file tree + content)
12. https://docs.rs/tokio/latest/tokio/ - Rust docs (sidebar + main)
13. https://kubernetes.io/docs/concepts/ - Technical docs with nav
14. https://www.postgresql.org/docs/current/ - Database documentation
15. https://developer.mozilla.org/en-US/docs/Web/JavaScript - MDN docs

### Forums/Q&A (3)
16. https://stackoverflow.com/questions/tagged/rust - Stack Overflow Q&A
17. https://news.ycombinator.com/ - Hacker News (comments vs. article)
18. https://www.reddit.com/r/rust/ - Reddit post (sidebar + comments)

### Other (2)
19. https://en.wikipedia.org/wiki/Rust_(programming_language) - Wikipedia (infobox + article)
20. https://arxiv.org/abs/2301.07041 - Academic paper (arXiv abstract)

---

## Extraction Quality Rubric

For each URL, document:

- ✅ **Correct**: Extracted main article/content only (no nav, footer, ads, comments)
- ⚠️ **Partial**: Got primary content but also included secondary elements
- ❌ **Failed**: Extracted wrong section, too little content, or mostly non-content

**Success threshold:** 15/20 URLs should be ✅ Correct (75% accuracy)

---

## Scoring Log Template

Use this format when testing each URL:

```markdown
### URL: https://example.com/article

**Top 3 Scoring Candidates:**
1. <article class="main"> - Score: 120 ← SELECTED
   - Reasons: +10 (semantic tag), +50 (text density), +30 (paragraph count)
2. <div class="sidebar"> - Score: 85
   - Reasons: +40 (text density), +20 (link count), -10 (class name penalty)
3. <div class="comments"> - Score: 70
   - Reasons: +30 (text density), +10 (paragraph count), -5 (class name penalty)

**Extraction Result:** ✅ Correct

**Notes:** Semantic `<article>` tag made selection obvious. Sidebar and comments correctly scored lower.

**Word Count:** 1,247 words

**Title Extracted:** "Example Article Title"
```

---

## Known Edge Cases

Document special handling for these patterns:

### 1. Medium.com
- **Challenge:** Nested `<article>` → `<div class="metabar">` + `<div class="content">` + `<div class="responses">`
- **Expected:** Extract only `<div class="content">` (deepest high-scoring node)
- **Test:** Prefer depth when scores are close (within 10%)

### 2. GitHub README
- **Challenge:** File tree nav vs. README content (both in `<article>`)
- **Expected:** Extract `<article class="markdown-body">`, not `<nav class="file-tree">`
- **Test:** Heavy penalty for `<nav>` tags (-20 score)

### 3. News Sites (Ads)
- **Challenge:** Sponsored content disguised as paragraphs
- **Expected:** Exclude content with class names: "ad", "sponsor", "promo"
- **Test:** Negative score for ad-related class names (-15 score)

### 4. Stack Overflow
- **Challenge:** Question vs. Answers section (both high text density)
- **Expected:** Prefer `<div id="question">` over `<div id="answers">`
- **Test:** Semantic ID bonus for "question", "content", "article" (+10 score)

### 5. Wikipedia
- **Challenge:** Infobox sidebar vs. article text
- **Expected:** Extract main article, include infobox (it's content, not nav)
- **Test:** Don't penalize "infobox" class (it's legitimate content)

---

## Testing Workflow

1. **During Task 6 implementation:**
   - Add logging: `tracing::debug!("Top 3 nodes: {:?}", top_candidates);`
   - For each test URL, capture the top 3 scoring nodes
   - Manually verify extraction quality

2. **Acceptance criteria:**
   - Minimum 15/20 URLs extract correctly (75% accuracy)
   - All news sites (1-5) extract correctly
   - At least 3/5 tech blogs extract correctly
   - GitHub README extracts correctly (critical for docs)

3. **Iteration:**
   - If <75% accuracy: Adjust scoring heuristics
   - Log failures with scoring breakdown
   - Iterate on tie-breaking logic (depth preference)

---

## Scoring Heuristics Reference

From plan Task 6, these heuristics should be implemented:

### Positive Signals (+score)
- `<article>`, `<main>` tags: +10
- Class names: "content", "article", "post", "entry": +5 each
- Paragraph count: +1 per `<p>` tag
- Text length: +0.1 per 10 characters
- Semantic IDs: "content", "article", "main": +5

### Negative Signals (-score)
- `<nav>`, `<footer>`, `<aside>` tags: -5 each
- Class names: "comment", "ad", "sidebar", "nav", "footer", "promo", "sponsor": -5 each
- Low text density (many tags, little text): -1 per empty tag

### Tie-Breaking (when scores within 10%)
- Prefer deepest node (most nested)
- Prefer semantic tags (`<article>` > `<div>`)
- Prefer node with title match (if `<h1>` matches page `<title>`)

---

*Test corpus created: 2026-05-21*  
*Source: Phase 1 review M2 recommendation*
