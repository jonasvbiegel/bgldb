mod database;

use crate::database::{DatabaseBuilder, KeyTypeSize, page::Header};
use axum::{
    Router,
    body::Body,
    extract::Path,
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::get,
};
use std::error::Error;
use std::io::Cursor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("hosting on localhost:8000");

    let app = Router::new().route("/{*id}", get(get_data));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn get_data(Path(id): Path<usize>) -> impl IntoResponse {
    let mut db = DatabaseBuilder::new(Cursor::new(Vec::<u8>::new()))
        .key(b"id".to_vec())
        .keytype(KeyTypeSize::Identity)
        .build_mock_u64();

    let found = db.get(&id.to_le_bytes());

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
