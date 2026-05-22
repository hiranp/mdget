# Test Fixtures for mdget Phase 1

This directory contains HTML fixtures for testing article extraction and markdown conversion.

## Fixtures

### simple_article.html
Minimal `<article>` page with basic content:
- One `<h1>` heading
- One `<p>` paragraph with **bold** text and a relative link
- A simple `<table>` with headers and one data row
- Navigation and footer elements (should be removed by extraction)

**Purpose:** Verify basic extraction, markdown conversion, table rendering, and DOM cleaning.

### github_readme.html
GitHub-style README page with semantic HTML:
- `<article class="markdown-body">` container
- Multiple heading levels
- Code blocks with syntax highlighting classes
- Lists (ordered and unordered)
- Navigation sidebar (should be removed)

**Purpose:** Test extraction of semantic HTML and class-based scoring heuristics.

### news_article.html
Ad-heavy news page simulating real-world content:
- `<article>` with actual content
- Multiple `<div class="ad">` blocks (should be removed)
- `<nav>` and `<footer>` (should be removed)
- `<aside>` sidebars (should be removed)

**Purpose:** Verify aggressive DOM cleaning removes ads, nav, and sidebars while preserving article content.

## Usage

These fixtures are used by:
- Unit tests in `tests/extract_test.rs` (offline, no network required)
- Manual validation with `cargo run -- fetch file://$(pwd)/tests/fixtures/simple_article.html`

## Maintenance

Update these fixtures when:
- Adding new HTML elements to the converter
- Improving extraction heuristics
- Testing edge cases discovered in production
