use std::fs;
use std::process::Command;

fn normalize_output(s: &str) -> String {
    s.lines()
        .map(|line| {
            if line.contains("fetched_at:") {
                "fetched_at: 'PLACEHOLDER'"
            } else if line.contains("\"fetched_at\"") {
                "  \"fetched_at\": \"PLACEHOLDER\","
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[tokio::test]
async fn parity_default_mode_stdout_equals_file_bytes() {
    let mut server = mockito::Server::new_async().await;
    let html = "<html><head><title>Parity Test</title></head><body><article><p>Hello Parity World!</p></article></body></html>";

    let _mock = server
        .mock("GET", "/ok")
        .with_status(200)
        .with_header("content-type", "text/html; charset=utf-8")
        .with_body(html)
        .create_async()
        .await;

    let url = format!("{}/ok", server.url());
    let bin_path = env!("CARGO_BIN_EXE_mdget");

    // 1. Fetch to stdout
    let stdout_output =
        Command::new(bin_path).arg("fetch").arg(&url).output().expect("failed to run binary");
    assert!(stdout_output.status.success(), "stdout command failed with: {:?}", stdout_output);
    let stdout_str = String::from_utf8(stdout_output.stdout).expect("valid utf-8 stdout");

    // 2. Fetch to file
    let out_file = std::env::temp_dir().join("parity_temp_default.txt");
    let file_output = Command::new(bin_path)
        .arg("fetch")
        .arg(&url)
        .arg("-o")
        .arg(&out_file)
        .output()
        .expect("failed to run binary with -o");
    assert!(file_output.status.success(), "file command failed with: {:?}", file_output);

    let file_str = fs::read_to_string(&out_file).expect("failed to read written file");
    let _ = fs::remove_file(&out_file);

    assert_eq!(normalize_output(&stdout_str), normalize_output(&file_str));
}

#[tokio::test]
async fn parity_no_frontmatter_mode_stdout_equals_file_bytes() {
    let mut server = mockito::Server::new_async().await;
    let html = "<html><head><title>Parity Test</title></head><body><article><p>Hello Parity World!</p></article></body></html>";

    let _mock = server
        .mock("GET", "/ok")
        .with_status(200)
        .with_header("content-type", "text/html; charset=utf-8")
        .with_body(html)
        .create_async()
        .await;

    let url = format!("{}/ok", server.url());
    let bin_path = env!("CARGO_BIN_EXE_mdget");

    // 1. Fetch to stdout
    let stdout_output = Command::new(bin_path)
        .arg("fetch")
        .arg(&url)
        .arg("--no-frontmatter")
        .output()
        .expect("failed to run binary");
    assert!(stdout_output.status.success(), "stdout command failed with: {:?}", stdout_output);
    let stdout_str = String::from_utf8(stdout_output.stdout).expect("valid utf-8 stdout");

    // 2. Fetch to file
    let out_file = std::env::temp_dir().join("parity_temp_no_frontmatter.txt");
    let file_output = Command::new(bin_path)
        .arg("fetch")
        .arg(&url)
        .arg("--no-frontmatter")
        .arg("-o")
        .arg(&out_file)
        .output()
        .expect("failed to run binary with -o");
    assert!(file_output.status.success(), "file command failed with: {:?}", file_output);

    let file_str = fs::read_to_string(&out_file).expect("failed to read written file");
    let _ = fs::remove_file(&out_file);

    assert_eq!(normalize_output(&stdout_str), normalize_output(&file_str));
}

#[tokio::test]
async fn parity_json_mode_stdout_equals_file_bytes() {
    let mut server = mockito::Server::new_async().await;
    let html = "<html><head><title>Parity Test</title></head><body><article><p>Hello Parity World!</p></article></body></html>";

    let _mock = server
        .mock("GET", "/ok")
        .with_status(200)
        .with_header("content-type", "text/html; charset=utf-8")
        .with_body(html)
        .create_async()
        .await;

    let url = format!("{}/ok", server.url());
    let bin_path = env!("CARGO_BIN_EXE_mdget");

    // 1. Fetch to stdout
    let stdout_output = Command::new(bin_path)
        .arg("fetch")
        .arg(&url)
        .arg("--json")
        .output()
        .expect("failed to run binary");
    assert!(stdout_output.status.success(), "stdout command failed with: {:?}", stdout_output);
    let stdout_str = String::from_utf8(stdout_output.stdout).expect("valid utf-8 stdout");

    // 2. Fetch to file
    let out_file = std::env::temp_dir().join("parity_temp_json.txt");
    let file_output = Command::new(bin_path)
        .arg("fetch")
        .arg(&url)
        .arg("--json")
        .arg("-o")
        .arg(&out_file)
        .output()
        .expect("failed to run binary with -o");
    assert!(file_output.status.success(), "file command failed with: {:?}", file_output);

    let file_str = fs::read_to_string(&out_file).expect("failed to read written file");
    let _ = fs::remove_file(&out_file);

    assert_eq!(normalize_output(&stdout_str), normalize_output(&file_str));
}
