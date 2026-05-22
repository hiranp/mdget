use crate::fetch::handlers::HandlerResult;

pub fn handle(body_bytes: &[u8], _content_type: Option<&str>) -> HandlerResult {
    // Attempt to decode as UTF-8
    let raw_str = String::from_utf8_lossy(body_bytes).into_owned();

    // Try to parse and prettify JSON
    let rendered_body = match serde_json::from_slice::<serde_json::Value>(body_bytes) {
        Ok(json_val) => {
            if let Ok(pretty) = serde_json::to_string_pretty(&json_val) {
                format!("```json\n{}\n```", pretty)
            } else {
                format!("```json\n{}\n```", raw_str.trim())
            }
        }
        Err(_) => {
            // Fallback for invalid JSON: wrap the raw content in a generic code block
            format!("```json\n{}\n```", raw_str.trim())
        }
    };

    let word_count = rendered_body.split_whitespace().count();

    HandlerResult {
        title: None,
        body: rendered_body,
        word_count,
        body_word_count: word_count,
        truncated: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_valid_json() {
        let json_data = b"{\"a\": 1, \"b\": \"value\"}";
        let res = handle(json_data, Some("application/json"));
        assert_eq!(res.title, None);
        assert!(res.body.contains("```json"));
        assert!(res.body.contains("\"a\": 1"));
        assert!(res.body.contains("\"b\": \"value\""));
        assert!(!res.truncated);
    }

    #[test]
    fn test_handle_invalid_json_fallback() {
        let bad_json = b"{invalid json";
        let res = handle(bad_json, Some("application/json"));
        assert!(res.body.contains("```json"));
        assert!(res.body.contains("{invalid json"));
        assert!(!res.truncated);
    }
}
