use crate::query::{Product, QueryPayload};
use axum::routing::post;
use axum::{extract::State, routing::get, Json, Router, debug_handler};
use database::cache::RedisServer;
use log::info;
use mongodb::bson::doc;
use query::{AppError, QueryBody};
use crate::database::mongo::MongoClient;

mod database;
mod query;

#[derive(Clone)]
struct AppState {
    rdb: RedisServer,
    mongo: MongoClient,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let rdb = RedisServer::new("127.0.0.1".to_string(), 6379).await;
    let mongo = MongoClient::new("localhost".to_string(), 27017).await;

    let app = Router::new()
        .route("/query", post(query).with_state(AppState { rdb, mongo }))
        .route("/health", get(health));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[debug_handler]
async fn query(
    State(mut s): State<AppState>,
    Json(payload): Json<QueryPayload>,
) -> Result<Json<QueryBody>, AppError> {
    let hash_key = query::get_hash_key(&payload).map_err(|_| AppError::InternalServerError)?;
    info!("hash key: {}", hash_key);
    let result = s.rdb.get(hash_key.as_str()).await.map_err(|_| AppError::InternalServerError)?;

    let db = s.mongo.conn.database("query_cache");
    let collection = db.collection::<Product>("products");
    let filter = doc! { "product_id": payload.product_id };
    let product = collection.find_one(filter, None).await.map_err(|_| AppError::InternalServerError)?;
    info!("product: {:?}", product);

    match result {
        Some(value) => Ok(Json(QueryBody::new(value))),
        None => {
            // TODO: query to the database, and cache the result
            Err(AppError::DataNotFound)
        },
    }
}

async fn health() -> &'static str {
    return "OK";
}
