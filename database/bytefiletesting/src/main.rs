use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
};

fn main() {
    let mut file = File::create("./lol").unwrap();
}

struct Document {
    fields: Vec<Field>,
}

struct Field {
    name: String,
    data: DataType,
}

impl Field {
    fn new(name: String, data: DataType) -> Self {
        Self { name, data }
    }

    fn to_bytes(&self) -> Option<Vec<u8>> {
        let mut bytes = Vec::<u8>::new();

        // if the size of the field name is under 256 in len, push the len in bytes
        if self.name.len() <= 0xFFusize {
            bytes.push(*self.name.len().to_le_bytes().first().unwrap());
        } else {
            return None;
        }

        // push the name as chars in bytes
        for b in self.name.chars() {
            bytes.push(b as u8);
        }

        match &self.data {
            // if string push the string length then the string
            DataType::String(s) => {
                let data_len = s.len() as u32;
                bytes.push(0x01u8);
                data_len.to_le_bytes().iter().for_each(|b| bytes.push(*b));
                for c in s.chars() {
                    bytes.push(c as u8);
                }
            }
            // if i32 push it, will always be 4 bytes (for reading)
            DataType::Int32(i) => {
                bytes.push(0x02u8);
                i.to_le_bytes().iter().for_each(|b| bytes.push(*b));
            }
            // if float push the float, will also be 4 bytes
            DataType::Float32(f) => {
                bytes.push(0x03u8);
                f.to_le_bytes().iter().for_each(|b| bytes.push(*b));
            }
        }

        Some(bytes)
    }
}

enum DataType {
    String(String), //1
    Int32(i32),     //2
    Float32(f32),   //3
}
