//! Pure logic for each of clock-mcp's tools.
//!
//! Each submodule owns its request/response types and a `run()` function that takes the
//! request and returns either a serializable response or a structured [`ToolError`].
//! The MCP plumbing in `main.rs` simply calls these and wraps the outcome.

use chrono_tz::Tz;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod convert_timezone;
pub mod now;
pub mod time_between;
pub mod time_since;
pub mod time_until;

/// A structured error returned to the MCP client.
///
/// Serializes to `{ "error": "...", "hint": "..." }` (the `hint` key is omitted when absent).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ToolError {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl ToolError {
    pub fn with_hint(error: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            hint: Some(hint.into()),
        }
    }
}

/// Parse an IANA timezone name, returning a ToolError with a helpful hint on failure.
pub fn parse_timezone(name: &str) -> Result<Tz, ToolError> {
    name.parse::<Tz>().map_err(|_| {
        ToolError::with_hint(
            format!("Unknown timezone: {name:?}"),
            "Use an IANA timezone name like 'America/Denver', 'Europe/Berlin', 'Asia/Tokyo', or 'UTC'.",
        )
    })
}

/// Parse an ISO 8601 / RFC 3339 datetime, returning a ToolError with a hint on failure.
pub fn parse_datetime(s: &str) -> Result<chrono::DateTime<chrono::FixedOffset>, ToolError> {
    chrono::DateTime::parse_from_rfc3339(s).map_err(|e| {
        ToolError::with_hint(
            format!("Could not parse datetime {s:?}: {e}"),
            "Use ISO 8601 / RFC 3339 format with a timezone offset, e.g. '2026-04-20T15:30:00-06:00' or '2026-04-20T21:30:00Z'.",
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_timezone() {
        let tz = parse_timezone("America/Denver").unwrap();
        assert_eq!(tz.name(), "America/Denver");
    }

    #[test]
    fn parse_invalid_timezone_has_hint() {
        let err = parse_timezone("Mars/Olympus_Mons").unwrap_err();
        assert!(err.error.contains("Mars/Olympus_Mons"));
        assert!(err.hint.is_some());
    }

    #[test]
    fn parse_valid_datetime() {
        let dt = parse_datetime("2026-04-20T15:30:00-06:00").unwrap();
        assert_eq!(dt.offset().local_minus_utc(), -6 * 3600);
    }

    #[test]
    fn parse_bad_datetime_has_hint() {
        let err = parse_datetime("not a date").unwrap_err();
        assert!(err.hint.is_some());
    }

    #[test]
    fn tool_error_serializes_without_hint_key() {
        let err = ToolError {
            error: "boom".into(),
            hint: None,
        };
        let s = serde_json::to_string(&err).unwrap();
        assert_eq!(s, r#"{"error":"boom"}"#);
    }

    #[test]
    fn tool_error_serializes_with_hint() {
        let err = ToolError::with_hint("boom", "try this");
        let v: serde_json::Value = serde_json::to_value(&err).unwrap();
        assert_eq!(v["error"], "boom");
        assert_eq!(v["hint"], "try this");
    }
}
