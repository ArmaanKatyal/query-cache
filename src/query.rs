use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Error};
use sha2::{Digest, Sha256};

#[derive(Debug)]
pub struct Query {
    pub key: String,
    pub value: String,
}

impl Query {
    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QueryPayload {
    pub product_id: String,
    pub price: u64,
    pub product_display_name: String,
    pub brand_name: String,
}

pub fn get_hash_key(document: &QueryPayload) -> Result<String, Error> {
    let text = serde_json::to_string(document).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(text);
    let hash = hasher.finalize();
    Ok(format!("CACHE_ASIDE_{:x}", hash))
}

#[derive(Serialize)]
pub struct QueryBody {
    pub result: String,
}

impl QueryBody {
    pub fn new(query: String) -> Self {
        Self { result: query }
    }
}

#[derive(Debug)]
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
