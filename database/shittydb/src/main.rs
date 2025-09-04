use serde::{Deserialize, Serialize, de::DeserializeOwned};
use simple_stopwatch::Stopwatch;
use std::{
    fs::{self, File, OpenOptions, remove_file},
    io::{BufRead, Write},
};

fn main() {
    let p = Person::new("Benchmark Man", "123123123", 100);

    let _ = remove_file("./people");

    let mut d = Database::new("people");

    d.insert(&p);

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
    path: String,
    file: File,
}

impl Database {
    fn new(name: &str) -> Database {
        Database {
            path: format!("./{name}"),
            file: OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(format!("./{name}"))
                .unwrap(),
        }
    }

    fn insert<'a, T: Serialize + Deserialize<'a>>(&mut self, item: &T) -> bool {
        let item_ron = ron::to_string(&item).unwrap();
        let success = self.file.write_all(item_ron.as_bytes()).is_ok();
        if success {
            let _ = self.file.write_all(b"\n");
            true
        } else {
            false
        }
    }

    fn find_all<T: Serialize + DeserializeOwned>(&mut self) -> Option<Vec<T>> {
        let items: Vec<T> = fs::read(&self.path)
            .unwrap()
            .lines()
            .map(|l| l.unwrap())
            .map(|l| ron::from_str(&l).unwrap())
            .collect();

        if !items.is_empty() { Some(items) } else { None }
    }

    fn filter<T: Serialize + DeserializeOwned, F>(&self, constraint: F) -> Option<Vec<T>>
    where
        F: FnMut(&T) -> bool,
    {
        let items: Vec<T> = fs::read(&self.path)
            .unwrap()
            .lines()
            .map(|l| l.unwrap())
            .map(|n| ron::from_str(&n).unwrap())
            .filter(constraint)
            .collect();

        if !items.is_empty() { Some(items) } else { None }
    }

    // for these, find 2 byte locations and either change or delete from one byte to another

    fn update() {
        todo!()
    }

    fn delete() {
        todo!()
    }
}
