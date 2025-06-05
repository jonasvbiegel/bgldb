use std::{
    collections::{BTreeMap, HashMap},
    num::ParseIntError,
};

use db::TableBuilder;
mod db;

fn main() {
    let row_version: u32 = 1;

    let mut table = TableBuilder::new()
        .str("name".to_string())
        .int("age".to_string())
        .build("People".to_string());

    let s1 = table
        .get_column_mut("name".to_string())
        .unwrap()
        .insert_row(row_version, db::Value::Str("dam".to_string()));

    let s2 = table
        .get_column_mut("age".to_string())
        .unwrap()
        .insert_row(row_version, db::Value::Int(13));

    if s1 & s2 {
        println!("gaming")
    }
}
