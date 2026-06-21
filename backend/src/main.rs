use anyhow::Result;
use clap::Parser;
use tent_backend::config::load_config;
use tent_backend::discovery::ServiceDiscovery;
use tent_backend::messaging::MessageBroker;
use tent_backend::registry::ServiceRegistry;
use tracing_subscriber::EnvFilter;

const ENV_SHUTDOWN_GRACE_SECS: &str = "TOT_SHUTDOWN_GRACE_SECS";

fn parse_shutdown_grace_secs() -> u64 {
    // Default 30 seconds
    match std::env::var(ENV_SHUTDOWN_GRACE_SECS) {
        Ok(v) => match v.parse::<u64>() {
            Ok(n) if n > 0 && n <= 300 => n,
            _ => 30, // invalid values use default
        },
        Err(_) => 30, // unset uses default
    }
}

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

    let grace_secs = parse_shutdown_grace_secs();
    let shutdown_state = tent_backend::shutdown::ShutdownState::new(grace_secs);
    shutdown_state.begin();

    tracing::info!(
        node_id = %cli.node_id,
        consensus = %cli.consensus,
        max_connections = %cli.max_connections,
        config = %cli.config,
        grace_secs = %grace_secs,
        shutdown_state = ?shutdown_state.snapshot(),
        "initializing tent backend orchestration framework"
    );

    let config = load_config(&cli.config).await?;
    let registry = ServiceRegistry::new(config.registry.clone());
    let discovery = ServiceDiscovery::new(config.discovery.clone());
    let broker = MessageBroker::new(config.messaging.clone());

    registry.initialize().await?;
    discovery.announce(&cli.node_id).await?;
    broker.connect().await?;

    tracing::info!("all subsystems initialized successfully, entering main loop");

    let mut signal = tokio::signal::unix::signal(
        tokio::signal::unix::SignalKind::terminate(),
    )?;

    tokio::select! {
        _ = signal.recv() => {
            tracing::info!("received SIGTERM, initiating graceful shutdown");
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("received SIGINT, initiating graceful shutdown");
        }
    }

    // Graceful drain with timeout
    tokio::select! {
        _ = broker.disconnect() => {
            shutdown_state.set_drain_completed();
            tracing::info!("broker disconnected gracefully");
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(grace_secs)) => {
            shutdown_state.set_drain_timed_out();
            tracing::warn!("grace period expired, forcing shutdown");
        }
    }

    discovery.withdraw(&cli.node_id).await?;
    registry.shutdown().await?;

    tracing::info!(shutdown_state = ?shutdown_state.snapshot(), "shutdown complete");
    Ok(())
}
