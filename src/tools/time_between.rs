use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{parse_datetime, ToolError};
use crate::duration::DurationBreakdown;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[schemars(description = "Parameters for the `time_between` tool.")]
pub struct TimeBetweenRequest {
    /// Start datetime in ISO 8601 / RFC 3339 format.
    pub start: String,
    /// End datetime in ISO 8601 / RFC 3339 format.
    pub end: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct TimeBetweenResponse {
    /// The start datetime, normalized to ISO 8601.
    pub from: String,
    /// The end datetime, normalized to ISO 8601.
    pub to: String,
    /// Duration from `start` to `end`. Negative if `end` is before `start`.
    pub duration: DurationBreakdown,
}

pub fn run(req: TimeBetweenRequest) -> Result<TimeBetweenResponse, ToolError> {
    let start = parse_datetime(&req.start)?;
    let end = parse_datetime(&req.end)?;
    let d = end.signed_duration_since(start);
    Ok(TimeBetweenResponse {
        from: start.to_rfc3339(),
        to: end.to_rfc3339(),
        duration: DurationBreakdown::from_duration(d),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forward_duration_is_positive() {
        let r = run(TimeBetweenRequest {
            start: "2026-01-01T00:00:00Z".into(),
            end: "2026-01-02T00:00:00Z".into(),
        })
        .unwrap();
        assert_eq!(r.duration.total_seconds, 86_400);
        assert_eq!(r.duration.days, 1);
    }

    #[test]
    fn reversed_duration_is_negative() {
        let r = run(TimeBetweenRequest {
            start: "2026-01-02T00:00:00Z".into(),
            end: "2026-01-01T00:00:00Z".into(),
        })
        .unwrap();
        assert_eq!(r.duration.total_seconds, -86_400);
    }

    #[test]
    fn handles_cross_timezone_inputs() {
        // Same instant, different zones → 0 duration.
        let r = run(TimeBetweenRequest {
            start: "2026-04-20T15:00:00-06:00".into(),
            end: "2026-04-20T21:00:00Z".into(),
        })
        .unwrap();
        assert_eq!(r.duration.total_seconds, 0);
    }

    #[test]
    fn bad_start_has_hint() {
        let err = run(TimeBetweenRequest {
            start: "nope".into(),
            end: "2026-01-01T00:00:00Z".into(),
        })
        .unwrap_err();
        assert!(err.hint.is_some());
    }
}
