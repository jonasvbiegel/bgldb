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
