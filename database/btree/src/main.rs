use std::iter::zip;

fn main() {
    println!("gaming");
    let mut n1 = Node::<i32>::new(4);
    n1.keys.push(1);
    n1.keys.push(2);

    let mut n2 = Node::<i32>::new(4);
    n2.keys.push(3);
    n2.keys.push(4);

    let mut n3 = Node::<i32>::new(4);
    n3.keys.push(5);
    n3.keys.push(6);
    n3.keys.push(7);

    let mut root = Node::<i32>::new(4);

    root.keys.push(3);
    root.keys.push(5);
    root.children.push(n1);
    root.children.push(n2);
    root.children.push(n3);

    root.display();
}

struct Node<T> {
    keys: Vec<T>,
    children: Vec<Node<T>>,
    next_leaf: Option<Box<Node<T>>>,
}

impl<T: std::fmt::Display> Node<T> {
    fn new(max_children: usize) -> Self {
        Self {
            keys: Vec::with_capacity(max_children - 1),
            children: Vec::with_capacity(max_children),
            next_leaf: None,
        }
    }

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
