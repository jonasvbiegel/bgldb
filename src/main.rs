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
    db.insert("6969");
    db.insert("123123123");
    db.insert("0303030303");

    let page = PageHandler::get_page(&mut db.source, 0);

    println!("{:#?}", page);

    if let PageType::Leaf(leaf) = page.unwrap().pagetype {
        for k in leaf.keys {
            let key = usize::from_le_bytes(k.try_into().expect("lol"));
            println!("{key}");
        }
    }

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

        let leaf = dbg!(Leaf::new(self.keytype));
        let _ = dbg!(PageHandler::new_page(
            &mut self.source,
            PageType::Leaf(leaf)
        ));
    }

    // takes data in the future
    fn insert(&mut self, key: &str) -> bool {
        let mut page = match PageHandler::get_page(&mut self.source, self.root.try_into().unwrap())
        {
            Ok(page) => page,
            Err(e) => {
                println!("insert failure: {e}");
                return false;
            }
        };

        match page.pagetype {
            PageType::Leaf(ref mut leaf) => match leaf.keytype {
                KeyType::String => {
                    let mut vec: Vec<u8> = Vec::new();
                    key.bytes().for_each(|byte| vec.push(byte));
                    leaf.keys.push(vec);
                }
                KeyType::UInt64 => {
                    if let Ok(v) = key.parse::<usize>() {
                        let mut vec: Vec<u8> = Vec::new();
                        v.to_le_bytes().iter().for_each(|byte| vec.push(*byte));
                        leaf.keys.push(vec);
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
