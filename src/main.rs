mod database;
mod handler;
mod page;

use database::*;
use std::error::Error;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn Error>> {
    let mut db = DatabaseBuilder::new(Cursor::new(Vec::<u8>::new()))
        .key(b"id".to_vec())
        .keytype(KeyTypeSize::Identity)
        .build_mock_u64();

    // let mut db_file = DatabaseBuilder::new(
    //     OpenOptions::new()
    //         .read(true)
    //         .write(true)
    //         .truncate(true)
    //         .create(true)
    //         .open("./test.db")
    //         .unwrap(),
    // )
    // .build_mock_u64();

    for i in 1_usize..=6_usize {
        let found = db.get(&i.to_le_bytes());

        if let Ok(data) = found {
            println!("{}", data.json());
        }
    }

    Ok(())
}
