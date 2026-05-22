use crate::fetch::{OutputMode, SuccessEnvelope};
use miette::Result;

/// Format the success envelope and body according to the chosen output mode.
pub fn format_output(envelope: &SuccessEnvelope, body: &str, mode: OutputMode) -> Result<String> {
    match mode {
        OutputMode::FrontmatterMarkdown => envelope.to_output(body),
        OutputMode::MarkdownOnly => Ok(body.to_string()),
        OutputMode::JsonEnvelope => {
            let mut val = serde_json::to_value(envelope).map_err(|e| {
                miette::miette!("Failed to serialize SuccessEnvelope to JSON: {}", e)
            })?;
            if let Some(obj) = val.as_object_mut() {
                obj.insert("markdown".to_string(), serde_json::Value::String(body.to_string()));
            }
            let json_str = serde_json::to_string_pretty(&val).map_err(|e| {
                miette::miette!("Failed to serialize envelope value to JSON: {}", e)
            })?;
            Ok(json_str)
        }
    }
}
