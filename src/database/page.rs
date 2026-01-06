use nom::Parser;
use nom::multi::{count, length_count};
use nom::number::{Endianness, u8, u64};
use serde::ser::{Serialize, SerializeMap, Serializer};
use thiserror::Error;

// NOTE: LITTLE ENDIAN BYTES
const PAGESIZE: u64 = 4096;
type Id = u64;

pub trait SerializeDeserialize: Sized {
    fn serialize(self) -> Vec<u8>;
    fn deserialize(bytes: &[u8]) -> Result<Self, FileError>;
}

#[derive(Debug)]
pub struct Header {
    pub elements: u64,
    pub keytype: KeyType,
    pub keytype_size: u8,
    pub key: Vec<u8>,
    pub root: Id,
    pub order: u8,
}

impl SerializeDeserialize for Header {
    fn deserialize(bytes: &[u8]) -> Result<Header, FileError> {
        if bytes.len() != PAGESIZE as usize {
            return Err(FileError::Pagesize(PAGESIZE as usize, bytes.len()));
        }

        let (_, (elements, keytype, keytype_size, key, root, order)) = (
            u64(Endianness::Little),
            u8(),
            u8(),
            length_count(u8(), u8()),
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
            key,
            root,
            order,
        })
    }

    fn serialize(self) -> Vec<u8> {
        let mut b = Vec::<u8>::new();

        b.extend(self.elements.to_le_bytes());

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

        b.push(self.key.len().try_into().expect("couldnt parse key len"));

        b.extend(self.key);

        b.extend(self.root.to_le_bytes());

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

impl Page {
    pub fn split(&mut self, new_id: Id) -> Result<Page, FileError> {
        let new_page = match &mut self.pagetype {
            PageType::Node(node) => Page {
                id: new_id,
                pagetype: PageType::Node(node.split()),
            },
            PageType::Leaf(leaf) => Page {
                id: new_id,
                pagetype: PageType::Leaf(leaf.split()),
            },
            // TODO: return error,
            _ => todo!(),
        };

        Ok(new_page)
    }
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
            _ => return Err(FileError::Pagetype(pagetype)),
        };

        Ok(Page { id, pagetype })
    }

    fn serialize(self) -> Vec<u8> {
        let mut b: Vec<u8> = Vec::new();

        b.extend(self.id.to_le_bytes());

        b.extend(match self.pagetype {
            PageType::Node(node) => node.serialize(),
            PageType::Leaf(leaf) => leaf.serialize(),
            PageType::Data(data) => data.serialize(),
        });

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

    pub fn split(&mut self) -> Node {
        let mut new_node = Node::new(self.keytype);

        for _ in self.keys.len() / 2..self.keys.len() {
            new_node.keys.push(self.keys.pop().unwrap());
        }
        new_node.keys.reverse();

        for _ in self.pointers.len().div_ceil(2)..self.pointers.len() {
            new_node.pointers.push(self.pointers.pop().unwrap());
        }
        new_node.pointers.reverse();

        new_node
    }
}

impl SerializeDeserialize for Node {
    fn deserialize(bytes: &[u8]) -> Result<Node, FileError> {
        if bytes.len() != PAGESIZE_NO_HEADER {
            return Err(FileError::Pagesize(PAGESIZE_NO_HEADER, bytes.len()));
        }

        let (input, (keys_len, keytype)) = (u8(), u8()).parse(bytes)?;

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

        b.push(
            self.keys
                .len()
                .try_into()
                .expect("failed to write keys_len"),
        );

        match self.keytype {
            KeyType::String => {
                b.push(0x01);
                for key in self.keys {
                    b.push(key.len() as u8);
                    b.extend(key);
                }
            }
            KeyType::UInt64 => {
                b.push(0x02);
                for key in self.keys {
                    b.extend(key);
                }
            }
        }

        for p in self.pointers {
            b.extend(p.to_le_bytes());
        }

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

    pub fn split(&mut self) -> Leaf {
        let mut new_leaf = Leaf::new(self.keytype);

        for _ in self.keys.len() / 2..self.keys.len() {
            new_leaf.keys.push(self.keys.pop().unwrap());
            new_leaf.pointers.push(self.pointers.pop().unwrap());
        }

        if !new_leaf.keys.is_sorted() && !new_leaf.pointers.is_sorted() {
            new_leaf.keys.reverse();
            new_leaf.pointers.reverse();
        }

        new_leaf
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
                    b.extend(key);
                }
            }
            KeyType::UInt64 => {
                b.push(0x02);
                b.push(u8::try_from(self.keys.len()).expect("couldnt parse keys_len"));
                for key in &self.keys {
                    b.extend(key);
                }
            }
        }

        for p in self.pointers {
            b.extend(p.to_le_bytes());
        }

        b.extend(self.next_leaf_pointer.to_le_bytes());

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
            _ => count(u64(Endianness::Little), keys_len as usize).parse(input)?,
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
    pub object: Vec<Field>,
}

impl Data {
    pub fn json(&self) -> String {
        let mut json = "{".to_string();

        for (idx, field) in self.object.iter().enumerate() {
            json.push_str(format!("\"{}\"", &field.get_key()).as_str());
            json.push_str(": ");
            match field.datatype {
                KeyType::String => json.push_str(format!("\"{}\"", field.get_data()).as_str()),
                KeyType::UInt64 => json.push_str(&field.get_data()),
            }
            if idx != self.object.len() - 1 {
                json.push(',');
            }
        }

        json.push('}');

        json
    }

    pub fn is_valid(&self) -> bool {
        for field in &self.object {
            match field.datatype {
                KeyType::UInt64 => {
                    if field.key.len() != 8 {
                        return false;
                    }
                }
                KeyType::String => {
                    if String::from_utf8(field.key.clone()).is_err() {
                        return false;
                    }
                }
            }
        }

        true
    }

    pub fn get_field(&self, key: &[u8]) -> Option<&Field> {
        if let Some(field) = self.object.iter().find(|field| field.key == key) {
            Some(field)
        } else {
            None
        }
    }
}

impl SerializeDeserialize for Data {
    fn serialize(self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.push(0x03);

        bytes.push(self.object.len() as u8);

        for field in self.object {
            let f = field.serialize();
            for byte in f {
                bytes.push(byte);
            }
        }

        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Data, FileError> {
        if bytes.len() != PAGESIZE_NO_HEADER {
            return Err(FileError::Pagesize(PAGESIZE_NO_HEADER, bytes.len()));
        }

        // TODO: this is wrong, field parsing is super wrong
        let (input, object_len) = u8().parse(bytes)?;

        let (_, fields) = count(length_count(u8(), u8()), object_len.into()).parse(input)?;

        let fields: Result<Vec<Field>, FileError> =
            fields.into_iter().map(|f| Field::deserialize(&f)).collect();

        Ok(Data { object: fields? })
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    key: Vec<u8>,
    datatype: KeyType,
    pub data: Vec<u8>,
}

impl Field {
    pub fn new(key: Vec<u8>, datatype: KeyType, data: Vec<u8>) -> Field {
        Field {
            key,
            datatype,
            data,
        }
    }

    pub fn len(&self) -> usize {
        let mut size = size_of::<u8>() // size of key
        + self.key.len() // the key
        + size_of::<u8>() // size of data type
        + self.data.len(); // the data

        if self.datatype == KeyType::String {
            size += 0x01 // size of the data len
        }

        size
    }

    pub fn get_key(&self) -> String {
        String::from_utf8(self.key.clone()).expect("couldnt parse key")
    }

    pub fn get_data(&self) -> String {
        match self.datatype {
            KeyType::String => {
                String::from_utf8(self.data.clone()).expect("couldnt parse data to string")
            }
            KeyType::UInt64 => usize::from_le_bytes(
                self.data
                    .clone()
                    .try_into()
                    .expect("couldnt parse data to usize"),
            )
            .to_string(),
        }
    }
}

impl SerializeDeserialize for Field {
    fn serialize(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.push(
            self.len()
                .try_into()
                .expect("couldnt parse length of field"),
        );

        bytes.push(self.key.len().try_into().expect("couldnt parse key len"));

        bytes.extend(self.key);

        match self.datatype {
            KeyType::String => {
                bytes.push(0x01);
                bytes.push(self.data.len().try_into().expect("couldnt parse data len"));
            }
            KeyType::UInt64 => bytes.push(0x02),
        }

        bytes.extend(self.data);

        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Field, FileError> {
        let (input, key) = length_count(u8(), u8()).parse(bytes)?;

        let (input, datatype) = u8().parse(input)?;

        let datatype = match datatype {
            0x01 => KeyType::String,
            0x02 => KeyType::UInt64,
            _ => return Err(FileError::Keytype(datatype)),
        };

        let (_, data) = match datatype {
            KeyType::String => length_count(u8(), u8()).parse(input)?,
            KeyType::UInt64 => count(u8(), 8).parse(input)?,
        };

        Ok(Field {
            key,
            datatype,
            data,
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

#[cfg(test)]
mod test {
    use super::*;

    mod pagetests {
        #[test]
        fn serialize() {}

        #[test]
        fn deserialize() {}
    }

    mod datatests {
        use super::*;
        use ::std::io::Write;

        #[test]
        fn serialize() {
            let data = Data {
                object: vec![
                    Field::new(b"foo".to_vec(), KeyType::String, b"bar".to_vec()),
                    Field::new(
                        b"test".to_vec(),
                        KeyType::UInt64,
                        1234_usize.to_le_bytes().to_vec(),
                    ),
                ],
            };

            let mut expected = vec![
                0x03, //pagetype
                0x02, // has 2 fields
                0x09, // size of field
                0x03, b'f', b'o', b'o', // key is 3 chars long, foo
                0x01, 0x03, b'b', b'a',
                b'r', // data is of type string and is 3 chars long, bar
                0x0E, // size of field
                0x04, b't', b'e', b's', b't', // key is 4 chars long, test
                0x02, // data is uint64
            ];

            expected.extend(1234_usize.to_le_bytes());

            let bytes = data.serialize();

            assert_eq!(expected.to_vec(), bytes)
        }

        #[test]
        fn deserialize() {
            let mut bytes = vec![
                0x02, // has 2 fields
                0x09, // size of field
                0x03, b'f', b'o', b'o', // key is 3 chars long, foo
                0x01, 0x03, b'b', b'a',
                b'r', // data is of type string and is 3 chars long, bar
                0x0E, // size of field
                0x04, b't', b'e', b's', b't', // key is 4 chars long, test
                0x02, // data is uint64
            ];
            bytes.extend(1234_usize.to_le_bytes()); // data 1234

            let data_expected = Data {
                object: vec![
                    Field::new(b"foo".to_vec(), KeyType::String, b"bar".to_vec()),
                    Field::new(
                        b"test".to_vec(),
                        KeyType::UInt64,
                        1234_usize.to_le_bytes().to_vec(),
                    ),
                ],
            };

            let mut buf = [0x00; PAGESIZE_NO_HEADER];
            buf[0..bytes.len()].as_mut().write_all(&bytes).unwrap();

            let data = Data::deserialize(&buf);

            if let Ok(data) = data {
                for (field_expected, field) in data_expected.object.iter().zip(data.object) {
                    assert_eq!(field_expected.key, field.key);
                    assert_eq!(field_expected.data, field.data);
                    assert_eq!(field_expected.datatype, field.datatype);
                }
            } else if let Err(e) = data {
                eprint!("{e}");
                panic!()
            }
        }
    }

    mod leaftests {
        use super::*;
        use ::std::io::Write;

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
