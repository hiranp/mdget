use crate::fetch::handlers::HandlerResult;

pub fn handle(body_bytes: &[u8], _content_type: Option<&str>) -> HandlerResult {
    let raw_str = String::from_utf8_lossy(body_bytes).into_owned();

    // Try to parse using feed-rs
    match feed_rs::parser::parse(body_bytes) {
        Ok(feed) => {
            let mut md = String::new();
            
            // Format feed title
            let feed_title = feed.title.map(|t| t.content).unwrap_or_else(|| "Feed".to_string());
            md.push_str(&format!("# {}\n\n", feed_title));

            for entry in feed.entries {
                let entry_title = entry.title.map(|t| t.content).unwrap_or_else(|| "Untitled".to_string());
                md.push_str(&format!("## {}\n", entry_title));

                if let Some(link) = entry.links.first() {
                    md.push_str(&format!("- **Link:** {}\n", link.href));
                }

                let date = entry
                    .published
                    .or(entry.updated)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                    .unwrap_or_else(|| "N/A".to_string());
                md.push_str(&format!("- **Published:** {}\n", date));

                if let Some(summary) = entry.summary {
                    md.push_str(&format!("- **Summary:** {}\n", summary.content.trim()));
                }
                md.push('\n');
            }

            let word_count = md.split_whitespace().count();
            HandlerResult {
                title: Some(feed_title),
                body: md.trim().to_string(),
                word_count,
                body_word_count: word_count,
                truncated: false,
            }
        }
        Err(_) => {
            // Fallback for failed XML/feed parse: wrap raw XML in a fenced code block
            let rendered_body = format!("```xml\n{}\n```", raw_str.trim());
            let word_count = rendered_body.split_whitespace().count();
            HandlerResult {
                title: None,
                body: rendered_body,
                word_count,
                body_word_count: word_count,
                truncated: false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_valid_atom_feed() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Test Feed</title>
  <entry>
    <title>First Post</title>
    <link href="http://example.com/1"/>
    <updated>2026-05-22T12:00:00Z</updated>
    <summary>Short summary of first post.</summary>
  </entry>
</feed>"#;
        
        let res = handle(xml.as_bytes(), Some("application/atom+xml"));
        assert_eq!(res.title, Some("Test Feed".to_string()));
        assert!(res.body.contains("# Test Feed"));
        assert!(res.body.contains("## First Post"));
        assert!(res.body.contains("- **Link:** http://example.com/1"));
        assert!(res.body.contains("- **Summary:** Short summary of first post."));
    }

    #[test]
    fn test_handle_invalid_xml_fallback() {
        let invalid_xml = "<invalid>xml";
        let res = handle(invalid_xml.as_bytes(), Some("application/xml"));
        assert_eq!(res.title, None);
        assert!(res.body.contains("```xml"));
        assert!(res.body.contains("<invalid>xml"));
    }
}
