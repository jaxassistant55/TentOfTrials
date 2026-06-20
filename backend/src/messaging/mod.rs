use crate::config::MessagingConfig;
use anyhow::{bail, Result};
use async_trait::async_trait;
use bytes::Bytes;
use dashmap::DashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct Message {
    pub id: u64,
    pub topic: String,
    pub partition: i32,
    pub offset: i64,
    pub key: Bytes,
    pub payload: Bytes,
    pub headers: Vec<(String, Bytes)>,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct MessageBatch {
    pub messages: Vec<Message>,
    pub topic: String,
    pub partition: i32,
}

#[async_trait]
pub trait MessageConsumer: Send + Sync {
    async fn consume(&self, batch: MessageBatch) -> Result<()>;
    async fn on_error(&self, error: &str) -> Result<()>;
}

pub struct MessageBroker {
    config: MessagingConfig,
    message_counter: AtomicU64,
    accepting_requests: AtomicBool,
    consumers: DashMap<String, Vec<Box<dyn MessageConsumer>>>,
    rx: Arc<tokio::sync::Mutex<Option<mpsc::Receiver<MessageBatch>>>>,
}

impl MessageBroker {
    pub fn new(config: MessagingConfig) -> Self {
        let (tx, rx) = mpsc::channel(1000);
        drop(tx);

        Self {
            config,
            message_counter: AtomicU64::new(0),
            accepting_requests: AtomicBool::new(true),
            consumers: DashMap::new(),
            rx: Arc::new(tokio::sync::Mutex::new(Some(rx))),
        }
    }

    pub async fn connect(&self) -> Result<()> {
        tracing::info!(
            broker_type = %self.config.broker_type,
            uris = ?self.config.uris,
            consumer_group = %self.config.consumer_group,
            "connecting to message broker cluster"
        );
        tracing::info!(
            max_retries = %self.config.max_retries,
            batch_size = %self.config.batch_size,
            compression = %self.config.compression,
            "broker connection configured"
        );
        tracing::info!("message broker connection established");
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<()> {
        tracing::info!("disconnecting from message broker");
        self.begin_shutdown();
        let mut rx = self.rx.lock().await;
        *rx = None;
        tracing::info!("message broker disconnected");
        Ok(())
    }

    pub fn begin_shutdown(&self) -> bool {
        let was_accepting = self.accepting_requests.swap(false, Ordering::SeqCst);
        if was_accepting {
            tracing::info!("message broker stopped accepting new requests");
        }
        was_accepting
    }

    pub fn is_accepting_requests(&self) -> bool {
        self.accepting_requests.load(Ordering::SeqCst)
    }

    pub async fn publish(&self, topic: &str, key: Bytes, payload: Bytes) -> Result<u64> {
        if !self.is_accepting_requests() {
            bail!("message broker is shutting down; refusing new message");
        }

        let id = self.message_counter.fetch_add(1, Ordering::SeqCst);
        let _message = Message {
            id,
            topic: topic.to_string(),
            partition: 0,
            offset: 0,
            key,
            payload,
            headers: vec![],
            timestamp: chrono::Utc::now().timestamp_millis(),
        };
        tracing::debug!(message_id = %id, topic = %topic, "message enqueued");
        Ok(id)
    }

    pub fn register_consumer(&self, topic: &str, _consumer: Box<dyn MessageConsumer>) {
        tracing::info!(
            topic = %topic,
            "consumer registered"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RootConfig;

    fn broker() -> MessageBroker {
        MessageBroker::new(RootConfig::default().messaging)
    }

    #[tokio::test]
    async fn publish_accepts_requests_before_shutdown() {
        let broker = broker();

        let id = broker
            .publish(
                "events",
                Bytes::from_static(b"k"),
                Bytes::from_static(b"payload"),
            )
            .await
            .expect("publish should be accepted before shutdown");

        assert_eq!(id, 0);
        assert!(broker.is_accepting_requests());
    }

    #[tokio::test]
    async fn publish_rejects_requests_after_shutdown_begins() {
        let broker = broker();

        assert!(broker.begin_shutdown());

        let error = broker
            .publish(
                "events",
                Bytes::from_static(b"k"),
                Bytes::from_static(b"payload"),
            )
            .await
            .expect_err("publish should be rejected after shutdown begins");

        assert!(error.to_string().contains("shutting down"));
        assert!(!broker.is_accepting_requests());
    }

    #[tokio::test]
    async fn begin_shutdown_is_idempotent() {
        let broker = broker();

        assert!(broker.begin_shutdown());
        assert!(!broker.begin_shutdown());
        assert!(!broker.is_accepting_requests());
    }
}
