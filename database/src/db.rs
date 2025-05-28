use std::collections::{BTreeMap, HashMap};

type RowVersion = u32;

pub struct Table {
    name: String,
    columns: HashMap<String, Column>,
}

impl Table {
    pub fn new(n: String) -> Table {
        Table {
            name: n,
            columns: HashMap::new(),
        }
    }
}

pub struct Column {
    primary: bool,
    rows: ColumnContent,
}

impl Column {
    pub fn new(p: bool, content: ColumnContent) -> Column {
        match content {
            ColumnContent::Str(_) => Column {
                primary: p,
                rows: ColumnContent::Str(BTreeMap::new()),
            },
            ColumnContent::Int(_) => Column {
                primary: p,
                rows: ColumnContent::Int(BTreeMap::new()),
            },
        }
    }

    pub fn insert(&mut self, rv: RowVersion, v: Value) -> bool {
        self.rows.insert(rv, v)
    }
}

pub enum Value {
    Str(String),
    Int(i32),
}

pub enum ColumnContent {
    Str(BTreeMap<RowVersion, String>),
    Int(BTreeMap<RowVersion, i32>),
}

impl ColumnContent {
    pub fn insert(&mut self, key: RowVersion, value: Value) -> bool {
        match (self, value) {
            (ColumnContent::Str(col), Value::Str(v)) => {
                col.insert(key, v);
                true
            }
            (ColumnContent::Int(col), Value::Int(v)) => {
                col.insert(key, v);
                true
            }
            _ => false,
        }
    }
}
