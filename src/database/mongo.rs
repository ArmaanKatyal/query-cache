use async_trait::async_trait;
use log::info;
use mongodb::options::ClientOptions;
use mongodb::Client;

#[async_trait]
pub trait MongoTrait {
    async fn new(host: String, port: u16) -> Self;
}

#[derive(Clone)]
pub struct MongoClient {
    pub host: String,
    pub port: u16,
    pub conn: Client,
}

#[async_trait]
impl MongoTrait for MongoClient {
    async fn new(host: String, port: u16) -> Self {
        let mut client_options = ClientOptions::parse(format!("mongodb://{host}:{port}"))
            .await
            .unwrap();
        info!("connected to mongodb server on {}:{}", host, port);
        client_options.app_name = Some("query_cache".to_string());
        let client = Client::with_options(client_options).unwrap();
        Self {
            host,
            port,
            conn: client,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    pub struct MockMongoClient {
        pub host: String,
        pub port: u16,
    }

    #[async_trait]
    impl MongoTrait for MockMongoClient {
        async fn new(host: String, port: u16) -> Self {
            Self { host, port }
        }
    }
}
