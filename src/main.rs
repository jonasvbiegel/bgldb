mod page;
use page::*;
use std::error::Error;
use std::io::{Cursor, Write};
use std::io::{Read, Seek};

fn main() -> Result<(), Box<dyn Error>> {
    let mut db = Database::new(
        Cursor::new(Vec::<u8>::new()),
        KeyType::UInt64,
        size_of::<u64>().try_into().unwrap(),
    );

    db.init();

    db.insert("1234");

    // let mut lol = Cursor::new(Vec::<u8>::new());
    // lol.write_all(&[0x00; 4096])?;
    // let header = Header {
    //     elements: 909090,
    //     keytype: KeyType::String,
    //     keytype_size: 10,
    //     order: 10,
    //     root: 0,
    // };
    //
    // HeaderHandler::write(&mut lol, header)?;
    //
    // println!("{:#?}", HeaderHandler::get(&mut lol)?);

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

struct Database<T: Read + Write + Seek> {
    source: T,
    keytype: KeyType,
    keytype_size: u8,
    root: usize,
}

impl<T: Read + Write + Seek> Database<T> {
    fn new(source: T, keytype: KeyType, keytype_size: u8) -> Database<T> {
        Database {
            source,
            keytype,
            keytype_size,
            root: 0,
        }
    }

    fn init(&mut self) {
        let header = Header {
            elements: 0,
            keytype: self.keytype,
            keytype_size: self.keytype_size,

            // this should be dynamic going forward, determined by keytype size
            order: 4,

            root: self.root.try_into().expect("u64 to usize failure"),
        };

        HeaderHandler::write(&mut self.source, header).expect("couldnt initialize header");

        let node = Node::new(self.keytype);
        let _ = PageHandler::new_page(&mut self.source, PageType::Node(node));
    }

    // takes data in the future
    fn insert(&mut self, key: &str) -> bool {
        let mut page = PageHandler::get_page(&mut self.source, self.root.try_into().unwrap())
            .expect("couldnt get root");

        match page.pagetype {
            PageType::Node(ref mut node) => match node.keytype {
                KeyType::String => {
                    let mut vec: Vec<u8> = Vec::new();
                    key.bytes().for_each(|byte| vec.push(byte));
                    node.keys.push(vec);
                }
                KeyType::UInt64 => {
                    if let Ok(v) = key.parse::<usize>() {
                        let mut vec: Vec<u8> = Vec::new();
                        v.to_le_bytes().iter().for_each(|byte| vec.push(*byte));
                        node.keys.push(vec);
                    } else {
                        return false;
                    }
                }
            },
            _ => return false,
        }

        PageHandler::write(&mut self.source, page).expect("couldnt write page");

        true
    }
}
