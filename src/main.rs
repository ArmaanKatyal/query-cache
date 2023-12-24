use crate::query::QueryPayload;
use axum::routing::post;
use axum::{extract::State, routing::get, Json, Router};
use database::cache::RedisServer;
use log::info;
use query::{AppError, QueryBody};

mod database;
mod query;

#[tokio::main]
async fn main() {
    env_logger::init();

    let rdb = RedisServer::new("127.0.0.1".to_string(), 6379).await;

    let app = Router::new()
        .route("/query", post(query).with_state(rdb))
        .route("/health", get(health));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn query(
    State(mut rdb): State<RedisServer>,
    Json(payload): Json<QueryPayload>,
) -> Result<Json<QueryBody>, AppError> {
    let hash_key = query::get_hash_key(&payload).map_err(|_| AppError::InternalServerError)?;
    info!("hash key: {}", hash_key);

    let result = rdb.get("test").await.unwrap();
    match result {
        Some(value) => Ok(Json(QueryBody::new(value))),
        None => Err(AppError::DataNotFound),
    }
}

async fn health() -> &'static str {
    return "OK";
}
