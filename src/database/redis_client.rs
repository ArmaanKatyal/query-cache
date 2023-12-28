use async_trait::async_trait;
use log::info;
use redis::AsyncCommands;

#[async_trait]
pub trait RedisTrait {
    async fn new(host: String, port: u16) -> Self;
    async fn get(&mut self, key: &str) -> Result<Option<String>, redis::RedisError>;
    async fn set(&mut self, key: &str, value: &str) -> Result<(), redis::RedisError>;
    async fn del(&mut self, key: &str) -> Result<(), redis::RedisError>;
}

#[derive(Clone)]
pub struct RedisServer {
    pub host: String,
    pub port: u16,
    pub conn: redis::aio::MultiplexedConnection,
}

#[async_trait]
impl RedisTrait for RedisServer {
    async fn new(host: String, port: u16) -> Self {
        let client = redis::Client::open(format!("redis://{host}:{port}")).unwrap();
        let conn = client.get_multiplexed_async_connection().await.unwrap();
        info!("connected to redis server on {}:{}", host, port);
        Self { host, port, conn }
    }

    async fn get(&mut self, key: &str) -> Result<Option<String>, redis::RedisError> {
        let result = self.conn.get(key).await?;
        Ok(result)
    }

    async fn set(&mut self, key: &str, value: &str) -> Result<(), redis::RedisError> {
        let result = self.conn.set(key, value).await?;
        Ok(result)
    }

    async fn del(&mut self, key: &str) -> Result<(), redis::RedisError> {
        let result = self.conn.del(key).await?;
        Ok(result)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    pub struct MockRedisServer {
        pub host: String,
        pub port: u16,
    }
    #[async_trait]
    impl RedisTrait for MockRedisServer {
        async fn new(host: String, port: u16) -> Self {
            Self { host, port }
        }
        async fn get(&mut self, _key: &str) -> Result<Option<String>, redis::RedisError> {
            Ok(None)
        }
        async fn set(&mut self, _key: &str, _value: &str) -> Result<(), redis::RedisError> {
            Ok(())
        }
        async fn del(&mut self, _key: &str) -> Result<(), redis::RedisError> {
            Ok(())
        }
    }
}
