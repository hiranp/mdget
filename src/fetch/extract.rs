// Readability-style article extraction with DOM cleaning

use miette::{Result, miette};
use scraper::{ElementRef, Html, Selector};

pub struct ExtractedArticle {
    pub title: Option<String>,
    pub description: Option<String>,
    pub canonical_url: Option<String>,
    pub content_html: String,
    pub word_count: usize,
}

struct ReadabilityExtractor {
    min_score: f64,
}

impl ReadabilityExtractor {
    fn new(min_score: f64) -> Self {
        Self { min_score }
    }

    pub fn extract(&self, html: &str) -> Result<ExtractedArticle> {
        let document = Html::parse_document(html);
        let title = self.extract_title(&document);
        let description = self.extract_description(&document);
        let canonical_url = self.extract_canonical_url(&document);
        let article_html = self.find_best_article_html(&document)?;
        let cleaned_html = self.clean_fragment(&article_html);
        let word_count = self.count_words(&cleaned_html);

        Ok(ExtractedArticle {
            title,
            description,
            canonical_url,
            content_html: cleaned_html,
            word_count,
        })
    }

    fn extract_title(&self, doc: &Html) -> Option<String> {
        let title_selector = Selector::parse("title").ok()?;
        if let Some(title_elem) = doc.select(&title_selector).next() {
            let text = title_elem.text().collect::<String>().trim().to_string();
            if !text.is_empty() {
                return Some(text);
            }
        }

        let h1_selector = Selector::parse("h1").ok()?;
        doc.select(&h1_selector)
            .next()
            .map(|h1| h1.text().collect::<String>().trim().to_string())
            .filter(|text| !text.is_empty())
    }

    fn extract_description(&self, doc: &Html) -> Option<String> {
        let selector = Selector::parse("meta[name='description']").ok()?;
        if let Some(elem) = doc.select(&selector).next() {
            let desc = elem.value().attr("content").map(|s| s.trim().to_string());
            if desc.as_deref().is_some_and(|s| !s.is_empty()) {
                return desc;
            }
        }
        let og_selector = Selector::parse("meta[property='og:description']").ok()?;
        if let Some(elem) = doc.select(&og_selector).next() {
            let desc = elem.value().attr("content").map(|s| s.trim().to_string());
            if desc.as_deref().is_some_and(|s| !s.is_empty()) {
                return desc;
            }
        }
        None
    }

    fn extract_canonical_url(&self, doc: &Html) -> Option<String> {
        let selector = Selector::parse("link[rel='canonical']").ok()?;
        if let Some(elem) = doc.select(&selector).next() {
            let href = elem.value().attr("href").map(|s| s.trim().to_string());
            if href.as_deref().is_some_and(|s| !s.is_empty()) {
                return href;
            }
        }
        let og_selector = Selector::parse("meta[property='og:url']").ok()?;
        if let Some(elem) = doc.select(&og_selector).next() {
            let href = elem.value().attr("content").map(|s| s.trim().to_string());
            if href.as_deref().is_some_and(|s| !s.is_empty()) {
                return href;
            }
        }
        None
    }

    fn find_best_article_html(&self, doc: &Html) -> Result<String> {
        let candidates = ["article", "main", "section", "div"];
        let p_selector = Selector::parse("p").map_err(|_| miette!("Invalid paragraph selector"))?;

        let mut best_any: Option<(String, f64, usize)> = None;
        let mut best_above_min: Option<(String, f64, usize)> = None;

        for tag in candidates {
            let selector =
                Selector::parse(tag).map_err(|_| miette!("Invalid candidate selector: {tag}"))?;
            for elem in doc.select(&selector) {
                let score = self.score_element(elem, &p_selector);
                let depth = self.element_depth(elem);
                let candidate = (elem.html(), score, depth);

                if Self::is_better_candidate(best_any.as_ref(), score, depth) {
                    best_any = Some(candidate.clone());
                }

                if score >= self.min_score
                    && Self::is_better_candidate(best_above_min.as_ref(), score, depth)
                {
                    best_above_min = Some(candidate);
                }
            }
        }

        best_above_min
            .or(best_any)
            .map(|(html, _, _)| html)
            .ok_or_else(|| miette!("No article-like node found in document"))
    }

    fn is_better_candidate(
        current: Option<&(String, f64, usize)>,
        score: f64,
        depth: usize,
    ) -> bool {
        let Some((_, best_score, best_depth)) = current else {
            return true;
        };

        let within_ten_percent = (score - best_score).abs() <= (best_score.abs() * 0.1);
        score > *best_score || (within_ten_percent && depth > *best_depth)
    }

    fn score_element(&self, elem: ElementRef<'_>, p_selector: &Selector) -> f64 {
        let mut score = 0.0;
        let tag = elem.value().name();

        if tag == "article" || tag == "main" {
            score += 10.0;
        }

        if let Some(class) = elem.value().attr("class") {
            let class = class.to_ascii_lowercase();
            if class.contains("content")
                || class.contains("article")
                || class.contains("post")
                || class.contains("entry")
            {
                score += 5.0;
            }
            if class.contains("comment")
                || class.contains("ad")
                || class.contains("sidebar")
                || class.contains("nav")
            {
                score -= 5.0;
            }
        }

        if let Some(id) = elem.value().attr("id") {
            let id = id.to_ascii_lowercase();
            if id.contains("content") || id.contains("article") || id.contains("main") {
                score += 5.0;
            }
        }

        let p_count = elem.select(p_selector).count();
        score += p_count as f64;

        let text_length: usize = elem.text().map(str::len).sum();
        score += text_length as f64 / 100.0;

        if tag == "nav" {
            score -= 20.0;
        }
        if tag == "footer" || tag == "aside" {
            score -= 5.0;
        }

        tracing::debug!(tag = tag, score, "scored candidate element");
        score
    }

    fn element_depth(&self, elem: ElementRef<'_>) -> usize {
        elem.ancestors().count()
    }

    fn clean_fragment(&self, html: &str) -> String {
        let fragment = Html::parse_fragment(html);
        let root = fragment.root_element();
        let bad_tags = ["nav", "footer", "aside", "script", "style"];
        let bad_classes = ["ad", "comment", "sidebar", "nav"];

        let mut out = String::new();
        for child in root.children() {
            if let Some(elem) = ElementRef::wrap(child) {
                Self::serialize_clean_element(elem, &bad_tags, &bad_classes, &mut out);
            } else if let Some(text_node) = child.value().as_text() {
                out.push_str(text_node);
            }
        }
        out
    }

    fn serialize_clean_element(
        elem: ElementRef<'_>,
        bad_tags: &[&str],
        bad_classes: &[&str],
        out: &mut String,
    ) {
        let tag = elem.value().name();
        if bad_tags.contains(&tag) {
            return;
        }

        if let Some(class) = elem.value().attr("class") {
            let class_lower = class.to_ascii_lowercase();
            if bad_classes.iter().any(|needle| class_lower.contains(needle)) {
                return;
            }
        }

        out.push('<');
        out.push_str(tag);
        for (k, v) in elem.value().attrs() {
            out.push(' ');
            out.push_str(k);
            out.push_str("=\"");
            // Escape attribute values to prevent attribute injection
            out.push_str(&v.replace('&', "&amp;").replace('"', "&quot;"));
            out.push('"');
        }
        out.push('>');

        for child in elem.children() {
            if let Some(child_elem) = ElementRef::wrap(child) {
                Self::serialize_clean_element(child_elem, bad_tags, bad_classes, out);
            } else if let Some(text_node) = child.value().as_text() {
                out.push_str(text_node);
            }
        }

        out.push_str("</");
        out.push_str(tag);
        out.push('>');
    }

    fn count_words(&self, html: &str) -> usize {
        let fragment = Html::parse_fragment(html);
        let text = fragment.root_element().text().collect::<Vec<_>>().join(" ");
        text.split_whitespace().filter(|word| word.len() >= 2).count()
    }
}

pub fn extract_article(html: &str) -> Result<ExtractedArticle> {
    let extractor = ReadabilityExtractor::new(20.0);
    extractor.extract(html)
}

#[cfg(test)]
mod tests {
    use super::{ReadabilityExtractor, extract_article};

    #[test]
    fn extracts_simple_article_fixture() {
        let html = include_str!("../../tests/fixtures/simple_article.html");
        let article = extract_article(html).expect("extract article");
        assert!(article.content_html.contains("<article"));
        assert!(article.content_html.contains("Test Article"));
        assert!(!article.content_html.contains("Copyright 2026"));
        assert!(article.word_count > 5);
    }

    #[test]
    fn extracts_news_fixture_without_ads() {
        let html = include_str!("../../tests/fixtures/news_article.html");
        let article = extract_article(html).expect("extract article");
        assert!(article.content_html.contains("Important News Event"));
        assert!(!article.content_html.contains("Sponsored content"));
        assert!(!article.content_html.contains("Related Stories"));
    }

    #[test]
    fn extracts_github_fixture_title() {
        let html = include_str!("../../tests/fixtures/github_readme.html");
        let article = extract_article(html).expect("extract article");
        assert_eq!(article.title.as_deref(), Some("GitHub README Example"));
        assert!(article.content_html.contains("Project Name"));
    }

    #[test]
    fn falls_back_to_h1_when_title_missing() {
        let html = "<html><body><h1>Fallback Heading</h1><article><p>Body copy here.</p></article></body></html>";
        let article = extract_article(html).expect("extract article");
        assert_eq!(article.title.as_deref(), Some("Fallback Heading"));
    }

    #[test]
    fn removes_script_and_comment_classes() {
        let html = "<html><head><title>T</title></head><body><article><p>Keep this.</p><script>alert('x')</script><div class='comments'>drop me</div></article></body></html>";
        let article = extract_article(html).expect("extract article");
        assert!(article.content_html.contains("Keep this."));
        assert!(!article.content_html.contains("alert('x')"));
        assert!(!article.content_html.contains("drop me"));
    }

    #[test]
    fn falls_back_to_best_candidate_when_min_score_too_high() {
        let html = "<html><body><div><p>Short but valid content block.</p></div></body></html>";
        let extractor = ReadabilityExtractor::new(999.0);
        let article = extractor.extract(html).expect("extract article");
        assert!(article.content_html.contains("Short but valid content block."));
    }

    #[test]
    fn extracts_malformed_fixture_content() {
        let html = include_str!("../../tests/fixtures/malformed_article.html");
        let article = extract_article(html).expect("extract malformed article");

        assert!(article.content_html.contains("Broken Markup Story"));
        assert!(article.content_html.contains("Second paragraph with useful content."));
        assert!(article.word_count > 8);
    }

    #[test]
    fn strips_navigation_noise_in_noisy_fixture() {
        let html = include_str!("../../tests/fixtures/noisy_navigation_page.html");
        let article = extract_article(html).expect("extract noisy article");

        assert!(article.content_html.contains("Noise Resistant Article"));
        assert!(article.content_html.contains("meaningful content"));
        assert!(!article.content_html.contains("Subscribe now"));
        assert!(!article.content_html.contains("Top stories"));
        assert!(!article.content_html.contains("Comments"));
    }

    #[test]
    fn extracts_description_and_canonical_url() {
        let html = r#"
            <html>
                <head>
                    <title>Test Page</title>
                    <meta name="description" content="This is a test description.">
                    <link rel="canonical" href="https://example.com/canonical">
                </head>
                <body>
                    <article><p>Hello world</p></article>
                </body>
            </html>
        "#;
        let article = extract_article(html).expect("extract article");
        assert_eq!(article.description.as_deref(), Some("This is a test description."));
        assert_eq!(article.canonical_url.as_deref(), Some("https://example.com/canonical"));
    }

    #[test]
    fn extracts_og_description_and_url_fallback() {
        let html = r#"
            <html>
                <head>
                    <title>Test Page</title>
                    <meta property="og:description" content="OG description fallback">
                    <meta property="og:url" content="https://example.com/og-url">
                </head>
                <body>
                    <article><p>Hello world</p></article>
                </body>
            </html>
        "#;
        let article = extract_article(html).expect("extract article");
        assert_eq!(article.description.as_deref(), Some("OG description fallback"));
        assert_eq!(article.canonical_url.as_deref(), Some("https://example.com/og-url"));
    }
}
