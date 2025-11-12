mod database;
use database::{Data, DataBuilder, Database, DatabaseError, Header, KeyType, Node};
use std::error::Error;

fn main() -> Result<(), DatabaseError> {
    let mut handler = Database::new_test("test")?;

    let page_id = dbg!(handler.new_page()?);

    let mut header = Header {
        elements: 909090,
        keytype: KeyType::String,
        keytype_size: 10,
    };

    handler.write_to_header(&header.serialize())?;

    dbg!(Header::deserialize(&handler.read_header()?))?;

    let mut b: Vec<u8> = Vec::new();
    for i in page_id.to_le_bytes() {
        b.push(i);
    }

    // nodetype
    b.push(0x01);

    //keytype
    b.push(0x01);

    //keys_len
    b.push(0x01);

    b.push(0x09);
    for m in "missemand".bytes() {
        b.push(m)
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

    dbg!(Node::deserialize(&handler.read_page(page_id)?)?);

    Ok(())
}
