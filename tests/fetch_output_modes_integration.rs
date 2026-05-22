use mdget::fetch::{FetchOptions, OutputMode, fetch_url};
use serde_json::Value as JsonValue;
use serde_yml::Value as YamlValue;

fn parse_frontmatter_and_body(output: &str) -> (YamlValue, &str) {
    let stripped = output.strip_prefix("---\n").expect("output must start with YAML frontmatter");
    let split_index = stripped.find("\n---\n").expect("output must contain closing YAML delimiter");

    let yaml = &stripped[..split_index];
    let body = &stripped[split_index + "\n---\n".len()..];
    let frontmatter: YamlValue = serde_yml::from_str(yaml).expect("valid yaml frontmatter");

    (frontmatter, body)
}

#[tokio::test]
async fn output_mode_frontmatter_markdown_default() {
    let mut server = mockito::Server::new_async().await;
    let html = r#"
        <html>
            <head>
                <title>Test Page</title>
                <meta name="description" content="This is a test description.">
                <link rel="canonical" href="https://example.com/canonical">
            </head>
            <body>
                <article><p>Hello world</p></article>
            </body>
        </html>
    "#;

    let _mock = server
        .mock("GET", "/ok")
        .with_status(200)
        .with_header("content-type", "text/html; charset=utf-8")
        .with_body(html)
        .create_async()
        .await;

    let url = format!("{}/ok", server.url());
    let options = FetchOptions {
        output_mode: OutputMode::FrontmatterMarkdown,
        ..Default::default()
    };
    let output = fetch_url(&url, options).await.expect("fetch output");

    let (frontmatter, body) = parse_frontmatter_and_body(&output);

    assert_eq!(frontmatter["success"], YamlValue::Bool(true));
    assert_eq!(frontmatter["status"], YamlValue::Number(200.into()));
    assert_eq!(frontmatter["title"], YamlValue::String("Test Page".to_string()));
    assert_eq!(frontmatter["description"], YamlValue::String("This is a test description.".to_string()));
    assert_eq!(frontmatter["canonical_url"], YamlValue::String("https://example.com/canonical".to_string()));
    assert_eq!(body.trim(), "Hello world");
}

#[tokio::test]
async fn output_mode_markdown_only() {
    let mut server = mockito::Server::new_async().await;
    let html = "<html><head><title>Test Page</title></head><body><article><p>Hello world</p></article></body></html>";

    let _mock = server
        .mock("GET", "/ok")
        .with_status(200)
        .with_header("content-type", "text/html; charset=utf-8")
        .with_body(html)
        .create_async()
        .await;

    let url = format!("{}/ok", server.url());
    let options = FetchOptions {
        output_mode: OutputMode::MarkdownOnly,
        ..Default::default()
    };
    let output = fetch_url(&url, options).await.expect("fetch output");

    // Output must be only the markdown body, no frontmatter or -- delimiters
    assert!(!output.starts_with("---"));
    assert_eq!(output.trim(), "Hello world");
}

#[tokio::test]
async fn output_mode_json_envelope() {
    let mut server = mockito::Server::new_async().await;
    let html = r#"
        <html>
            <head>
                <title>Test Page</title>
                <meta name="description" content="This is a test description.">
                <link rel="canonical" href="https://example.com/canonical">
            </head>
            <body>
                <article><p>Hello world</p></article>
            </body>
        </html>
    "#;

    let _mock = server
        .mock("GET", "/ok")
        .with_status(200)
        .with_header("content-type", "text/html; charset=utf-8")
        .with_body(html)
        .create_async()
        .await;

    let url = format!("{}/ok", server.url());
    let options = FetchOptions {
        output_mode: OutputMode::JsonEnvelope,
        ..Default::default()
    };
    let output = fetch_url(&url, options).await.expect("fetch output");

    // Must be valid JSON
    let json: JsonValue = serde_json::from_str(&output).expect("valid json envelope");

    assert_eq!(json["success"], JsonValue::Bool(true));
    assert_eq!(json["status"], JsonValue::Number(200.into()));
    assert_eq!(json["title"], JsonValue::String("Test Page".to_string()));
    assert_eq!(json["description"], JsonValue::String("This is a test description.".to_string()));
    assert_eq!(json["canonical_url"], JsonValue::String("https://example.com/canonical".to_string()));
    assert_eq!(json["markdown"], JsonValue::String("Hello world".to_string()));
}
