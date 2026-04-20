use chrono::Utc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{parse_datetime, ToolError};
use crate::duration::DurationBreakdown;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[schemars(description = "Parameters for the `time_since` tool.")]
pub struct TimeSinceRequest {
    /// Past datetime in ISO 8601 / RFC 3339 format (e.g. "2024-01-01T00:00:00Z").
    pub past: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct TimeSinceResponse {
    /// The past datetime, normalized to ISO 8601.
    pub from: String,
    /// The current time (UTC), ISO 8601.
    pub to: String,
    /// Duration from `from` to `to`. Negative if the input is actually in the future.
    pub duration: DurationBreakdown,
}

pub fn run(req: TimeSinceRequest) -> Result<TimeSinceResponse, ToolError> {
    let past = parse_datetime(&req.past)?;
    let now = Utc::now();
    let d = now.signed_duration_since(past);
    Ok(TimeSinceResponse {
        from: past.to_rfc3339(),
        to: now.to_rfc3339(),
        duration: DurationBreakdown::from_duration(d),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn past_target_is_positive() {
        let past = (Utc::now() - chrono::Duration::hours(3)).to_rfc3339();
        let r = run(TimeSinceRequest { past }).unwrap();
        assert!(r.duration.total_seconds > 0);
    }

    #[test]
    fn future_target_is_negative() {
        let future = (Utc::now() + chrono::Duration::hours(3)).to_rfc3339();
        let r = run(TimeSinceRequest { past: future }).unwrap();
        assert!(r.duration.total_seconds < 0);
    }

    #[test]
    fn bad_datetime_has_hint() {
        let err = run(TimeSinceRequest {
            past: "last tuesday".into(),
        })
        .unwrap_err();
        assert!(err.hint.is_some());
    }
}
