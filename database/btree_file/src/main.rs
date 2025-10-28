use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Error, Read, Seek, SeekFrom, Write};

const PAGESIZE: u64 = 4096;
type Id = u64;

fn main() -> Result<(), Error> {
    let mut handler = Handler::new("test")?;

    handler.write_to_header("gaming".as_bytes())?;

    let page_id = handler.new_page()?;

    handler.get_max_id()?;
    handler.write_to_page(page_id, "hej gamer".as_bytes())?;
    let read = handler.read_all()?;

    println!("{read:?}");

    Ok(())
}

struct Handler {
    file: File,
}

impl Handler {
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
        let id = self.file.seek(std::io::SeekFrom::End(0))?;
        self.write(&[0x00; PAGESIZE as usize])?;
        Ok((id / PAGESIZE) - 1)
    }

    fn read_all(&mut self) -> Result<Vec<u8>, Error> {
        let mut buf = Vec::new();
        self.file.rewind()?;
        self.file.read_to_end(&mut buf)?;
        Ok(buf)
    }

    fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        self.file.seek(SeekFrom::End(0))?;
        let mut writer = BufWriter::new(self.file.try_clone().unwrap());
        writer.write_all(buf)?;
        Ok(())
    }

    fn write_to_page(&mut self, id: Id, buf: &[u8]) -> Result<bool, Error> {
        if dbg!(id <= self.get_max_id()?) {
            let mut writer = BufWriter::new(self.file.try_clone().unwrap());
            self.file
                .seek(SeekFrom::Start(PAGESIZE + (PAGESIZE * id)))?;
            writer.write_all(buf)?;
            return Ok(true);
        }
        Ok(false)
    }

    fn write_to_header(&mut self, buf: &[u8]) -> Result<(), Error> {
        let mut writer = BufWriter::new(self.file.try_clone().unwrap());
        self.file.seek(SeekFrom::Start(0))?;
        writer.write_all(buf)?;
        Ok(())
    }

    fn get_max_id(&mut self) -> Result<u64, Error> {
        Ok((self.file.metadata()?.len() / PAGESIZE) - 2)
    }

    fn get_header(&self) -> Result<Header, Error> {
        todo!()
    }
}

struct Header;
