use std::fs::{File, OpenOptions};
use std::io::{Error, Read, Seek, SeekFrom, Write};
use std::ops::Index;

// LITTLE ENDIAN BYTES
const PAGESIZE: u64 = 4096;
type Id = u64;

pub struct FileHandler {
    file: File,
}

impl FileHandler {
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
}

impl Header {
    pub fn deserialize(bytes: &[u8]) -> Option<Header> {
        if bytes.len() != PAGESIZE as usize {
            return None;
        }

        let e = u64::from_le_bytes(bytes[0..8].try_into().unwrap());

        Some(Self { elements: e })
    }

    pub fn serialize(&mut self) -> Vec<u8> {
        let mut b: [u8; PAGESIZE as usize] = [0x00; PAGESIZE as usize];

        for (idx, byte) in self.elements.to_le_bytes().iter().enumerate() {
            b[idx] = *byte;
        }

        b.to_vec()
    }
}

#[derive(Debug)]
// pagetype and data_len are stored in the header, which is the first 3 bytes. after that is the
// keys and pointers, stored in the order c_1, k_1, c_2, k_2 .. c_n, k_n, c_last
pub struct Page {
    pagetype: PageType,
    keys_len: u16,
    keys: Vec<u64>,
    pointers: Vec<u32>,
}

impl Page {
    pub fn deserialize(bytes: &[u8]) -> Option<Page> {
        if bytes.len() != PAGESIZE as usize {
            return None;
        }

        let pagetype = match bytes.index(0) {
            0x01 => PageType::Root,
            0x02 => PageType::Node,
            0x03 => PageType::Leaf,
            _ => return None,
        };

        let keys_len = u16::from_le_bytes(bytes[1..=2].try_into().unwrap());

        let mut keys = Vec::new();
        let mut pointers = Vec::new();

        match pagetype {
            PageType::Leaf => todo!(),
            _ => {
                for (i, b) in bytes[3..].chunks(12).enumerate() {
                    if i >= (keys_len).into() {
                        pointers.push(u32::from_le_bytes(b[0..=3].try_into().unwrap()));
                        break;
                    } else {
                        pointers.push(u32::from_le_bytes(b[0..=3].try_into().unwrap()));
                        keys.push(u64::from_le_bytes(b[4..].try_into().unwrap()));
                    }
                }
            }
        }

        Some(Page {
            pagetype,
            keys_len,
            keys,
            pointers,
        })
    }

    pub fn serialize(&mut self) -> Vec<u8> {
        todo!()
    }
}

#[derive(Debug)]
pub enum PageType {
    Root, //0x01
    Node, //0x02
    Leaf, //0x03
}
