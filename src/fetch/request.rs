use miette::{Result, miette};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    FrontmatterMarkdown, // default
    MarkdownOnly,        // --no-frontmatter
    JsonEnvelope,        // --json
}

#[derive(Debug, Clone, Default)]
pub struct RequestOptions {
    pub headers: Vec<(String, String)>,
    pub cookies: Vec<String>,
    pub bearer: Option<String>,
}

pub fn parse_header(raw: &str) -> Result<(String, String)> {
    let Some((name, value)) = raw.split_once(':') else {
        return Err(miette!("Invalid header format: missing colon in '{}'", raw));
    };
    let name = name.trim().to_string();
    let value = value.trim().to_string();

    if name.is_empty() {
        return Err(miette!("Invalid header: empty header name in '{}'", raw));
    }

    // Header names cannot contain control chars, spaces, or certain special chars
    for c in name.chars() {
        if c.is_control() || c.is_whitespace() || c == ':' {
            return Err(miette!("Invalid header name '{}': contains invalid characters", name));
        }
    }

    // Header values cannot contain control characters
    for c in value.chars() {
        if c.is_control() {
            return Err(miette!(
                "Invalid header value for '{}': contains control characters",
                name
            ));
        }
    }

    Ok((name, value))
}

pub fn parse_cookie(raw: &str) -> Result<String> {
    let cookie = raw.trim();
    let Some((name, _value)) = cookie.split_once('=') else {
        return Err(miette!("Invalid cookie format: missing '=' in '{}'", raw));
    };

    let name = name.trim();
    if name.is_empty() {
        return Err(miette!("Invalid cookie format: empty cookie name in '{}'", raw));
    }

    if cookie.chars().any(char::is_control) {
        return Err(miette!("Invalid cookie '{}': contains control characters", raw));
    }

    Ok(cookie.to_string())
}

pub fn validate_bearer(token: &str) -> Result<()> {
    if token.trim().is_empty() {
        return Err(miette!("Invalid bearer token: token cannot be empty"));
    }

    if token.chars().any(char::is_control) {
        return Err(miette!("Invalid bearer token: contains control characters"));
    }

    Ok(())
}

pub fn merge_cookies(cookies: &[String]) -> Option<String> {
    if cookies.is_empty() { None } else { Some(cookies.join("; ")) }
}
