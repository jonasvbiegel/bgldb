mod database;

use crate::database::{Database, DatabaseBuilder, KeyTypeSize};
use axum::{
    Router,
    body::Body,
    extract::{Path, State},
    http::{Response, StatusCode},
    routing::get,
};
use std::{error::Error, sync::Mutex};
use std::{
    fs::{File, OpenOptions},
    sync::Arc,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("hosting on localhost:8000");

    let state = Arc::new(Mutex::new(DatabaseApi::new()));
    let address = "localhost:8000".to_string();

    let app = Router::new()
        .route("/{*key}", get(get_data))
        .route("/", get("bgldb"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn get_data(
    State(handler): State<Arc<Mutex<DatabaseApi>>>,
    Path(key): Path<usize>,
) -> Response<Body> {
    handler.lock().unwrap().get_data(key)
}

struct DatabaseApi {
    db: Database<File>,
}

impl DatabaseApi {
    fn new() -> DatabaseApi {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open("database.db")
            .unwrap();

        let db = DatabaseBuilder::new(file)
            .key(b"id".to_vec())
            .keytype(KeyTypeSize::Identity)
            .build_mock_u64();

        DatabaseApi { db }
    }

    fn get_data(&mut self, key: usize) -> Response<Body> {
        let found = self.db.get(&key.to_le_bytes());

        if let Ok(data) = found {
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Body::from(data.json()))
                .unwrap()
        } else {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Content-Type", "text/plain")
                .body(Body::from("key not found"))
                .unwrap()
        }
    }
}
