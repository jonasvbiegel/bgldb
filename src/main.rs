mod database;

use crate::database::handler::HandlerError;
use crate::database::page::Data;
use crate::database::{DatabaseBuilder, KeyTypeSize};
use std::error::Error;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn Error>> {
    let mut db = DatabaseBuilder::new(Cursor::new(Vec::<u8>::new()))
        .key(b"id".to_vec())
        .keytype(KeyTypeSize::Identity)
        .build_mock_u64();

    for i in 1_usize..=6_usize {
        let found: Result<Data, HandlerError> = db.get(&i.to_le_bytes());

        if let Ok(data) = found {
            println!("{}", data.json());
        }
    }

    Ok(())
}
