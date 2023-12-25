use log::{error, info};
use serde_json::Error;
use sha2::{Digest, Sha256};

use crate::{
    database::redis_client::RedisServer,
    query::{Product, QueryPayload},
};

#[derive(Clone)]
pub struct Cache {
    pub redis: RedisServer,
}

impl Cache {
    pub async fn init() -> Self {
        let rdb = RedisServer::new("127.0.0.1".to_string(), 6379).await;
        info!("connected to redis server on {}:{}", rdb.host, rdb.port);
        Self { redis: rdb }
    }

    pub async fn get(
        &mut self,
        payload: &QueryPayload,
    ) -> Result<Option<String>, redis::RedisError> {
        let hash_key = get_hash_key(payload).map_err(|_| {
            error!("hash-key operation failed");
            redis::RedisError::from((redis::ErrorKind::TypeError, "failed to get hash key"))
        });
        let result = self.redis.get(hash_key.unwrap().as_str()).await;
        return result;
    }

    pub async fn set(
        &mut self,
        payload: &QueryPayload,
        product: &Product,
    ) -> Result<(), redis::RedisError> {
        let hash_key = get_hash_key(payload).map_err(|_| {
            error!("hash-key opertation failed");
            redis::RedisError::from((redis::ErrorKind::TypeError, "failed to get hash key"))
        });
        let result = self
            .redis
            .set(
                hash_key.unwrap().as_str(),
                serde_json::to_string(product).unwrap().as_str(),
            )
            .await;
        return result;
    }
}

pub fn get_hash_key(document: &QueryPayload) -> Result<String, Error> {
    let text = serde_json::to_string(document).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(text);
    let hash = hasher.finalize();
    let hash = hex::encode(hash);
    Ok(format!("CACHE_ASIDE_{hash}"))
}
