pub use crate::database::page::KeyType;
use crate::database::{Database, DatabaseBuilder, KeyTypeSize};
use axum::http::{HeaderName, StatusCode, header};
use std::fs::{File, OpenOptions};

pub type DatabaseResponse = (StatusCode, [(HeaderName, String); 1], String);

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

    pub fn get_data(&mut self, key: &[u8]) -> DatabaseResponse {
        let found = self.db.get(key);

        if let Ok(Some(data)) = found {
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/json".to_string())],
                data.json(),
            )
        } else {
            (
                StatusCode::NOT_FOUND,
                [(header::CONTENT_TYPE, "text/plain".to_string())],
                "key not found".to_string(),
            )
        }
    }
}
