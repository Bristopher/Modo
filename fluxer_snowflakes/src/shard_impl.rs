// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::generator::{MAX_WORKER_ID, SnowflakeGenerator};
use crate::types::{SERVICE_NAME, SnowflakeRequest, SnowflakeResponse};
use fluxer_svc::shard::ShardService;
use tokio::sync::Mutex;

pub const MAX_BATCH_SIZE: u32 = 512;

pub struct SnowflakesShard {
    generator: Mutex<SnowflakeGenerator>,
}

impl SnowflakesShard {
    pub fn new(worker_id: u32) -> anyhow::Result<Self> {
        if worker_id > MAX_WORKER_ID {
            anyhow::bail!("snowflake shard id {worker_id} exceeds max worker id {MAX_WORKER_ID}");
        }
        Ok(Self {
            generator: Mutex::new(SnowflakeGenerator::new(worker_id)?),
        })
    }

    async fn generate_batch(&self, count: u32) -> anyhow::Result<SnowflakeResponse> {
        if count == 0 || count > MAX_BATCH_SIZE {
            anyhow::bail!("batch count must be between 1 and {MAX_BATCH_SIZE}");
        }
        let mut generator = self.generator.lock().await;
        let ids = generator
            .generate_many(count)
            .await?
            .into_iter()
            .map(|id| id.to_string())
            .collect();
        Ok(SnowflakeResponse { ids })
    }
}

impl ShardService for SnowflakesShard {
    type Request = SnowflakeRequest;
    type Response = SnowflakeResponse;

    fn service_name(&self) -> &str {
        SERVICE_NAME
    }

    async fn handle(&self, request: SnowflakeRequest) -> anyhow::Result<SnowflakeResponse> {
        match request {
            SnowflakeRequest::GenerateBatch { count, .. } => self.generate_batch(count).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::parse_snowflake;

    #[tokio::test]
    async fn shard_generates_requested_batch() {
        let shard = SnowflakesShard::new(12).unwrap();
        let response = shard
            .handle(SnowflakeRequest::GenerateBatch {
                count: 8,
                routing_key: None,
            })
            .await
            .unwrap();
        assert_eq!(response.ids.len(), 8);
        let mut previous = 0;
        for id in response.ids {
            let parsed_id = id.parse::<u64>().unwrap();
            assert!(parsed_id > previous);
            assert_eq!(parse_snowflake(parsed_id).worker_id, 12);
            previous = parsed_id;
        }
    }

    #[tokio::test]
    async fn shard_rejects_empty_and_oversized_batches() {
        let shard = SnowflakesShard::new(0).unwrap();
        assert!(
            shard
                .handle(SnowflakeRequest::GenerateBatch {
                    count: 0,
                    routing_key: None,
                })
                .await
                .is_err()
        );
        assert!(
            shard
                .handle(SnowflakeRequest::GenerateBatch {
                    count: MAX_BATCH_SIZE + 1,
                    routing_key: None,
                })
                .await
                .is_err()
        );
    }

    #[test]
    fn shard_id_must_fit_worker_bits() {
        assert!(SnowflakesShard::new(MAX_WORKER_ID).is_ok());
        assert!(SnowflakesShard::new(MAX_WORKER_ID + 1).is_err());
    }
}
