use chrono::Utc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{parse_timezone, ToolError};

#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
#[schemars(description = "Parameters for the `now` tool.")]
pub struct NowRequest {
    /// IANA timezone name (e.g. "America/Denver", "Europe/Berlin"). Defaults to "UTC".
    #[serde(default)]
    pub timezone: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct NowResponse {
    /// ISO 8601 / RFC 3339 formatted timestamp in the resolved timezone.
    pub iso8601: String,
    /// Unix timestamp in seconds (timezone-independent).
    pub unix_seconds: i64,
    /// The resolved IANA timezone name.
    pub timezone: String,
}

pub fn run(req: NowRequest) -> Result<NowResponse, ToolError> {
    let tz_name = req.timezone.as_deref().unwrap_or("UTC");
    let tz = parse_timezone(tz_name)?;
    let local = Utc::now().with_timezone(&tz);
    Ok(NowResponse {
        iso8601: local.to_rfc3339(),
        unix_seconds: local.timestamp(),
        timezone: tz.name().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_utc() {
        let r = run(NowRequest::default()).unwrap();
        assert_eq!(r.timezone, "UTC");
        assert!(r.iso8601.ends_with("+00:00") || r.iso8601.ends_with('Z'));
    }

    #[test]
    fn honours_explicit_timezone() {
        let r = run(NowRequest {
            timezone: Some("America/Denver".into()),
        })
        .unwrap();
        assert_eq!(r.timezone, "America/Denver");
    }

    #[test]
    fn unix_timestamp_is_close_to_system_clock() {
        let before = chrono::Utc::now().timestamp();
        let r = run(NowRequest::default()).unwrap();
        let after = chrono::Utc::now().timestamp();
        assert!(r.unix_seconds >= before && r.unix_seconds <= after);
    }

    #[test]
    fn bad_timezone_returns_hint() {
        let err = run(NowRequest {
            timezone: Some("Narnia/Cair_Paravel".into()),
        })
        .unwrap_err();
        assert!(err.hint.is_some());
    }
}
