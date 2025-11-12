use nom::Parser;
use nom::multi::{count, length_count};
use nom::number::{Endianness, u8, u64};
use std::io::{Read, Seek, SeekFrom, Write};
use thiserror::Error;

// NOTE: LITTLE ENDIAN BYTES
const PAGESIZE: u64 = 4096;
type Id = u64;

pub trait Pageable<T: Write + Read + Seek> {
    fn new_page(source: &mut T) -> Result<Id, PageError>;
    fn write_to_page(source: &mut T, id: Id, buf: &[u8]) -> Result<(), PageError>;
    fn write_to_header(source: &mut T, buf: &[u8]) -> Result<(), PageError>;
    fn read_page(source: &mut T, id: Id) -> Result<Vec<u8>, PageError>;
    fn read_header(source: &mut T) -> Result<Vec<u8>, PageError>;
}

pub struct PageHandler;

impl<T: Write + Read + Seek> Pageable<T> for PageHandler {
    fn new_page(source: &mut T) -> Result<Id, PageError> {
        let id = source.seek(SeekFrom::End(0))?;
        let w = source.write(&[0x00; PAGESIZE as usize])?;
        if w != PAGESIZE as usize {
            return Err(PageError::WriteBytesExact(w));
        }
        Ok((id / PAGESIZE) - 1)
    }

    fn write_to_page(source: &mut T, id: Id, buf: &[u8]) -> Result<(), PageError> {
        if buf.len() > PAGESIZE as usize {
            return Err(PageError::BiggerBuffer(buf.len()));
        }

        let pos = PAGESIZE + (PAGESIZE * id);
        source.seek(SeekFrom::Start(pos))?;
        source.write_all(buf)?;
        Ok(())
    }

    fn write_to_header(source: &mut T, buf: &[u8]) -> Result<(), PageError> {
        source.rewind()?;
        source.write_all(buf)?;
        Ok(())
    }

    fn read_page(source: &mut T, id: Id) -> Result<Vec<u8>, PageError> {
        let pos = PAGESIZE + (PAGESIZE * id);
        let mut buf: [u8; PAGESIZE as usize] = [0x00; PAGESIZE as usize];
        source.seek(SeekFrom::Start(pos))?;
        source.read_exact(&mut buf)?;
        Ok(buf.to_vec())
    }

    fn read_header(source: &mut T) -> Result<Vec<u8>, PageError> {
        let mut buf: [u8; PAGESIZE as usize] = [0x00; PAGESIZE as usize];
        source.rewind()?;
        source.read_exact(&mut buf)?;
        Ok(buf.to_vec())
    }
}

#[derive(Debug)]
pub struct Header {
    pub elements: u64,
    pub keytype: KeyType,
    pub keytype_size: u8,
    pub root: Id,
    pub order: u8,
}

impl Header {
    pub fn deserialize(bytes: &[u8]) -> Result<Header, PageError> {
        if bytes.len() != PAGESIZE as usize {
            return Err(PageError::Pagesize(bytes.len()));
        }

        let (_, (elements, keytype, keytype_size, root, order)) = (
            u64(Endianness::Little),
            u8(),
            u8(),
            u64(Endianness::Little),
            u8(),
        )
            .parse(bytes)?;

        let keytype = match keytype {
            0x01 => KeyType::String,
            0x02 => KeyType::UInt64,
            _ => return Err(PageError::Keytype(keytype)),
        };

        Ok(Header {
            elements,
            keytype,
            keytype_size,
            root,
            order,
        })
    }

    pub fn serialize(&mut self) -> Vec<u8> {
        let mut b = Vec::<u8>::new();

        for (idx, byte) in self.elements.to_le_bytes().iter().enumerate() {
            b.insert(idx, *byte);
        }

        match self.keytype {
            KeyType::String => {
                b.push(0x01);
                b.push(self.keytype_size);
            }
            KeyType::UInt64 => {
                b.push(0x02);
                b.push(self.keytype_size);
            }
        }

        for byte in self.root.to_le_bytes() {
            b.push(byte);
        }

        b.push(self.order);

        b
    }
}

const ID_SIZE: usize = size_of::<u64>();
const NODETYPE_SIZE: usize = size_of::<u8>();
pub struct Page {
    id: Id,
    nodetype: NodeType,
}

impl Page {
    pub fn deserialize(bytes: &[u8]) -> Result<Page, PageError> {
        if bytes.len() != PAGESIZE as usize {
            return Err(PageError::Pagesize(bytes.len()));
        }

        let (input, (id, nodetype)) = (u64(Endianness::Little), u8()).parse(bytes)?;

        let nodetype = match nodetype {
            0x01 => NodeType::Node(Node::deserialize(input)?),
            0x02 => NodeType::Leaf(Leaf::deserialize(input)?),
            0x03 => NodeType::Data(Data::deserialize(input)?),
            _ => return Err(PageError::Keytype(nodetype)),
        };

        Ok(Page { id, nodetype })
    }

    pub fn serialize(self) -> Vec<u8> {
        let mut b: Vec<u8> = Vec::new();

        self.id.to_le_bytes().iter().for_each(|byte| b.push(*byte));

        match self.nodetype {
            NodeType::Node(node) => node.serialize().iter().for_each(|byte| b.push(*byte)),
            NodeType::Leaf(leaf) => leaf.serialize().iter().for_each(|byte| b.push(*byte)),
            NodeType::Data(data) => data.serialize().iter().for_each(|byte| b.push(*byte)),
        }

        b
    }
}

#[derive(Debug)]
pub struct Node {
    keytype: KeyType,
    keys_len: u8,
    keys: Vec<Vec<u8>>, //
    pointers: Vec<u64>, //
}

impl Node {
    pub fn deserialize(bytes: &[u8]) -> Result<Node, PageError> {
        if bytes.len() != PAGESIZE as usize - ID_SIZE - NODETYPE_SIZE {
            return Err(PageError::Pagesize(bytes.len()));
        }

        let (input, (keytype, keys_len)) = (u8(), u8()).parse(bytes)?;

        let keytype = match keytype {
            0x01 => KeyType::String,
            0x02 => KeyType::UInt64,
            _ => return Err(PageError::Keytype(keytype)),
        };

        let (input, keys) = match keytype {
            KeyType::String => count(length_count(u8(), u8()), keys_len as usize).parse(input)?,
            KeyType::UInt64 => count(count(u8(), 8), keys_len as usize).parse(input)?,
        };

        let (_input, pointers) =
            count(u64(Endianness::Little), keys_len as usize + 1).parse(input)?;

        Ok(Node {
            keytype,
            keys_len,
            keys,
            pointers,
        })
    }

    pub fn serialize(self) -> Vec<u8> {
        let mut b: Vec<u8> = Vec::new();

        b.push(0x01);

        match self.keytype {
            KeyType::String => {
                b.push(0x01);
                for key in self.keys {
                    b.push(key.len() as u8);
                    key.iter().for_each(|x| b.push(*x));
                }
            }
            KeyType::UInt64 => {
                b.push(0x02);
                for key in self.keys {
                    key.iter().for_each(|x| b.push(*x));
                }
            }
        }

        b.push(self.keys_len);

        self.pointers
            .iter()
            .for_each(|p| p.to_le_bytes().iter().for_each(|byte| b.push(*byte)));

        b
    }
}

#[derive(Debug)]
pub struct Leaf {
    keytype: KeyType,
    keys_len: u8,
    keys: Vec<Vec<u8>>,
    pointers: Vec<u64>,
    next_leaf_pointer: u8,
}

impl Leaf {
    pub fn serialize(self) -> Vec<u8> {
        todo!()
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Leaf, PageError> {
        todo!()
    }
}

// id   fields_len | field_n_len field_n
// Id   u16          u16         field

#[derive(Debug)]
pub struct Data {
    id: Id,
    fields: Vec<Field>,
}

impl Data {
    fn serialize(self) -> Vec<u8> {
        todo!()
    }

    fn deserialize(bytes: &[u8]) -> Result<Data, PageError> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Field {
    keysize: u8,
    key: Vec<u8>,
    datasize: u8,
    data: Vec<u8>,
}

#[derive(Debug)]
pub enum KeyType {
    String, // 0x01
    UInt64, // 0x02
}

#[derive(Debug)]
pub enum NodeType {
    Node(Node), // 0x01
    Leaf(Leaf), // 0x02
    Data(Data), //0x03
}

#[derive(Error, Debug)]
pub enum PageError {
    #[error("page was not 4096 bytes ({0})")]
    Pagesize(usize),

    #[error("keytype could not be parsed ({0})")]
    Keytype(u8),

    #[error("node type was not correct")]
    NodeType(u8),

    #[error("did not write exact bytes ({0})")]
    WriteBytesExact(usize),

    #[error("buffer was bigger than pagesize ({0})")]
    BiggerBuffer(usize),

    #[error("failed to read or write from file: ({0})")]
    Io(#[from] std::io::Error),

    #[error("nom failed parsing")]
    Nom(nom::Err<nom::error::Error<Vec<u8>>>),
}

impl From<nom::Err<nom::error::Error<&[u8]>>> for PageError {
    fn from(err: nom::Err<nom::error::Error<&[u8]>>) -> Self {
        Self::Nom(err.map_input(|input| input.to_vec()))
    }
}
