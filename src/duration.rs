use chrono::Duration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A signed duration broken into days/hours/minutes/seconds plus a human-readable string.
///
/// All component fields carry the sign of the total. A duration of -3665 seconds renders as
/// `{ total_seconds: -3665, days: 0, hours: -1, minutes: -1, seconds: -5, human: "-1h 1m 5s" }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DurationBreakdown {
    pub total_seconds: i64,
    pub days: i64,
    pub hours: i64,
    pub minutes: i64,
    pub seconds: i64,
    pub human: String,
}

impl DurationBreakdown {
    pub fn from_seconds(total_seconds: i64) -> Self {
        let sign: i64 = if total_seconds < 0 { -1 } else { 1 };
        let abs = total_seconds.unsigned_abs();
        let days = (abs / 86_400) as i64;
        let hours = ((abs % 86_400) / 3_600) as i64;
        let minutes = ((abs % 3_600) / 60) as i64;
        let seconds = (abs % 60) as i64;
        let human = humanize(total_seconds, days, hours, minutes, seconds);
        Self {
            total_seconds,
            days: sign * days,
            hours: sign * hours,
            minutes: sign * minutes,
            seconds: sign * seconds,
            human,
        }
    }

    pub fn from_duration(d: Duration) -> Self {
        Self::from_seconds(d.num_seconds())
    }
}

fn humanize(total_seconds: i64, days: i64, hours: i64, minutes: i64, seconds: i64) -> String {
    if total_seconds == 0 {
        return "0s".to_string();
    }
    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{days}d"));
    }
    if hours > 0 {
        parts.push(format!("{hours}h"));
    }
    if minutes > 0 {
        parts.push(format!("{minutes}m"));
    }
    if seconds > 0 {
        parts.push(format!("{seconds}s"));
    }
    let joined = parts.join(" ");
    if total_seconds < 0 {
        format!("-{joined}")
    } else {
        joined
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_duration() {
        let b = DurationBreakdown::from_seconds(0);
        assert_eq!(b.total_seconds, 0);
        assert_eq!(b.days, 0);
        assert_eq!(b.human, "0s");
    }

    #[test]
    fn positive_breakdown() {
        // 1d 2h 3m 4s = 86400 + 7200 + 180 + 4 = 93784
        let b = DurationBreakdown::from_seconds(93_784);
        assert_eq!(b.days, 1);
        assert_eq!(b.hours, 2);
        assert_eq!(b.minutes, 3);
        assert_eq!(b.seconds, 4);
        assert_eq!(b.human, "1d 2h 3m 4s");
    }

    #[test]
    fn negative_breakdown_carries_sign() {
        let b = DurationBreakdown::from_seconds(-3_665);
        assert_eq!(b.total_seconds, -3_665);
        assert_eq!(b.hours, -1);
        assert_eq!(b.minutes, -1);
        assert_eq!(b.seconds, -5);
        assert_eq!(b.human, "-1h 1m 5s");
    }

    #[test]
    fn skips_zero_components() {
        let b = DurationBreakdown::from_seconds(3_600);
        assert_eq!(b.human, "1h");
    }
}
