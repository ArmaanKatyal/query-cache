// use axum::async_trait;
use redis::AsyncCommands;

#[derive(Clone)]
pub struct RedisServer {
    pub host: String,
    pub port: u16,
    pub conn: redis::aio::MultiplexedConnection,
}

#[allow(dead_code)]
impl RedisServer {
    pub async fn new(host: String, port: u16) -> Self {
        let client = redis::Client::open(format!("redis://{}:{}", host, port)).unwrap();
        let conn = client.get_multiplexed_async_connection().await.unwrap();
        Self { host, port, conn }
    }

    pub async fn get(&mut self, key: &str) -> Result<Option<String>, redis::RedisError> {
        let result = self.conn.get(key).await?;
        Ok(result)
    }

    pub async fn set(&mut self, key: &str, value: &str) -> Result<(), redis::RedisError> {
        let result = self.conn.set(key, value).await?;
        Ok(result)
    }
}
