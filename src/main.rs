mod filehandler;
use filehandler::{FileHandler, Header, KeyType, Page};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut handler = FileHandler::new_test("test")?;

    let page_id = dbg!(handler.new_page()?);

    let mut header = Header {
        elements: 909090,
        keytype: KeyType::String(10),
    };

    handler.write_to_header(&header.serialize())?;

    dbg!(Header::deserialize(&handler.read_header()?))?;

    let mut b: Vec<u8> = Vec::new();
    // page type
    b.push(0x01);

    // keys_len
    for x in u16::to_le_bytes(1) {
        b.push(x);
    }

    // first pointer
    for y in u64::to_le_bytes(123) {
        b.push(y);
    }

    // keys
    for z in u64::to_le_bytes(999) {
        b.push(z);
    }

    // last pointer
    for æ in u64::to_le_bytes(456) {
        b.push(æ);
    }

    handler.write_to_page(page_id, &b)?;

    dbg!(Page::deserialize(&handler.read_page(page_id)?));

    Ok(())
}
