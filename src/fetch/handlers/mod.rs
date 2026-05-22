pub mod feed;
pub mod json;
pub mod pdf;
pub mod text;

#[derive(Debug, Clone)]
pub struct HandlerResult {
    pub title: Option<String>,
    pub body: String,
    pub word_count: usize,
    pub body_word_count: usize,
    pub truncated: bool,
}
