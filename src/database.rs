use nom::multi::{count, length_count};
use nom::number::{Endianness, u8, u64};
use nom::{IResult, Parser};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::vec;
use thiserror::Error;

// NOTE: LITTLE ENDIAN BYTES
const PAGESIZE: u64 = 4096;
type Id = u64;

pub struct Database {
    file: File,
}

impl Database {
    pub fn new(name: &str) -> Result<Self, DatabaseError> {
        let mut h = Self {
            file: OpenOptions::new()
                .create(true)
                .truncate(false)
                .write(true)
                .read(true)
                .open(format!("./{name}"))
                .unwrap(),
        };

        if h.file.metadata()?.len() == 0 {
            h.write(&[0x00; PAGESIZE as usize])?;
        }

        Ok(h)
    }

    pub fn new_test(name: &str) -> Result<Self, DatabaseError> {
        let mut h = Self {
            file: OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .read(true)
                .open(format!("test/{name}"))
                .unwrap(),
        };

        if h.file.metadata()?.len() == 0 {
            h.write(&[0x00; PAGESIZE as usize])?;
        }

        Ok(h)
    }

    pub fn new_page(&mut self) -> Result<Id, DatabaseError> {
        let id = self.file.seek(SeekFrom::End(0))?;
        self.write(&[0x00; PAGESIZE as usize])?;
        Ok((id / PAGESIZE) - 1)
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<(), DatabaseError> {
        self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(buf)?;
        Ok(())
    }

    pub fn write_to_page(&mut self, id: Id, buf: &[u8]) -> Result<bool, DatabaseError> {
        if id <= self.get_max_id()? {
            let pos = PAGESIZE + (PAGESIZE * id);
            self.file.seek(SeekFrom::Start(pos))?;
            self.file.write_all(buf)?;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn write_to_header(&mut self, buf: &[u8]) -> Result<(), DatabaseError> {
        self.file.rewind()?;
        self.file.write_all(buf)?;
        Ok(())
    }

    pub fn read_all(&mut self) -> Result<Vec<u8>, DatabaseError> {
        let mut buf = Vec::new();
        self.file.rewind()?;
        self.file.read_to_end(&mut buf)?;
        Ok(buf)
    }

    pub fn read_page(&mut self, id: Id) -> Result<Vec<u8>, DatabaseError> {
        let pos = PAGESIZE + (PAGESIZE * id);
        let mut buf: [u8; PAGESIZE as usize] = [0x00; PAGESIZE as usize];
        self.file.seek(SeekFrom::Start(pos))?;
        self.file.read_exact(&mut buf)?;
        Ok(buf.to_vec())
    }

    pub fn read_header(&mut self) -> Result<Vec<u8>, DatabaseError> {
        let mut buf: [u8; PAGESIZE as usize] = [0x00; PAGESIZE as usize];
        self.file.rewind()?;
        self.file.read_exact(&mut buf)?;
        Ok(buf.to_vec())
    }

    pub fn get_max_id(&mut self) -> Result<u64, DatabaseError> {
        let len = self.file.metadata()?.len();
        Ok((len / PAGESIZE) - 1)
    }
}

#[derive(Debug)]
pub struct Header {
    pub elements: u64,
    pub keytype: KeyType,
    pub keytype_size: u8,
}

impl Header {
    pub fn deserialize(bytes: &[u8]) -> Result<Header, DatabaseError> {
        if bytes.len() != PAGESIZE as usize {
            return Err(DatabaseError::Pagesize(bytes.len()));
        }

        let header = || -> IResult<&[u8], Header> {
            let (input, (elements, keytype, keytype_size)) =
                (u64(Endianness::Little), u8(), u8()).parse(bytes)?;

            let keytype = match keytype {
                0x01 => KeyType::String,
                0x02 => KeyType::UInt64,
                _ => KeyType::Undefined(keytype),
            };

            Ok((
                input,
                Header {
                    elements,
                    keytype,
                    keytype_size,
                },
            ))
        };

        match header() {
            Ok((_, h)) => match h.keytype {
                KeyType::Undefined(b) => return Err(DatabaseError::Keytype(b)),
                _ => Ok(h),
            },
            Err(_) => return Err(DatabaseError::Nom),
        }
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
            _ => {}
        }

        b.to_vec()
    }
}

// NOTE: can maybe hold more information in the future
//
// id    | pagetype | keys_len | keys(n)      | pointers(n + 1)
// usize | u8       | u16      | Vec<Vec<u8>> | Vec<u64>
//
// n = ( PAGESIZE - id(16 bytes) - pagetype(1 byte) - keys_len(2 bytes) ) / sizeof(type)

#[derive(Debug)]
pub struct Node {
    id: Id,             // 0..=7
    nodetype: NodeType, // 8
    keytype: KeyType,
    keys_len: u8,
    keys: Vec<Vec<u8>>, // n
    pointers: Vec<u64>, // n + 1
}

impl Node {
    pub fn deserialize(bytes: &[u8]) -> Result<Node, DatabaseError> {
        if bytes.len() != PAGESIZE as usize {
            return Err(DatabaseError::Pagesize(bytes.len()));
        }

        let node = || -> IResult<&[u8], Node> {
            let (input, (id, nodetype, keytype, keys_len)) =
                (u64(Endianness::Little), u8(), u8(), u8()).parse(bytes)?;

            let nodetype = match nodetype {
                0x01 => NodeType::Root,
                0x02 => NodeType::Node,
                0x03 => NodeType::Leaf,
                _ => NodeType::Undefined(nodetype),
            };

            let keytype = match keytype {
                0x01 => KeyType::String,
                0x02 => KeyType::UInt64,
                _ => KeyType::Undefined(keytype),
            };

            let (input, keys) = match keytype {
                KeyType::String => {
                    count(length_count(u8(), u8()), keys_len as usize).parse(input)?
                }
                KeyType::UInt64 => count(count(u8(), 8), keys_len as usize).parse(input)?,
                _ => (input, vec![vec![0x00]]),
            };

            let (_input, pointers) =
                count(u64(Endianness::Little), keys_len as usize + 1).parse(input)?;

            Ok((
                input,
                Node {
                    id,
                    nodetype,
                    keytype,
                    keys_len,
                    keys,
                    pointers,
                },
            ))
        };

        match node() {
            Ok((_, n)) => {
                if let KeyType::Undefined(b) = n.keytype {
                    return Err(DatabaseError::Keytype(b));
                }
                if let NodeType::Undefined(b) = n.nodetype {
                    return Err(DatabaseError::NodeType(b));
                }

                Ok(n)
            }
            Err(_) => return Err(DatabaseError::Nom),
        }
    }

    pub fn serialize(self) -> Vec<u8> {
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

    fn deserialize(bytes: &[u8]) -> Data {
        todo!()
    }
}

// keytype      field_len   field       primary     data_len    data
// keytype      u8         string       bool        u16         [u8]

#[derive(Debug)]
struct Field {
    field: Vec<u8>,
    keytype: KeyType,
    data: Vec<u8>,
}

impl Field {
    fn new(field: &str, keytype: KeyType, data: Vec<u8>) -> Field {
        Self {
            field: field.as_bytes().into(),
            keytype,
            data,
        }
    }

    // fn serialize(self) -> Result<Vec<u8>, DatabaseError> {
    //     let mut vec: Vec<u8> = Vec::new();
    //
    //     let len = match self.keytype {
    //         KeyType::String(n) => {
    //             vec.push(0x01);
    //             if n as usize > size_of::<u8>() {
    //                 return Err(DatabaseError::Fieldsize(n as usize));
    //             }
    //             n
    //         }
    //         KeyType::UInt64 => {
    //             vec.push(0x02);
    //             size_of::<u64>().try_into().unwrap()
    //         }
    //     };
    //
    //     vec.push(len);
    //
    //     for b in self.field {
    //         vec.push(b);
    //     }
    //
    //     u16::to_le_bytes(self.data.len() as u16)
    //         .iter()
    //         .for_each(|b| vec.push(*b));
    //
    //     self.data.iter().for_each(|b| vec.push(*b));
    //
    //     Ok(vec)
    // }

    // keytype      field_len   field       data_len    data
    // keytype      u8         string       u32         [u8]

    // this should just use nom, because this is getting stupid at this point
    // fn deserialize(bytes: &[u8]) -> Result<Field, DatabaseError> {
    //     let keytype = match bytes.index(0) {
    //         0x01 => (KeyType::String, *bytes.index(1)),
    //         0x02 => (KeyType::UInt64, *bytes.index(1)),
    //         _ => return Err(DatabaseError::Keytype(*bytes.index(0))),
    //     };
    //
    //     let field_len = bytes.index(2);
    //     let field = &bytes[3..(*field_len + 3)
    //         .try_into()
    //         .expect("couldnt parse field_len")];
    //     let data_len = bytes.index(*field_len as usize + 4);
    //     let data = &bytes[*field_len as usize + 5..*data_len as usize + *field_len as usize + 5]; // what
    //     // is this?
    //
    //     Ok(Self {
    //         field: field.to_vec(),
    //         keytype,
    //         data: data.to_vec(),
    //     })
    // }
}

pub struct DataBuilder {
    id: Id,
    fields: Vec<Field>,
}

impl DataBuilder {
    pub fn new(id: Id) -> DataBuilder {
        Self {
            id,
            fields: Vec::new(),
        }
    }

    pub fn field(
        mut self,
        name: &str,
        keytype: KeyType,
        data: Vec<u8>,
    ) -> Result<DataBuilder, DatabaseError> {
        self.fields.push(Field::new(name, keytype, data));
        Ok(self)
    }

    pub fn build(self) -> Data {
        Data {
            id: self.id,
            fields: self.fields,
        }
    }
}

#[derive(Debug)]
pub enum KeyType {
    String,        // 0x01
    UInt64,        // 0x02
    Undefined(u8), // error
}

#[derive(Debug)]
pub enum NodeType {
    Root,          // 0x01
    Node,          // 0x02
    Leaf,          // 0x03
    Undefined(u8), // error
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("page was not 4096 bytes ({0})")]
    Pagesize(usize),

    #[error("keytype could not be parsed ({0})")]
    Keytype(u8),

    #[error("node type was not correct")]
    NodeType(u8),

    #[error("data already contains a primary key ({0})")]
    Fieldname(String),

    #[error("field was too big ({0})")]
    Fieldsize(usize),

    #[error("failed to read or write from file: ({0})")]
    Io(#[from] std::io::Error),

    #[error("parsing failed with nom")]
    Nom,
}
