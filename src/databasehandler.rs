pub use crate::database::page::KeyType;
use crate::database::{Database, DatabaseBuilder, KeyTypeSize};
use axum::http::{HeaderName, StatusCode, header};
use std::fs::{File, OpenOptions};

pub type DatabaseResponse = (StatusCode, [(HeaderName, String); 1], String);

pub struct DatabaseHandler {
    db: Database<File>,
}

impl DatabaseHandler {
    #[allow(dead_code)]
    pub fn new_u64() -> DatabaseHandler {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(".db")
            .unwrap();

        let db = DatabaseBuilder::new(file)
            .key(b"id".to_vec())
            .keytype(KeyTypeSize::UInt64)
            .build_mock_u64();

        DatabaseHandler { db }
    }

    #[allow(dead_code)]
    pub fn new_string() -> DatabaseHandler {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(".db")
            .unwrap();

        let db = DatabaseBuilder::new(file)
            .key(b"id".to_vec())
            .keytype(KeyTypeSize::String(10))
            .build_mock_string();

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
