// Readability-style article extraction with DOM cleaning

use miette::Result;
use scraper::{ElementRef, Html, Selector};

pub struct ExtractedArticle {
    pub title: Option<String>,
    pub content_html: String,
    pub word_count: usize,
}

pub struct ReadabilityExtractor {
    pub min_score: f64,
}

impl ReadabilityExtractor {
    pub fn new(min_score: f64) -> Self {
        Self { min_score }
    }

    pub fn extract(&self, html: &str) -> Result<ExtractedArticle> {
        let document = Html::parse_document(html);

        // 1. Extract title
        let title = self.extract_title(&document);

        // 2. Find article node (highest scoring candidate)
        let article_node = self.find_article_node(&document)?;

        // 3. Clean unwanted elements BEFORE serialization
        let cleaned_html = self.clean_and_serialize(article_node);

        // 4. Count words
        let word_count = self.count_words(&cleaned_html);

        Ok(ExtractedArticle {
            title,
            content_html: cleaned_html,
            word_count,
        })
    }

    fn extract_title(&self, doc: &Html) -> Option<String> {
        // Try <title> first
        if let Ok(selector) = Selector::parse("title") {
            if let Some(title_elem) = doc.select(&selector).next() {
                let text = title_elem.text().collect::<String>().trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }

        // Fall back to first <h1>
        if let Ok(selector) = Selector::parse("h1") {
            if let Some(h1_elem) = doc.select(&selector).next() {
                let text = h1_elem.text().collect::<String>().trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }

        None
    }

    fn find_article_node(&self, doc: &Html) -> Result<ElementRef> {
        let candidates = ["article", "main", "section", "div"];
        let mut best_candidate: Option<(ElementRef, f64)> = None;

        for tag in &candidates {
            if let Ok(selector) = Selector::parse(tag) {
                for elem in doc.select(&selector) {
                    let score = self.score_element(elem);

                    if score >= self.min_score {
                        if best_candidate.is_none()
                            || score > best_candidate.as_ref().unwrap().1
                        {
                            best_candidate = Some((elem, score));
                        }
                    }
                }
            }
        }

        best_candidate
            .map(|(elem, _)| elem)
            .ok_or_else(|| {
                miette::miette!("No suitable article node found (min_score: {})", self.min_score)
            })
    }

    fn score_element(&self, elem: ElementRef) -> f64 {
        let mut score = 0.0;

        // Positive signals
        if elem.value().name() == "article" {
            score += 10.0;
        }
        if elem.value().name() == "main" {
            score += 10.0;
        }

        // Check class and ID
        if let Some(class) = elem.value().attr("class") {
            if class.contains("content")
                || class.contains("article")
                || class.contains("post")
                || class.contains("entry")
            {
                score += 5.0;
            }
        }

        if let Some(id) = elem.value().attr("id") {
            if id.contains("content") || id.contains("article") || id.contains("main") {
                score += 5.0;
            }
        }

        // Count paragraphs and text
        if let Ok(p_selector) = Selector::parse("p") {
            let p_count = elem.select(&p_selector).count();
            score += p_count as f64;
        }

        let text_length: usize = elem.text().map(|t| t.len()).sum();
        score += (text_length as f64) / 100.0;

        // Negative signals
        if elem.value().name() == "nav" {
            score -= 20.0; // Heavy penalty
        }
        if elem.value().name() == "footer" {
            score -= 5.0;
        }
        if elem.value().name() == "aside" {
            score -= 5.0;
        }

        if let Some(class) = elem.value().attr("class") {
            if class.contains("comment")
                || class.contains("ad")
                || class.contains("sidebar")
                || class.contains("nav")
            {
                score -= 5.0;
            }
        }

        tracing::debug!("Scored {} element: {}", elem.value().name(), score);
        score
    }

    fn clean_and_serialize(&self, elem: ElementRef) -> String {
        let bad_tags = ["nav", "footer", "aside", "script", "style"];
        let bad_classes = ["ad", "comment", "sidebar", "nav"];

        // Clone the element and remove bad nodes
        let html = elem.html();
        let fragment = Html::parse_fragment(&html);

        // Simple approach: serialize with bad tags removed
        let mut output = String::new();
        self.serialize_clean(
            &fragment.root_element(),
            &bad_tags,
            &bad_classes,
            &mut output,
        );
        output
    }

    fn serialize_clean(
        &self,
        node: &ElementRef,
        bad_tags: &[&str],
        bad_classes: &[&str],
        output: &mut String,
    ) {
        if bad_tags.contains(&node.value().name()) {
            return; // Skip this element entirely
        }

        if let Some(class) = node.value().attr("class") {
            if bad_classes.iter().any(|c| class.contains(c)) {
                return; // Skip this element
            }
        }

        // Serialize this node
        output.push_str(&format!("<{}", node.value().name()));
        for attr in node.value().attrs() {
            output.push_str(&format!(" {}=\"{}\"", attr.0, attr.1));
        }
        output.push('>');

        // Recurse into children
        for child in node.children() {
            if let Some(elem) = ElementRef::wrap(child) {
                self.serialize_clean(&elem, bad_tags, bad_classes, output);
            } else if let Some(text_node) = child.value().as_text() {
                output.push_str(text_node);
            }
        }

        output.push_str(&format!("</{}>", node.value().name()));
    }

    fn count_words(&self, html: &str) -> usize {
        html.split_whitespace()
            .filter(|word| word.len() >= 2)
            .count()
    }
}

/// Public API
pub fn extract_article(html: &str) -> Result<ExtractedArticle> {
    let extractor = ReadabilityExtractor::new(20.0);
    extractor.extract(html)
}
