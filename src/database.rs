use crate::handler::*;
use crate::page::*;

use std::io::{Read, Seek, Write};
use std::ops::Index;

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
        let mut page = match PageHandler::get_page(&mut self.source, self.root.try_into().unwrap())
        {
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
