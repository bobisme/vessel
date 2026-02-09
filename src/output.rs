//! Output format resolution and helpers.
//!
//! Provides consistent output formatting across all botty commands,
//! with auto-detection of terminal capabilities.

use serde_json::{json, Value};
use std::io::IsTerminal;

/// Output format for command results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Concise plain text for agents and pipes. ID-first, two-space delimiters.
    Text,
    /// Structured JSON with envelope: { key: data, "advice": [...] }
    Json,
    /// Colored human-friendly output for terminals.
    Pretty,
}

impl OutputFormat {
    /// Parse a format string into an OutputFormat.
    ///
    /// Returns None for unknown format strings.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "text" => Some(Self::Text),
            "json" => Some(Self::Json),
            "pretty" => Some(Self::Pretty),
            _ => None,
        }
    }
}

/// Resolve output format from flag, env var, and TTY detection.
///
/// Priority: --format flag > FORMAT env var > TTY detection (pretty for TTY, text for pipe).
///
/// Unknown format strings default to Text for robustness.
#[must_use]
pub fn resolve_format(flag: Option<&str>) -> OutputFormat {
    // 1. Check explicit flag first
    if let Some(fmt) = flag {
        if let Some(parsed) = OutputFormat::parse(fmt) {
            return parsed;
        }
        // Unknown format defaults to Text
        return OutputFormat::Text;
    }

    // 2. Check FORMAT env var
    if let Ok(env_format) = std::env::var("FORMAT") {
        if let Some(parsed) = OutputFormat::parse(&env_format) {
            return parsed;
        }
        // Unknown format defaults to Text
        return OutputFormat::Text;
    }

    // 3. Auto-detect based on TTY
    if std::io::stdout().is_terminal() {
        OutputFormat::Pretty
    } else {
        OutputFormat::Text
    }
}

/// Wrap data in a standard JSON envelope with advice.
///
/// Returns: { "<key>": <data>, "advice": [<advice strings>] }
#[must_use]
pub fn json_envelope(key: &str, data: Value, advice: Vec<String>) -> Value {
    json!({
        key: data,
        "advice": advice,
    })
}

/// Format a single record as ID-first text with two-space delimiters.
///
/// Example: `text_record(&["agent-abc", "running", "bash", "pid=1234"])`
/// Returns: "agent-abc  running  bash  pid=1234"
#[must_use]
pub fn text_record(fields: &[&str]) -> String {
    fields.join("  ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_formats() {
        assert_eq!(OutputFormat::parse("text"), Some(OutputFormat::Text));
        assert_eq!(OutputFormat::parse("json"), Some(OutputFormat::Json));
        assert_eq!(OutputFormat::parse("pretty"), Some(OutputFormat::Pretty));

        // Case insensitive
        assert_eq!(OutputFormat::parse("TEXT"), Some(OutputFormat::Text));
        assert_eq!(OutputFormat::parse("Json"), Some(OutputFormat::Json));
        assert_eq!(OutputFormat::parse("PRETTY"), Some(OutputFormat::Pretty));
    }

    #[test]
    fn test_parse_invalid_format() {
        assert_eq!(OutputFormat::parse("unknown"), None);
        assert_eq!(OutputFormat::parse("xml"), None);
        assert_eq!(OutputFormat::parse(""), None);
    }

    #[test]
    fn test_resolve_format_with_flag() {
        assert_eq!(resolve_format(Some("json")), OutputFormat::Json);
        assert_eq!(resolve_format(Some("text")), OutputFormat::Text);
        assert_eq!(resolve_format(Some("pretty")), OutputFormat::Pretty);
    }

    #[test]
    fn test_resolve_format_unknown_flag() {
        // Unknown format should default to Text
        assert_eq!(resolve_format(Some("unknown")), OutputFormat::Text);
        assert_eq!(resolve_format(Some("xml")), OutputFormat::Text);
    }

    #[test]
    fn test_resolve_format_with_env() {
        // Clear any existing FORMAT env var for this test
        unsafe {
            std::env::remove_var("FORMAT");
        }

        // Set FORMAT env var
        unsafe {
            std::env::set_var("FORMAT", "json");
        }
        assert_eq!(resolve_format(None), OutputFormat::Json);

        unsafe {
            std::env::set_var("FORMAT", "text");
        }
        assert_eq!(resolve_format(None), OutputFormat::Text);

        unsafe {
            std::env::set_var("FORMAT", "pretty");
        }
        assert_eq!(resolve_format(None), OutputFormat::Pretty);

        // Unknown env var should default to Text
        unsafe {
            std::env::set_var("FORMAT", "unknown");
        }
        assert_eq!(resolve_format(None), OutputFormat::Text);

        // Cleanup
        unsafe {
            std::env::remove_var("FORMAT");
        }
    }

    #[test]
    fn test_resolve_format_flag_overrides_env() {
        // Set env to json
        unsafe {
            std::env::set_var("FORMAT", "json");
        }

        // Flag should override env
        assert_eq!(resolve_format(Some("text")), OutputFormat::Text);

        // Cleanup
        unsafe {
            std::env::remove_var("FORMAT");
        }
    }

    #[test]
    fn test_json_envelope_with_data() {
        let data = json!({"id": "agent-123", "status": "running"});
        let advice = vec!["Use --json for structured output".to_string()];

        let result = json_envelope("agent", data.clone(), advice.clone());

        assert_eq!(result["agent"], data);
        assert_eq!(result["advice"], json!(advice));
    }

    #[test]
    fn test_json_envelope_empty_advice() {
        let data = json!({"id": "agent-123"});
        let result = json_envelope("agent", data.clone(), vec![]);

        assert_eq!(result["agent"], data);
        assert_eq!(result["advice"], json!([]));
    }

    #[test]
    fn test_json_envelope_multiple_advice() {
        let data = json!(null);
        let advice = vec![
            "First tip".to_string(),
            "Second tip".to_string(),
            "Third tip".to_string(),
        ];

        let result = json_envelope("result", data, advice.clone());

        assert_eq!(result["advice"], json!(advice));
    }

    #[test]
    fn test_text_record_simple() {
        let result = text_record(&["agent-abc", "running", "bash"]);
        assert_eq!(result, "agent-abc  running  bash");
    }

    #[test]
    fn test_text_record_with_keyvalue() {
        let result = text_record(&["agent-abc", "running", "bash", "pid=1234"]);
        assert_eq!(result, "agent-abc  running  bash  pid=1234");
    }

    #[test]
    fn test_text_record_single_field() {
        let result = text_record(&["agent-abc"]);
        assert_eq!(result, "agent-abc");
    }

    #[test]
    fn test_text_record_empty() {
        let result = text_record(&[]);
        assert_eq!(result, "");
    }

    #[test]
    fn test_text_record_fields_with_spaces() {
        // Fields should be joined with two spaces regardless of content
        let result = text_record(&["field one", "field two", "field three"]);
        assert_eq!(result, "field one  field two  field three");
    }
}
