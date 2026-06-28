use serde::Serialize;
use std::time::{Duration, Instant};

pub const DEFAULT_SHUTDOWN_GRACE_PERIOD: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShutdownTerminalStatus {
    NotStarted,
    Draining,
    Completed,
    TimedOut,
}

impl ShutdownTerminalStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            ShutdownTerminalStatus::NotStarted => "not_started",
            ShutdownTerminalStatus::Draining => "draining",
            ShutdownTerminalStatus::Completed => "completed",
            ShutdownTerminalStatus::TimedOut => "timed_out",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ShutdownMetricsSnapshot {
    pub shutdown_started: bool,
    pub grace_period_seconds: u64,
    pub elapsed_seconds: u64,
    pub terminal_status: ShutdownTerminalStatus,
}

#[derive(Debug, Clone)]
pub struct ShutdownMetrics {
    grace_period: Duration,
    started_at: Option<Instant>,
    completed_at: Option<Instant>,
    timed_out_at: Option<Instant>,
}

impl ShutdownMetrics {
    pub fn new(grace_period: Duration) -> Self {
        Self {
            grace_period,
            started_at: None,
            completed_at: None,
            timed_out_at: None,
        }
    }

    pub fn mark_started(&mut self) {
        self.mark_started_at(Instant::now());
    }

    pub fn mark_completed(&mut self) {
        self.mark_completed_at(Instant::now());
    }

    pub fn mark_timed_out(&mut self) {
        self.mark_timed_out_at(Instant::now());
    }

    pub fn snapshot(&self) -> ShutdownMetricsSnapshot {
        self.snapshot_at(Instant::now())
    }

    fn mark_started_at(&mut self, now: Instant) {
        self.started_at = Some(now);
        self.completed_at = None;
        self.timed_out_at = None;
    }

    fn mark_completed_at(&mut self, now: Instant) {
        if self.started_at.is_none() {
            self.started_at = Some(now);
        }
        self.completed_at = Some(now);
        self.timed_out_at = None;
    }

    fn mark_timed_out_at(&mut self, now: Instant) {
        if self.started_at.is_none() {
            self.started_at = Some(now);
        }
        self.timed_out_at = Some(now);
        self.completed_at = None;
    }

    fn snapshot_at(&self, now: Instant) -> ShutdownMetricsSnapshot {
        let Some(started_at) = self.started_at else {
            return ShutdownMetricsSnapshot {
                shutdown_started: false,
                grace_period_seconds: self.grace_period.as_secs(),
                elapsed_seconds: 0,
                terminal_status: ShutdownTerminalStatus::NotStarted,
            };
        };

        let elapsed = self.elapsed_since(started_at, now);
        let terminal_status = if self.completed_at.is_some() {
            ShutdownTerminalStatus::Completed
        } else if self.timed_out_at.is_some() || elapsed >= self.grace_period {
            ShutdownTerminalStatus::TimedOut
        } else {
            ShutdownTerminalStatus::Draining
        };

        ShutdownMetricsSnapshot {
            shutdown_started: true,
            grace_period_seconds: self.grace_period.as_secs(),
            elapsed_seconds: elapsed.as_secs(),
            terminal_status,
        }
    }

    fn elapsed_since(&self, started_at: Instant, now: Instant) -> Duration {
        self.completed_at
            .or(self.timed_out_at)
            .unwrap_or(now)
            .saturating_duration_since(started_at)
    }
}

#[cfg(test)]
mod shutdown_metrics_tests {
    use super::*;

    #[test]
    fn snapshot_reports_pre_shutdown_state() {
        let now = Instant::now();
        let metrics = ShutdownMetrics::new(Duration::from_secs(30));

        assert_eq!(
            metrics.snapshot_at(now),
            ShutdownMetricsSnapshot {
                shutdown_started: false,
                grace_period_seconds: 30,
                elapsed_seconds: 0,
                terminal_status: ShutdownTerminalStatus::NotStarted,
            }
        );
    }

    #[test]
    fn snapshot_reports_draining_state() {
        let start = Instant::now();
        let mut metrics = ShutdownMetrics::new(Duration::from_secs(30));
        metrics.mark_started_at(start);

        assert_eq!(
            metrics.snapshot_at(start + Duration::from_secs(8)),
            ShutdownMetricsSnapshot {
                shutdown_started: true,
                grace_period_seconds: 30,
                elapsed_seconds: 8,
                terminal_status: ShutdownTerminalStatus::Draining,
            }
        );
    }

    #[test]
    fn snapshot_reports_completed_state() {
        let start = Instant::now();
        let mut metrics = ShutdownMetrics::new(Duration::from_secs(30));
        metrics.mark_started_at(start);
        metrics.mark_completed_at(start + Duration::from_secs(12));

        assert_eq!(
            metrics.snapshot_at(start + Duration::from_secs(45)),
            ShutdownMetricsSnapshot {
                shutdown_started: true,
                grace_period_seconds: 30,
                elapsed_seconds: 12,
                terminal_status: ShutdownTerminalStatus::Completed,
            }
        );
    }

    #[test]
    fn snapshot_reports_explicit_timeout_state() {
        let start = Instant::now();
        let mut metrics = ShutdownMetrics::new(Duration::from_secs(30));
        metrics.mark_started_at(start);
        metrics.mark_timed_out_at(start + Duration::from_secs(31));

        assert_eq!(
            metrics.snapshot_at(start + Duration::from_secs(60)),
            ShutdownMetricsSnapshot {
                shutdown_started: true,
                grace_period_seconds: 30,
                elapsed_seconds: 31,
                terminal_status: ShutdownTerminalStatus::TimedOut,
            }
        );
    }

    #[test]
    fn snapshot_reports_implicit_timeout_state_after_grace_period() {
        let start = Instant::now();
        let mut metrics = ShutdownMetrics::new(Duration::from_secs(30));
        metrics.mark_started_at(start);

        assert_eq!(
            metrics.snapshot_at(start + Duration::from_secs(30)),
            ShutdownMetricsSnapshot {
                shutdown_started: true,
                grace_period_seconds: 30,
                elapsed_seconds: 30,
                terminal_status: ShutdownTerminalStatus::TimedOut,
            }
        );
    }
}
