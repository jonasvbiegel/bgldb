use std::fs::{File, OpenOptions};

use axum::{
    body::Body,
    http::{Response, StatusCode},
};

use crate::database::{Database, DatabaseBuilder, KeyTypeSize};

pub struct DatabaseHandler {
    db: Database<File>,
}

impl DatabaseHandler {
    pub fn new() -> DatabaseHandler {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(".db")
            .unwrap();

        let db = DatabaseBuilder::new(file)
            .key(b"id".to_vec())
            .keytype(KeyTypeSize::Identity)
            .build_mock_u64();

        DatabaseHandler { db }
    }

    pub fn get_data(&mut self, key: usize) -> Response<Body> {
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
