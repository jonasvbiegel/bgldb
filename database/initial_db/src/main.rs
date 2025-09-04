use db::TableBuilder;
use smallvec::SmallVec;
mod db;

fn main() {
    let row_version: u32 = 1;

    let mut table = TableBuilder::new()
        .str("name".to_string())
        .int("age".to_string())
        .build("People".to_string());

    let v1 = db::Value::Str("lol".to_string());
    let v2 = db::Value::Int(123);

    let mut sv = SmallVec::new();
    sv.insert(0, v1);
    sv.insert(1, v2);

    let mut s = table.insert(sv);

    println!("{s}");

    let v3 = db::Value::Str("lol".to_string());

    let mut sv2 = SmallVec::new();
    sv2.insert(0, v3.clone());
    sv2.insert(1, v3);

    s = table.insert(sv2);

    println!("{s}");

    // table
    //     .get_column("name".to_string())
    //     .expect("troll")
    //     .get_rows();

    // table.get_column("name".to_string()).expect("t").get_rows();

    // for (k, v) in table
    //     .get_column("name".to_string())
    //     .expect("troll")
    //     .get_rows()
    //     .get_rows
    // {
    //     println!(v);
    // }
}
