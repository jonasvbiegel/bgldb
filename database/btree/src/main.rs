use std::{iter::zip, ops::Index};

fn main() {
    // println!("gaming");
    // let mut n1 = Node::<i32>::new(4);
    // n1.keys.push(1);
    // n1.keys.push(2);
    //
    // let mut n2 = Node::<i32>::new(4);
    // n2.keys.push(3);
    // n2.keys.push(4);
    //
    // let mut n3 = Node::<i32>::new(4);
    // n3.keys.push(5);
    // n3.keys.push(6);
    // n3.keys.push(7);
    //
    // let mut root = Node::<i32>::new(4);
    //
    // root.keys.push(3);
    // root.keys.push(5);
    // root.children.push(n1);
    // root.children.push(n2);
    // root.children.push(n3);
    //
    // root.display();
    //
    // println!("{}", root.search(1));
    // println!("{}", root.search(7));
    // println!("{}", root.search(11));
}

struct Node<T> {
    order: usize,
    keys: Vec<T>,
    children: Vec<Node<T>>,
    next_leaf: Option<Box<Node<T>>>,
}

impl<T: std::fmt::Display + std::cmp::PartialOrd> Node<T> {
    fn new(order: usize) -> Self {
        Self {
            order,
            keys: Vec::with_capacity(order - 1),
            children: Vec::with_capacity(order),
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

    // --------------------
    // | INSERT FUNCTIONS |
    // --------------------
    fn split_child(&self) {
        todo!()
    }

    fn insert_non_full(&self) {
        todo!()
    }

    fn insert(&self) {
        todo!()
    }

    // --------------------
    // | DELETE FUNCTIONS |
    // --------------------
    fn delete_key_helper(&self, key: T) {
        todo!()
    }

    fn find_key(&self, key: T) -> Option<i32> {
        // find index of key in node, maybe useless? idk
        todo!()
    }

    // CHECK IF INDICES OF THESE FUNCTIONS ARE ACTUALLY NEEDED LOL
    fn remove_from_leaf(&self, index: i32) {
        todo!()
    }

    fn get_predecessor(&self) -> Option<T> {
        todo!()
    }

    fn fill(&self, index: i32) {
        todo!()
    }

    fn borrow_from_prev(&self, index: i32) {
        todo!()
    }

    fn borrow_from_next(&self, index: i32) {
        todo!()
    }

    fn merge(&self, index: i32) {
        todo!()
    }
}

// goofy implementation, ignore plz

// struct Node<T> {
//     max_keys: usize,
//     keys: Vec<Key<T>>,
// }
//
// impl<T: std::fmt::Display> Node<T> {
//     fn new(max_keys: usize) -> Self {
//         Self {
//             max_keys,
//             keys: Vec::with_capacity(max_keys),
//         }
//     }
//
//     fn display(&self) {
//         if self.is_leaf() {
//             for k in &self.keys {
//                 println!("{}", k.value);
//             }
//         } else {
//             for n in self.keys.iter() {
//                 for c in n.children.iter().take(n.children.len() - 1) {
//                     c.display();
//                 }
//                 println!("{}", n.value);
//                 n.children.last().unwrap().display();
//             }
//         }
//     }
//
//     fn is_leaf(&self) -> bool {
//         for k in &self.keys {
//             if !k.children.is_empty() {
//                 return false;
//             }
//         }
//         true
//     }
// }
//
// struct Key<T> {
//     value: T,
//     children: Vec<Node<T>>,
// }
//
// impl<T> Key<T> {
//     fn new(value: T) -> Self {
//         Self {
//             value,
//             children: Vec::with_capacity(2),
//         }
//     }
// }
