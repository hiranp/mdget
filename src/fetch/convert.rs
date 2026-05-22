// HTML to markdown conversion with absolute URL support

use std::sync::OnceLock;

use miette::{Result, miette};
use scraper::{ElementRef, Html, Selector};
use url::Url;

pub struct MarkdownRenderer {
    output: String,
    base_url: String,
}

impl MarkdownRenderer {
    pub fn new(base_url: String) -> Self {
        Self { output: String::new(), base_url }
    }

    pub fn render(&mut self, html: &str) -> Result<String> {
        let document = Html::parse_fragment(html);
        let root = document.root_element();

        for child in root.children() {
            if let Some(elem) = ElementRef::wrap(child) {
                self.render_block(elem)?;
            }
        }

        Ok(self.output.trim_end().to_string())
    }

    fn render_block(&mut self, node: ElementRef<'_>) -> Result<()> {
        match node.value().name() {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level =
                    node.value().name().chars().nth(1).and_then(|c| c.to_digit(10)).unwrap_or(1);
                let text = self.render_inline_children(node)?;
                self.output.push_str(&"#".repeat(level as usize));
                self.output.push(' ');
                self.output.push_str(text.trim());
                self.output.push_str("\n\n");
            }
            "p" => {
                let text = self.render_inline_children(node)?;
                if !text.trim().is_empty() {
                    self.output.push_str(text.trim());
                    self.output.push_str("\n\n");
                }
            }
            "pre" => {
                let code = node.text().collect::<String>();
                self.output.push_str("```\n");
                self.output.push_str(code.trim_end());
                self.output.push_str("\n```\n\n");
            }
            "blockquote" => {
                let text = self.render_inline_children(node)?;
                for line in text.lines() {
                    self.output.push_str("> ");
                    self.output.push_str(line);
                    self.output.push('\n');
                }
                self.output.push('\n');
            }
            "ul" | "ol" => {
                self.render_list(node, node.value().name() == "ol")?;
            }
            "table" => {
                self.render_table(node)?;
            }
            "img" => {
                let alt = node.value().attr("alt").unwrap_or("");
                let src = node.value().attr("src").unwrap_or("");
                let absolute = self.resolve_url(src);
                self.output.push_str(&format!("![{}]({})\n\n", escape_markdown(alt), absolute));
            }
            _ => {
                for child in node.children() {
                    if let Some(elem) = ElementRef::wrap(child) {
                        self.render_block(elem)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn render_inline_children(&self, node: ElementRef<'_>) -> Result<String> {
        let mut out = String::new();
        for child in node.children() {
            if let Some(elem) = ElementRef::wrap(child) {
                out.push_str(&self.render_inline_element(elem)?);
            } else if let Some(text) = child.value().as_text() {
                out.push_str(&escape_markdown(text));
            }
        }
        Ok(out)
    }

    fn render_inline_element(&self, node: ElementRef<'_>) -> Result<String> {
        match node.value().name() {
            "a" => {
                let text = self.render_inline_children(node)?;
                let href = node.value().attr("href").unwrap_or("");
                let absolute = self.resolve_url(href);
                Ok(format!("[{}]({})", text.trim(), absolute))
            }
            "strong" | "b" => {
                let text = self.render_inline_children(node)?;
                Ok(format!("**{}**", text.trim()))
            }
            "em" | "i" => {
                let text = self.render_inline_children(node)?;
                Ok(format!("*{}*", text.trim()))
            }
            "code" => {
                let text = node.text().collect::<String>();
                Ok(format!("`{}`", text.trim()))
            }
            "img" => {
                let alt = node.value().attr("alt").unwrap_or("");
                let src = node.value().attr("src").unwrap_or("");
                Ok(format!("![{}]({})", escape_markdown(alt), self.resolve_url(src)))
            }
            _ => self.render_inline_children(node),
        }
    }

    fn render_list(&mut self, list_node: ElementRef<'_>, is_ordered: bool) -> Result<()> {
        let mut index = 1usize;
        for child in list_node.children() {
            let Some(elem) = ElementRef::wrap(child) else {
                continue;
            };
            if elem.value().name() != "li" {
                continue;
            }

            let prefix = if is_ordered { format!("{}. ", index) } else { "- ".to_string() };
            let text = self.render_inline_children(elem)?;
            self.output.push_str(&prefix);
            self.output.push_str(text.trim());
            self.output.push('\n');
            index += 1;
        }
        self.output.push('\n');
        Ok(())
    }

    fn render_table(&mut self, table_node: ElementRef<'_>) -> Result<()> {
        static TR_SELECTOR: OnceLock<Selector> = OnceLock::new();
        static CELL_SELECTOR: OnceLock<Selector> = OnceLock::new();

        let tr_selector =
            TR_SELECTOR.get_or_init(|| Selector::parse("tr").expect("valid selector: tr"));
        let cell_selector =
            CELL_SELECTOR.get_or_init(|| Selector::parse("th,td").expect("valid selector: th,td"));

        let rows: Vec<Vec<String>> = table_node
            .select(tr_selector)
            .map(|row| {
                row.select(cell_selector)
                    .map(|cell| cell.text().collect::<String>().trim().to_string())
                    .collect::<Vec<_>>()
            })
            .filter(|cells| !cells.is_empty())
            .collect();

        if rows.is_empty() {
            return Ok(());
        }

        for cell in &rows[0] {
            self.output.push('|');
            self.output.push(' ');
            self.output.push_str(cell);
            self.output.push(' ');
        }
        self.output.push_str("|\n");

        for _ in &rows[0] {
            self.output.push_str("| --- ");
        }
        self.output.push_str("|\n");

        for row in rows.iter().skip(1) {
            for cell in row {
                self.output.push('|');
                self.output.push(' ');
                self.output.push_str(cell);
                self.output.push(' ');
            }
            self.output.push_str("|\n");
        }
        self.output.push('\n');

        Ok(())
    }

    fn resolve_url(&self, input: &str) -> String {
        if input.is_empty() {
            return String::new();
        }

        if input.starts_with("http://") || input.starts_with("https://") {
            return input.to_string();
        }

        match Url::parse(&self.base_url).and_then(|base| base.join(input)) {
            Ok(absolute) => absolute.to_string(),
            Err(_) => input.to_string(),
        }
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

pub fn html_to_markdown(html: &str, base_url: &str) -> Result<String> {
    if base_url.is_empty() {
        return Err(miette!("base_url must not be empty"));
    }

    let mut renderer = MarkdownRenderer::new(base_url.to_string());
    renderer.render(html)
}

#[cfg(test)]
mod tests {
    use super::html_to_markdown;

    #[test]
    fn resolves_relative_links() {
        let html = "<p><a href='/docs'>Docs</a></p>";
        let md = html_to_markdown(html, "https://example.com/base/").expect("markdown");
        assert!(md.contains("[Docs](https://example.com/docs)"));
    }

    #[test]
    fn renders_tables_to_gfm() {
        let html = "<table><tr><th>A</th><th>B</th></tr><tr><td>1</td><td>2</td></tr></table>";
        let md = html_to_markdown(html, "https://example.com").expect("markdown");
        assert!(md.contains("| A | B |"));
        assert!(md.contains("| --- | --- |"));
        assert!(md.contains("| 1 | 2 |"));
    }

    #[test]
    fn keeps_absolute_links_unchanged() {
        let html = "<p><a href='https://rust-lang.org/learn'>Learn</a></p>";
        let md = html_to_markdown(html, "https://example.com").expect("markdown");
        assert!(md.contains("[Learn](https://rust-lang.org/learn)"));
    }

    #[test]
    fn resolves_relative_images_and_escapes_alt_text() {
        let html = "<img alt='A [demo] image' src='/img/logo.png'>";
        let md = html_to_markdown(html, "https://example.com/docs/").expect("markdown");
        assert!(md.contains("![A \\[demo\\] image](https://example.com/img/logo.png)"));
    }

    #[test]
    fn escapes_markdown_in_plain_text() {
        let html = "<p>Use * and _ and [link] chars</p>";
        let md = html_to_markdown(html, "https://example.com").expect("markdown");
        assert!(md.contains("Use \\* and \\_ and \\[link\\] chars"));
    }

    #[test]
    fn rejects_empty_base_url() {
        let err = html_to_markdown("<p>x</p>", "").expect_err("must fail on empty base url");
        assert!(err.to_string().contains("base_url must not be empty"));
    }
}
