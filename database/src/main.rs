use std::{
    collections::{BTreeMap, HashMap},
    num::ParseIntError,
};
mod db;

fn main() {
    let row_version: u32 = 1;

    let mut table = db::Table::new("Gaming".to_string());
    let col_string = db::Column::new(true, db::ColumnContent::Str(BTreeMap::new()));

    table.add_column("StringColumn".to_string(), col_string);

    table
        .get_column_mut("StringColumn".to_string())
        .unwrap()
        .insert_row(row_version, db::Value::Str("hej".to_string()));

    let rows = table
        .get_column("StringColumn".to_string())
        .expect("TROLL")
        .get_rows();

    match rows {
        db::ColumnContent::Str(col) => {
            for (key, _value) in col.iter() {
                println!("{}", col.get(key).unwrap_or(&"empty".to_string()))
            }
        }
        db::ColumnContent::Int(col) => {
            for (key, _value) in col.iter() {
                println!("{}", col.get(key).unwrap_or(&0))
            }
        }
    }
}
