use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufWriter, Write},
};

fn main() {}

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

    fn update<T: Serialize + DeserializeOwned + std::fmt::Debug, F, U>(
        &mut self,
        mut constraint: F,
        mut action: U,
    ) -> bool
    where
        F: Clone + FnMut(&T) -> bool,
        U: FnMut(&mut T),
    {
        let mut items: Vec<T> = fs::read(&self.path)
            .unwrap()
            .lines()
            .map(|l| l.unwrap())
            .map(|x| ron::from_str(&x).unwrap())
            .collect();

        for item in &mut items {
            if constraint(item) {
                action(item)
            }
        }

        let items: Vec<String> = items.iter().map(|x| ron::to_string(&x).unwrap()).collect();

        let new_file = OpenOptions::new()
            .truncate(true)
            .write(true)
            .open(&self.path);

        let mut writer = BufWriter::new(new_file.unwrap());

        for item in items {
            let success = writeln!(writer, "{item}");
            if success.is_err() {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::remove_file;

    static NAME: &str = "test";

    fn cleanup() {
        let _ = remove_file(format!("./{NAME}"));
    }

    #[test]
    fn insert() {
        let p = Person::new("Jonas", "291220021234", 22);
        let mut d = Database::new(NAME);
        let success = d.insert(&p);
        assert!(success);
        cleanup();
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
        cleanup();
    }

    #[test]
    fn remove() {
        let p1 = Person::new("Jonas", "1234", 22);
        let p2 = Person::new("Hans", "4321", 22);
        let mut d = Database::new(NAME);

        d.insert(&p1);
        d.insert(&p2);

        let rows_deleted = d.remove(|x: &Person| x.cpr == "1234");
        assert!(rows_deleted == 1);
        cleanup();
    }

    #[test]
    fn update() {
        let p1 = Person::new("Jonas", "1234", 22);
        let p2 = Person::new("Hans", "4321", 22);
        let mut d = Database::new(NAME);
        d.insert(&p1);
        d.insert(&p2);

        let success = d.update(
            |x: &Person| x.name == "Jonas",
            |x: &mut Person| x.name = "John".to_string(),
        );

        assert!(success);
        assert_ne!(
            p1.name,
            d.filter(|x: &Person| x.cpr == "1234")
                .unwrap()
                .first()
                .unwrap()
                .name
        );
        assert_eq!(
            p2.name,
            d.filter(|x: &Person| x.cpr == "4321")
                .unwrap()
                .first()
                .unwrap()
                .name
        );
    }
}
