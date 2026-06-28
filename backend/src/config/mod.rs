use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

pub const SHUTDOWN_GRACE_ENV: &str = "TOT_SHUTDOWN_GRACE_SECS";
pub const DEFAULT_SHUTDOWN_GRACE_SECS: u64 = 30;
pub const MAX_SHUTDOWN_GRACE_SECS: u64 = 300;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub version: String,
    pub host: String,
    pub port: u16,
    pub tls_enabled: bool,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub backend: String,
    pub endpoints: Vec<String>,
    pub heartbeat_interval_ms: u64,
    pub ttl_seconds: u64,
    pub replication_factor: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    pub provider: String,
    pub namespace: String,
    pub tags: Vec<String>,
    pub health_check_path: String,
    pub health_check_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagingConfig {
    pub broker_type: String,
    pub uris: Vec<String>,
    pub consumer_group: String,
    pub max_retries: u32,
    pub retry_backoff_ms: u64,
    pub batch_size: u32,
    pub compression: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootConfig {
    pub service: ServiceConfig,
    pub registry: RegistryConfig,
    pub discovery: DiscoveryConfig,
    pub messaging: MessagingConfig,
}

impl Default for RootConfig {
    fn default() -> Self {
        Self {
            service: ServiceConfig {
                name: "tent-backend".into(),
                version: "0.1.0".into(),
                host: "0.0.0.0".into(),
                port: 8080,
                tls_enabled: false,
                tls_cert_path: None,
                tls_key_path: None,
            },
            registry: RegistryConfig {
                backend: "etcd".into(),
                endpoints: vec!["localhost:2379".into()],
                heartbeat_interval_ms: 5000,
                ttl_seconds: 30,
                replication_factor: 3,
            },
            discovery: DiscoveryConfig {
                provider: "consul".into(),
                namespace: "tent".into(),
                tags: vec!["microservice".into(), "orchestration".into()],
                health_check_path: "/health".into(),
                health_check_interval_ms: 10000,
            },
            messaging: MessagingConfig {
                broker_type: "kafka".into(),
                uris: vec!["localhost:9092".into()],
                consumer_group: "tent-consumers".into(),
                max_retries: 3,
                retry_backoff_ms: 1000,
                batch_size: 500,
                compression: "snappy".into(),
            },
        }
    }
}

pub async fn load_config(path: &str) -> Result<RootConfig> {
    let path = Path::new(path);
    if path.exists() {
        let contents = tokio::fs::read_to_string(path).await?;
        let config: RootConfig = toml::from_str(&contents)?;
        tracing::info!("configuration loaded from {}", path.display());
        Ok(config)
    } else {
        tracing::warn!("config file {} not found, using defaults", path.display());
        Ok(RootConfig::default())
    }
}

pub fn parse_shutdown_grace_secs(raw: Option<&str>) -> std::result::Result<u64, String> {
    let Some(raw) = raw else {
        return Ok(DEFAULT_SHUTDOWN_GRACE_SECS);
    };

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(DEFAULT_SHUTDOWN_GRACE_SECS);
    }
    if trimmed.starts_with('-') {
        return Err(format!(
            "{SHUTDOWN_GRACE_ENV} must be between 1 and {MAX_SHUTDOWN_GRACE_SECS} seconds"
        ));
    }

    let value: u64 = trimmed
        .parse()
        .map_err(|_| format!("{SHUTDOWN_GRACE_ENV} must be a whole number of seconds"))?;

    if value == 0 || value > MAX_SHUTDOWN_GRACE_SECS {
        return Err(format!(
            "{SHUTDOWN_GRACE_ENV} must be between 1 and {MAX_SHUTDOWN_GRACE_SECS} seconds"
        ));
    }

    Ok(value)
}

pub fn shutdown_grace_duration_from_env() -> Result<Duration> {
    let raw = std::env::var(SHUTDOWN_GRACE_ENV).ok();
    let seconds = parse_shutdown_grace_secs(raw.as_deref()).map_err(anyhow::Error::msg)?;
    Ok(Duration::from_secs(seconds))
}

#[cfg(test)]
mod tests {
    use super::{parse_shutdown_grace_secs, DEFAULT_SHUTDOWN_GRACE_SECS, MAX_SHUTDOWN_GRACE_SECS};

    #[test]
    fn shutdown_grace_unset_uses_default() {
        assert_eq!(
            DEFAULT_SHUTDOWN_GRACE_SECS,
            parse_shutdown_grace_secs(None).unwrap()
        );
    }

    #[test]
    fn shutdown_grace_accepts_valid_value() {
        assert_eq!(45, parse_shutdown_grace_secs(Some("45")).unwrap());
    }

    #[test]
    fn shutdown_grace_rejects_zero() {
        assert!(parse_shutdown_grace_secs(Some("0")).is_err());
    }

    #[test]
    fn shutdown_grace_rejects_negative_value() {
        assert!(parse_shutdown_grace_secs(Some("-5")).is_err());
    }

    #[test]
    fn shutdown_grace_rejects_too_large_value() {
        assert!(
            parse_shutdown_grace_secs(Some(&(MAX_SHUTDOWN_GRACE_SECS + 1).to_string())).is_err()
        );
    }

    #[test]
    fn shutdown_grace_rejects_non_numeric_value() {
        assert!(parse_shutdown_grace_secs(Some("soon")).is_err());
    }
}
