use crate::fetch::handlers::HandlerResult;
use crate::fetch::reduce::reduce_markdown;

pub fn handle(text: &str, max_body_words: Option<usize>) -> HandlerResult {
    // Plain text is passed through directly. We apply word limits without structural compaction.
    let reduced = reduce_markdown(text, false, max_body_words);
    let original_word_count = text.split_whitespace().count();

    HandlerResult {
        title: None,
        body: reduced.body,
        word_count: original_word_count,
        body_word_count: reduced.body_word_count,
        truncated: reduced.truncated,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_plain_text() {
        let text = "one two three four five";
        let res = handle(text, Some(3));
        assert_eq!(res.title, None);
        assert_eq!(res.body, "one two three");
        assert_eq!(res.word_count, 5);
        assert_eq!(res.body_word_count, 3);
        assert!(res.truncated);
    }
}
