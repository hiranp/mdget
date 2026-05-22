pub struct ReducedMarkdown {
    pub body: String,
    pub body_word_count: usize,
    pub truncated: bool,
}

pub fn reduce_markdown(
    markdown: &str,
    compact: bool,
    max_body_words: Option<usize>,
) -> ReducedMarkdown {
    let body = if compact { compact_markdown(markdown) } else { markdown.trim().to_string() };
    let (body, truncated) = truncate_words(&body, max_body_words);

    ReducedMarkdown { body_word_count: count_words(&body), body, truncated }
}

fn compact_markdown(markdown: &str) -> String {
    let mut blocks = Vec::new();

    for block in markdown.split("\n\n") {
        let trimmed = block.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('#') {
            blocks.push(trimmed.to_string());
            continue;
        }

        if trimmed.starts_with("```") {
            continue;
        }

        blocks.push(first_sentence_or_excerpt(trimmed, 24));
    }

    blocks.join("\n\n")
}

fn first_sentence_or_excerpt(text: &str, max_words: usize) -> String {
    let sentence_end =
        text.find('.').or_else(|| text.find('!')).or_else(|| text.find('?')).map(|index| index + 1);
    let excerpt = sentence_end.map(|end| text[..end].trim()).unwrap_or(text);
    truncate_to_words(excerpt, max_words)
}

fn truncate_words(text: &str, max_words: Option<usize>) -> (String, bool) {
    match max_words {
        Some(limit) => {
            let truncated = truncate_to_words(text, limit);
            let was_truncated = count_words(text) > limit;
            (truncated, was_truncated)
        }
        None => (text.trim().to_string(), false),
    }
}

fn truncate_to_words(text: &str, max_words: usize) -> String {
    if max_words == 0 {
        return String::new();
    }

    let mut words = Vec::new();
    for word in text.split_whitespace().take(max_words) {
        words.push(word);
    }

    words.join(" ")
}

fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

#[cfg(test)]
mod tests {
    use super::{compact_markdown, reduce_markdown};

    #[test]
    fn compact_markdown_removes_code_blocks_and_shortens_paragraphs() {
        let markdown = "# Heading\n\nThis is a sentence. This is extra detail.\n\n```rust\nfn main() {}\n```\n\n## Next\n\nAnother paragraph with a second sentence.";

        let compacted = compact_markdown(markdown);

        assert!(compacted.contains("# Heading"));
        assert!(compacted.contains("## Next"));
        assert!(compacted.contains("This is a sentence."));
        assert!(!compacted.contains("fn main"));
        assert!(!compacted.contains("extra detail"));
    }

    #[test]
    fn reduce_markdown_tracks_truncation() {
        let markdown = "one two three four five six";
        let reduced = reduce_markdown(markdown, false, Some(3));

        assert_eq!(reduced.body, "one two three");
        assert_eq!(reduced.body_word_count, 3);
        assert!(reduced.truncated);
    }
}
