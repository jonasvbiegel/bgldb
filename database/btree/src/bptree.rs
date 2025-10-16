use std::ops::{Index, IndexMut};

#[derive(Debug)]
/// A tree structure with multiple keys and children for each node.
pub struct BPTree<T> {
    order: usize,
    root: Node<T>,
}

impl<T> BPTree<T>
where
    T: std::fmt::Display + std::cmp::PartialOrd + std::cmp::Ord + Clone,
{
    /// Creates and returns a new BPTree with type T.
    /// `Order` is the maximum amount of children each node can hold.
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
            let child = self.root.split();

            new_root.keys.push(child.keys.index(0).clone());
            new_root.children.push(child);
            new_root.children.push(self.root.clone());

            self.root = new_root;
            self.root.sort();
        }
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
    T: std::fmt::Display + std::cmp::PartialOrd + std::cmp::Ord + Clone,
{
    /// Creates a new Node
    fn new(order: usize) -> Self {
        Self {
            order,
            keys: Vec::new(),
            children: Vec::new(),
            // next_leaf: None,
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

    /// Inserts a value recursively into the node
    fn insert(&mut self, value: &T) {
        if let Some(c) = self.get_child_mut(value) {
            if !c.children.is_empty() {
                c.insert(value);
            } else {
                c.keys.push(value.clone());
                c.sort();
            }

            if c.keys.len() > c.order - 1 {
                let n = c.split();
                self.keys.push(n.keys.first().unwrap().clone());
                self.children.push(n);
            }
        } else {
            self.keys.push(value.clone());
        }
    }

    // HACK: sort this while running by looking for the value and inserting after instead of always
    // sorting the node
    // see sort() under this function

    /// Splits the child, returning the child created after the split
    fn split(&mut self) -> Node<T> {
        let mut new_child: Node<T> = Node::new(self.order);

        // insert the rightmost keys of the node into the new node and sort
        for _ in self.keys.len() / 2..self.keys.len() {
            new_child
                .keys
                .push(self.keys.remove(self.keys.len().div_ceil(2)));
        }

        new_child.keys.sort();

        // if node is not a leaf, do the same for the children
        if !self.children.is_empty() {
            for _ in self.children.len() / 2..self.children.len() {
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

    /// Sorts the keys and children of the node
    fn sort(&mut self) {
        self.keys.sort();
        self.children.sort_by(|x, y| {
            x.keys
                .first()
                .partial_cmp(&y.keys.first())
                .expect("couldnt sort")
        });
    }
}
