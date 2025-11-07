use std::fs::{File, OpenOptions};
use std::io::{Error, Read, Seek, SeekFrom, Write};
use std::ops::Index;
use thiserror::Error;

// NOTE: LITTLE ENDIAN BYTES
const PAGESIZE: u64 = 4096;
type Id = u64;

pub struct Database {
    file: File,
}

impl Database {
    pub fn new(name: &str) -> Result<Self, Error> {
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

    pub fn new_test(name: &str) -> Result<Self, Error> {
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

    pub fn new_page(&mut self) -> Result<Id, Error> {
        let id = self.file.seek(SeekFrom::End(0))?;
        self.write(&[0x00; PAGESIZE as usize])?;
        Ok((id / PAGESIZE) - 1)
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(buf)?;
        Ok(())
    }

    pub fn write_to_page(&mut self, id: Id, buf: &[u8]) -> Result<bool, Error> {
        if id <= self.get_max_id()? {
            let pos = PAGESIZE + (PAGESIZE * id);
            self.file.seek(SeekFrom::Start(pos))?;
            self.file.write_all(buf)?;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn write_to_header(&mut self, buf: &[u8]) -> Result<(), Error> {
        // self.file.seek(SeekFrom::Start(0))?;
        self.file.rewind()?;
        self.file.write_all(buf)?;
        Ok(())
    }

    pub fn read_all(&mut self) -> Result<Vec<u8>, Error> {
        let mut buf = Vec::new();
        self.file.rewind()?;
        self.file.read_to_end(&mut buf)?;
        Ok(buf)
    }

    pub fn read_page(&mut self, id: Id) -> Result<Vec<u8>, Error> {
        let pos = PAGESIZE + (PAGESIZE * id);
        let mut buf: [u8; PAGESIZE as usize] = [0x00; PAGESIZE as usize];
        self.file.seek(SeekFrom::Start(pos))?;
        self.file.read_exact(&mut buf)?;
        Ok(buf.to_vec())
    }

    pub fn read_header(&mut self) -> Result<Vec<u8>, Error> {
        let mut buf: [u8; PAGESIZE as usize] = [0x00; PAGESIZE as usize];
        self.file.rewind()?;
        self.file.read_exact(&mut buf)?;
        Ok(buf.to_vec())
    }

    pub fn get_max_id(&mut self) -> Result<u64, Error> {
        let len = self.file.metadata()?.len();
        Ok((len / PAGESIZE) - 1)
    }
}

#[derive(Debug)]
pub struct Header {
    pub elements: u64,
    pub keytype: KeyType,
}

impl Header {
    pub fn deserialize(bytes: &[u8]) -> Result<Header, DatabaseError> {
        if bytes.len() != PAGESIZE as usize {
            return Err(DatabaseError::Pagesize(bytes.len()));
        }

        let elements = u64::from_le_bytes(bytes[0..8].try_into().unwrap());

        let keytype = match bytes.index(8) {
            0x01 => KeyType::String(*bytes.index(9)),
            0x02 => KeyType::UInt64,
            _ => return Err(DatabaseError::Keytype(*bytes.index(8))),
        };

        let _content_start = match keytype {
            KeyType::String(_) => 10,
            KeyType::UInt64 => 9,
        };

        Ok(Header { elements, keytype })
    }

    pub fn serialize(&mut self) -> Vec<u8> {
        let mut b = Vec::<u8>::new();

        for (idx, byte) in self.elements.to_le_bytes().iter().enumerate() {
            b.insert(idx, *byte);
        }

        match self.keytype {
            KeyType::String(len) => {
                b.push(0x01);
                b.push(len);
            }
            KeyType::UInt64 => b.push(0x02),
        }

        b.to_vec()
    }
}

// NOTE: can maybe hold more information in the future
//
// id      pagetype    keys_len    |   keys(n)      pointers(n + 1)
// usize   u8          u16         |   Vec<u64>     Vec<u64>
//
// n = PAGESIZE - id(16 bytes) - pagetype(1 byte) - keys_len(2 bytes)
// NOTE: this should also account for String(n) which has a variable amount of bytes it takes

#[derive(Debug)]
pub struct Node {
    id: Id,             // 0..=7
    pagetype: PageType, // 8
    keys_len: u16,      // 9..=10
    keys: Vec<u64>,     // n
    pointers: Vec<u64>, // n + 1
}

impl Node {
    pub fn deserialize(bytes: &[u8]) -> Result<Node, DatabaseError> {
        if bytes.len() != PAGESIZE as usize {
            return Err(DatabaseError::Pagesize(bytes.len()));
        }

        let id = usize::from_le_bytes(bytes[0..=7].try_into().unwrap());

        let pagetype = match bytes.index(8) {
            0x01 => PageType::Root,
            0x02 => PageType::Node,
            0x03 => PageType::Leaf,
            // _ => return None,
            _ => return Err(DatabaseError::Pagetype(*bytes.index(8))),
        };

        let keys_len = u16::from_le_bytes(bytes[9..11].try_into().unwrap());

        let mut keys = Vec::new();
        let mut pointers = Vec::new();

        match pagetype {
            PageType::Leaf => todo!(),
            _ => {
                for (i, b) in bytes[11..].chunks(size_of::<u64>()).enumerate() {
                    if i >= keys_len as usize {
                        break;
                    }
                    keys.push(u64::from_le_bytes(b.try_into().unwrap()));
                }

                for (i, b) in bytes[11 + size_of::<u64>() * keys_len as usize..]
                    .chunks(size_of::<u64>())
                    .enumerate()
                {
                    if i > keys_len as usize {
                        break;
                    }

                    pointers.push(u64::from_le_bytes(b.try_into().unwrap()));
                }
            }
        }

        Ok(Node {
            id: id.try_into().unwrap(),
            pagetype,
            keys_len,
            keys,
            pointers,
        })
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
    fn serialize(self) -> Vec<u8> {}

    fn deserialize(bytes: &[u8]) -> Data {
        todo!()
    }
}

// keytype      field_len   field       primary     data_len    data
// keytype      u8         string       bool        u32         [u8]

#[derive(Debug)]
struct Field {
    field: Box<[u8]>,
    primary: bool,
    keytype: KeyType,
    data: Vec<u8>,
}

impl Field {
    fn new(field: &str, primary: bool, keytype: KeyType, data: Vec<u8>) -> Field {
        Self {
            field: field.as_bytes().into(),
            primary,
            keytype,
            data,
        }
    }

    fn serialize(self) -> Result<Vec<u8>, DatabaseError> {
        let mut vec: Vec<u8> = Vec::new();

        let len = match self.keytype {
            KeyType::String(n) => {
                vec.push(0x01);
                if n as usize > size_of::<u8>() {
                    return Err(DatabaseError::Fieldsize(n as usize));
                }
                n
            }
            KeyType::UInt64 => {
                vec.push(0x02);
                size_of::<u64>().try_into().unwrap()
            }
        };

        vec.push(len);

        for b in self.field {
            vec.push(b);
        }

        match self.primary {
            true => vec.push(0x02),
            false => vec.push(0x01),
        }

        u32::to_le_bytes(self.data.len() as u32)
            .iter()
            .for_each(|b| vec.push(*b));

        self.data.iter().for_each(|b| vec.push(*b));

        Ok(vec)
    }

    fn deserialize() -> Field {
        todo!()
    }
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

    pub fn primary(
        mut self,
        name: &str,
        keytype: KeyType,
        data: Vec<u8>,
    ) -> Result<DataBuilder, DatabaseError> {
        match self.fields.iter().find(|x| x.primary) {
            Some(f) => Err(DatabaseError::Fieldname(
                String::from_utf8(f.field.to_vec()).unwrap(),
            )),
            None => {
                self.fields.push(Field::new(name, true, keytype, data));
                Ok(self)
            }
        }
    }

    pub fn field(mut self, name: &str, keytype: KeyType, data: Vec<u8>) -> DataBuilder {
        self.fields.push(Field::new(name, false, keytype, data));
        self
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
    String(u8), //0x01
    UInt64,     //0x02
}

#[derive(Debug)]
pub enum PageType {
    Root, //0x01
    Node, //0x02
    Leaf, //0x03
    Data, //0x04
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("page was not 4096 bytes ({0})")]
    Pagesize(usize),

    #[error("keytype could not be parsed ({0})")]
    Keytype(u8),

    #[error("page type was not correct")]
    Pagetype(u8),

    #[error("data already contains a primary key ({0})")]
    Fieldname(String),

    #[error("field was too big ({0})")]
    Fieldsize(usize),
}
