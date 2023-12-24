use mongodb::Client;
use mongodb::options::ClientOptions;

#[derive(Clone)]
pub struct MongoClient {
    pub host: String,
    pub port: u16,
    pub conn: Client,
}

impl MongoClient {
    pub async fn new(host: String, port: u16) -> Self {
        let mut client_options = ClientOptions::parse(format!("mongodb://{host}:{port}")).await.unwrap();
        client_options.app_name = Some("query_cache".to_string());
        let client = Client::with_options(client_options).unwrap();
        Self {
            host,
            port,
            conn: client,
        }
    }
}