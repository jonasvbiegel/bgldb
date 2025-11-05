use std::fs::{File, OpenOptions};
use std::io::{Error, Read, Seek, SeekFrom, Write};
use std::ops::Index;

// BIG ENDIAN BYTES

const PAGESIZE: u64 = 4096;
type Id = u64;

fn main() -> Result<(), Error> {
    let mut handler = FileHandler::new("test")?;

    let page_id = dbg!(handler.new_page()?);
    // handler.write_to_page(page_id, "GUNGA GAMER".as_bytes())?;

    let read = handler.read_page(page_id);

    println!("{read:?}");

    let mut header = Header { elements: 909090 };
    handler.write_to_header(&header.serialize())?;

    let header_read = Header::deserialize(&handler.read_header().unwrap());

    println!("{header_read:?}");

    let mut b: Vec<u8> = Vec::new();
    b.push(0x01);

    for x in u16::to_be_bytes(1) {
        b.push(x);
    }

    for y in u32::to_be_bytes(123) {
        b.push(y);
    }

    for z in u64::to_be_bytes(7123123173) {
        b.push(z);
    }

    for æ in u32::to_be_bytes(456) {
        b.push(æ);
    }

    handler.write_to_page(page_id, &b)?;

    dbg!(Page::deserialize(&handler.read_page(page_id)?));

    Ok(())
}

struct FileHandler {
    file: File,
}

impl FileHandler {
    fn new(name: &str) -> Result<Self, Error> {
        let mut h = Self {
            file: OpenOptions::new()
                .create(true)
                .truncate(true)
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

    fn new_page(&mut self) -> Result<Id, Error> {
        let id = self.file.seek(SeekFrom::End(0))?;
        self.write(&[0x00; PAGESIZE as usize])?;
        Ok((id / PAGESIZE) - 1)
    }

    fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(buf)?;
        Ok(())
    }

    fn write_to_page(&mut self, id: Id, buf: &[u8]) -> Result<bool, Error> {
        if id <= self.get_max_id()? {
            let pos = PAGESIZE + (PAGESIZE * id);
            self.file.seek(SeekFrom::Start(pos))?;
            self.file.write_all(buf)?;
            return Ok(true);
        }
        Ok(false)
    }

    fn write_to_header(&mut self, buf: &[u8]) -> Result<(), Error> {
        // self.file.seek(SeekFrom::Start(0))?;
        self.file.rewind()?;
        self.file.write_all(buf)?;
        Ok(())
    }

    fn read_all(&mut self) -> Result<Vec<u8>, Error> {
        let mut buf = Vec::new();
        self.file.rewind()?;
        self.file.read_to_end(&mut buf)?;
        Ok(buf)
    }

    fn read_page(&mut self, id: Id) -> Result<Vec<u8>, Error> {
        let pos = PAGESIZE + (PAGESIZE * id);
        let mut buf: [u8; PAGESIZE as usize] = [0x00; PAGESIZE as usize];
        self.file.seek(SeekFrom::Start(pos))?;
        self.file.read_exact(&mut buf)?;
        Ok(buf.to_vec())
    }

    fn read_header(&mut self) -> Result<Vec<u8>, Error> {
        let mut buf: [u8; PAGESIZE as usize] = [0x00; PAGESIZE as usize];
        self.file.rewind()?;
        self.file.read_exact(&mut buf)?;
        Ok(buf.to_vec())
    }

    fn get_max_id(&mut self) -> Result<u64, Error> {
        let len = self.file.metadata()?.len();
        Ok((len / PAGESIZE) - 1)
    }
}

#[derive(Debug)]
struct Header {
    elements: u64,
}

impl Header {
    fn deserialize(bytes: &[u8]) -> Option<Header> {
        if bytes.len() != PAGESIZE as usize {
            return None;
        }

        let e = u64::from_be_bytes(bytes[0..8].try_into().unwrap());

        Some(Self { elements: e })
    }

    fn serialize(&mut self) -> Vec<u8> {
        let mut b: [u8; PAGESIZE as usize] = [0x00; PAGESIZE as usize];

        for (idx, byte) in self.elements.to_be_bytes().iter().enumerate() {
            b[idx] = *byte;
        }

        b.to_vec()
    }
}

#[derive(Debug)]
// pagetype and data_len are stored in the header, which is the first 3 bytes. after that is the
// keys and pointers, stored in the order c_1, k_1, c_2, k_2 .. c_n, k_n, c_last
struct Page {
    pagetype: PageType,
    keys_len: u16,
    keys: Vec<u64>,
    pointers: Vec<u32>,
}

impl Page {
    fn deserialize(bytes: &[u8]) -> Option<Page> {
        if bytes.len() != PAGESIZE as usize {
            return None;
        }

        let pagetype = match bytes.index(0) {
            0x01 => PageType::Root,
            0x02 => PageType::Node,
            0x03 => PageType::Leaf,
            _ => return None,
        };

        let keys_len = u16::from_be_bytes(bytes[1..=2].try_into().unwrap());

        let mut keys = Vec::new();
        let mut pointers = Vec::new();

        match pagetype {
            PageType::Leaf => todo!(),
            _ => {
                for (i, b) in bytes[3..].chunks(12).enumerate() {
                    if i >= (keys_len).into() {
                        pointers.push(u32::from_be_bytes(b[0..=3].try_into().unwrap()));
                        break;
                    } else {
                        pointers.push(u32::from_be_bytes(b[0..=3].try_into().unwrap()));
                        keys.push(u64::from_be_bytes(b[4..].try_into().unwrap()));
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

    fn serialize(&mut self) -> Vec<u8> {
        todo!()
    }
}

#[derive(Debug)]
enum PageType {
    Root, //0x01
    Node, //0x02
    Leaf, //0x03
}
