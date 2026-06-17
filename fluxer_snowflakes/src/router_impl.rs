// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::types::{SERVICE_NAME, SnowflakeRequest, SnowflakeResponse};
use fluxer_svc::config::ServiceConfig;
use fluxer_svc::hash_ring::HashRing;
use fluxer_svc::metrics::ServiceMetrics;
use fluxer_svc::transport::{NatsTransport, TransportMessage, TransportSubscriber};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;
use tokio::task::JoinSet;
use tracing::{debug, info, warn};

const SHARD_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_ROUTER_REQUEST_BYTES: usize = 64 * 1024;

pub struct RoundRobinShardPicker {
    next: AtomicU32,
    shard_count: u32,
}

impl RoundRobinShardPicker {
    pub fn new(shard_count: u32) -> Self {
        Self::with_start(shard_count, 0)
    }

    fn with_start(shard_count: u32, start: u32) -> Self {
        Self {
            next: AtomicU32::new(start),
            shard_count: shard_count.max(1),
        }
    }

    pub fn next_shard(&self) -> u32 {
        self.next.fetch_add(1, Ordering::Relaxed) % self.shard_count
    }
}

pub struct SnowflakeShardPicker {
    round_robin: RoundRobinShardPicker,
    hash_ring: HashRing,
}

impl SnowflakeShardPicker {
    pub fn new(shard_count: u32) -> Self {
        Self {
            round_robin: RoundRobinShardPicker::new(shard_count),
            hash_ring: HashRing::new(shard_count.max(1)),
        }
    }

    pub fn pick_shard(&self, request: &SnowflakeRequest) -> u32 {
        match request.routing_key() {
            Some(routing_key) => self.hash_ring.owner(routing_key),
            None => self.round_robin.next_shard(),
        }
    }
}

pub async fn run_round_robin_router(
    config: &ServiceConfig,
    transport: NatsTransport,
) -> anyhow::Result<()> {
    let request_subject = format!("svc.{SERVICE_NAME}");
    let queue_group = format!("{SERVICE_NAME}-router");
    let picker = Arc::new(SnowflakeShardPicker::new(config.shard_count));
    let mut tasks = JoinSet::new();
    let health_addr = config.listen_addr;
    let router_serving = Arc::new(AtomicBool::new(true));
    let http_serving = router_serving.clone();
    let http_metrics = Arc::new(ServiceMetrics::default());
    http_metrics.init();
    tasks.spawn({
        let metrics = http_metrics.clone();
        async move {
            fluxer_svc::server::run_http(
                health_addr,
                http_serving,
                metrics,
                SERVICE_NAME.to_owned(),
            )
            .await
        }
    });
    let req_transport = transport.clone();
    tasks.spawn(async move {
        loop {
            let mut sub = req_transport
                .subscribe_queue(&request_subject, &queue_group)
                .await?;
            info!(subject = request_subject, "snowflake router listening for requests");
            loop {
                let msg = tokio::select! {
                    msg_opt = sub.next() => {
                        let Some(msg) = msg_opt else {
                            warn!("snowflake router request subscription stream ended, will re-subscribe");
                            break;
                        };
                        msg
                    }
                    _ = req_transport.wait_for_reconnect() => {
                        info!("NATS reconnected, re-subscribing snowflake router request listener");
                        break;
                    }
                };
                let transport = req_transport.clone();
                let picker = picker.clone();
                if msg.payload().len() > MAX_ROUTER_REQUEST_BYTES {
                    warn!(
                        payload_bytes = msg.payload().len(),
                        max_payload_bytes = MAX_ROUTER_REQUEST_BYTES,
                        "rejecting oversized snowflake request"
                    );
                    reply_json_error(&msg, &transport, "request_too_large").await;
                    continue;
                }
                let request: SnowflakeRequest = match serde_json::from_slice(msg.payload()) {
                    Ok(request) => request,
                    Err(error) => {
                        warn!(error = %error, "failed to decode snowflake request");
                        reply_json_error(&msg, &transport, "decode_error").await;
                        continue;
                    }
                };
                tokio::spawn(async move {
                    let shard_id = picker.pick_shard(&request);
                    let shard_subject = format!("svc.{SERVICE_NAME}.shard.{shard_id}");
                    let payload = match rmp_serde::to_vec(&request) {
                        Ok(payload) => payload,
                        Err(error) => {
                            warn!(error = %error, "failed to encode snowflake shard request");
                            reply_json_error(&msg, &transport, "encode_error").await;
                            return;
                        }
                    };
                    let response_bytes = match transport
                        .request(&shard_subject, &payload, SHARD_REQUEST_TIMEOUT)
                        .await
                    {
                        Ok(response_bytes) => response_bytes,
                        Err(error) => {
                            debug!(error = %error, shard_id, "snowflake shard request failed");
                            reply_json_error(&msg, &transport, "shard_unavailable").await;
                            return;
                        }
                    };
                    let response: SnowflakeResponse = match rmp_serde::from_slice(&response_bytes) {
                        Ok(response) => response,
                        Err(error) => {
                            debug!(error = %error, shard_id, "failed to decode snowflake shard response");
                            reply_json_error(&msg, &transport, "shard_decode_error").await;
                            return;
                        }
                    };
                    if msg.has_reply() {
                        let response_json = serde_json::to_vec(&response).unwrap_or_default();
                        let _ = msg.reply(&transport, &response_json).await;
                    }
                });
            }
        }
    });
    tokio::select! {
        result = tasks.join_next() => {
            match result {
                Some(Ok(Ok(()))) => Ok(()),
                Some(Ok(Err(error))) => Err(error),
                Some(Err(error)) => Err(error.into()),
                None => Ok(()),
            }
        }
        _ = fluxer_svc::shutdown::wait_for_shutdown() => {
            info!("snowflake router shutting down");
            router_serving.store(false, Ordering::SeqCst);
            Ok(())
        }
    }
}

async fn reply_json_error(
    msg: &fluxer_svc::transport::NatsMessage,
    transport: &NatsTransport,
    error: &str,
) {
    if msg.has_reply() {
        let error_response =
            serde_json::to_vec(&serde_json::json!({ "error": error })).unwrap_or_default();
        let _ = msg.reply(transport, &error_response).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picker_round_robins_across_shards() {
        let picker = RoundRobinShardPicker::new(3);
        let shards = (0..8).map(|_| picker.next_shard()).collect::<Vec<_>>();
        assert_eq!(shards, vec![0, 1, 2, 0, 1, 2, 0, 1]);
    }

    #[test]
    fn picker_handles_atomic_wraparound() {
        let picker = RoundRobinShardPicker::with_start(4, u32::MAX - 1);
        let shards = (0..4).map(|_| picker.next_shard()).collect::<Vec<_>>();
        assert_eq!(shards, vec![2, 3, 0, 1]);
    }

    #[test]
    fn picker_routes_same_key_to_same_shard() {
        let picker = SnowflakeShardPicker::new(8);
        let request = SnowflakeRequest::GenerateBatch {
            count: 1,
            routing_key: Some("channel:1510189013330296832".to_owned()),
        };
        let first = picker.pick_shard(&request);
        for _ in 0..100 {
            assert_eq!(picker.pick_shard(&request), first);
        }
    }

    #[test]
    fn picker_round_robins_unrouted_requests() {
        let picker = SnowflakeShardPicker::new(3);
        let request = SnowflakeRequest::GenerateBatch {
            count: 1,
            routing_key: None,
        };
        let shards = (0..8)
            .map(|_| picker.pick_shard(&request))
            .collect::<Vec<_>>();
        assert_eq!(shards, vec![0, 1, 2, 0, 1, 2, 0, 1]);
    }
}
