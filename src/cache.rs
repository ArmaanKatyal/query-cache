use chrono::Utc;
use log::{error, info};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize, Deserialize)]
struct CacheValue {
    product: Vec<Product>,
    ttl: i64,
}

impl CacheValue {
    pub fn new(product: Vec<Product>) -> Self {
        let ttl = Utc::now().timestamp() + 60;
        Self { product, ttl }
    }
}

impl Cache {
    pub async fn init() -> Self {
        let rdb = RedisServer::new("redis".to_string(), 6379).await;
        info!("connected to redis server on {}:{}", rdb.host, rdb.port);
        Self { redis: rdb }
    }

    pub async fn get(
        &mut self,
        payload: &QueryPayload,
    ) -> Result<Option<Vec<Product>>, redis::RedisError> {
        let hash_key = get_hash_key(payload).map_err(|_| {
            error!("hash-key operation failed");
            redis::RedisError::from((redis::ErrorKind::TypeError, "failed to get hash key"))
        }).ok();
        let result = self.redis.get(hash_key.clone().unwrap().as_str()).await;
        match result {
            Ok(val) => {
                match val {
                    Some(val) => {
                        let val: CacheValue = serde_json::from_str(val.as_str()).unwrap();
                        if val.ttl < Utc::now().timestamp() {
                            // cache expired
                            self.redis.del(hash_key.unwrap().as_str()).await?;
                            return Ok(None);
                        }
                        return Ok(Some(val.product));
                    }
                    None => {
                        return Ok(None);
                    }
                }
            }
            Err(e) => {
                error!("redis get operation failed: {:?}", e);
                return Err(e);
            }
        }
    }

    pub async fn set(
        &mut self,
        payload: &QueryPayload,
        product: &Vec<Product>,
    ) -> Result<(), redis::RedisError> {
        let hash_key = get_hash_key(payload).map_err(|_| {
            error!("hash-key opertation failed");
            redis::RedisError::from((redis::ErrorKind::TypeError, "failed to get hash key"))
        });
        let val = CacheValue::new(product.clone());
        let result = self
            .redis
            .set(
                hash_key.unwrap().as_str(),
                serde_json::to_string(&val).unwrap().as_str(),
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
