use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{parse_datetime, parse_timezone, ToolError};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[schemars(description = "Parameters for the `convert_timezone` tool.")]
pub struct ConvertTimezoneRequest {
    /// Input datetime in ISO 8601 / RFC 3339 format (must include a timezone offset).
    pub datetime: String,
    /// Target IANA timezone to convert to (e.g. "Asia/Tokyo").
    pub target_timezone: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ConvertTimezoneResponse {
    /// The input datetime, echoed in normalized ISO 8601 form.
    pub original: String,
    /// The equivalent local time in the target timezone, ISO 8601.
    pub converted: String,
    /// The resolved target IANA timezone name.
    pub target_timezone: String,
    /// Unix timestamp (timezone-independent) — identical for input and output.
    pub unix_seconds: i64,
}

pub fn run(req: ConvertTimezoneRequest) -> Result<ConvertTimezoneResponse, ToolError> {
    let tz = parse_timezone(&req.target_timezone)?;
    let dt = parse_datetime(&req.datetime)?;
    let converted = dt.with_timezone(&tz);
    Ok(ConvertTimezoneResponse {
        original: dt.to_rfc3339(),
        converted: converted.to_rfc3339(),
        target_timezone: tz.name().to_string(),
        unix_seconds: dt.timestamp(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_utc_to_denver() {
        let r = run(ConvertTimezoneRequest {
            datetime: "2026-04-20T21:00:00Z".into(),
            target_timezone: "America/Denver".into(),
        })
        .unwrap();
        // April 20 → MDT (UTC-6). 21:00Z → 15:00 local.
        assert!(r.converted.starts_with("2026-04-20T15:00:00"));
        assert!(r.converted.contains("-06:00"));
        assert_eq!(r.target_timezone, "America/Denver");
    }

    #[test]
    fn preserves_instant_across_zones() {
        let r = run(ConvertTimezoneRequest {
            datetime: "2026-04-20T21:00:00Z".into(),
            target_timezone: "Asia/Tokyo".into(),
        })
        .unwrap();
        // Converting timezones must not change the absolute instant.
        let expected = chrono::DateTime::parse_from_rfc3339("2026-04-20T21:00:00Z")
            .unwrap()
            .timestamp();
        assert_eq!(r.unix_seconds, expected);
        // Tokyo is UTC+9 year-round, so 21:00Z → 06:00 next day local.
        assert!(r.converted.starts_with("2026-04-21T06:00:00"));
        assert!(r.converted.contains("+09:00"));
    }

    #[test]
    fn unknown_timezone_returns_hint() {
        let err = run(ConvertTimezoneRequest {
            datetime: "2026-04-20T21:00:00Z".into(),
            target_timezone: "Middle-earth/Rivendell".into(),
        })
        .unwrap_err();
        assert!(err.hint.is_some());
    }

    #[test]
    fn bad_datetime_returns_hint() {
        let err = run(ConvertTimezoneRequest {
            datetime: "not iso".into(),
            target_timezone: "UTC".into(),
        })
        .unwrap_err();
        assert!(err.hint.is_some());
    }
}
