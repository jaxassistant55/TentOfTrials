//! Graceful shutdown grace-period configuration.
//!
//! The backend supports a configurable shutdown grace period via the
//! `TOT_SHUTDOWN_GRACE_SECS` environment variable. This module parses,
//! validates, and applies bounds to that setting so deploys use it
//! consistently.

use std::env;
use std::time::Duration;

/// Environment variable name for the shutdown grace period.
pub const ENV_SHUTDOWN_GRACE_SECS: &str = "TOT_SHUTDOWN_GRACE_SECS";

/// Default grace period in seconds when the variable is unset.
pub const DEFAULT_GRACE_SECS: u64 = 30;

/// Maximum allowed grace period in seconds (5 minutes).
pub const MAX_GRACE_SECS: u64 = 300;

/// Minimum allowed grace period in seconds.
pub const MIN_GRACE_SECS: u64 = 1;

/// Parsed grace-period result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GracePeriod {
    /// The resolved duration.
    pub duration: Duration,
    /// How the value was determined.
    pub source: GraceSource,
}

/// How the grace period was determined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraceSource {
    /// No env var set; the default was used.
    Default,
    /// The env var contained a valid value within bounds.
    Env(u64),
    /// The env var was zero or below minimum; clamped to the minimum.
    ClampedLow { raw: i64, clamped_to: u64 },
    /// The env var exceeded the maximum; clamped to the maximum.
    ClampedHigh { raw: u64, clamped_to: u64 },
    /// The env var was not a valid integer; the default was used instead.
    Invalid { raw: String, fallback: u64 },
}

/// Parse the `TOT_SHUTDOWN_GRACE_SECS` environment variable and return a
/// validated [`GracePeriod`].
///
/// | Input                       | Behaviour                                   |
/// |-----------------------------|---------------------------------------------|
/// | Unset                       | Use `DEFAULT_GRACE_SECS`                    |
/// | Valid integer in range      | Use the parsed value                        |
/// | Zero                        | Clamp to `MIN_GRACE_SECS`                  |
/// | Negative                    | Clamp to `MIN_GRACE_SECS`                  |
/// | Above `MAX_GRACE_SECS`     | Clamp to `MAX_GRACE_SECS`                  |
/// | Non-numeric                 | Fall back to `DEFAULT_GRACE_SECS`           |
pub fn parse_grace_period() -> GracePeriod {
    parse_grace_period_from(env::var(ENV_SHUTDOWN_GRACE_SECS).ok())
}

/// Testable entry-point that accepts an optional raw string instead of
/// reading the real environment.
pub fn parse_grace_period_from(raw: Option<String>) -> GracePeriod {
    match raw {
        None => GracePeriod {
            duration: Duration::from_secs(DEFAULT_GRACE_SECS),
            source: GraceSource::Default,
        },
        Some(s) => {
            let trimmed = s.trim();
            // Try parsing as i64 first so we can detect negatives.
            if let Ok(val) = trimmed.parse::<i64>() {
                if val <= 0 {
                    GracePeriod {
                        duration: Duration::from_secs(MIN_GRACE_SECS),
                        source: GraceSource::ClampedLow {
                            raw: val,
                            clamped_to: MIN_GRACE_SECS,
                        },
                    }
                } else if (val as u64) > MAX_GRACE_SECS {
                    GracePeriod {
                        duration: Duration::from_secs(MAX_GRACE_SECS),
                        source: GraceSource::ClampedHigh {
                            raw: val as u64,
                            clamped_to: MAX_GRACE_SECS,
                        },
                    }
                } else {
                    GracePeriod {
                        duration: Duration::from_secs(val as u64),
                        source: GraceSource::Env(val as u64),
                    }
                }
            } else {
                GracePeriod {
                    duration: Duration::from_secs(DEFAULT_GRACE_SECS),
                    source: GraceSource::Invalid {
                        raw: trimmed.to_string(),
                        fallback: DEFAULT_GRACE_SECS,
                    },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unset_uses_default() {
        let gp = parse_grace_period_from(None);
        assert_eq!(gp.duration, Duration::from_secs(DEFAULT_GRACE_SECS));
        assert_eq!(gp.source, GraceSource::Default);
    }

    #[test]
    fn test_valid_value_in_range() {
        let gp = parse_grace_period_from(Some("60".into()));
        assert_eq!(gp.duration, Duration::from_secs(60));
        assert_eq!(gp.source, GraceSource::Env(60));
    }

    #[test]
    fn test_valid_value_at_minimum() {
        let gp = parse_grace_period_from(Some("1".into()));
        assert_eq!(gp.duration, Duration::from_secs(1));
        assert_eq!(gp.source, GraceSource::Env(1));
    }

    #[test]
    fn test_valid_value_at_maximum() {
        let gp = parse_grace_period_from(Some("300".into()));
        assert_eq!(gp.duration, Duration::from_secs(300));
        assert_eq!(gp.source, GraceSource::Env(300));
    }

    #[test]
    fn test_zero_clamped_to_minimum() {
        let gp = parse_grace_period_from(Some("0".into()));
        assert_eq!(gp.duration, Duration::from_secs(MIN_GRACE_SECS));
        assert!(matches!(
            gp.source,
            GraceSource::ClampedLow {
                raw: 0,
                clamped_to: MIN_GRACE_SECS
            }
        ));
    }

    #[test]
    fn test_negative_clamped_to_minimum() {
        let gp = parse_grace_period_from(Some("-5".into()));
        assert_eq!(gp.duration, Duration::from_secs(MIN_GRACE_SECS));
        assert!(matches!(
            gp.source,
            GraceSource::ClampedLow {
                raw: -5,
                clamped_to: MIN_GRACE_SECS
            }
        ));
    }

    #[test]
    fn test_too_large_clamped_to_maximum() {
        let gp = parse_grace_period_from(Some("9999".into()));
        assert_eq!(gp.duration, Duration::from_secs(MAX_GRACE_SECS));
        assert!(matches!(
            gp.source,
            GraceSource::ClampedHigh {
                raw: 9999,
                clamped_to: MAX_GRACE_SECS
            }
        ));
    }

    #[test]
    fn test_non_numeric_falls_back_to_default() {
        let gp = parse_grace_period_from(Some("abc".into()));
        assert_eq!(gp.duration, Duration::from_secs(DEFAULT_GRACE_SECS));
        assert!(matches!(
            gp.source,
            GraceSource::Invalid {
                raw,
                fallback: DEFAULT_GRACE_SECS
            } if raw == "abc"
        ));
    }

    #[test]
    fn test_whitespace_trimmed() {
        let gp = parse_grace_period_from(Some("  45  ".into()));
        assert_eq!(gp.duration, Duration::from_secs(45));
        assert_eq!(gp.source, GraceSource::Env(45));
    }

    #[test]
    fn test_float_string_falls_back_to_default() {
        let gp = parse_grace_period_from(Some("30.5".into()));
        assert_eq!(gp.duration, Duration::from_secs(DEFAULT_GRACE_SECS));
        assert!(matches!(
            gp.source,
            GraceSource::Invalid {
                raw,
                fallback: DEFAULT_GRACE_SECS
            } if raw == "30.5"
        ));
    }
}
