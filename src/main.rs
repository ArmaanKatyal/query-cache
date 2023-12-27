use axum::{
    routing::{get, post},
    Router,
};
use cache::Cache;
use database::mongo::MongoClient;

mod cache;
mod database;
mod query;

#[derive(Clone)]
struct AppState {
    cache: Cache,
    mongo: MongoClient,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let mongo = MongoClient::new("mongo".to_string(), 27017).await;
    let cache = Cache::init().await;
    let app = Router::new()
        .route(
            "/query",
            post(query::query_handler).with_state(AppState { cache, mongo }),
        )
        .route("/health", get(health));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn health() -> &'static str {
    return "OK";
}
