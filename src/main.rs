use axum::{Router, routing::get, Json, http::StatusCode, response::{IntoResponse, Response}, debug_handler, extract::State};
use database::cache::RedisServer;
use log::info;
use serde_json::json;

mod database;

#[derive(serde::Serialize)]
struct QueryBody {
    query: String,
}

impl QueryBody {
    fn new(query: String) -> Self {
        Self {
            query,
        }
    }
}

#[derive(Debug)]
pub enum AppError {
    DataNotFound,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::DataNotFound => (StatusCode::BAD_REQUEST, "Data not found"),
        };
        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let rdb = database::cache::RedisServer::new("127.0.0.1".to_string(), 6379).await;

    let app = Router::new()
    .route("/query", get(query).with_state(rdb))
    .route("/health", get(health));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
    .serve(app.into_make_service())
    .await
    .unwrap();
}

#[debug_handler]
async fn query(State(mut rdb): State<RedisServer>) -> Result<Json<QueryBody>, AppError> {
    info!("querying redis");
    let result = rdb.get("test").await.unwrap();
    match result {
        Some(value) => Ok(Json(QueryBody::new(value))),
        None => Err(AppError::DataNotFound),
    }
}

async fn health() -> &'static str {
    return "OK"
}
