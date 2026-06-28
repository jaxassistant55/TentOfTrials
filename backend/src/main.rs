use anyhow::Result;
use clap::Parser;
use tent_backend::discovery::ServiceDiscovery;
use tent_backend::messaging::MessageBroker;
use tent_backend::registry::ServiceRegistry;
use tent_backend::shutdown::ShutdownMetrics;
use tracing_subscriber::EnvFilter;

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

    #[arg(long, default_value = "/etc/tent/config.toml")]
    config: String,
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

    tracing::info!(
        node_id = %cli.node_id,
        consensus = %cli.consensus,
        max_connections = %cli.max_connections,
        config = %cli.config,
        "initializing tent backend orchestration framework"
    );

    let config = tent_backend::config::load_config(&cli.config).await?;
    let registry = ServiceRegistry::new(config.registry.clone());
    let discovery = ServiceDiscovery::new(config.discovery.clone());
    let broker = MessageBroker::new(config.messaging.clone());
    let shutdown_grace = tent_backend::config::shutdown_grace_duration_from_env()?;
    let mut shutdown_metrics = ShutdownMetrics::new(shutdown_grace);

    registry.initialize().await?;
    discovery.announce(&cli.node_id).await?;
    broker.connect().await?;

    tracing::info!("all subsystems initialized successfully, entering main loop");

    let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;

    tokio::select! {
        _ = signal.recv() => {
            tracing::info!(
                accepting_new_work = false,
                "received SIGTERM, initiating graceful shutdown"
            );
            shutdown_metrics.mark_started();
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!(
                accepting_new_work = false,
                "received SIGINT, initiating graceful shutdown"
            );
            shutdown_metrics.mark_started();
        }
    }

    let snapshot = shutdown_metrics.snapshot();
    tracing::info!(
        shutdown_started = snapshot.shutdown_started,
        grace_period_seconds = snapshot.grace_period_seconds,
        elapsed_seconds = snapshot.elapsed_seconds,
        terminal_status = snapshot.terminal_status.as_str(),
        "shutdown metrics snapshot"
    );

    let shutdown_result = tokio::time::timeout(shutdown_grace, async {
        broker.disconnect().await?;
        discovery.withdraw(&cli.node_id).await?;
        registry.shutdown().await
    })
    .await;

    match shutdown_result {
        Ok(result) => result?,
        Err(_) => {
            shutdown_metrics.mark_timed_out();
            let snapshot = shutdown_metrics.snapshot();
            tracing::error!(
                shutdown_started = snapshot.shutdown_started,
                grace_period_seconds = snapshot.grace_period_seconds,
                elapsed_seconds = snapshot.elapsed_seconds,
                terminal_status = snapshot.terminal_status.as_str(),
                "shutdown metrics snapshot"
            );
            return Err(anyhow::anyhow!(
                "shutdown exceeded {} second grace period",
                shutdown_grace.as_secs()
            ));
        }
    }

    shutdown_metrics.mark_completed();
    let snapshot = shutdown_metrics.snapshot();
    tracing::info!(
        shutdown_started = snapshot.shutdown_started,
        grace_period_seconds = snapshot.grace_period_seconds,
        elapsed_seconds = snapshot.elapsed_seconds,
        terminal_status = snapshot.terminal_status.as_str(),
        "shutdown metrics snapshot"
    );

    tracing::info!("shutdown complete");
    Ok(())
}
