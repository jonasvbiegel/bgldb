use serde::{Deserialize, Serialize, de::DeserializeOwned};
use simple_stopwatch::Stopwatch;
use std::{
    fmt::format,
    fs::{self, File},
    io::{BufRead, Read, Write},
};

fn main() {
    let p = Person::new("Benchmark Man", "123123123", 100);

    let mut d = Database::new("people");

    let mut sw = Stopwatch::start_new();
    for i in 1..=1_000_000 {
        d.insert(&p);

        match i {
            1 => {
                println!("inserted 1 element in {}ms", sw.ms())
            }
            10 => {
                println!("inserted 10 elements in {}ms", sw.ms())
            }
            100 => {
                println!("inserted 100 elements in {}ms", sw.ms())
            }
            1_000 => {
                println!("inserted 1000 elements in {}ms", sw.ms())
            }
            10_000 => {
                println!("inserted 10000 elements in {}ms", sw.ms())
            }
            100_000 => {
                println!("inserted 100000 elements in {}ms", sw.ms())
            }
            1_000_000 => {
                println!("inserted 1000000 elements in {}ms", sw.ms())
            }
            _ => {}
        }
    }

    sw.restart();
    let _: Vec<Person> = d.find_all().unwrap();
    println!("retrieved all elements in {}ms", sw.ms());
}

#[derive(Serialize, Deserialize, Debug)]
struct Person {
    name: String,
    cpr: String,
    age: usize,
}

impl Person {
    fn new(name: &str, cpr: &str, age: usize) -> Person {
        Person {
            name: name.to_string(),
            cpr: cpr.to_string(),
            age,
        }
    }
}

struct Database {
    name: String,
    file: File,
}

// impl<'a, T: Serialize + Deserialize<'a>> Database {
impl Database {
    fn new(name: &str) -> Database {
        Database {
            name: name.to_string(),
            file: File::create(format!("./{}", name)).unwrap(),
        }
    }

    fn insert<'a, T: Serialize + Deserialize<'a>>(&mut self, item: &T) -> bool {
        // todo!()

        let item_ron = ron::to_string(&item).unwrap();
        let success = self.file.write_all(item_ron.as_bytes()).is_ok();
        if success {
            let _ = self.file.write_all(b"\n");
            true
        } else {
            false
        }
    }

    // fn find<'a, T: Serialize + Deserialize<'a>>(item: T) {
    //     todo!()
    // }
    //
    fn find_all<T: Serialize + DeserializeOwned>(&mut self) -> Option<Vec<T>> {
        let items: Vec<T> = fs::read(format!("./{}", self.name))
            .unwrap()
            .lines()
            .map(|l| l.unwrap())
            .map(|l| ron::from_str(&l).unwrap())
            .collect();

        if !items.is_empty() { Some(items) } else { None }
    }

    fn find<T: Serialize>(constraint: String) -> T {
        todo!()
    }
}
