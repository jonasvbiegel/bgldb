use std::{
    collections::{BTreeMap, HashMap},
    num::ParseIntError,
};
mod db;

fn main() {
    let row_version: u32 = 1;

    let mut col_string = db::Column::new(true, db::ColumnContent::Str(BTreeMap::new()));
    let success_string = col_string.insert(row_version, db::Value::Str("hej".to_string()));

    let mut col_int = db::Column::new(false, db::ColumnContent::Int(BTreeMap::new()));
    let success_int = col_int.insert(row_version, db::Value::Int(2));

    if success_int {
        println!("gaming");
    } else {
        println!("no");
    }

    let table = db::Table::new("Gaming".to_string());
}
