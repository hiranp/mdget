// HTML to markdown conversion with absolute URL support

use miette::Result;
use scraper::{ElementRef, Html, Selector};
use url::Url;

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

    pub fn render(&mut self, html: &str) -> Result<String> {
        let document = Html::parse_fragment(html);
        let root = document.root_element();

        // Process each child of the root
        for child in root.children() {
            if let Some(elem) = ElementRef::wrap(child) {
                self.visit_node(elem)?;
            }
        }

        Ok(self.output.clone())
    }

    fn visit_node(&mut self, node: ElementRef) -> Result<()> {
        let tag_name = node.value().name();

        match tag_name {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level = tag_name[1..].parse::<usize>().unwrap_or(1);
                let text = self.extract_text(node);
                self.output
                    .push_str(&format!("{} {}\n\n", "#".repeat(level), escape_markdown(&text)));
            }
            "p" => {
                let text = self.extract_text(node);
                self.output.push_str(&format!("{}\n\n", text));
            }
            "a" => {
                let text = self.extract_text(node);
                let href = node.value().attr("href").unwrap_or("");
                let absolute_url = self.resolve_url(href);
                self.output
                    .push_str(&format!("[{}]({})", text, absolute_url));
            }
            "strong" | "b" => {
                let text = self.extract_text(node);
                self.output.push_str(&format!("**{}**", text));
            }
            "em" | "i" => {
                let text = self.extract_text(node);
                self.output.push_str(&format!("*{}*", text));
            }
            "code" => {
                let text = self.extract_text(node);
                self.output.push_str(&format!("`{}`", text)); // Do NOT escape in code
            }
            "pre" => {
                let text = self.extract_text(node);
                self.output.push_str(&format!("```\n{}\n```\n\n", text));
            }
            "blockquote" => {
                let text = self.extract_text(node);
                for line in text.lines() {
                    self.output.push_str(&format!("> {}\n", line));
                }
                self.output.push('\n');
            }
            "ul" | "ol" => {
                self.render_list(node, tag_name == "ol")?;
            }
            "img" => {
                let alt = node.value().attr("alt").unwrap_or("");
                let src = node.value().attr("src").unwrap_or("");
                let absolute_url = self.resolve_url(src);
                self.output
                    .push_str(&format!("![{}]({})\n\n", alt, absolute_url));
            }
            "table" => {
                self.render_table(node)?;
            }
            _ => {
                // Recursively process children
                for child in node.children() {
                    if let Some(elem) = ElementRef::wrap(child) {
                        self.visit_node(elem)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn resolve_url(&self, url: &str) -> String {
        if url.is_empty() {
            return String::new();
        }

        // If already absolute, return as-is
        if url.starts_with("http://") || url.starts_with("https://") {
            return url.to_string();
        }

        // Try to join with base URL
        match Url::parse(&self.base_url) {
            Ok(base) => match base.join(url) {
                Ok(absolute) => absolute.to_string(),
                Err(_) => url.to_string(),
            },
            Err(_) => url.to_string(),
        }
    }

    fn render_table(&mut self, table_node: ElementRef) -> Result<()> {
        let selector = Selector::parse("tr")
            .map_err(|_| miette::miette!("Invalid CSS selector"))?;
        let rows: Vec<_> = table_node.select(&selector).collect();

        for (row_idx, row) in rows.iter().enumerate() {
            let cell_selector = Selector::parse("th,td")
                .map_err(|_| miette::miette!("Invalid CSS selector"))?;
            let cells: Vec<String> = row
                .select(&cell_selector)
                .map(|cell| self.extract_text(cell))
                .collect();

            self.output.push('|');
            for cell in cells {
                self.output.push_str(&format!(" {} |", cell));
            }
            self.output.push('\n');

            // Add separator after header row
            if row_idx == 0 {
                self.output.push_str("| --- ");
                for _ in 1..cells.len() {
                    self.output.push_str("| --- ");
                }
                self.output.push_str("|\n");
            }
        }
        self.output.push('\n');

        Ok(())
    }

    fn render_list(&mut self, list_node: ElementRef, is_ordered: bool) -> Result<()> {
        let li_selector =
            Selector::parse("li").map_err(|_| miette::miette!("Invalid CSS selector"))?;

        for (idx, li) in list_node.select(&li_selector).enumerate() {
            let prefix = if is_ordered {
                format!("{}. ", idx + 1)
            } else {
                "- ".to_string()
            };
            let text = self.extract_text(li);
            self.output.push_str(&format!("{}{}\n", prefix, text));
        }
        self.output.push('\n');
        Ok(())
    }

    fn extract_text(&self, elem: ElementRef) -> String {
        elem.text().collect::<String>()
    }
}

fn escape_markdown(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('<', "\\<")
        .replace('>', "\\>")
        .replace('#', "\\#")
        .replace('`', "\\`")
}

/// Public API — used by the orchestrator in fetch/mod.rs
pub fn html_to_markdown(html: &str, base_url: &str) -> Result<String> {
    let mut renderer = MarkdownRenderer::new(base_url.to_string());
    renderer.render(html)
}
