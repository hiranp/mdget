pub mod json;
pub mod text;
pub mod feed;
pub mod pdf;

#[derive(Debug, Clone)]
pub struct HandlerResult {
    pub title: Option<String>,
    pub body: String,
    pub word_count: usize,
    pub body_word_count: usize,
    pub truncated: bool,
}
