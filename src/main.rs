use crate::query::{Product, QueryPayload};
use axum::routing::post;
use axum::{debug_handler, extract::State, routing::get, Json, Router};
use cache::Cache;
use database::mongo::MongoClient;
use log::info;
use mongodb::bson::doc;
use query::{AppError, QueryBody};

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

    let mongo = MongoClient::new("localhost".to_string(), 27017).await;
    let cache = Cache::init().await;
    let app = Router::new()
        .route(
            "/query",
            post(query_handler).with_state(AppState { cache, mongo }),
        )
        .route("/health", get(health));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[debug_handler]
async fn query_handler(
    State(mut s): State<AppState>,
    Json(payload): Json<QueryPayload>,
) -> Result<Json<QueryBody>, AppError> {
    let result = s
        .cache
        .get(&payload)
        .await
        .map_err(|_| AppError::InternalServerError)?;
    match result {
        Some(value) => {
            info!("cache hit; product_id:{}", payload.product_id);
            let payload = serde_json::from_str::<Product>(&value)
                .map_err(|_| AppError::InternalServerError)?;
            let result = QueryBody::new(vec![payload]);
            Ok(Json(result))
        }
        None => {
            info!("cache miss; product_id:{}", payload.product_id);
            let db = s.mongo.conn.database("query_cache");
            let cache_payload = payload.clone();
            let collection = db.collection::<Product>("products");
            let filter = doc! { "product_id": payload.product_id };
            let product = collection
                .find_one(filter, None)
                .await
                .map_err(|_| AppError::InternalServerError)?;
            match product {
                Some(value) => {
                    s.cache
                        .set(&cache_payload, &value)
                        .await
                        .map_err(|_| AppError::InternalServerError)?;
                    let result = QueryBody::new(vec![value]);
                    Ok(Json(result))
                }
                None => Err(AppError::DataNotFound),
            }
        }
    }
}

async fn health() -> &'static str {
    return "OK";
}
