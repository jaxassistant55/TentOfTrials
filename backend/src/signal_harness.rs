use std::sync::{Arc, Mutex};
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Signal that triggered the shutdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShutdownSignal {
    Sigterm,
    Sigint,
}

/// Outcome of a shutdown drain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShutdownOutcome {
    /// Shutdown completed within the grace period.
    Completed,
    /// Grace period expired before shutdown finished.
    TimedOut,
}

/// State of the shutdown harness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessState {
    Idle,
    Notified,
    Draining,
    Finished(ShutdownOutcome),
}

/// Integration harness for exercising the signal-based shutdown path.
pub struct SignalShutdownHarness {
    grace_period: Duration,
    state: Arc<Mutex<HarnessState>>,
    signal: Arc<Mutex<Option<ShutdownSignal>>>,
}

impl SignalShutdownHarness {
    pub fn new(grace_period: Duration) -> Self {
        Self {
            grace_period,
            state: Arc::new(Mutex::new(HarnessState::Idle)),
            signal: Arc::new(Mutex::new(None)),
        }
    }

    /// Notify the harness that a shutdown signal was received.
    pub fn notify(&self, sig: ShutdownSignal) {
        *self.signal.lock().unwrap() = Some(sig);
        *self.state.lock().unwrap() = HarnessState::Notified;
    }

    /// Begin the drain phase.
    pub fn begin_drain(&self) {
        *self.state.lock().unwrap() = HarnessState::Draining;
    }

    /// Mark shutdown as completed.
    pub fn complete(&self) {
        *self.state.lock().unwrap() = HarnessState::Finished(ShutdownOutcome::Completed);
    }

    /// Simulate a grace period timeout.
    pub fn timeout(&self) {
        *self.state.lock().unwrap() = HarnessState::Finished(ShutdownOutcome::TimedOut);
    }

    /// Get the current state.
    pub fn state(&self) -> HarnessState {
        *self.state.lock().unwrap()
    }

    /// Get the signal that triggered shutdown, if any.
    pub fn trigger_signal(&self) -> Option<ShutdownSignal> {
        *self.signal.lock().unwrap()
    }

    /// Get the configured grace period.
    pub fn grace_period(&self) -> Duration {
        self.grace_period
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sigterm_notification() {
        let harness = SignalShutdownHarness::new(Duration::from_secs(30));
        harness.notify(ShutdownSignal::Sigterm);
        assert_eq!(harness.trigger_signal(), Some(ShutdownSignal::Sigterm));
        assert_eq!(harness.state(), HarnessState::Notified);
    }

    #[test]
    fn test_sigint_notification() {
        let harness = SignalShutdownHarness::new(Duration::from_secs(30));
        harness.notify(ShutdownSignal::Sigint);
        assert_eq!(harness.trigger_signal(), Some(ShutdownSignal::Sigint));
        assert_eq!(harness.state(), HarnessState::Notified);
    }

    #[test]
    fn test_grace_period_expiration() {
        let harness = SignalShutdownHarness::new(Duration::from_secs(5));
        harness.notify(ShutdownSignal::Sigterm);
        harness.begin_drain();
        harness.timeout();
        match harness.state() {
            HarnessState::Finished(ShutdownOutcome::TimedOut) => {}
            other => panic!("expected TimedOut, got {:?}", other),
        }
    }

    #[test]
    fn test_successful_completion() {
        let harness = SignalShutdownHarness::new(Duration::from_secs(30));
        harness.notify(ShutdownSignal::Sigterm);
        harness.begin_drain();
        harness.complete();
        match harness.state() {
            HarnessState::Finished(ShutdownOutcome::Completed) => {}
            other => panic!("expected Completed, got {:?}", other),
        }
    }
}
