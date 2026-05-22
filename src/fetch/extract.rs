// Readability-style article extraction with DOM cleaning
// Implementation in Task 6

use miette::Result;

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

    pub fn extract(&self, _html: &str) -> Result<ExtractedArticle> {
        todo!("Implement in Task 6")
    }
}

/// Public API
pub fn extract_article(_html: &str) -> Result<ExtractedArticle> {
    let extractor = ReadabilityExtractor::new(20.0);
    extractor.extract(_html)
}
