use axum::{
    debug_handler,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use futures::stream::TryStreamExt;
use log::info;
use mongodb::{bson::doc, options::FindOptions};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::AppState;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct QueryPayload {
    pub product_id: Option<String>,
    pub price: Option<u64>,
    pub product_display_name: Option<String>,
    pub brand_name: Option<String>,
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
pub enum AppError {
    DataNotFound,
    InternalServerError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::DataNotFound => (StatusCode::BAD_REQUEST, "Data not found"),
            AppError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };
        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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
            info!("cache hit");
            let result = QueryBody::new(value);
            Ok(Json(result))
        }
        None => {
            let cache_payload = payload.clone();
            let db = s.mongo.conn.database("query_cache");
            let collection = db.collection::<Product>("products");
            if let Some(product_id) = payload.product_id {
                info!("cache miss; product_id:{}", product_id);
                let product = collection
                    .find_one(doc! {"product_id": product_id}, None)
                    .await
                    .map_err(|_| AppError::InternalServerError)?;
                match product {
                    Some(value) => {
                        let into_vec = vec![value];
                        s.cache
                            .set(&cache_payload, &into_vec)
                            .await
                            .map_err(|_| AppError::InternalServerError)?;
                        Ok(Json(QueryBody::new(into_vec)))
                    }
                    None => Err(AppError::DataNotFound),
                }
            } else if let Some(product_name) = payload.product_display_name {
                info!("cache miss; product_name:{}", product_name);
                let re = mongodb::bson::Regex {
                    pattern: format!(".*{}.*", product_name),
                    options: String::from("i"),
                };
                let mut prods = collection
                    .find(
                        doc! {"product_display_name": re},
                        FindOptions::builder().limit(10).build(),
                    )
                    .await
                    .map_err(|_| AppError::InternalServerError)?;
                let mut result = Vec::new();
                while let Some(prod) = prods
                    .try_next()
                    .await
                    .map_err(|_| AppError::InternalServerError)?
                {
                    result.push(prod);
                }
                s.cache
                    .set(&cache_payload, &result)
                    .await
                    .map_err(|_| AppError::InternalServerError)?;
                Ok(Json(QueryBody::new(result)))
            } else {
                Err(AppError::DataNotFound)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_error() {
        let app_error = AppError::DataNotFound;
        let response = app_error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
