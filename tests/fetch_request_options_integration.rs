use mdget::fetch::{FetchOptions, RequestOptions, fetch_url};
use mdget::fetch::request::{parse_header, merge_cookies};

fn parse_frontmatter_and_body(output: &str) -> (serde_yml::Value, &str) {
    let stripped = output.strip_prefix("---\n").expect("output must start with YAML frontmatter");
    let split_index = stripped.find("\n---\n").expect("output must contain closing YAML delimiter");

    let yaml = &stripped[..split_index];
    let body = &stripped[split_index + "\n---\n".len()..];
    let frontmatter: serde_yml::Value = serde_yml::from_str(yaml).expect("valid yaml frontmatter");

    (frontmatter, body)
}

#[tokio::test]
async fn fetch_with_custom_headers() {
    let mut server = mockito::Server::new_async().await;
    let html = include_str!("fixtures/simple_article.html");

    let _mock = server
        .mock("GET", "/headers")
        .match_header("X-Custom-Header", "Value1")
        .match_header("X-Another-Header", "Value2")
        .with_status(200)
        .with_header("content-type", "text/html; charset=utf-8")
        .with_body(html)
        .create_async()
        .await;

    let url = format!("{}/headers", server.url());
    let options = FetchOptions {
        request: RequestOptions {
            headers: vec![
                ("X-Custom-Header".to_string(), "Value1".to_string()),
                ("X-Another-Header".to_string(), "Value2".to_string()),
            ],
            ..Default::default()
        },
        ..Default::default()
    };

    let output = fetch_url(&url, options).await.expect("fetch output");
    let (_frontmatter, body) = parse_frontmatter_and_body(&output);
    assert!(body.contains("# Test Article"));
}

#[tokio::test]
async fn fetch_with_cookies_merged() {
    let mut server = mockito::Server::new_async().await;
    let html = include_str!("fixtures/simple_article.html");

    let _mock = server
        .mock("GET", "/cookies")
        .match_header("Cookie", "session=abc123_xyz; theme=dark")
        .with_status(200)
        .with_header("content-type", "text/html; charset=utf-8")
        .with_body(html)
        .create_async()
        .await;

    let url = format!("{}/cookies", server.url());
    let options = FetchOptions {
        request: RequestOptions {
            cookies: vec![
                "session=abc123_xyz".to_string(),
                "theme=dark".to_string(),
            ],
            ..Default::default()
        },
        ..Default::default()
    };

    let output = fetch_url(&url, options).await.expect("fetch output");
    let (_frontmatter, body) = parse_frontmatter_and_body(&output);
    assert!(body.contains("# Test Article"));
}

#[tokio::test]
async fn fetch_with_bearer_token_precedence() {
    let mut server = mockito::Server::new_async().await;
    let html = include_str!("fixtures/simple_article.html");

    let _mock = server
        .mock("GET", "/bearer")
        .match_header("Authorization", "Bearer my_secret_token")
        .with_status(200)
        .with_header("content-type", "text/html; charset=utf-8")
        .with_body(html)
        .create_async()
        .await;

    let url = format!("{}/bearer", server.url());
    
    // Explicit Authorization header is provided via custom headers,
    // but the bearer option should override it.
    let options = FetchOptions {
        request: RequestOptions {
            headers: vec![
                ("Authorization".to_string(), "Bearer override-me".to_string()),
                ("X-Extra".to_string(), "Stay".to_string()),
            ],
            bearer: Some("my_secret_token".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    let output = fetch_url(&url, options).await.expect("fetch output");
    let (_frontmatter, body) = parse_frontmatter_and_body(&output);
    assert!(body.contains("# Test Article"));
}

#[test]
fn test_header_validation_parser() {
    // Valid headers
    assert_eq!(parse_header("X-Header: value").unwrap(), ("X-Header".to_string(), "value".to_string()));
    assert_eq!(parse_header("Authorization:  Bearer 123 ").unwrap(), ("Authorization".to_string(), "Bearer 123".to_string()));
    assert_eq!(parse_header("Content-Type:application/json").unwrap(), ("Content-Type".to_string(), "application/json".to_string()));

    // Invalid format
    assert!(parse_header("NoColonHere").is_err());
    assert!(parse_header(": emptyname").is_err());
    
    // Control characters in name
    assert!(parse_header("Header\nName: value").is_err());
    assert!(parse_header("Header\rName: value").is_err());
    
    // Whitespace or colon in name
    assert!(parse_header("Header Name: value").is_err());
    assert_eq!(parse_header("Header:Name: value").unwrap(), ("Header".to_string(), "Name: value".to_string()));

    // Control characters in value
    assert!(parse_header("Header: value\nwithnewline").is_err());
}

#[test]
fn test_cookie_merging() {
    assert_eq!(merge_cookies(&[]), None);
    assert_eq!(merge_cookies(&["a=1".to_string()]), Some("a=1".to_string()));
    assert_eq!(merge_cookies(&["a=1".to_string(), "b=2".to_string()]), Some("a=1; b=2".to_string()));
}

#[test]
fn test_cli_conflict_json_no_frontmatter() {
    let output = std::process::Command::new("cargo")
        .args(&[
            "run", 
            "--quiet", 
            "--bin", 
            "mdget", 
            "--", 
            "fetch", 
            "http://example.com", 
            "--json", 
            "--no-frontmatter"
        ])
        .output()
        .expect("failed to execute cargo run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("cannot be used with"));
}

#[test]
fn test_cli_header_flag_does_not_conflict_with_home() {
    let output = std::process::Command::new("cargo")
        .args(&[
            "run", 
            "--quiet", 
            "--bin", 
            "mdget", 
            "--", 
            "fetch", 
            "http://invalid-url-12345.com", 
            "-H", 
            "X-Test: 123"
        ])
        .output()
        .expect("failed to execute cargo run");

    let stderr = String::from_utf8_lossy(&output.stderr);
    // CLI parsing succeeds (does not fail with clap error on arguments)
    assert!(!stderr.contains("error:"));
}
