mod page;
use page::{FileError, Header, KeyType, Node, Page, PageHandler, Pageable, SerializeDeserialize};
use std::io::{Cursor, Write};

fn main() -> Result<(), FileError> {
    // let mut file = OpenOptions::new()
    //     .create(true)
    //     .truncate(true)
    //     .write(true)
    //     .read(true)
    //     .open("test/test")
    //     .unwrap();
    // file.write_all(&[0x00; 4096])?;

    // let mut lol = Cursor::new(Vec::<u8>::new());
    // lol.write_all(&[0x00; 4096])?;
    //
    // let page_id = dbg!(PageHandler::new_page(&mut lol)?);
    //
    // let mut header = Header {
    //     elements: 909090,
    //     keytype: KeyType::String,
    //     keytype_size: 10,
    //     order: 10,
    //     root: 0,
    // };
    //
    // PageHandler::write_to_header(&mut lol, &header.serialize())?;
    //
    // dbg!(Header::deserialize(&PageHandler::read_header(&mut lol)?))?;
    //
    // let mut b: Vec<u8> = Vec::new();
    // for i in page_id.to_le_bytes() {
    //     b.push(i);
    // }
    //
    // // nodetype
    // b.push(0x01);
    //
    // //keytype
    // b.push(0x01);
    //
    // //keys_len
    // b.push(0x01);
    //
    // // string
    // b.push(0x09);
    // for m in "missemand".bytes() {
    //     b.push(m)
    // }
    //
    // // first pointer
    // for y in u64::to_le_bytes(123) {
    //     b.push(y);
    // }
    //
    // // last pointer
    // for æ in u64::to_le_bytes(456) {
    //     b.push(æ);
    // }
    //
    // dbg!(PageHandler::write_to_page(&mut lol, page_id, &b))?;
    //
    // // let k = Node::deserialize(&PageHandler::read_page(&mut lol, page_id)?);
    //
    // let node = Page::deserialize(&PageHandler::read_page(&mut lol, page_id)?);
    //
    // match node {
    //     Ok(n) => println!("{n:#?}"),
    //     Err(e) => println!("{e}"),
    // }
    //
    Ok(())
}
