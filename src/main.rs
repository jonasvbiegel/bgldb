mod handler;
mod page;

use handler::*;
use page::*;
use std::error::Error;
use std::io::{Cursor, Write};
use std::io::{Read, Seek};
use std::ops::Index;

fn main() -> Result<(), Box<dyn Error>> {
    let mut db = Database::new(Cursor::new(Vec::<u8>::new()), KeyType::String, 10);

    db.init();

    // db.insert("hej");
    // db.insert("farvel");
    // db.insert("foo");
    // db.insert("bar");
    //
    // let page = PageHandler::get_page(&mut db.source, 0);
    //
    // println!("{:#?}", page);
    //
    // if let PageType::Leaf(leaf) = page.unwrap().pagetype {
    //     for k in leaf.keys {
    //         let key = String::from_utf8(k).unwrap();
    //         println!("{key}");
    //     }
    // }

    let node = PageType::Node(Node {
        keytype: KeyType::String,
        keys: vec![b"jonas".to_vec()],
        pointers: vec![1],
    });

    let leaf = PageType::Leaf(Leaf {
        keytype: KeyType::String,
        keys: vec![b"jonas".to_vec()],
        pointers: vec![2],
        next_leaf_pointer: 0,
    });

    let data = PageType::Data(Data {
        object: vec![
            Field::new(b"name".to_vec(), KeyType::String, b"jonas".to_vec()),
            Field::new(
                b"age".to_vec(),
                KeyType::UInt64,
                22_usize.to_le_bytes().to_vec(),
            ),
        ],
    });

    println!("{node:#?}");
    println!("{leaf:#?}");
    println!("{data:#?}");

    let node_page = Page {
        id: 0,
        pagetype: node,
    };

    let leaf_page = Page {
        id: 1,
        pagetype: leaf,
    };

    let data_page = Page {
        id: 2,
        pagetype: data,
    };

    PageHandler::write(&mut db.source, node_page).unwrap();
    PageHandler::write(&mut db.source, leaf_page).unwrap();
    PageHandler::write(&mut db.source, data_page).unwrap();

    let find = db.get("jonas");

    println!("{find:?}");

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

        let leaf = Leaf::new(self.keytype);
        let _ = PageHandler::new_page(&mut self.source, PageType::Leaf(leaf));
    }

    fn get_root(&mut self) -> Result<Page, HandlerError> {
        let root_id = HeaderHandler::get(&mut self.source)?.root;
        println!("root {root_id}");

        PageHandler::get_page(&mut self.source, root_id)
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

    fn get(&mut self, key: &str) -> Result<Data, HandlerError> {
        let mut current_node = self.get_root().expect("couldnt get root");

        while let PageType::Node(ref node) = current_node.pagetype {
            println!("{current_node:#?}");
            let child_id = if let Some(idx) = node
                .keys
                .iter()
                .position(|node_key| node_key > &key.bytes().collect())
            {
                node.pointers.index(idx)
            } else {
                node.pointers.last().unwrap()
            };

            current_node = PageHandler::get_page(&mut self.source, *child_id)?;
        }

        if let PageType::Leaf(ref leaf) = current_node.pagetype {
            let pointer_id = if let Some(idx) = leaf
                .keys
                .iter()
                .position(|leaf_key| leaf_key == &key.bytes().collect::<Vec<u8>>())
            {
                leaf.pointers.index(idx)
            } else {
                // TODO: implement error
                todo!()
            };

            let data = PageHandler::get_page(&mut self.source, *pointer_id)?;
            match data.pagetype {
                PageType::Data(data) => Ok(data),
                _ => todo!(), // TODO: implement error
            }
        } else {
            // TODO: implement error
            todo!()
        }
    }
}
