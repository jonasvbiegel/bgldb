use smallvec::SmallVec;
use std::collections::{BTreeMap, HashMap};

type RowVersion = u32;
const MAX_COLUMNS: u16 = 1024;

pub struct TableBuilder {
    columns: HashMap<String, Column>,
}

impl TableBuilder {
    pub fn new() -> TableBuilder {
        TableBuilder {
            columns: HashMap::new(),
        }
    }

    pub fn str(mut self, name: String) -> TableBuilder {
        self.columns.insert(
            name,
            Column::new(false, ColumnContent::Str(BTreeMap::new())),
        );

        self
    }

    pub fn int(mut self, name: String) -> TableBuilder {
        self.columns.insert(
            name,
            Column::new(false, ColumnContent::Int(BTreeMap::new())),
        );

        self
    }

    pub fn build(self, name: String) -> Table {
        if self.columns.len() > MAX_COLUMNS as usize {
            //throw error
        }

        Table {
            name,
            columns: self.columns,
        }
    }
}

pub struct Database {
    name: String,
    tables: HashMap<String, Table>,
}

impl Database {
    pub fn new(name: String) -> Database {
        Database {
            name,
            tables: HashMap::new(),
        }
    }

    pub fn insert_table(name: String) -> bool {
        todo!()
    }
}

pub struct Table {
    name: String,
    columns: HashMap<String, Column>,
}

impl Table {
    pub fn get_name(self) -> String {
        self.name
    }

    pub fn get_columns(self) -> HashMap<String, Column> {
        self.columns
    }

    pub fn get_column_mut(&mut self, column_name: String) -> Option<&mut Column> {
        self.columns.get_mut(&column_name)
    }

    pub fn get_column(&self, column_name: String) -> Option<&Column> {
        self.columns.get(&column_name)
    }

    pub fn insert(&mut self, items: SmallVec<[Value; MAX_COLUMNS as usize]>) -> bool {
        let mut gaming = true;

        for c in &mut self.columns.values_mut() {
            for v in &items {
                if c.insert_row(1, v.clone()) {
                    gaming = true;
                    continue;
                }

                gaming = false;
            }
        }

        gaming
    }

    pub fn print_table(self) -> bool {
        todo!()
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

    pub fn insert_row(&mut self, rv: RowVersion, v: Value) -> bool {
        self.rows.insert(rv, v)
    }

    pub fn get_rows(&self) -> &ColumnContent {
        &self.rows
    }
}

#[derive(Clone)]
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

    // pub fn get_rows(&self) -> Vec<Vec<u8>> {
    //     match self {
    //         ColumnContent::Str(col) => {}
    //     }
    // }
}
