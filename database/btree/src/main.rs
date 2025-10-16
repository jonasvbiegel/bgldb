use std::{
    iter::zip,
    ops::{Index, IndexMut},
};
mod btree;

fn main() {
    println!("lol");

    let mut n: Node<i32> = Node::new(2);
    n.keys.push(3);
    n.keys.push(5);

    for k in &n.keys {
        println!("{k}")
    }

    let mut c1: Node<i32> = Node::new(2);
    c1.keys.push(1);
    c1.keys.push(2);

    let mut c2: Node<i32> = Node::new(2);
    c2.keys.push(3);
    c2.keys.push(4);

    let mut c3: Node<i32> = Node::new(2);
    c3.keys.push(5);
    c3.keys.push(6);
    c3.keys.push(7);
    c3.keys.push(8);

    n.children.push(c1);
    n.children.push(c2);
    n.children.push(c3);

    // this splits wrong, should sort and find the child in a different way
    let success = n.split_child(9);
    println!("{success}");

    if success {
        for k in &n.keys {
            println!("{k}")
        }

        for (i, c) in n.children.iter().enumerate() {
            println!("child {i}");
            for k in c.keys.clone() {
                println!("{k}");
            }
        }
    }
}

struct Node<T> {
    order: usize,
    keys: Vec<T>,
    children: Vec<Node<T>>,
    next_leaf: Option<Box<Node<T>>>,
}

impl<T: std::fmt::Display + std::cmp::PartialOrd + Copy> Node<T> {
    fn new(order: usize) -> Self {
        Self {
            order,
            keys: Vec::new(),
            children: Vec::new(),
            next_leaf: None,
        }
    }

    // display the tree
    fn display(&self) {
        if self.children.is_empty() {
            for k in &self.keys {
                println!("{k}");
            }
        } else {
            for (k, c) in zip(
                &self.keys,
                self.children.iter().take(self.children.len() - 1),
            ) {
                c.display();
                println!("{k}");
            }
            self.children.last().unwrap().display();
        }
    }

    // search for a key, if key exists return true
    fn search(&self, key: T) -> bool {
        if let Some(n) = self.keys.iter().position(|x| *x >= key) {
            if key == *self.keys.index(n) {
                return true;
            } else if !self.children.is_empty() {
                return self.children.index(n).search(key);
            }
        } else if let Some(c) = self.children.last() {
            return c.search(key);
        }
        false
    }

    fn get_child_mut(&mut self, value: T) -> Option<&mut Node<T>> {
        if let Some(n) = self.keys.iter().position(|x| *x > value) {
            return Some(self.children.index_mut(n));
        } else if let Some(c) = self.children.last_mut() {
            return Some(c);
        }

        None
    }

    fn split_child(&mut self, value: T) -> bool {
        let mut new_child: Node<T> = Node::new(self.order);

        if let Some(c) = self.get_child_mut(value) {
            let leaf = c.children.is_empty();

            if !leaf {
                for i in c.children.len().div_ceil(2)..c.children.len() {
                    new_child.children.push(c.children.remove(i - 1));
                }
            }

            for _ in c.keys.len().div_ceil(2)..c.keys.len() {
                new_child.keys.push(c.keys.remove(c.keys.len().div_ceil(2)));
            }

            self.keys.push(*new_child.keys.first().unwrap());
            println!("pushing key to self: {}", new_child.keys.first().unwrap());
            self.children.push(new_child);

            // sort the children by key value
            self.children.sort_by(|a, b| {
                a.keys
                    .first()
                    .unwrap()
                    .partial_cmp(b.keys.first().unwrap())
                    .expect("troll")
            });

            return true;
        }

        false
    }

    // // --------------------
    // // | INSERT FUNCTIONS |
    // // --------------------
    // fn split_child(&mut self, child: &mut Node<T>, index: usize) {
    //     // index is index of child i
    //     // think?
    //     let mut new_child: Node<T> = Node::new(self.order);
    //
    //     for i in child.order..child.keys.len() - 1 {
    //         new_child.keys.push(child.keys.remove(i));
    //     }
    //
    //     if !child.children.is_empty() {
    //         for i in child.order..child.children.len() {
    //             new_child.children.push(child.children.remove(i));
    //         }
    //     }
    //
    //     self.keys.insert(index, *new_child.keys.last().unwrap());
    //     self.children.insert(index, new_child);
    // }
    //
    // fn insert_non_full(&self) {
    //     todo!()
    // }
    //
    // fn insert(&self) {
    //     todo!()
    // }
    //
    // // --------------------
    // // | DELETE FUNCTIONS |
    // // --------------------
    // fn delete_key_helper(&self, key: T) {
    //     todo!()
    // }
    //
    // fn find_key(&self, key: T) -> Option<i32> {
    //     // find index of key in node, maybe useless? idk
    //     todo!()
    // }
    //
    // // CHECK IF INDICES OF THESE FUNCTIONS ARE ACTUALLY NEEDED LOL
    // fn remove_from_leaf(&self, index: i32) {
    //     todo!()
    // }
    //
    // fn get_predecessor(&self) -> Option<T> {
    //     todo!()
    // }
    //
    // fn fill(&self, index: i32) {
    //     todo!()
    // }
    //
    // fn borrow_from_prev(&self, index: i32) {
    //     todo!()
    // }
    //
    // fn borrow_from_next(&self, index: i32) {
    //     todo!()
    // }
    //
    // fn merge(&self, index: i32) {
    //     todo!()
    // }
}
