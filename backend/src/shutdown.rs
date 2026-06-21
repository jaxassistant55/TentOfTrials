use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ShutdownMetrics {
    pub started: bool,
    pub grace_period_secs: u64,
    pub elapsed_secs: f64,
    pub drain_completed: bool,
    pub drain_timed_out: bool,
}

#[derive(Debug)]
pub struct ShutdownState {
    started: AtomicBool,
    start_time: std::sync::Mutex<Option<Instant>>,
    grace_period_secs: AtomicU64,
    drain_completed: AtomicBool,
    drain_timed_out: AtomicBool,
}

impl ShutdownState {
    pub fn new(grace_period_secs: u64) -> Arc<Self> {
        Arc::new(Self {
            started: AtomicBool::new(false),
            start_time: std::sync::Mutex::new(None),
            grace_period_secs: AtomicU64::new(grace_period_secs),
            drain_completed: AtomicBool::new(false),
            drain_timed_out: AtomicBool::new(false),
        })
    }

    pub fn begin(&self) {
        if !self.started.swap(true, Ordering::SeqCst) {
            let mut guard = self.start_time.lock().unwrap();
            *guard = Some(Instant::now());
        }
    }

    pub fn set_drain_completed(&self) {
        self.drain_completed.store(true, Ordering::SeqCst);
    }

    pub fn set_drain_timed_out(&self) {
        self.drain_timed_out.store(true, Ordering::SeqCst);
    }

    pub fn snapshot(&self) -> ShutdownMetrics {
        let started = self.started.load(Ordering::SeqCst);
        let grace_period = self.grace_period_secs.load(Ordering::SeqCst);
        let elapsed = {
            let guard = self.start_time.lock().unwrap();
            guard.map(|t| t.elapsed().as_secs_f64()).unwrap_or(0.0)
        };
        let drain_completed = self.drain_completed.load(Ordering::SeqCst);
        let drain_timed_out = self.drain_timed_out.load(Ordering::SeqCst);

        ShutdownMetrics {
            started,
            grace_period_secs: grace_period,
            elapsed_secs: elapsed,
            drain_completed,
            drain_timed_out,
        }
    }
}
