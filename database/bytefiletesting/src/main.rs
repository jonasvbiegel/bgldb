use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
};

fn main() {
    let f_name = Field::new("name".to_string(), DataType::String("Jonas".to_string()));
    let f_age = Field::new("age".to_string(), DataType::I32(22));
    let f_float = Field::new("float".to_string(), DataType::Float(-1.0));

    let mut file = File::create("./lol").unwrap();
    let _ = file.write_all(&f_name.to_bytes());
    let _ = file.write_all(&f_age.to_bytes());
    let _ = file.write_all(&f_float.to_bytes());

    let _ = &f_name.to_bytes().iter().for_each(|b| println!("{b}"));
    println!();
    let _ = &f_float.to_bytes().iter().for_each(|b| println!("{b}"));
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

    fn get_size(&self) -> u8 {
        todo!()
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();

        // if the size of the field name is under 256 in len, push the len in bytes
        if self.name.len() <= 0xFFusize {
            bytes.push(*self.name.len().to_le_bytes().first().unwrap());
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
            DataType::I32(i) => {
                bytes.push(0x02u8);
                i.to_le_bytes().iter().for_each(|b| bytes.push(*b));
            }
            // if float push the float
            DataType::Float(f) => {
                bytes.push(0x03u8);
                f.to_le_bytes().iter().for_each(|b| bytes.push(*b));
            }
        }

        bytes
    }
}

enum DataType {
    String(String), //1
    I32(i32),       //2
    Float(f32),     //3
}
