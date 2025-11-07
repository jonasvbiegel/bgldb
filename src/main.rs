mod database;
use database::{Data, DataBuilder, Database, Header, KeyType, Node};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut handler = Database::new_test("test")?;

    let page_id = dbg!(handler.new_page()?);

    let mut header = Header {
        elements: 909090,
        keytype: KeyType::String(10),
    };

    handler.write_to_header(&header.serialize())?;

    dbg!(Header::deserialize(&handler.read_header()?))?;

    let mut b: Vec<u8> = Vec::new();
    for i in page_id.to_le_bytes() {
        b.push(i);
    }

    // page type
    b.push(0x01);

    // keys_len
    for x in u16::to_le_bytes(1) {
        b.push(x);
    }

    // keys
    for z in u64::to_le_bytes(999) {
        b.push(z);
    }

    // first pointer
    for y in u64::to_le_bytes(123) {
        b.push(y);
    }

    // last pointer
    for æ in u64::to_le_bytes(456) {
        b.push(æ);
    }

    handler.write_to_page(page_id, &b)?;

    dbg!(Node::deserialize_node(&handler.read_page(page_id)?));

    dbg!(
        DataBuilder::new(0)
            .primary("id", KeyType::Int, u64::to_le_bytes(1234).to_vec())
            .unwrap()
            .field("name", KeyType::String(10), str::as_bytes("jonas").to_vec())
            .field("age", KeyType::Int, usize::to_le_bytes(1000).to_vec())
            .build()
    );

    Ok(())
}
