use after_test::cleanup;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    fs::{self, File, OpenOptions, remove_file},
    io::{BufRead, Write},
};

fn main() {
    let p = Person::new("Benchmark Man", "123123123", 100);
    let p1 = Person::new("Missefar", "123123123", 100);
    let p2 = Person::new("Jonas", "123123123", 100);
    let p3 = Person::new("John", "addsd", 123123123);

    let _ = remove_file("./people");

    let mut d = Database::new("people");

    d.insert(&p);
    d.insert(&p1);
    d.insert(&p2);
    d.insert(&p3);

    let lol = d.filter(|p: &Person| p.name == "Benchmark Man");
    if let Some(n) = lol {
        println!("{}", n.first().unwrap().name)
    }

    let items = d.remove(|x: &Person| x.name == "Jonas");
    println!("{items}");
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

    fn get_all<T: Serialize + DeserializeOwned>(&mut self) -> Option<Vec<T>> {
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

    fn remove<T: Serialize + DeserializeOwned, F>(&mut self, mut constraint: F) -> usize
    where
        F: FnMut(&T) -> bool,
    {
        let items_count = fs::read_to_string(&self.path).unwrap().lines().count();

        let items_string: Vec<String> = fs::read_to_string(&self.path)
            .unwrap()
            .lines()
            .map(|x| ron::from_str(x).unwrap())
            .filter(|x| !constraint(x))
            .map(|x| ron::to_string(&x).unwrap())
            .collect();

        let items_deleted = items_count - items_string.len();

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&self.path)
            .unwrap();

        for l in items_string {
            let _ = file.write_all(l.as_bytes());
            let _ = file.write(b"\n");
        }
        items_deleted
    }

    fn update() {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static NAME: &str = "test";

    fn cleanup() {
        let _ = remove_file(format!("./{NAME}"));
    }

    #[test]
    fn insert() {
        let p = Person::new("Jonas", "291220021234", 22);
        let mut d = Database::new(NAME);
        let success = d.insert(&p);
        cleanup();
        assert!(success);
    }

    #[test]
    fn filter() {
        let p1 = Person::new("Jonas", "1234", 22);
        let p2 = Person::new("Hans", "4321", 22);
        let mut d = Database::new(NAME);

        d.insert(&p1);
        d.insert(&p2);

        let found = d.filter(|p: &Person| p.cpr == "4321");
        assert!(found.is_some());
    }

    #[test]
    fn remove() {
        todo!()
    }

    #[test]
    fn update() {
        todo!()
    }
}
