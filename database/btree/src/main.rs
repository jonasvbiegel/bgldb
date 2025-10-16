mod bptree;
use std::ops::IndexMut;

use crate::bptree::BPTree;

fn main() {
    let mut bptree = BPTree::<i32>::new(4);
    bptree.insert(1);
    bptree.insert(2);
    bptree.insert(3);
    bptree.insert(4);
    bptree.insert(5);
    bptree.insert(6);
    bptree.insert(7);

    println!("{bptree:?}")

    // let mut n: Node<i32> = Node::new(4);
    // n.keys.push(3);
    // n.keys.push(5);
    //
    // let mut c1: Node<i32> = Node::new(4);
    // c1.keys.push(1);
    // c1.keys.push(2);
    //
    // let mut c2: Node<i32> = Node::new(4);
    // c2.keys.push(3);
    // c2.keys.push(4);
    //
    // let mut c3: Node<i32> = Node::new(4);
    // c3.keys.push(5);
    // c3.keys.push(6);
    // c3.keys.push(7);
    //
    // n.children.push(c1);
    // n.children.push(c2);
    // n.children.push(c3);
    //
    // n.insert(&8);
    // print_tree(&n);
    // println!();
    //
    // n.insert(&9);
    // print_tree(&n);
    // println!();
    //
    // n.insert(&0);
    // print_tree(&n);
    // println!();
    //
    // n.insert(&10);
    // print_tree(&n);
    // println!();
}

fn print_tree<T: std::fmt::Display>(root: &Node<T>) {
    for k in &root.keys {
        print!("{k} ");
    }
    println!();
    for c in &root.children {
        for k in &c.keys {
            print!("{k} ");
        }
        print!("-> ");
    }
    println!()
}

#[derive(Debug)]
struct Node<T> {
    order: usize,
    keys: Vec<T>,
    children: Vec<Node<T>>,
    // next_leaf: Option<Box<Node<T>>>,
}

impl<T> Node<T>
where
    T: std::fmt::Display + std::cmp::PartialOrd + std::cmp::Ord + Clone + std::fmt::Debug,
{
    fn new(order: usize) -> Self {
        Self {
            order,
            keys: Vec::new(),
            children: Vec::new(),
            // next_leaf: None,
        }
    }

    fn get_child_mut(&mut self, value: &T) -> Option<&mut Node<T>> {
        if !self.children.is_empty() {
            if let Some(n) = self.keys.iter().position(|x| *x > *value) {
                return Some(self.children.index_mut(n));
            } else if let Some(c) = self.children.last_mut() {
                return Some(c);
            }
        }

        None
    }

    fn sort(&mut self) {
        self.keys.sort();
        self.children.sort_by(|x, y| {
            x.keys
                .first()
                .partial_cmp(&y.keys.first())
                .expect("couldnt sort")
        });
    }

    fn insert(&mut self, value: &T) {
        println!("inserting {value} on {self:?}");
        if let Some(c) = self.get_child_mut(value) {
            println!("found child {c:?}");
            if !c.children.is_empty() {
                c.insert(value);
            } else {
                c.keys.push(value.clone());
                c.sort();
            }

            println!("keylen {} splitif {}", c.keys.len(), c.order - 1);
            if c.keys.len() > c.order - 1 {
                println!("splitting {c:?}");
                let n = c.split();
                println!("{n:?}");
                self.keys.push(n.keys.first().unwrap().clone());
                self.children.push(n);
            }
        }
    }

    // HACK: sort this while running by looking for the value and inserting after instead of always
    // sorting the node
    //
    // TODO: if the node is the root, do something to make a new root (should probably be done in
    // the actual b+ tree struct?)
    fn split(&mut self) -> Node<T> {
        let mut new_child: Node<T> = Node::new(self.order);

        // insert the rightmost keys of the node into the new node and sort
        for _ in self.keys.len().div_ceil(2)..self.keys.len() {
            new_child
                .keys
                .push(self.keys.remove(self.keys.len().div_ceil(2)));
        }

        new_child.keys.sort();

        // if node is not a leaf, do the same for the children
        if !self.children.is_empty() {
            for _ in self.children.len().div_ceil(2)..self.children.len() {
                new_child
                    .children
                    .push(self.children.remove(self.children.len().div_ceil(2)));
            }

            new_child.children.sort_by(|x, y| {
                x.keys
                    .first()
                    .unwrap()
                    .partial_cmp(y.keys.first().unwrap())
                    .expect("couldnt compare or sort")
            });
        }

        new_child
    }
}

// garbage
//
// display the tree
// fn display(&self) {
//     if self.children.is_empty() {
//         for k in &self.keys {
//             println!("{k}");
//         }
//     } else {
//         for (k, c) in zip(
//             &self.keys,
//             self.children.iter().take(self.children.len() - 1),
//         ) {
//             c.display();
//             println!("{k}");
//         }
//         self.children.last().unwrap().display();
//     }
// }
//
// // search for a key, if key exists return true
// fn search(&self, key: T) -> bool {
//     if let Some(n) = self.keys.iter().position(|x| *x >= key) {
//         if key == *self.keys.index(n) {
//             return true;
//         } else if !self.children.is_empty() {
//             return self.children.index(n).search(key);
//         }
//     } else if let Some(c) = self.children.last() {
//         return c.search(key);
//     }
//     false
// }
