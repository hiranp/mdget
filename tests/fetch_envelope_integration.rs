use mdget::fetch::{FetchOptions, fetch_url};
use serde_yml::Value;

fn parse_frontmatter_and_body(output: &str) -> (Value, &str) {
    let stripped = output.strip_prefix("---\n").expect("output must start with YAML frontmatter");
    let split_index = stripped.find("\n---\n").expect("output must contain closing YAML delimiter");

    let yaml = &stripped[..split_index];
    let body = &stripped[split_index + "\n---\n".len()..];
    let frontmatter: Value = serde_yml::from_str(yaml).expect("valid yaml frontmatter");

    (frontmatter, body)
}

#[tokio::test]
async fn fetch_success_formats_yaml_and_markdown_body() {
    let mut server = mockito::Server::new_async().await;
    let html = include_str!("fixtures/noisy_navigation_page.html");

    let _mock = server
        .mock("GET", "/ok")
        .with_status(200)
        .with_header("content-type", "text/html; charset=utf-8")
        .with_body(html)
        .create_async()
        .await;

    let url = format!("{}/ok", server.url());
    let output = fetch_url(&url, FetchOptions::default()).await.expect("fetch output");

    let (frontmatter, body) = parse_frontmatter_and_body(&output);

    assert_eq!(frontmatter["success"], Value::Bool(true));
    assert_eq!(frontmatter["status"], Value::Number(200.into()));
    assert_eq!(frontmatter["url"], Value::String(url));
    assert!(frontmatter["fetched_at"].as_str().is_some());
    assert!(frontmatter["word_count"].as_u64().unwrap_or(0) > 10);

    assert!(body.contains("# Noise Resistant Article"));
    assert!(body.contains("[a docs link]("));
    assert!(!body.contains("Subscribe now"));
    assert!(!body.contains("Top stories"));
}

#[tokio::test]
async fn fetch_http_error_formats_yaml_without_body() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/missing")
        .with_status(404)
        .with_header("content-type", "text/html")
        .with_body("<html><body>not found</body></html>")
        .create_async()
        .await;

    let url = format!("{}/missing", server.url());
    let output = fetch_url(&url, FetchOptions::default()).await.expect("error envelope output");

    let (frontmatter, body) = parse_frontmatter_and_body(&output);

    assert_eq!(frontmatter["success"], Value::Bool(false));
    assert_eq!(frontmatter["status"], Value::Number(404.into()));
    assert_eq!(frontmatter["error"], Value::String("http_404".to_string()));
    assert!(frontmatter["message"].as_str().unwrap_or("").contains("HTTP 404: Not Found"));
    assert_eq!(frontmatter["url"], Value::String(url));
    assert!(frontmatter["fetched_at"].as_str().is_some());

    assert!(body.trim().is_empty());
}

#[tokio::test]
async fn fetch_success_includes_redirect_chain_and_final_url() {
    let mut server = mockito::Server::new_async().await;
    let final_html = include_str!("fixtures/simple_article.html");

    let _redirect = server
        .mock("GET", "/start")
        .with_status(302)
        .with_header("location", &format!("{}/final", server.url()))
        .create_async()
        .await;

    let _final = server
        .mock("GET", "/final")
        .with_status(200)
        .with_header("content-type", "text/html; charset=utf-8")
        .with_body(final_html)
        .create_async()
        .await;

    let start_url = format!("{}/start", server.url());
    let final_url = format!("{}/final", server.url());
    let output =
        fetch_url(&start_url, FetchOptions::default()).await.expect("redirect fetch output");

    let (frontmatter, body) = parse_frontmatter_and_body(&output);

    assert_eq!(frontmatter["success"], Value::Bool(true));
    assert_eq!(frontmatter["status"], Value::Number(200.into()));
    assert_eq!(frontmatter["url"], Value::String(final_url));

    let redirects = frontmatter["redirect_chain"]
        .as_sequence()
        .expect("redirect_chain should be present for redirected response");
    assert!(!redirects.is_empty());
    assert!(redirects.iter().filter_map(Value::as_str).any(|entry| entry.contains("/start")));

    assert!(body.contains("# Test Article"));
}

#[tokio::test]
async fn fetch_json_content() {
    let mut server = mockito::Server::new_async().await;
    let json_body = r#"{"name": "mdget", "features": ["markdown", "html", "json"]}"#;

    let _mock = server
        .mock("GET", "/data.json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json_body)
        .create_async()
        .await;

    let url = format!("{}/data.json", server.url());
    let output = fetch_url(&url, FetchOptions::default()).await.expect("json output");

    let (frontmatter, body) = parse_frontmatter_and_body(&output);

    assert_eq!(frontmatter["success"], Value::Bool(true));
    assert_eq!(frontmatter["status"], Value::Number(200.into()));
    assert_eq!(frontmatter["url"], Value::String(url.clone()));

    assert!(body.contains("```json"));
    assert!(body.contains("\"name\": \"mdget\""));
    assert!(body.contains("\"features\": ["));

    let limited_output =
        fetch_url(&url, FetchOptions { max_body_words: Some(6), ..Default::default() })
            .await
            .expect("json output with word limit");

    let (limited_frontmatter, limited_body) = parse_frontmatter_and_body(&limited_output);
    assert_eq!(limited_frontmatter["body_truncated"], Value::Bool(true));
    assert!(limited_frontmatter["body_word_count"].as_u64().unwrap_or(0) <= 6);
    assert!(limited_body.split_whitespace().count() <= 6);
}

#[tokio::test]
async fn fetch_plain_text_content() {
    let mut server = mockito::Server::new_async().await;
    let text_body = "This is a simple plain text response with a few words.";

    let _mock = server
        .mock("GET", "/doc.txt")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body(text_body)
        .create_async()
        .await;

    let url = format!("{}/doc.txt", server.url());
    let output = fetch_url(&url, FetchOptions::default()).await.expect("plain text output");

    let (frontmatter, body) = parse_frontmatter_and_body(&output);

    assert_eq!(frontmatter["success"], Value::Bool(true));
    assert_eq!(frontmatter["status"], Value::Number(200.into()));
    assert_eq!(frontmatter["url"], Value::String(url.clone()));
    assert_eq!(body.trim(), text_body);

    // Let's test with max_body_words option
    let options = FetchOptions { max_body_words: Some(5), ..Default::default() };
    let output_truncated = fetch_url(&url, options).await.expect("plain text output");
    let (frontmatter_truncated, body_truncated) = parse_frontmatter_and_body(&output_truncated);

    assert_eq!(frontmatter_truncated["body_truncated"], Value::Bool(true));
    assert_eq!(body_truncated.trim(), "This is a simple plain");
}

#[tokio::test]
async fn fetch_feed_content() {
    let mut server = mockito::Server::new_async().await;
    let atom_feed = r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Example Feed</title>
  <entry>
    <title>First Entry</title>
    <link href="http://example.org/1"/>
    <updated>2026-05-22T00:00:00Z</updated>
    <summary>This is the summary of the first entry.</summary>
  </entry>
</feed>"#;

    let _mock = server
        .mock("GET", "/feed.xml")
        .with_status(200)
        .with_header("content-type", "application/atom+xml")
        .with_body(atom_feed)
        .create_async()
        .await;

    let url = format!("{}/feed.xml", server.url());
    let output = fetch_url(&url, FetchOptions::default()).await.expect("feed output");

    let (frontmatter, body) = parse_frontmatter_and_body(&output);

    assert_eq!(frontmatter["success"], Value::Bool(true));
    assert_eq!(frontmatter["title"], Value::String("Example Feed".to_string()));
    assert!(body.contains("# Example Feed"));
    assert!(body.contains("## First Entry"));
    assert!(body.contains("- **Link:** http://example.org/1"));
    assert!(body.contains("- **Summary:** This is the summary of the first entry."));

    let limited_output =
        fetch_url(&url, FetchOptions { max_body_words: Some(8), ..Default::default() })
            .await
            .expect("feed output with word limit");

    let (limited_frontmatter, limited_body) = parse_frontmatter_and_body(&limited_output);
    assert_eq!(limited_frontmatter["body_truncated"], Value::Bool(true));
    assert!(limited_frontmatter["body_word_count"].as_u64().unwrap_or(0) <= 8);
    assert!(limited_body.split_whitespace().count() <= 8);
}

#[tokio::test]
async fn fetch_non_feed_xml_content() {
    let mut server = mockito::Server::new_async().await;
    let xml_content = r#"<?xml version="1.0" encoding="utf-8"?>
<config>
  <setting name="enabled">true</setting>
</config>"#;

    let _mock = server
        .mock("GET", "/config.xml")
        .with_status(200)
        .with_header("content-type", "application/xml")
        .with_body(xml_content)
        .create_async()
        .await;

    let url = format!("{}/config.xml", server.url());
    let output = fetch_url(&url, FetchOptions::default()).await.expect("xml output");

    let (frontmatter, body) = parse_frontmatter_and_body(&output);

    assert_eq!(frontmatter["success"], Value::Bool(true));
    assert_eq!(frontmatter["title"], Value::Null);
    assert!(body.contains("```xml"));
    assert!(body.contains("<config>"));
}

#[tokio::test]
async fn fetch_pdf_extraction_failure() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/invalid.pdf")
        .with_status(200)
        .with_header("content-type", "application/pdf")
        .with_body(b"not a valid pdf content")
        .create_async()
        .await;

    let url = format!("{}/invalid.pdf", server.url());
    let output = fetch_url(&url, FetchOptions::default()).await.expect("pdf output");

    let (frontmatter, body) = parse_frontmatter_and_body(&output);

    assert_eq!(frontmatter["success"], Value::Bool(false));
    assert_eq!(frontmatter["error"], Value::String("pdf_extraction_failed".to_string()));
    assert!(body.trim().is_empty());
}

#[tokio::test]
async fn fetch_html_metadata_in_envelope() {
    let mut server = mockito::Server::new_async().await;
    let html = r#"
        <html>
            <head>
                <title>Metadata Page</title>
                <meta name="description" content="This description must be extracted.">
                <link rel="canonical" href="https://example.com/canonical-url">
            </head>
            <body>
                <article><p>Some interesting article content.</p></article>
            </body>
        </html>
    "#;

    let _mock = server
        .mock("GET", "/metadata")
        .with_status(200)
        .with_header("content-type", "text/html; charset=utf-8")
        .with_body(html)
        .create_async()
        .await;

    let url = format!("{}/metadata", server.url());
    let output = fetch_url(&url, FetchOptions::default()).await.expect("fetch output");

    let (frontmatter, body) = parse_frontmatter_and_body(&output);

    assert_eq!(frontmatter["success"], Value::Bool(true));
    assert_eq!(frontmatter["title"], Value::String("Metadata Page".to_string()));
    assert_eq!(
        frontmatter["description"],
        Value::String("This description must be extracted.".to_string())
    );
    assert_eq!(
        frontmatter["canonical_url"],
        Value::String("https://example.com/canonical-url".to_string())
    );
    assert!(body.contains("Some interesting article content."));
}
