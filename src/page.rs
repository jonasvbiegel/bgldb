use nom::Parser;
use nom::multi::{count, length_count};
use nom::number::{Endianness, u8, u64};
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use thiserror::Error;

// NOTE: LITTLE ENDIAN BYTES
const PAGESIZE: u64 = 4096;
type Id = u64;

pub trait PageHandlerFuncs<T: Write + Read + Seek> {
    fn new_page(source: &mut T) -> Result<Page, PageHandlerError>;
    fn get_page(source: &mut T, id: Id) -> Result<Page, PageHandlerError>;
    fn write_page(source: &mut T, page: Page) -> Result<(), PageHandlerError>;
    fn get_header(source: &mut T) -> Result<Header, PageHandlerError>;
    fn write_header(source: &mut T, header: Header) -> Result<(), PageHandlerError>;
}

pub struct PageHandler;

impl<T: Write + Read + Seek> PageHandlerFuncs<T> for PageHandler {
    fn new_page(source: &mut T) -> Result<Page, PageHandlerError> {
        let id = FileHandler::new_page(source)?;
        let page = Page {
            id,
            nodetype: NodeType::Empty,
        };

        FileHandler::write_page(source, id, &page.clone().serialize());

        Ok(page)
    }

    fn get_page(source: &mut T, id: Id) -> Result<Page, PageHandlerError> {
        let page = FileHandler::read_page(source, id)?;

        Ok(Page::deserialize(&page)?)
    }

    fn write_page(source: &mut T, page: Page) -> Result<(), PageHandlerError> {
        FileHandler::write_page(source, page.id, &page.serialize())?;
        Ok(())
    }

    fn get_header(source: &mut T) -> Result<Header, PageHandlerError> {
        let header = FileHandler::read_header(source)?;
        Ok(Header::deserialize(&header)?)
    }
    fn write_header(source: &mut T, header: Header) -> Result<(), PageHandlerError> {
        FileHandler::write_header(source, &header.serialize())?;
        Ok(())
    }
}

impl PageHandler {
    fn get_pagetype(id: Id) -> Result<Data, PageHandlerError> {
        todo!()
    }

    fn get_data(id: Id) -> Result<Data, PageHandlerError> {
        todo!()
    }

    fn write_data(id: Id) -> Result<(), PageHandlerError> {
        todo!()
    }
}

pub trait FileHandlerFuncs<T: Write + Read + Seek> {
    fn new_page(source: &mut T) -> Result<Id, FileError>;
    fn write_page(source: &mut T, id: Id, buf: &[u8]) -> Result<(), FileError>;
    fn write_header(source: &mut T, buf: &[u8]) -> Result<(), FileError>;
    fn read_page(source: &mut T, id: Id) -> Result<Vec<u8>, FileError>;
    fn read_header(source: &mut T) -> Result<Vec<u8>, FileError>;
}

pub struct FileHandler;

impl<T: Write + Read + Seek> FileHandlerFuncs<T> for FileHandler {
    fn new_page(source: &mut T) -> Result<Id, FileError> {
        let id = source.seek(SeekFrom::End(0))?;
        let id_write = source.write(&[0x00; PAGESIZE as usize])?;
        if id_write != PAGESIZE as usize {
            return Err(FileError::WriteBytesExact(id_write));
        }
        Ok((id / PAGESIZE) - 1)
    }

    fn write_page(source: &mut T, id: Id, buf: &[u8]) -> Result<(), FileError> {
        if buf.len() > PAGESIZE as usize {
            return Err(FileError::BiggerBuffer(buf.len()));
        }

        let pos = PAGESIZE + (PAGESIZE * id);
        source.seek(SeekFrom::Start(pos))?;
        source.write_all(buf)?;
        Ok(())
    }

    fn write_header(source: &mut T, buf: &[u8]) -> Result<(), FileError> {
        source.rewind()?;
        source.write_all(buf)?;
        Ok(())
    }

    fn read_page(source: &mut T, id: Id) -> Result<Vec<u8>, FileError> {
        let pos = PAGESIZE + (PAGESIZE * id);
        let mut buf: [u8; PAGESIZE as usize] = [0x00; PAGESIZE as usize];
        source.seek(SeekFrom::Start(pos))?;
        source.read_exact(&mut buf)?;
        Ok(buf.to_vec())
    }

    fn read_header(source: &mut T) -> Result<Vec<u8>, FileError> {
        let mut buf: [u8; PAGESIZE as usize] = [0x00; PAGESIZE as usize];
        source.rewind()?;
        source.read_exact(&mut buf)?;
        Ok(buf.to_vec())
    }
}

pub trait SerializeDeserialize: Sized {
    fn serialize(self) -> Vec<u8>;
    fn deserialize(bytes: &[u8]) -> Result<Self, FileError>;
}

#[derive(Debug)]
pub struct Header {
    pub elements: u64,
    pub keytype: KeyType,
    pub keytype_size: u8,
    pub root: Id,
    pub order: u8,
}

impl SerializeDeserialize for Header {
    fn deserialize(bytes: &[u8]) -> Result<Header, FileError> {
        if bytes.len() != PAGESIZE as usize {
            return Err(FileError::Pagesize(PAGESIZE as usize, bytes.len()));
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
            _ => return Err(FileError::Keytype(keytype)),
        };

        Ok(Header {
            elements,
            keytype,
            keytype_size,
            root,
            order,
        })
    }

    fn serialize(self) -> Vec<u8> {
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

const PAGESIZE_NO_HEADER: usize = PAGESIZE as usize - ID_SIZE - NODETYPE_SIZE;

#[derive(Debug, Clone)]
pub struct Page {
    id: Id,
    nodetype: NodeType,
}

impl SerializeDeserialize for Page {
    fn deserialize(bytes: &[u8]) -> Result<Page, FileError> {
        if bytes.len() != PAGESIZE as usize {
            return Err(FileError::Pagesize(PAGESIZE as usize, bytes.len()));
        }

        let (input, (id, nodetype)) = (u64(Endianness::Little), u8()).parse(bytes)?;

        let nodetype = match nodetype {
            0x01 => NodeType::Node(Node::deserialize(input)?),
            0x02 => NodeType::Leaf(Leaf::deserialize(input)?),
            0x03 => NodeType::Data(Data::deserialize(input)?),
            0x04 => NodeType::Empty,
            _ => return Err(FileError::Keytype(nodetype)),
        };

        Ok(Page { id, nodetype })
    }

    fn serialize(self) -> Vec<u8> {
        let mut b: Vec<u8> = Vec::new();

        self.id.to_le_bytes().iter().for_each(|byte| b.push(*byte));

        match self.nodetype {
            NodeType::Node(node) => node.serialize().iter().for_each(|byte| b.push(*byte)),
            NodeType::Leaf(leaf) => leaf.serialize().iter().for_each(|byte| b.push(*byte)),
            NodeType::Data(data) => data.serialize().iter().for_each(|byte| b.push(*byte)),
            NodeType::Empty => b.push(0x04),
        }

        b
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    keytype: KeyType,
    keys: Vec<Vec<u8>>,
    pointers: Vec<u64>,
}

impl Node {
    fn new(keytype: KeyType) -> Node {
        Node {
            keytype,
            keys: Vec::new(),
            pointers: Vec::new(),
        }
    }
}

impl SerializeDeserialize for Node {
    fn deserialize(bytes: &[u8]) -> Result<Node, FileError> {
        if bytes.len() != PAGESIZE_NO_HEADER {
            return Err(FileError::Pagesize(PAGESIZE_NO_HEADER, bytes.len()));
        }

        let (input, (keytype, keys_len)) = (u8(), u8()).parse(bytes)?;

        let keytype = match keytype {
            0x01 => KeyType::String,
            0x02 => KeyType::UInt64,
            _ => return Err(FileError::Keytype(keytype)),
        };

        let (input, keys) = match keytype {
            KeyType::String => count(length_count(u8(), u8()), keys_len as usize).parse(input)?,
            KeyType::UInt64 => count(count(u8(), 8), keys_len as usize).parse(input)?,
        };

        let (_input, pointers) =
            count(u64(Endianness::Little), keys_len as usize + 1).parse(input)?;

        Ok(Node {
            keytype,
            keys,
            pointers,
        })
    }

    fn serialize(self) -> Vec<u8> {
        let mut b: Vec<u8> = Vec::new();

        b.push(0x01);

        usize::to_le_bytes(self.keys.len())
            .iter()
            .for_each(|byte| b.push(*byte));

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

        self.pointers
            .iter()
            .for_each(|p| p.to_le_bytes().iter().for_each(|byte| b.push(*byte)));

        b
    }
}

#[derive(Debug, Clone)]
pub struct Leaf {
    keytype: KeyType,
    keys: Vec<Vec<u8>>,
    pointers: Vec<u64>,
    next_leaf_pointer: u8,
}

impl Leaf {
    fn from_node(node: Node) -> Leaf {
        Leaf {
            keytype: node.keytype,
            keys: node.keys,
            pointers: node.pointers,
            next_leaf_pointer: 0x00,
        }
    }

    fn set_next_leaf_pointer(&mut self, pointer: u8) {
        self.next_leaf_pointer = pointer
    }
}

impl SerializeDeserialize for Leaf {
    fn serialize(self) -> Vec<u8> {
        let mut v = Node::serialize(Node {
            keytype: self.keytype,
            keys: self.keys,
            pointers: self.pointers,
        });

        v.push(self.next_leaf_pointer);

        v
    }

    fn deserialize(bytes: &[u8]) -> Result<Leaf, FileError> {
        if bytes.len() != PAGESIZE_NO_HEADER {
            return Err(FileError::Pagesize(PAGESIZE_NO_HEADER, bytes.len()));
        }

        let (input, (keytype, keys_len)) = (u8(), u8()).parse(bytes)?;

        let keytype = match keytype {
            0x01 => KeyType::String,
            0x02 => KeyType::UInt64,
            _ => return Err(FileError::Keytype(keytype)),
        };

        let (input, keys) = match keytype {
            KeyType::String => count(length_count(u8(), u8()), keys_len as usize).parse(input)?,
            KeyType::UInt64 => count(count(u8(), 8), keys_len as usize).parse(input)?,
        };

        let (input, pointers) =
            count(u64(Endianness::Little), keys_len as usize + 1).parse(input)?;

        let (_input, next_leaf_pointer) = u8().parse(input)?;

        Ok(Leaf {
            keytype,
            keys,
            pointers,
            next_leaf_pointer,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Data {
    extend: u64,
    objects: Vec<Vec<Field>>,
}

impl SerializeDeserialize for Data {
    fn serialize(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.extend
            .to_le_bytes()
            .iter()
            .for_each(|b| bytes.push(*b));

        for object in self.objects {
            bytes.push(object.len().try_into().expect("couldnt parse object len"));
            for field in object {
                let f = Field::serialize(field);
                bytes.push(f.len().try_into().expect("couldnt parse field len"));
                f.iter().for_each(|b| bytes.push(*b));
            }
        }

        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Data, FileError> {
        todo!()
    }
}

// this should not exist
#[derive(Debug)]
pub struct DataExtend {
    extend: u64,
    data: Vec<u8>,
}

impl SerializeDeserialize for DataExtend {
    fn serialize(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.extend
            .to_le_bytes()
            .iter()
            .for_each(|b| bytes.push(*b));

        self.data.iter().for_each(|b| bytes.push(*b));

        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<DataExtend, FileError> {
        let (_, extend) = (u64(Endianness::Little)).parse(bytes)?;

        let mut data: Vec<u8> = Vec::new();
        let mut cursor = Cursor::new(bytes);
        cursor.seek(SeekFrom::Start(size_of::<u64>().try_into()?))?;
        cursor.read_to_end(&mut data)?;

        match data.len() == PAGESIZE as usize - size_of::<u64>() {
            true => Ok(DataExtend { data, extend }),
            false => Err(FileError::Pagesize(
                data.len(),
                PAGESIZE as usize - size_of::<u64>(),
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    keysize: u8,
    key: Vec<u8>,
    datasize: u8,
    data: Vec<u8>,
}

impl SerializeDeserialize for Field {
    fn serialize(self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.push(self.keysize);
        self.key.iter().for_each(|b| bytes.push(*b));

        bytes.push(self.datasize);
        self.data.iter().for_each(|b| bytes.push(*b));

        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Field, FileError> {
        let (input, keysize) = u8().parse(bytes)?;
        let (input, key) = count(u8(), keysize as usize).parse(input)?;

        let (input, datasize) = u8().parse(input)?;
        let (_, data) = count(u8(), datasize as usize).parse(input)?;

        Ok(Field {
            keysize,
            key,
            datasize,
            data,
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub enum KeyType {
    String, // 0x01
    UInt64, // 0x02

            // Int64,   // 0x03
            // Float64, // 0x04,
            // Float32, // 0x05
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Node(Node), // 0x01
    Leaf(Leaf), // 0x02
    Data(Data), // 0x03
    Empty,      //0x04
}

#[derive(Error, Debug)]
pub enum FileError {
    #[error("page was not the correct size (expected {0}, found {1})")]
    Pagesize(usize, usize),

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

    #[error("couldnt convert to int ({0})")]
    TryFromInt(#[from] std::num::TryFromIntError),

    #[error("nom failed parsing")]
    Nom(nom::Err<nom::error::Error<Vec<u8>>>),
}

impl From<nom::Err<nom::error::Error<&[u8]>>> for FileError {
    fn from(err: nom::Err<nom::error::Error<&[u8]>>) -> Self {
        Self::Nom(err.map_input(|input| input.to_vec()))
    }
}

#[derive(Error, Debug)]
pub enum PageHandlerError {
    #[error("file handler error: {0}")]
    FileHandler(#[from] FileError),
}
