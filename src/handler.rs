use crate::page::*;
use std::io::SeekFrom;
use std::io::{Read, Seek, Write};
use thiserror::Error;

const PAGESIZE: u64 = 4096;
type Id = u64;

pub trait PageHandlerFuncs<T: Write + Read + Seek> {
    fn new_page(source: &mut T, pagetype: PageType) -> Result<Page, HandlerError>;
    fn get_page(source: &mut T, id: Id) -> Result<Page, HandlerError>;
    fn write(source: &mut T, page: Page) -> Result<(), HandlerError>;
}

pub struct PageHandler;
impl<T: Write + Read + Seek> PageHandlerFuncs<T> for PageHandler {
    fn new_page(source: &mut T, pagetype: PageType) -> Result<Page, HandlerError> {
        let id = FileHandler::new_page(source)?;

        let page = Page { id, pagetype };

        let mut buf: [u8; PAGESIZE as usize] = [0x00; PAGESIZE as usize];
        let bytes = page.clone().serialize();
        buf[..bytes.len()].as_mut().write_all(&bytes)?;

        FileHandler::write_page(source, id, &buf)?;

        let mut new_header = HeaderHandler::get(source)?;
        new_header.elements += 1;
        HeaderHandler::write(source, new_header)?;

        Ok(page)
    }

    fn get_page(source: &mut T, id: Id) -> Result<Page, HandlerError> {
        println!("{id}");
        let page = Page::deserialize(&FileHandler::read_page(source, id)?)?;
        Ok(page)
    }

    fn write(source: &mut T, page: Page) -> Result<(), HandlerError> {
        FileHandler::write_page(source, page.id, &page.serialize())?;
        Ok(())
    }
}

pub trait HeaderHandlerFuncs<T: Write + Read + Seek> {
    fn get(source: &mut T) -> Result<Header, HandlerError>;
    fn write(source: &mut T, header: Header) -> Result<(), HandlerError>;
}

pub struct HeaderHandler;
impl<T: Read + Write + Seek> HeaderHandlerFuncs<T> for HeaderHandler {
    fn get(source: &mut T) -> Result<Header, HandlerError> {
        let header = Header::deserialize(&FileHandler::read_header(source)?)?;
        Ok(header)
    }

    fn write(source: &mut T, header: Header) -> Result<(), HandlerError> {
        let bytes = header.serialize();

        let mut page: [u8; PAGESIZE as usize] = [0x00; PAGESIZE as usize];

        page[..bytes.len()].as_mut().write_all(&bytes)?;

        FileHandler::write_header(source, &page)?;
        Ok(())
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
        if buf.len() > PAGESIZE as usize {
            return Err(FileError::BiggerBuffer(buf.len()));
        }
        source.rewind()?;
        source.write_all(buf)?;
        Ok(())
    }

    fn read_page(source: &mut T, id: Id) -> Result<Vec<u8>, FileError> {
        let pos = PAGESIZE + (PAGESIZE * id);
        println!("{pos}");
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

#[derive(Error, Debug)]
pub enum HandlerError {
    #[error("file handler error: {0}")]
    FileHandler(#[from] FileError),

    #[error("failed to initialize header")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use super::*;

    fn init_file(pages: usize) -> Cursor<Vec<u8>> {
        Cursor::new(vec![0x00; pages * PAGESIZE as usize])
    }

    mod filehandlertests {
        use std::io::Read;
        use std::io::Seek;
        use std::io::Write;

        use super::*;

        #[test]
        fn new_page() {
            let mut file = init_file(1);
            let id = FileHandler::new_page(&mut file).unwrap();
            assert_eq!(id, 0);
        }

        #[test]
        fn write_page() {
            let mut file = init_file(2);
            let result_ok = FileHandler::write_page(&mut file, 0, b"test");
            assert!(result_ok.is_ok());

            let mut buf: [u8; 4] = [0x00; 4];
            file.seek(std::io::SeekFrom::Start(PAGESIZE)).unwrap();
            file.read_exact(&mut buf).unwrap();
            assert_eq!(&buf, b"test")
        }

        #[test]
        fn write_page_err() {
            let mut file = init_file(2);
            let result_err = FileHandler::write_page(&mut file, 0, &[0x00; PAGESIZE as usize + 1]);
            assert!(result_err.is_err());
        }

        #[test]
        fn write_header() {
            let mut file = init_file(2);
            let result_ok = FileHandler::write_header(&mut file, b"test");
            assert!(result_ok.is_ok());

            let result_err = FileHandler::write_header(&mut file, &[0x00; PAGESIZE as usize + 1]);
            assert!(result_err.is_err());

            let mut buf: [u8; 4] = [0x00; 4];
            file.rewind().unwrap();
            file.read_exact(&mut buf).unwrap();
            assert_eq!(&buf, b"test");
        }

        #[test]
        fn read_page() {
            let mut file = init_file(2);
            file.seek(std::io::SeekFrom::Start(PAGESIZE)).unwrap();
            file.write_all(b"test").unwrap();

            let buf = FileHandler::read_page(&mut file, 0);
            assert!(buf.is_ok());
            assert_eq!(&buf.unwrap()[0..4], b"test");
        }

        #[test]
        fn read_header() {
            let mut file = init_file(2);
            file.write_all(b"test").unwrap();

            let buf = FileHandler::read_header(&mut file);
            assert!(buf.is_ok());
            assert_eq!(&buf.unwrap()[0..4], b"test");
        }
    }

    mod pagehandlertests {
        use super::*;

        #[test]
        fn new_page_leaf() {
            let mut file = init_file(1);
            let _ = HeaderHandler::write(
                &mut file,
                Header {
                    root: 0,
                    order: 4,
                    keytype: KeyType::UInt64,
                    elements: 0,
                    keytype_size: 8,
                },
            );

            let pagetype = PageType::Leaf(Leaf::new(KeyType::UInt64));

            let page = PageHandler::new_page(&mut file, pagetype);
            if let Ok(ref page) = page
                && let PageType::Leaf(leaf) = &page.pagetype
            {
                assert_eq!(leaf.keytype, KeyType::UInt64);
                assert_eq!(leaf.keys.len(), 0);
                assert_eq!(leaf.pointers.len(), 0);
                assert_eq!(leaf.next_leaf_pointer, 0);
            } else if let Err(err) = page {
                eprintln!("{err}");
                panic!()
            }
        }

        #[test]
        fn new_page_node() {
            let mut file = init_file(1);
            let _ = HeaderHandler::write(
                &mut file,
                Header {
                    root: 0,
                    order: 4,
                    keytype: KeyType::UInt64,
                    elements: 0,
                    keytype_size: 8,
                },
            );

            let pagetype = PageType::Node(Node::new(KeyType::UInt64));

            let page = PageHandler::new_page(&mut file, pagetype);

            if let Ok(ref page) = page
                && let PageType::Node(node) = &page.pagetype
            {
                assert_eq!(node.keytype, KeyType::UInt64);
                assert!(node.keys.is_empty());
                assert!(node.pointers.is_empty());
            } else if let Err(err) = page {
                eprintln!("{err}");
                panic!()
            }
        }

        #[test]
        fn new_page_raw() {}

        #[test]
        fn get_page_leaf() {
            let mut file = init_file(2);
            let _ = HeaderHandler::write(
                &mut file,
                Header {
                    root: 0,
                    order: 4,
                    keytype: KeyType::UInt64,
                    elements: 0,
                    keytype_size: 8,
                },
            );

            let _ = file.seek(SeekFrom::Start(PAGESIZE));
            let _ = file.write_all(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x02]);

            let page = PageHandler::get_page(&mut file, 0);

            if let Ok(ref page) = page
                && let PageType::Leaf(leaf) = (&page.pagetype)
            {
                assert_eq!(leaf.keytype, KeyType::UInt64);
                assert!(leaf.keys.is_empty());
                assert!(leaf.pointers.is_empty());
                assert_eq!(leaf.next_leaf_pointer, 0);
            } else if let Err(err) = page {
                eprintln!("{err}");
                panic!()
            }
        }

        #[test]
        fn write() {}
    }
}
