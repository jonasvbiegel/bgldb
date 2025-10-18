use std::ops::{Index, IndexMut};

#[derive(Debug)]
/// A tree structure with multiple keys and children for each node.
pub struct BPTree<T> {
    order: usize,
    root: Node<T>,
}

impl<T> BPTree<T>
where
    T: std::fmt::Display + std::cmp::PartialOrd + std::cmp::Ord + Clone + std::fmt::Debug,
{
    /// Creates and returns a new BPTree with type T.
    /// - `Order` is the maximum amount of children each node can hold.
    pub fn new(order: usize) -> Self {
        Self {
            order,
            root: Node::new(order),
        }
    }

    /// Inserts a value `T` into the tree.
    pub fn insert(&mut self, value: T) {
        self.root.insert(&value);

        if self.root.keys.len() > self.order - 1 {
            let mut new_root: Node<T> = Node::new(self.order);
            let mut child = self.root.split();

            if child.children.is_empty() {
                new_root.keys.push(child.keys.index(0).clone());
            } else {
                new_root.keys.push(child.keys.remove(0));
            }

            new_root.children.push(self.root.clone());
            new_root.children.push(child);

            self.root = new_root;
        }
    }

    /// Searches for a value T in the tree. Returns true if the value is found.
    pub fn search(&self, value: T) -> bool {
        self.root.search(value)
    }
}

#[derive(Clone, Debug)]
struct Node<T> {
    order: usize,
    keys: Vec<T>,
    children: Vec<Node<T>>,
}

impl<T> Node<T>
where
    T: std::fmt::Display + std::cmp::PartialOrd + std::cmp::Ord + Clone + std::fmt::Debug,
{
    /// Creates a new Node
    fn new(order: usize) -> Self {
        Self {
            order,
            keys: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Searches for a node with value T in the children of `self`. Returns `true` if a node with value T is
    /// found.
    fn search(&self, value: T) -> bool {
        if let Some(i) = self.keys.iter().position(|x| *x >= value) {
            if *self.keys.index(i) == value {
                return true;
            } else if !self.children.is_empty() {
                return self.children.index(i).search(value);
            }
        } else if !self.children.is_empty() {
            return self.children.last().unwrap().search(value);
        }
        false
    }

    // -----------------
    // | GET FUNCTIONS |
    // -----------------

    fn get(&self, value: T) -> &Node<T> {
        todo!()
    }

    fn get_mut(&mut self, value: T) -> &mut Node<T> {
        todo!()
    }

    // --------------------
    // | INSERT FUNCTIONS |
    // --------------------

    /// Inserts a value recursively into the node
    fn insert(&mut self, value: &T) {
        if !self.children.is_empty()
            && let Some(c) = self.get_child_mut(value)
        {
            if !c.children.is_empty() {
                c.insert(value);
            } else if let Some(i) = c.keys.iter().position(|x| *x > *value) {
                c.keys.insert(i + 1, value.clone());
            } else {
                c.keys.push(value.clone());
            }

            if c.keys.len() > c.order - 1 {
                let mut n = c.split();
                if !n.children.is_empty() {
                    self.keys.push(n.keys.remove(0));
                } else {
                    self.keys.push(n.keys.first().unwrap().clone());
                }
                self.children.push(n);
            }
        } else {
            self.keys.push(value.clone());
        }
    }

    /// Returns Some(Node<T>) if a child with the value is found, else None
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

    /// Splits the child, returning the child created after the split
    fn split(&mut self) -> Node<T> {
        let mut new_child: Node<T> = Node::new(self.order);

        // insert the rightmost keys of the node into the new node and sort
        for _ in self.keys.len() / 2..self.keys.len() {
            new_child.keys.push(self.keys.pop().unwrap());
        }
        new_child.keys.reverse();

        // if node is not a leaf, do the same for the children
        if !self.children.is_empty() {
            for _ in self.children.len().div_ceil(2)..self.children.len() {
                new_child.children.push(self.children.pop().unwrap());
            }
            new_child.children.reverse();
        }

        new_child
    }

    // --------------------
    // | DELETE FUNCTIONS |
    // --------------------
}
