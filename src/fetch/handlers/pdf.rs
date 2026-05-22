use crate::fetch::handlers::HandlerResult;

pub fn handle(body_bytes: &[u8]) -> Result<HandlerResult, String> {
    match pdf_extract::extract_text_from_mem(body_bytes) {
        Ok(text) => {
            let trimmed = text.trim().to_string();
            let word_count = trimmed.split_whitespace().count();
            Ok(HandlerResult {
                title: None,
                body: trimmed,
                word_count,
                body_word_count: word_count,
                truncated: false,
            })
        }
        Err(e) => Err(format!("PDF extraction failed: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_invalid_pdf_fails() {
        let bad_pdf = b"not a pdf at all";
        let res = handle(bad_pdf);
        assert!(res.is_err());
        assert!(res.err().unwrap().contains("PDF extraction failed"));
    }
}
