use crate::database::{Database, DatabaseBuilder, KeyTypeSize, page::KeyType};
use axum::{
    body::Body,
    http::{Response, StatusCode},
};
use std::fs::{File, OpenOptions};

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
            .keytype(KeyTypeSize::UInt64)
            .build_mock_u64();

        DatabaseHandler { db }
    }

    pub fn get_keytype(&mut self) -> KeyType {
        self.db.get_keytype().unwrap()
    }

    pub fn get_data(&mut self, key: &[u8]) -> Response<Body> {
        let found = self.db.get(key);

        if let Ok(Some(data)) = found {
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
