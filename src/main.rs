mod database;
mod databasehandler;

use crate::databasehandler::KeyType;
use databasehandler::{DatabaseHandler, DatabaseResponse};
use serde_json::Value;

use axum::{
    Json, Router,
    extract::State,
    http::{StatusCode, header},
    routing::{get, post},
};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    println!("hosting on localhost:8000");

    let database = Arc::new(Mutex::new(DatabaseHandler::new()));
    let address = "localhost:8000".to_string();

    let cors = CorsLayer::new().allow_origin(Any);

    let app = Router::new()
        .route("/", post(operation))
        .route("/", get("Hello from bgldb!"))
        .with_state(database)
        .layer(cors);

    let listener = TcpListener::bind(address).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn operation(
    State(handler): State<Arc<Mutex<DatabaseHandler>>>,
    Json(payload): Json<Value>,
) -> DatabaseResponse {
    if let Value::String(operation) = &payload["operation"] {
        match operation.to_uppercase().as_str() {
            "GET" => {
                if let Ok(mut locked) = handler.lock() {
                    match (&payload["key"], locked.get_keytype()) {
                        (Value::String(key), KeyType::String) => locked.get_data(key.as_bytes()),
                        (Value::Number(key), KeyType::UInt64) => {
                            if let Some(key) = key.as_u64() {
                                locked.get_data(&key.to_le_bytes())
                            } else {
                                (
                                    StatusCode::BAD_REQUEST,
                                    [(header::CONTENT_TYPE, "text/plain".to_string())],
                                    "expected unsigned integer, found negative number".to_string(),
                                )
                            }
                        }
                        _ => (
                            StatusCode::BAD_REQUEST,
                            [(header::CONTENT_TYPE, "text/plain".to_string())],
                            "wrong keytype".to_string(),
                        ),
                    }
                } else {
                    (
                        StatusCode::LOCKED,
                        [(header::CONTENT_TYPE, "text/plain".to_string())],
                        "database is locked".to_string(),
                    )
                }
            }
            "INSERT" => todo!(),
            _ => (
                StatusCode::BAD_REQUEST,
                [(header::CONTENT_TYPE, "text/plain".to_string())],
                "invalid operation".to_string(),
            ),
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            [(header::CONTENT_TYPE, "text/plain".to_string())],
            "missing operation field".to_string(),
        )
    }
}
