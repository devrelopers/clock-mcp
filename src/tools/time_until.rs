use chrono::Utc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{parse_datetime, ToolError};
use crate::duration::DurationBreakdown;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[schemars(description = "Parameters for the `time_until` tool.")]
pub struct TimeUntilRequest {
    /// Target datetime in ISO 8601 / RFC 3339 format (e.g. "2026-12-31T23:59:59Z").
    pub target: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct TimeUntilResponse {
    /// The current time (UTC) used as the starting point, ISO 8601.
    pub from: String,
    /// The target datetime, normalized to ISO 8601.
    pub to: String,
    /// Duration from `from` to `to`. Negative if the target is already in the past.
    pub duration: DurationBreakdown,
}

pub fn run(req: TimeUntilRequest) -> Result<TimeUntilResponse, ToolError> {
    let target = parse_datetime(&req.target)?;
    let now = Utc::now();
    let d = target.signed_duration_since(now);
    Ok(TimeUntilResponse {
        from: now.to_rfc3339(),
        to: target.to_rfc3339(),
        duration: DurationBreakdown::from_duration(d),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn future_target_is_positive() {
        let future = (Utc::now() + chrono::Duration::hours(2)).to_rfc3339();
        let r = run(TimeUntilRequest { target: future }).unwrap();
        assert!(r.duration.total_seconds > 0);
    }

    #[test]
    fn past_target_is_negative() {
        let past = (Utc::now() - chrono::Duration::hours(2)).to_rfc3339();
        let r = run(TimeUntilRequest { target: past }).unwrap();
        assert!(r.duration.total_seconds < 0);
    }

    #[test]
    fn bad_datetime_has_hint() {
        let err = run(TimeUntilRequest {
            target: "tomorrow".into(),
        })
        .unwrap_err();
        assert!(err.hint.is_some());
    }
}
