mod database;
mod databasehandler;

use databasehandler::DatabaseHandler;

use axum::{
    Router,
    body::Body,
    extract::{Path, State},
    http::{Response, StatusCode},
    routing::get,
};
use std::sync::Arc;
use std::{error::Error, sync::Mutex};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("hosting on localhost:8000");

    let state = Arc::new(Mutex::new(DatabaseHandler::new()));
    let address = "localhost:8000".to_string();

    let app = Router::new()
        .route("/{*key}", get(get_data))
        .route("/", get("bgldb"))
        .with_state(state);

    let listener = TcpListener::bind(address).await.unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn get_data(
    State(handler): State<Arc<Mutex<DatabaseHandler>>>,
    Path(key): Path<usize>,
) -> Response<Body> {
    if let Ok(mut locked) = handler.lock() {
        locked.get_data(key)
    } else {
        Response::builder()
            .status(StatusCode::LOCKED)
            .header("Content-Type", "text/plain")
            .body(Body::from("database is locked"))
            .unwrap()
    }
}
