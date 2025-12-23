pub mod handler;
pub mod page;

use crate::database::handler::*;
use crate::database::page::*;
use std::io::{Read, Seek, Write};
use std::ops::Index;

pub struct DatabaseBuilder<T: Read + Write + Seek> {
    source: T,
    key: Vec<u8>,
    keytype: KeyTypeSize,
}

impl<T: Read + Write + Seek> DatabaseBuilder<T> {
    pub fn new(source: T) -> DatabaseBuilder<T> {
        DatabaseBuilder {
            source,
            key: Vec::new(),
            keytype: KeyTypeSize::Identity,
        }
    }

    pub fn key(mut self, key: Vec<u8>) -> DatabaseBuilder<T> {
        self.key = key;
        self
    }

    pub fn keytype(mut self, keytype: KeyTypeSize) -> DatabaseBuilder<T> {
        self.keytype = keytype;
        self
    }

    pub fn build(self) -> Database<T> {
        let mut db = Database {
            source: self.source,
            key: self.key,
            keytype: self.keytype.keytype(),
            keytype_size: self.keytype.size(),
            root: 0,
        };

        db.init();

        db
    }

    pub fn build_mock_u64(self) -> Database<T> {
        let mut db = self.build();

        let node = PageType::Node(Node {
            keytype: KeyType::UInt64,
            keys: vec![
                3_usize.to_le_bytes().to_vec(),
                5_usize.to_le_bytes().to_vec(),
            ],
            pointers: vec![1, 2, 3],
        });

        let leaf1 = PageType::Leaf(Leaf {
            keytype: KeyType::UInt64,
            keys: vec![
                1_usize.to_le_bytes().to_vec(),
                2_usize.to_le_bytes().to_vec(),
            ],
            pointers: vec![4, 5],
            next_leaf_pointer: 0,
        });

        let leaf2 = PageType::Leaf(Leaf {
            keytype: KeyType::UInt64,
            keys: vec![
                3_usize.to_le_bytes().to_vec(),
                4_usize.to_le_bytes().to_vec(),
            ],
            pointers: vec![6, 7],
            next_leaf_pointer: 0,
        });

        let leaf3 = PageType::Leaf(Leaf {
            keytype: KeyType::UInt64,
            keys: vec![
                5_usize.to_le_bytes().to_vec(),
                6_usize.to_le_bytes().to_vec(),
            ],
            pointers: vec![8, 9],
            next_leaf_pointer: 0,
        });

        let data1 = PageType::Data(Data {
            object: vec![
                Field::new(
                    b"id".to_vec(),
                    KeyType::UInt64,
                    1_usize.to_le_bytes().to_vec(),
                ),
                Field::new(b"name".to_vec(), KeyType::String, b"jonas".to_vec()),
                Field::new(
                    b"age".to_vec(),
                    KeyType::UInt64,
                    22_usize.to_le_bytes().to_vec(),
                ),
                Field::new(
                    b"weight".to_vec(),
                    KeyType::UInt64,
                    87_usize.to_le_bytes().to_vec(),
                ),
            ],
        });

        let data2 = PageType::Data(Data {
            object: vec![
                Field::new(
                    b"id".to_vec(),
                    KeyType::UInt64,
                    2_usize.to_le_bytes().to_vec(),
                ),
                Field::new(b"cpr".to_vec(), KeyType::String, b"0101009999".to_vec()),
                Field::new(b"name".to_vec(), KeyType::String, b"thea".to_vec()),
                Field::new(
                    b"age".to_vec(),
                    KeyType::UInt64,
                    25_usize.to_le_bytes().to_vec(),
                ),
            ],
        });

        let data3 = PageType::Data(Data {
            object: vec![
                Field::new(
                    b"id".to_vec(),
                    KeyType::UInt64,
                    3_usize.to_le_bytes().to_vec(),
                ),
                Field::new(b"name".to_vec(), KeyType::String, b"dam".to_vec()),
                Field::new(
                    b"age".to_vec(),
                    KeyType::UInt64,
                    300_usize.to_le_bytes().to_vec(),
                ),
            ],
        });

        let data4 = PageType::Data(Data {
            object: vec![
                Field::new(
                    b"id".to_vec(),
                    KeyType::UInt64,
                    4_usize.to_le_bytes().to_vec(),
                ),
                Field::new(b"name".to_vec(), KeyType::String, b"lars".to_vec()),
                Field::new(
                    b"age".to_vec(),
                    KeyType::UInt64,
                    55_usize.to_le_bytes().to_vec(),
                ),
            ],
        });

        let data5 = PageType::Data(Data {
            object: vec![
                Field::new(
                    b"id".to_vec(),
                    KeyType::UInt64,
                    5_usize.to_le_bytes().to_vec(),
                ),
                Field::new(b"name".to_vec(), KeyType::String, b"john".to_vec()),
                Field::new(
                    b"age".to_vec(),
                    KeyType::UInt64,
                    55_usize.to_le_bytes().to_vec(),
                ),
            ],
        });

        let data6 = PageType::Data(Data {
            object: vec![
                Field::new(
                    b"id".to_vec(),
                    KeyType::UInt64,
                    6_usize.to_le_bytes().to_vec(),
                ),
                Field::new(b"name".to_vec(), KeyType::String, b"hans".to_vec()),
                Field::new(
                    b"age".to_vec(),
                    KeyType::UInt64,
                    55_usize.to_le_bytes().to_vec(),
                ),
            ],
        });

        // println!("{leaf:#?}");

        let mut root = db.get_root().unwrap();
        root.pagetype = node;
        PageHandler::write(&mut db.source, root).unwrap();

        let _ = PageHandler::new_page(&mut db.source, leaf1);
        let _ = PageHandler::new_page(&mut db.source, leaf2);
        let _ = PageHandler::new_page(&mut db.source, leaf3);
        let _ = PageHandler::new_page(&mut db.source, data1);
        let _ = PageHandler::new_page(&mut db.source, data2);
        let _ = PageHandler::new_page(&mut db.source, data3);
        let _ = PageHandler::new_page(&mut db.source, data4);
        let _ = PageHandler::new_page(&mut db.source, data5);
        let _ = PageHandler::new_page(&mut db.source, data6);

        db
    }
}

pub enum KeyTypeSize {
    Identity,
    String(u8),
    UInt64,
}

impl KeyTypeSize {
    fn size(&self) -> u8 {
        match self {
            KeyTypeSize::Identity => 8,
            KeyTypeSize::String(n) => *n,
            KeyTypeSize::UInt64 => 8,
        }
    }

    fn keytype(&self) -> KeyType {
        match self {
            KeyTypeSize::Identity => KeyType::UInt64,
            KeyTypeSize::String(_) => KeyType::String,
            KeyTypeSize::UInt64 => KeyType::UInt64,
        }
    }
}

pub struct Database<T: Read + Write + Seek> {
    pub source: T,
    key: Vec<u8>,
    keytype: KeyType,
    keytype_size: u8,
    root: usize,
}

impl<T: Read + Write + Seek> Database<T> {
    pub fn new(source: T, key: &str, keytype: KeyType, keytype_size: u8) -> Database<T> {
        Database {
            source,
            key: key.bytes().collect(),
            keytype,
            keytype_size,
            root: 0,
        }
    }

    pub fn init(&mut self) {
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

        PageHandler::get_page(&mut self.source, root_id)
    }

    fn insert_key(&mut self, key: Vec<u8>) -> Data {
        todo!()
    }

    // takes data in the future
    fn insert(&mut self, key: &str) -> bool {
        let mut page = match dbg!(PageHandler::get_page(
            &mut self.source,
            self.root.try_into().unwrap()
        )) {
            Ok(page) => page,
            Err(e) => {
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

    pub fn get(&mut self, key: &[u8]) -> Result<Data, HandlerError> {
        let mut current_node = self.get_root().expect("couldnt get root");

        while let PageType::Node(ref node) = current_node.pagetype {
            let child_id = if let Some(idx) = node
                .keys
                .iter()
                .position(|node_key| *node_key > key.to_vec())
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
                .position(|leaf_key| *leaf_key == key.to_vec())
            {
                leaf.pointers.index(idx)
            } else {
                // TODO: implement error
                println!("FUCK");
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
