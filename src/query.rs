use axum::{
    debug_handler,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use log::info;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::AppState;

#[derive(Debug)]
pub struct Query {
    pub key: String,
    pub value: String,
}
#[allow(dead_code)]
impl Query {
    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct QueryPayload {
    pub product_id: String,
    pub price: u64,
    pub product_display_name: String,
    pub brand_name: String,
}

#[derive(Serialize)]
pub struct QueryBody {
    pub data: Vec<Product>,
}

impl QueryBody {
    pub fn new(result: Vec<Product>) -> Self {
        Self { data: result }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum AppError {
    DataNotFound,
    InvalidQuery,
    InternalServerError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::DataNotFound => (StatusCode::BAD_REQUEST, "Data not found"),
            AppError::InvalidQuery => (StatusCode::BAD_REQUEST, "Invalid query"),
            AppError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };
        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Product {
    pub product_id: String,
    pub price: u64,
    pub product_display_name: String,
    pub brand_name: String,
}

#[debug_handler]
pub async fn query_handler(
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
