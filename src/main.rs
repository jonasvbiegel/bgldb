mod filehandler;
use filehandler::{FileHandler, Header, Page};
use std::io::Error;

fn main() -> Result<(), Error> {
    let mut handler = FileHandler::new_test("test")?;

    let page_id = dbg!(handler.new_page()?);

    let mut header = Header { elements: 909090 };
    handler.write_to_header(&header.serialize())?;

    let header_read = Header::deserialize(&handler.read_header().unwrap());

    println!("{header_read:?}");

    let mut b: Vec<u8> = Vec::new();
    b.push(0x01);

    for x in u16::to_le_bytes(1) {
        b.push(x);
    }

    for y in u32::to_le_bytes(123) {
        b.push(y);
    }

    for z in u64::to_le_bytes(7123123173) {
        b.push(z);
    }

    for æ in u32::to_le_bytes(456) {
        b.push(æ);
    }

    handler.write_to_page(page_id, &b)?;

    dbg!(Page::deserialize(&handler.read_page(page_id)?));

    Ok(())
}
