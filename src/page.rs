use nom::Parser;
use nom::multi::{count, length_count};
use nom::number::{Endianness, u8, u64};
use std::io::{Read, Seek, SeekFrom, Write};
use thiserror::Error;

// NOTE: LITTLE ENDIAN BYTES
const PAGESIZE: u64 = 4096;
type Id = u64;

// NOTE: still need to figure out data handling
// maybe we should just always parse to raw, and then in the end parse all raws to a data struct?
// this could still work with the Data and Raw struct

pub trait PageHandlerFuncs<T: Write + Read + Seek> {
    fn new_page(source: &mut T, pagetype: PageType) -> Result<Page, HandlerError>;
    fn get_page(source: &mut T, id: Id) -> Result<Page, HandlerError>;
    fn get_raw(source: &mut T, id: Id) -> Result<Raw, HandlerError>;
    fn write(source: &mut T, page: Page) -> Result<(), HandlerError>;
}

pub struct PageHandler;
impl<T: Write + Read + Seek> PageHandlerFuncs<T> for PageHandler {
    fn new_page(source: &mut T, pagetype: PageType) -> Result<Page, HandlerError> {
        let id = FileHandler::new_page(source)?;

        let page = Page { id, pagetype };

        FileHandler::write_page(source, id, &page.clone().serialize())?;

        let mut new_header = HeaderHandler::get(source)?;
        new_header.elements += 1;
        HeaderHandler::write(source, new_header)?;

        Ok(page)
    }

    fn get_raw(source: &mut T, id: Id) -> Result<Raw, HandlerError> {
        let page = Page::deserialize(&FileHandler::read_page(source, id)?)?;

        match page.pagetype {
            PageType::Raw(raw) => Ok(raw),
            _ => Err(HandlerError::GetRawError),
        }
    }

    fn get_page(source: &mut T, id: Id) -> Result<Page, HandlerError> {
        let page = Page::deserialize(&FileHandler::read_page(source, id)?)?;

        match page.pagetype {
            PageType::Raw(root) => {
                let mut bytes: Vec<u8> = Vec::new();

                root.data.iter().for_each(|byte| bytes.push(*byte));

                for pointer in root.pointers {
                    let raw = PageHandler::get_raw(source, pointer)?;
                    raw.data.iter().for_each(|byte| bytes.push(*byte));
                }

                let data = Data::deserialize(&bytes)?;

                Ok(Page {
                    id,
                    pagetype: PageType::Data(data),
                })
            }
            _ => Ok(page),
        }
    }

    fn write(source: &mut T, page: Page) -> Result<(), HandlerError> {
        match page.pagetype {
            PageType::Raw(raw) => todo!(),
            _ => FileHandler::write_page(source, page.id, &page.serialize())?,
        }

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
const PAGETYPE_SIZE: usize = size_of::<u8>();

const PAGESIZE_NO_HEADER: usize = PAGESIZE as usize - ID_SIZE - PAGETYPE_SIZE;

#[derive(Debug, Clone)]
pub struct Page {
    pub id: Id,
    pub pagetype: PageType,
}

impl SerializeDeserialize for Page {
    fn deserialize(bytes: &[u8]) -> Result<Page, FileError> {
        if bytes.len() != PAGESIZE as usize {
            return Err(FileError::Pagesize(PAGESIZE as usize, bytes.len()));
        }

        let (input, (id, pagetype)) = (u64(Endianness::Little), u8()).parse(bytes)?;

        let pagetype = match pagetype {
            0x01 => PageType::Node(Node::deserialize(input)?),
            0x02 => PageType::Leaf(Leaf::deserialize(input)?),
            0x03 => PageType::Data(Data::deserialize(input)?),
            0x04 => PageType::Raw(Raw::deserialize(input)?),
            _ => return Err(FileError::Pagetype(pagetype)),
        };

        Ok(Page { id, pagetype })
    }

    fn serialize(self) -> Vec<u8> {
        let mut b: Vec<u8> = Vec::new();

        self.id.to_le_bytes().iter().for_each(|byte| b.push(*byte));

        match self.pagetype {
            PageType::Node(node) => node.serialize().iter().for_each(|byte| b.push(*byte)),
            PageType::Leaf(leaf) => leaf.serialize().iter().for_each(|byte| b.push(*byte)),
            PageType::Data(data) => data.serialize().iter().for_each(|byte| b.push(*byte)),
            PageType::Raw(raw) => raw.data.iter().for_each(|byte| b.push(*byte)),
        }

        b
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub keytype: KeyType,
    pub keys: Vec<Vec<u8>>,
    pub pointers: Vec<Id>,
}

impl Node {
    pub fn new(keytype: KeyType) -> Node {
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

        let (_, pointers) = count(u64(Endianness::Little), keys_len as usize + 1).parse(input)?;

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
    pub keytype: KeyType,
    pub keys: Vec<Vec<u8>>,
    pub pointers: Vec<Id>,
    pub next_leaf_pointer: Id,
}

impl Leaf {
    pub fn new(keytype: KeyType) -> Leaf {
        Leaf {
            keytype,
            keys: Vec::new(),
            pointers: Vec::new(),
            next_leaf_pointer: 0,
        }
    }

    fn from_node(node: Node) -> Leaf {
        Leaf {
            keytype: node.keytype,
            keys: node.keys,
            pointers: node.pointers,
            next_leaf_pointer: 0x00,
        }
    }

    fn set_next_leaf_pointer(&mut self, pointer: Id) {
        self.next_leaf_pointer = pointer
    }
}

impl SerializeDeserialize for Leaf {
    fn serialize(self) -> Vec<u8> {
        let mut b = Vec::new();
        b.push(0x02);

        match self.keytype {
            KeyType::String => {
                b.push(0x01);
                b.push(u8::try_from(self.keys.len()).expect("couldnt parse keys_len"));
                for key in &self.keys {
                    b.push(key.len() as u8);
                    key.iter().for_each(|x| b.push(*x));
                }
            }
            KeyType::UInt64 => {
                b.push(0x02);
                b.push(u8::try_from(self.keys.len()).expect("couldnt parse keys_len"));
                for key in &self.keys {
                    key.iter().for_each(|x| b.push(*x));
                }
            }
        }

        self.pointers
            .iter()
            .for_each(|p| p.to_le_bytes().iter().for_each(|byte| b.push(*byte)));

        u64::to_le_bytes(self.next_leaf_pointer)
            .iter()
            .for_each(|byte| b.push(*byte));

        b
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

        let (input, keys) = match keys_len == 0 {
            false => match keytype {
                KeyType::String => {
                    count(length_count(u8(), u8()), keys_len as usize).parse(input)?
                }
                KeyType::UInt64 => count(count(u8(), 8), keys_len as usize).parse(input)?,
            },
            _ => (input, Vec::new()),
        };

        let (input, pointers) = match keys_len == 0 {
            true => (input, Vec::new()),
            _ => count(u64(Endianness::Little), keys_len as usize + 1).parse(input)?,
        };

        let (_, next_leaf_pointer) = u64(Endianness::Little).parse(input)?;

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
    pointers: Vec<u64>,
    objects: Vec<Vec<Field>>,
}

impl Data {
    fn raw(self) -> Vec<Raw> {
        todo!()
    }
}

impl SerializeDeserialize for Data {
    fn serialize(self) -> Vec<u8> {
        let mut bytes = Vec::new();

        for object in self.objects {
            bytes.push(object.len().try_into().expect("couldnt parse object len"));
            for field in object {
                let f = field.serialize();
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

#[derive(Debug, Clone)]
pub struct Raw {
    pointers: Vec<u64>,
    data: Vec<u8>,
}

impl Raw {
    fn new() -> Raw {
        Raw {
            pointers: Vec::new(),
            data: Vec::new(),
        }
    }
}

impl SerializeDeserialize for Raw {
    fn serialize(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.push(0x04);

        usize::to_le_bytes(self.pointers.len())
            .iter()
            .for_each(|byte| bytes.push(*byte));

        for pointer in self.pointers {
            u64::to_le_bytes(pointer)
                .iter()
                .for_each(|byte| bytes.push(*byte));
        }

        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, FileError> {
        let (input, pointers_len) = u64(Endianness::Little).parse(bytes)?;

        let (input, pointers) =
            count(u64(Endianness::Little), pointers_len as usize).parse(input)?;

        Ok(Raw {
            pointers,
            data: input.to_vec(),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KeyType {
    String, // 0x01
    UInt64, // 0x02
}

#[derive(Debug, Clone)]
pub enum PageType {
    Node(Node), // 0x01
    Leaf(Leaf), // 0x02
    Data(Data), // 0x03
    Raw(Raw),   //0x04
}

#[derive(Error, Debug)]
pub enum FileError {
    #[error("page was not the correct size (expected {0}, found {1})")]
    Pagesize(usize, usize),

    #[error("keytype could not be parsed ({0})")]
    Keytype(u8),

    #[error("page type was not correct")]
    Pagetype(u8),

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
pub enum HandlerError {
    #[error("file handler error: {0}")]
    FileHandler(#[from] FileError),

    #[error("failed to initialize header")]
    Io(#[from] std::io::Error),

    #[error("expected raw page")]
    GetRawError,
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

        // fn new_page(source: &mut T, pagetype: PageType) -> Result<Page, HandlerError>;
        // fn get_page(source: &mut T, id: Id) -> Result<Page, HandlerError>;
        // fn get_raw(source: &mut T, id: Id) -> Result<Raw, HandlerError>;
        // fn write(source: &mut T, page: Page) -> Result<(), HandlerError>;

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
        fn get_raw() {}

        #[test]
        fn write() {}
    }

    mod pagetests {
        use super::*;

        #[test]
        fn serialize() {}

        #[test]
        fn deserialize() {}
    }

    mod leaftests {
        use super::*;

        #[test]
        fn serialize() {
            let leaf = Leaf {
                keys: Vec::new(),
                keytype: KeyType::UInt64,
                pointers: Vec::new(),
                next_leaf_pointer: 0,
            };

            let bytes = leaf.serialize();

            let expected = [
                0x02, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ];

            assert_eq!(bytes, expected)
        }

        #[test]
        fn serialize_with_data_uint64() {
            let leaf = Leaf {
                keys: vec![
                    usize::to_le_bytes(1).to_vec(),
                    usize::to_le_bytes(2).to_vec(),
                    usize::to_le_bytes(3).to_vec(),
                ],
                keytype: KeyType::UInt64,
                pointers: vec![4, 5, 6, 7],
                next_leaf_pointer: 1,
            };

            let bytes = leaf.serialize();

            let mut expected: Vec<u8> = vec![0x02, 0x02, 0x03];

            usize::to_le_bytes(1).iter().for_each(|b| expected.push(*b));
            usize::to_le_bytes(2).iter().for_each(|b| expected.push(*b));
            usize::to_le_bytes(3).iter().for_each(|b| expected.push(*b));

            usize::to_le_bytes(4).iter().for_each(|b| expected.push(*b));
            usize::to_le_bytes(5).iter().for_each(|b| expected.push(*b));
            usize::to_le_bytes(6).iter().for_each(|b| expected.push(*b));
            usize::to_le_bytes(7).iter().for_each(|b| expected.push(*b));

            usize::to_le_bytes(1).iter().for_each(|b| expected.push(*b));

            assert_eq!(expected, bytes)
        }

        #[test]
        fn serialize_with_data_string() {
            let leaf = Leaf {
                keys: vec![b"foo".to_vec(), b"bar".to_vec(), b"baz".to_vec()],
                keytype: KeyType::String,
                pointers: vec![4, 5, 6, 7],
                next_leaf_pointer: 1,
            };

            let bytes = leaf.serialize();

            let mut expected: Vec<u8> = vec![0x02, 0x01, 0x03];

            expected.push(0x03);
            b"foo".iter().for_each(|b| expected.push(*b));
            expected.push(0x03);
            b"bar".iter().for_each(|b| expected.push(*b));
            expected.push(0x03);
            b"baz".iter().for_each(|b| expected.push(*b));

            usize::to_le_bytes(4).iter().for_each(|b| expected.push(*b));
            usize::to_le_bytes(5).iter().for_each(|b| expected.push(*b));
            usize::to_le_bytes(6).iter().for_each(|b| expected.push(*b));
            usize::to_le_bytes(7).iter().for_each(|b| expected.push(*b));

            usize::to_le_bytes(1).iter().for_each(|b| expected.push(*b));

            assert_eq!(expected, bytes)
        }

        #[test]
        fn deserialize() {
            let bytes = [0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

            let mut page: [u8; PAGESIZE_NO_HEADER] = [0x00; PAGESIZE_NO_HEADER];

            let _ = page[..bytes.len()].as_mut().write_all(&bytes);

            let leaf = Leaf::deserialize(&page);
            if let Ok(leaf) = leaf {
                assert_eq!(leaf.keytype, KeyType::UInt64);
                assert_eq!(leaf.keys.len(), 0);
                assert_eq!(leaf.pointers.len(), 0);
                assert_eq!(leaf.next_leaf_pointer, 0);
            } else if let Err(e) = leaf {
                eprintln!("{e}");
                panic!()
            }
        }

        #[test]
        fn deserialize_with_data_uint64() {
            let mut bytes: Vec<u8> = vec![0x03, 0x02];

            usize::to_le_bytes(1).iter().for_each(|b| bytes.push(*b));
            usize::to_le_bytes(2).iter().for_each(|b| bytes.push(*b));
            usize::to_le_bytes(3).iter().for_each(|b| bytes.push(*b));

            usize::to_le_bytes(4).iter().for_each(|b| bytes.push(*b));
            usize::to_le_bytes(5).iter().for_each(|b| bytes.push(*b));
            usize::to_le_bytes(6).iter().for_each(|b| bytes.push(*b));
            usize::to_le_bytes(7).iter().for_each(|b| bytes.push(*b));

            usize::to_le_bytes(1).iter().for_each(|b| bytes.push(*b));

            let mut page: [u8; PAGESIZE_NO_HEADER] = [0x00; PAGESIZE_NO_HEADER];

            let _ = page[..bytes.len()].as_mut().write_all(&bytes);

            let leaf = Leaf::deserialize(&page);
            if let Ok(leaf) = leaf {
                assert_eq!(leaf.keytype, KeyType::UInt64);
                assert_eq!(
                    leaf.keys,
                    vec![
                        vec![0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                        vec![0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                        vec![0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                    ]
                );
                assert_eq!(leaf.pointers, vec![4, 5, 6, 7]);
                assert_eq!(leaf.next_leaf_pointer, 1);
            }
        }

        #[test]
        fn deserialize_with_data_string() {
            let mut bytes: Vec<u8> = vec![0x03, 0x01];

            bytes.push(0x03);
            b"foo".iter().for_each(|b| bytes.push(*b));

            bytes.push(0x03);
            b"bar".iter().for_each(|b| bytes.push(*b));

            bytes.push(0x03);
            b"baz".iter().for_each(|b| bytes.push(*b));

            usize::to_le_bytes(4).iter().for_each(|b| bytes.push(*b));
            usize::to_le_bytes(5).iter().for_each(|b| bytes.push(*b));
            usize::to_le_bytes(6).iter().for_each(|b| bytes.push(*b));
            usize::to_le_bytes(7).iter().for_each(|b| bytes.push(*b));

            usize::to_le_bytes(1).iter().for_each(|b| bytes.push(*b));

            let mut page: [u8; PAGESIZE_NO_HEADER] = [0x00; PAGESIZE_NO_HEADER];

            let _ = page[..bytes.len()].as_mut().write_all(&bytes);

            let leaf = Leaf::deserialize(&page);
            if let Ok(leaf) = leaf {
                assert_eq!(leaf.keytype, KeyType::String);
                assert_eq!(
                    leaf.keys,
                    vec![
                        vec![b'f', b'o', b'o'],
                        vec![b'b', b'a', b'r'],
                        vec![b'b', b'a', b'z'],
                    ]
                );
                assert_eq!(leaf.pointers, vec![4, 5, 6, 7]);
                assert_eq!(leaf.next_leaf_pointer, 1);
            }
        }
    }
}
