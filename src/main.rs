mod database;
mod databasehandler;

use databasehandler::DatabaseHandler;
use serde_json::Value;

use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    routing::{get, post},
};
use std::sync::Arc;
use std::{error::Error, sync::Mutex};
use tokio::net::TcpListener;

use crate::database::page::KeyType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("hosting on localhost:8000");

    let database = Arc::new(Mutex::new(DatabaseHandler::new()));
    let address = "localhost:8000".to_string();

    let app = Router::new()
        .route("/", post(operation))
        .route("/", get("Hello from bgldb!"))
        .with_state(database);

    let listener = TcpListener::bind(address).await.unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn operation(
    State(handler): State<Arc<Mutex<DatabaseHandler>>>,
    Json(payload): Json<Value>,
) -> Response<Body> {
    if let Value::String(operation) = &payload["operation"] {
        match operation.as_str() {
            "GET" => {
                if let Ok(mut locked) = handler.lock() {
                    let keytype = locked.get_keytype();
                    match (&payload["key"], keytype) {
                        (Value::String(key), KeyType::String) => locked.get_data(key.as_bytes()),
                        (Value::Number(key), KeyType::UInt64) => {
                            locked.get_data(&key.as_u64().unwrap().to_le_bytes())
                        }
                        _ => Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .header("Content-Type", "text/plain")
                            .body(Body::from("wrong keytype"))
                            .unwrap(),
                    }
                } else {
                    Response::builder()
                        .status(StatusCode::LOCKED)
                        .header("Content-Type", "text/plain")
                        .body(Body::from("database was locked"))
                        .unwrap()
                }
            }
            "INSERT" => todo!(),
            _ => Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "text/plain")
                .body(Body::from("invalid operation"))
                .unwrap(),
        }
    } else {
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "text/plain")
            .body(Body::from("missing operation field"))
            .unwrap()
    }
}
