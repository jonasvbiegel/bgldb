pub mod handler;
pub mod page;
pub use crate::database::page::Data;

use crate::database::handler::*;
use crate::database::page::*;
use std::collections::VecDeque;
use std::io::{Read, Seek, Write};
use std::ops::Index;
use thiserror::Error;

pub struct DatabaseBuilder<T: Read + Write + Seek> {
    source: T,
    key: Vec<u8>,
    keytype: KeyTypeSize,
    order: usize,
}

impl<T: Read + Write + Seek> DatabaseBuilder<T> {
    pub fn new(source: T) -> DatabaseBuilder<T> {
        DatabaseBuilder {
            source,
            key: Vec::new(),
            keytype: KeyTypeSize::UInt64,
            order: 0,
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

    pub fn order(mut self, order: usize) -> DatabaseBuilder<T> {
        self.order = order;
        self
    }

    pub fn build(self) -> Database<T> {
        let mut db = Database {
            source: self.source,
            key: self.key,
            keytype: self.keytype.keytype(),
            keytype_size: self.keytype.size(),
            order: self.order,
            root: 0,
        };

        db.init_header();

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
    String(u8),
    UInt64,
}

impl KeyTypeSize {
    fn size(&self) -> u8 {
        match self {
            KeyTypeSize::String(n) => *n,
            KeyTypeSize::UInt64 => 8,
        }
    }

    fn keytype(&self) -> KeyType {
        match self {
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
    order: usize,
    root: usize,
}

impl<T: Read + Write + Seek> Database<T> {
    pub fn init_header(&mut self) {
        let header = Header {
            elements: 0,
            keytype: self.keytype,
            keytype_size: self.keytype_size,
            key: self.key.clone(),

            // this should be dynamic going forward, determined by keytype size
            order: 4,

            root: self.root.try_into().expect("u64 to usize failure"),
        };

        HeaderHandler::write(&mut self.source, header).expect("couldnt initialize header");

        let leaf = Leaf::new(self.keytype);
        let _ = PageHandler::new_page(&mut self.source, PageType::Leaf(leaf));
    }

    pub fn get_keytype(&mut self) -> Result<KeyType, HandlerError> {
        Ok(HeaderHandler::get(&mut self.source)?.keytype)
    }

    fn get_root(&mut self) -> Result<Page, HandlerError> {
        let root_id = HeaderHandler::get(&mut self.source)?.root;

        PageHandler::get_page(&mut self.source, root_id)
    }

    #[allow(dead_code)]
    pub fn insert(&mut self, data: Data) -> Result<(), HandlerError> {
        if !data.is_valid() {
            // TODO: return error regarding undefined data
            todo!()
        }

        if let Some(field) = data.get_field(&self.key) {
            let mut nodestack = VecDeque::new();

            let mut current_node = self.get_root()?;

            while let PageType::Node(ref node) = current_node.pagetype {
                let child_id = if let Some(idx) =
                    node.keys.iter().position(|node_key| *node_key > field.data)
                {
                    node.pointers.index(idx)
                } else {
                    node.pointers.last().unwrap()
                };

                nodestack.push_front(current_node.id);
                current_node = PageHandler::get_page(&mut self.source, *child_id)?;
            }

            if let PageType::Leaf(mut leaf) = current_node.pagetype {
                if leaf.keys.contains(&field.data) {
                    // TODO: return error, db already contains this key
                    todo!()
                }

                let data_page =
                    PageHandler::new_page(&mut self.source, PageType::Leaf(leaf.clone()))?;

                if let Some(idx) = leaf.keys.iter().position(|leaf_key| *leaf_key > field.data) {
                    leaf.keys.insert(idx, field.data.clone());
                    leaf.pointers.insert(idx, data_page.id);
                } else {
                    leaf.keys.push(field.data.clone());
                    leaf.pointers.push(data_page.id);
                }

                if leaf.pointers.len() > self.order {
                    let split = leaf.split();

                    if let Some(parent) = nodestack.pop_front() {
                        // TODO: split logic, look at slotmap implementation
                        todo!()
                    }
                }

                todo!()
            }
        } else {
            // TODO: return error regarding data not having the required key
            todo!()
        }

        todo!();
    }

    pub fn get(&mut self, key: &[u8]) -> Result<Option<Data>, DatabaseError> {
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
                return Ok(None);
            };

            let data = PageHandler::get_page(&mut self.source, *pointer_id)?;
            match data.pagetype {
                PageType::Data(data) => Ok(Some(data)),
                _ => Err(DatabaseError::UnexpectedPagetype(
                    "data".to_string(),
                    "something else".to_string(),
                )),
            }
        } else {
            Err(DatabaseError::UnexpectedPagetype(
                "leaf".to_string(),
                "node".to_string(),
            ))
        }
    }
}

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("expected {0}, found {1}")]
    UnexpectedPagetype(String, String),

    #[error("handler error: {0}")]
    FileHandlerError(#[from] HandlerError),
}
