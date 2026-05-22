use mdget::fetch::{FetchOptions, fetch_url};
use serde_yml::Value;

fn parse_frontmatter_and_body(output: &str) -> (Value, &str) {
    let stripped = output
        .strip_prefix("---\n")
        .expect("output must start with YAML frontmatter");
    let split_index = stripped
        .find("\n---\n")
        .expect("output must contain closing YAML delimiter");

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
    let output = fetch_url(&start_url, FetchOptions::default())
        .await
        .expect("redirect fetch output");

    let (frontmatter, body) = parse_frontmatter_and_body(&output);

    assert_eq!(frontmatter["success"], Value::Bool(true));
    assert_eq!(frontmatter["status"], Value::Number(200.into()));
    assert_eq!(frontmatter["url"], Value::String(final_url));

    let redirects = frontmatter["redirect_chain"]
        .as_sequence()
        .expect("redirect_chain should be present for redirected response");
    assert!(!redirects.is_empty());
    assert!(redirects
        .iter()
        .filter_map(Value::as_str)
        .any(|entry| entry.contains("/start")));

    assert!(body.contains("# Test Article"));
}
