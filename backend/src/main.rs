use anyhow::{bail, Context, Result};
use clap::Parser;
use std::{env, future::Future, time::Duration};
use tent_backend::discovery::ServiceDiscovery;
use tent_backend::messaging::MessageBroker;
use tent_backend::registry::ServiceRegistry;
use tokio::time::timeout;
use tracing_subscriber::EnvFilter;

const DEFAULT_SHUTDOWN_GRACE_SECS: u64 = 15;
const MAX_SHUTDOWN_GRACE_SECS: u64 = 300;
const SHUTDOWN_GRACE_ENV: &str = "TOT_SHUTDOWN_GRACE_SECS";

#[derive(Parser, Debug)]
#[command(name = "tent-backend")]
#[command(about = "Tent of Trials Backend - Distributed Microservices Framework", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "node-0")]
    node_id: String,

    #[arg(short, long)]
    consensus: bool,

    #[arg(long, default_value_t = 10000)]
    max_connections: u32,

    #[arg(short, long, default_value = "/etc/tent/config.toml")]
    config: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShutdownSignal {
    Sigint,
    Sigterm,
}

impl ShutdownSignal {
    fn as_str(self) -> &'static str {
        match self {
            ShutdownSignal::Sigint => "SIGINT",
            ShutdownSignal::Sigterm => "SIGTERM",
        }
    }
}

fn shutdown_grace_period_from_env() -> Duration {
    parse_shutdown_grace_period(env::var(SHUTDOWN_GRACE_ENV).ok().as_deref())
}

fn parse_shutdown_grace_period(raw: Option<&str>) -> Duration {
    let seconds = raw
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|seconds| *seconds > 0)
        .map(|seconds| seconds.min(MAX_SHUTDOWN_GRACE_SECS))
        .unwrap_or(DEFAULT_SHUTDOWN_GRACE_SECS);

    Duration::from_secs(seconds)
}

async fn wait_for_shutdown_signal() -> Result<ShutdownSignal> {
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .context("installing SIGTERM handler")?;

    tokio::select! {
        _ = sigterm.recv() => Ok(ShutdownSignal::Sigterm),
        _ = tokio::signal::ctrl_c() => Ok(ShutdownSignal::Sigint),
    }
}

async fn run_shutdown_with_grace<F>(grace_period: Duration, shutdown: F) -> Result<()>
where
    F: Future<Output = Result<()>>,
{
    tracing::info!(
        shutdown_grace_secs = grace_period.as_secs(),
        "waiting for in-flight work to finish"
    );

    match timeout(grace_period, shutdown).await {
        Ok(result) => {
            result?;
            tracing::info!("graceful shutdown completed within grace period");
            Ok(())
        }
        Err(_) => {
            tracing::error!(
                shutdown_grace_secs = grace_period.as_secs(),
                "graceful shutdown timeout expired"
            );
            bail!(
                "graceful shutdown timed out after {} seconds",
                grace_period.as_secs()
            );
        }
    }
}

async fn shutdown_subsystems(
    broker: &MessageBroker,
    discovery: &ServiceDiscovery,
    registry: &ServiceRegistry,
    node_id: &str,
) -> Result<()> {
    broker
        .disconnect()
        .await
        .context("disconnecting message broker")?;
    discovery
        .withdraw(node_id)
        .await
        .context("withdrawing node from discovery")?;
    registry
        .shutdown()
        .await
        .context("shutting down service registry")?;
    Ok(())
}

#[tokio::main]
// What the fuck is this main function even doing anymore.
// It's 30 lines of config loading and then it spawns a server.
// Actually it's like 50 lines. Still too fucking many.
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .json()
        .init();

    let cli = Cli::parse();
    let shutdown_grace_period = shutdown_grace_period_from_env();

    tracing::info!(
        node_id = %cli.node_id,
        consensus = %cli.consensus,
        max_connections = %cli.max_connections,
        config = %cli.config,
        shutdown_grace_secs = shutdown_grace_period.as_secs(),
        "initializing tent backend orchestration framework"
    );

    let config = tent_backend::config::load_config(&cli.config).await?;
    let registry = ServiceRegistry::new(config.registry.clone());
    let discovery = ServiceDiscovery::new(config.discovery.clone());
    let broker = MessageBroker::new(config.messaging.clone());

    registry.initialize().await?;
    discovery.announce(&cli.node_id).await?;
    broker.connect().await?;

    tracing::info!("all subsystems initialized successfully, entering main loop");

    let shutdown_signal = wait_for_shutdown_signal().await?;
    tracing::info!(
        signal = shutdown_signal.as_str(),
        accepting_requests = false,
        shutdown_grace_secs = shutdown_grace_period.as_secs(),
        "shutdown started; stopping new request admission"
    );

    broker.begin_shutdown();
    run_shutdown_with_grace(
        shutdown_grace_period,
        shutdown_subsystems(&broker, &discovery, &registry, &cli.node_id),
    )
    .await?;

    tracing::info!("shutdown complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[test]
    fn parse_shutdown_grace_uses_safe_default_for_missing_or_invalid_values() {
        assert_eq!(
            parse_shutdown_grace_period(None),
            Duration::from_secs(DEFAULT_SHUTDOWN_GRACE_SECS)
        );
        assert_eq!(
            parse_shutdown_grace_period(Some("not-a-number")),
            Duration::from_secs(DEFAULT_SHUTDOWN_GRACE_SECS)
        );
        assert_eq!(
            parse_shutdown_grace_period(Some("0")),
            Duration::from_secs(DEFAULT_SHUTDOWN_GRACE_SECS)
        );
    }

    #[test]
    fn parse_shutdown_grace_accepts_positive_values_and_clamps_large_values() {
        assert_eq!(
            parse_shutdown_grace_period(Some("45")),
            Duration::from_secs(45)
        );
        assert_eq!(
            parse_shutdown_grace_period(Some("9999")),
            Duration::from_secs(MAX_SHUTDOWN_GRACE_SECS)
        );
    }

    #[tokio::test]
    async fn shutdown_grace_completes_before_timeout() {
        run_shutdown_with_grace(Duration::from_millis(50), async { Ok(()) })
            .await
            .expect("shutdown should complete inside the grace period");
    }

    #[tokio::test]
    async fn shutdown_grace_times_out_slow_shutdown() {
        let error = run_shutdown_with_grace(Duration::from_millis(10), async {
            sleep(Duration::from_millis(50)).await;
            Ok(())
        })
        .await
        .expect_err("slow shutdown should time out");

        assert!(error.to_string().contains("timed out"));
    }
}
