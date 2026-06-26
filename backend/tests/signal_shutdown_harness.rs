#![cfg(unix)]

use std::fs::{self, File};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const INIT_LOG: &str = "all subsystems initialized successfully, entering main loop";
const SHUTDOWN_START_LOG: &str = "shutdown signal received, initiating graceful shutdown";
const SHUTDOWN_GATE_LOG: &str = "shutdown gate closed to new work";
const SHUTDOWN_COMPLETE_LOG: &str = "shutdown complete";

#[test]
fn sigint_drives_graceful_shutdown_lifecycle() {
    let binary = env!("CARGO_BIN_EXE_tent-backend");
    let log_path = std::env::temp_dir().join(format!(
        "tent-backend-signal-harness-{}.log",
        std::process::id()
    ));
    let log_file = File::create(&log_path).expect("create backend harness log");
    let stderr_file = log_file.try_clone().expect("clone harness log handle");

    let mut child = Command::new(binary)
        .arg("--node-id")
        .arg("signal-harness-node")
        .arg("--config")
        .arg("/tmp/tent-signal-harness-missing.toml")
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .expect("spawn tent-backend");

    wait_for_log(&mut child, &log_path, INIT_LOG, Duration::from_secs(10));

    let status = Command::new("kill")
        .arg("-INT")
        .arg(child.id().to_string())
        .status()
        .expect("send SIGINT to tent-backend");
    assert!(status.success(), "kill -INT failed with status {status}");

    wait_for_log(
        &mut child,
        &log_path,
        SHUTDOWN_START_LOG,
        Duration::from_secs(10),
    );
    wait_for_log(
        &mut child,
        &log_path,
        SHUTDOWN_GATE_LOG,
        Duration::from_secs(10),
    );
    wait_for_log(
        &mut child,
        &log_path,
        SHUTDOWN_COMPLETE_LOG,
        Duration::from_secs(10),
    );

    let exit_status = wait_for_exit(&mut child, Duration::from_secs(10));
    assert!(
        exit_status.success(),
        "backend exited unsuccessfully after graceful shutdown: {exit_status}"
    );

    let _ = fs::remove_file(log_path);
}

fn wait_for_log(child: &mut Child, log_path: &std::path::Path, needle: &str, timeout: Duration) {
    let deadline = Instant::now() + timeout;

    loop {
        if let Ok(contents) = fs::read_to_string(log_path) {
            if contents.contains(needle) {
                return;
            }
        }

        if let Some(status) = child.try_wait().expect("poll backend process") {
            let contents = fs::read_to_string(log_path).unwrap_or_default();
            panic!(
                "backend exited before expected log {needle:?}; status={status}; logs:\n{contents}"
            );
        }

        if Instant::now() >= deadline {
            let contents = fs::read_to_string(log_path).unwrap_or_default();
            let _ = child.kill();
            panic!("timed out waiting for log {needle:?}; logs:\n{contents}");
        }

        thread::sleep(Duration::from_millis(100));
    }
}

fn wait_for_exit(child: &mut Child, timeout: Duration) -> std::process::ExitStatus {
    let deadline = Instant::now() + timeout;

    loop {
        if let Some(status) = child.try_wait().expect("poll backend process") {
            return status;
        }

        if Instant::now() >= deadline {
            let _ = child.kill();
            panic!("backend did not exit before timeout after shutdown completion");
        }

        thread::sleep(Duration::from_millis(100));
    }
}
