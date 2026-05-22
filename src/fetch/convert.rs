// HTML to markdown conversion with absolute URL support
// Implementation in Task 5

use miette::Result;

pub struct MarkdownRenderer {
    output: String,
    base_url: String,
    list_depth: usize,
}

impl MarkdownRenderer {
    pub fn new(base_url: String) -> Self {
        Self {
            output: String::new(),
            base_url,
            list_depth: 0,
        }
    }

    pub fn render(&mut self, _html: &str) -> Result<String> {
        todo!("Implement in Task 5")
    }
}

/// Public API — used by the orchestrator in fetch/mod.rs
pub fn html_to_markdown(_html: &str, _base_url: &str) -> Result<String> {
    let mut renderer = MarkdownRenderer::new(_base_url.to_string());
    renderer.render(_html)
}
