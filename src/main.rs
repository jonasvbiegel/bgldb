mod database;

use crate::database::{DatabaseBuilder, KeyTypeSize};
use axum::body::Body;
use axum::extract::Path;
use axum::http::{HeaderValue, Response};
use axum::response::IntoResponse;
use axum::{Router, routing::get};
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

    let found = db
        .get(&id.to_le_bytes())
        .expect("TODO: implement error handling");

    let mut res = Response::new(Body::from(found.json()));
    res.headers_mut()
        .insert("Content-Type", HeaderValue::from_static("application/json"));

    res
}
