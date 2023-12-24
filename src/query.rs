use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

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
