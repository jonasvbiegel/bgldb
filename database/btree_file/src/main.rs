use std::fs::{File, OpenOptions};
use std::io::{Error, Read, Seek, SeekFrom, Write};

// BIG ENDIAN BYTES

const PAGESIZE: u64 = 4096;
type Id = u64;

fn main() -> Result<(), Error> {
    let mut handler = FileHandler::new("test")?;

    let page_id = dbg!(handler.new_page()?);
    handler.write_to_page(page_id, "GUNGA GAMER".as_bytes())?;

    let read = handler.read_page(page_id);

    println!("{read:?}");

    let mut header = Header { elements: 909090 };
    handler.write_to_header(&header.serialize().unwrap())?;

    let header_read = Header::deserialize(&handler.read_header().unwrap());

    println!("{header_read:?}");

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
        // let id = self.file.seek(std::io::SeekFrom::End(0))?;
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
        self.file.seek(SeekFrom::Start(0))?;
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
    // fn new() -> Self {
    //     Self { elements: 0 }
    // }

    fn deserialize(bytes: &[u8]) -> Option<Header> {
        if bytes.len() != PAGESIZE as usize {
            return None;
        }

        let e = u64::from_be_bytes(bytes[0..8].try_into().unwrap());

        Some(Self { elements: e })
    }

    fn serialize(&mut self) -> Option<Vec<u8>> {
        let mut b: [u8; PAGESIZE as usize] = [0x00; PAGESIZE as usize];

        for (idx, byte) in self.elements.to_be_bytes().iter().enumerate() {
            b[idx] = *byte;
        }

        Some(b.to_vec())
    }
}
