mod database;
mod handler;
mod page;

use database::*;
use handler::*;
use page::*;
use std::error::Error;
use std::io::Cursor;
use std::time::Instant;

fn main() -> Result<(), Box<dyn Error>> {
    let mut db = Database::new(Cursor::new(Vec::<u8>::new()), "id", KeyType::UInt64, 8);

    db.init();

    let node = PageType::Node(Node {
        keytype: KeyType::UInt64,
        keys: vec![
            2_usize.to_le_bytes().to_vec(),
            5_usize.to_le_bytes().to_vec(),
        ],
        pointers: vec![0, 1, 0],
    });

    let leaf = PageType::Leaf(Leaf {
        keytype: KeyType::UInt64,
        keys: vec![2_usize.to_le_bytes().to_vec()],
        pointers: vec![2],
        next_leaf_pointer: 0,
    });

    let data = PageType::Data(Data {
        object: vec![
            Field::new(
                b"id".to_vec(),
                KeyType::UInt64,
                2_usize.to_le_bytes().to_vec(),
            ),
            Field::new(b"name".to_vec(), KeyType::String, b"jonas".to_vec()),
            Field::new(
                b"age".to_vec(),
                KeyType::UInt64,
                22_usize.to_le_bytes().to_vec(),
            ),
        ],
    });

    PageHandler::write(
        &mut db.source,
        Page {
            id: 0,
            pagetype: node,
        },
    )
    .unwrap();

    let _ = PageHandler::new_page(&mut db.source, leaf);
    let _ = PageHandler::new_page(&mut db.source, data);

    // let found = db.get(&2_usize.to_le_bytes());
    let found = db.get(b"lol");

    if let Ok(data) = found {
        println!("{}", data.json());
    }

    Ok(())
}
