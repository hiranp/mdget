use mime::Mime;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandlerKind {
    Html,
    Json,
    Text,
    Feed,
    Pdf,
    Unknown,
}

pub fn route_content_type(content_type: Option<&str>) -> HandlerKind {
    let Some(ct_str) = content_type else {
        // Fallback to Html if no Content-Type header is present, preserving baseline behavior
        return HandlerKind::Html;
    };

    let Ok(mime) = Mime::from_str(ct_str) else {
        return HandlerKind::Unknown;
    };

    let essence = mime.essence_str().to_ascii_lowercase();

    if essence == "text/html" || essence == "application/xhtml+xml" {
        HandlerKind::Html
    } else if essence == "application/json" || essence.ends_with("+json") {
        HandlerKind::Json
    } else if essence == "text/plain" {
        HandlerKind::Text
    } else if essence == "application/pdf" {
        HandlerKind::Pdf
    } else if essence == "application/rss+xml"
        || essence == "application/atom+xml"
        || essence == "application/xml"
        || essence == "text/xml"
        || essence.ends_with("+xml")
    {
        HandlerKind::Feed
    } else {
        HandlerKind::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routing() {
        assert_eq!(route_content_type(None), HandlerKind::Html);
        assert_eq!(route_content_type(Some("text/html")), HandlerKind::Html);
        assert_eq!(route_content_type(Some("application/xhtml+xml")), HandlerKind::Html);
        assert_eq!(route_content_type(Some("text/html; charset=utf-8")), HandlerKind::Html);

        assert_eq!(route_content_type(Some("application/json")), HandlerKind::Json);
        assert_eq!(route_content_type(Some("application/problem+json")), HandlerKind::Json);

        assert_eq!(route_content_type(Some("text/plain")), HandlerKind::Text);
        assert_eq!(route_content_type(Some("application/pdf")), HandlerKind::Pdf);

        assert_eq!(route_content_type(Some("application/rss+xml")), HandlerKind::Feed);
        assert_eq!(route_content_type(Some("application/atom+xml")), HandlerKind::Feed);
        assert_eq!(route_content_type(Some("application/xml")), HandlerKind::Feed);
        assert_eq!(route_content_type(Some("text/xml")), HandlerKind::Feed);

        assert_eq!(route_content_type(Some("image/png")), HandlerKind::Unknown);
        assert_eq!(route_content_type(Some("invalid-mime")), HandlerKind::Unknown);
    }
}
