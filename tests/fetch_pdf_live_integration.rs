use mdget::fetch::{FetchOptions, fetch_url};
use scraper::{Html, Selector};
use serde_yml::Value;

async fn fetch_text_with_retry(url: &'static str, attempts: usize) -> Result<String, String> {
    let mut last_error = String::new();

    for attempt in 1..=attempts {
        let result = tokio::task::spawn_blocking(move || {
            let config = ureq::Agent::config_builder()
                .http_status_as_error(false)
                .timeout_global(Some(std::time::Duration::from_secs(45)))
                .build();
            let agent = ureq::Agent::new_with_config(config);

            let response =
                agent.get(url).call().map_err(|e| format!("failed to fetch {url}: {e}"))?;

            if response.status().as_u16() >= 400 {
                return Err(format!("{url} returned HTTP {}", response.status().as_u16()));
            }

            response
                .into_body()
                .read_to_string()
                .map_err(|e| format!("failed to read {url} body: {e}"))
        })
        .await
        .map_err(|e| format!("blocking task join error: {e}"));

        match result {
            Ok(Ok(body)) => return Ok(body),
            Ok(Err(err)) | Err(err) => {
                last_error = err;
                if attempt < attempts {
                    let backoff_secs = 1_u64 << (attempt - 1);
                    tokio::time::sleep(std::time::Duration::from_secs(backoff_secs)).await;
                }
            }
        }
    }

    Err(format!("request failed after {} attempts for {}: {}", attempts, url, last_error))
}

fn parse_frontmatter_and_body(output: &str) -> (Value, &str) {
    let stripped = output.strip_prefix("---\n").expect("output must start with YAML frontmatter");
    let split_index = stripped.find("\n---\n").expect("output must contain closing YAML delimiter");

    let yaml = &stripped[..split_index];
    let body = &stripped[split_index + "\n---\n".len()..];
    let frontmatter: Value = serde_yml::from_str(yaml).expect("valid yaml frontmatter");

    (frontmatter, body)
}

fn extract_first_arxiv_pdf_link(html: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    let selector = Selector::parse("a[href^='/pdf/']").ok()?;
    let href = doc.select(&selector).next()?.value().attr("href")?;

    if href.starts_with("http://") || href.starts_with("https://") {
        Some(href.to_string())
    } else {
        Some(format!("https://arxiv.org{href}"))
    }
}

#[test]
fn extract_first_arxiv_pdf_link_from_fixture_like_html() {
    let html = r#"
        <html>
            <body>
                <a href="/abs/2601.00001">abstract</a>
                <a href="/pdf/2601.00001">pdf</a>
            </body>
        </html>
    "#;

    let pdf_url = extract_first_arxiv_pdf_link(html).expect("pdf link should exist");
    assert_eq!(pdf_url, "https://arxiv.org/pdf/2601.00001");
}

#[tokio::test]
#[ignore = "Live network test: fetches https://arxiv.org/list/cs.AI/recent"]
async fn live_arxiv_recent_page_discovers_pdf_and_extracts_text() {
    let list_url = "https://arxiv.org/list/cs.AI/recent";
    let list_html = fetch_text_with_retry(list_url, 3).await.expect("download arxiv list html");

    let pdf_url =
        extract_first_arxiv_pdf_link(&list_html).expect("arxiv list should expose a pdf link");

    let output = fetch_url(&pdf_url, FetchOptions { timeout_secs: 60, ..Default::default() })
        .await
        .expect("fetch pdf output");

    let (frontmatter, body) = parse_frontmatter_and_body(&output);

    assert_eq!(frontmatter["success"], Value::Bool(true));
    assert_eq!(frontmatter["error"], Value::Null);
    assert!(frontmatter["status"].as_u64().unwrap_or(0) < 400);
    assert!(body.split_whitespace().count() > 20, "expected extracted PDF text to be non-trivial");
}

#[tokio::test]
#[ignore = "Live network test: fetches https://arxiv.org/list/cs.AI/recent"]
async fn live_arxiv_pdf_respects_max_body_words_limit() {
    let list_url = "https://arxiv.org/list/cs.AI/recent";
    let list_html = fetch_text_with_retry(list_url, 3).await.expect("download arxiv list html");

    let pdf_url =
        extract_first_arxiv_pdf_link(&list_html).expect("arxiv list should expose a pdf link");

    let output = fetch_url(
        &pdf_url,
        FetchOptions { timeout_secs: 60, max_body_words: Some(30), ..Default::default() },
    )
    .await
    .expect("fetch pdf output with max body words");

    let (frontmatter, body) = parse_frontmatter_and_body(&output);

    assert_eq!(frontmatter["success"], Value::Bool(true));
    assert_eq!(frontmatter["body_truncated"], Value::Bool(true));
    assert!(frontmatter["body_word_count"].as_u64().unwrap_or(0) <= 30);
    assert!(body.split_whitespace().count() <= 30);
}
