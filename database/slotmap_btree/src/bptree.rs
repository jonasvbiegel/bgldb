use slotmap::*;
use std::ops::Index;
use thiserror::Error;

new_key_type! { struct Id; }

#[derive(Debug)]
pub struct Tree {
    order: usize,
    root: Option<Id>,
    items: SlotMap<Id, Node>,
    children: SecondaryMap<Id, Vec<Id>>,
}

impl Tree {
    pub fn new(order: usize) -> Tree {
        let o = match order {
            _ if order < 3 => 3,
            _ => order,
        };

        Self {
            order: o,
            root: None,
            items: SlotMap::with_key(),
            children: SecondaryMap::new(),
        }
    }

    // --------------------
    // | INSERT FUNCTIONS |
    // --------------------

    pub fn insert(&mut self, value: i32) -> Result<(), TreeError> {
        if let Some(root_id) = self.root {
            self.insert_recursive(value, root_id)?;
        } else {
            let root = self.create_item();
            self.push_key(value, root);
            self.root = Some(root);
        }

        if self.items[self.root.unwrap()].keys.len() > self.order - 1 {
            self.split_root();
        }

        Ok(())
    }

    fn split_root(&mut self) {
        let new_root = self.create_item();

        let root_id = self.root.unwrap();
        let split = self.split_item(root_id);

        let key = match self.children[split].is_empty() {
            true => *self.items[split].keys.index(0),
            false => self.items[split].keys.remove(0),
        };

        match self.items[new_root].keys.iter().position(|x| *x > key) {
            Some(idx) => self.items[new_root].keys.insert(idx, key),
            None => self.items[new_root].keys.push(key),
        }

        self.children[new_root].push(root_id);
        self.children[new_root].push(split);

        self.root = Some(new_root);
    }

    // TODO: garbage collect empty children on leaves
    // NOTE: i dont think this actually works, as we constantly look if the child of the current
    // node has children, well maybe we could also just check if the key exists in the children
    // map? using contains_key(id)

    fn insert_recursive(&mut self, value: i32, id: Id) -> Result<(), TreeError> {
        if let Some(child) = self.get_child(id, value) {
            if !self.children[child.id].is_empty() {
                self.insert_recursive(value, child.id)?;
            } else {
                self.insert_key(value, child.id)?
            }

            if self.items[child.id].keys.len() > self.order - 1 {
                let split_id = self.split_item(child.id);

                let key = match self.items[id]
                    .keys
                    .contains(self.items[split_id].keys.index(0))
                {
                    false => {
                        if !self.children[split_id].is_empty() {
                            self.items[split_id].keys.remove(0)
                        } else {
                            *self.items[split_id].keys.first().unwrap()
                        }
                    }
                    true => {
                        if !self.children[child.id].is_empty() {
                            self.items[child.id].keys.remove(0)
                        } else {
                            *self.items[child.id].keys.first().unwrap()
                        }
                    }
                };

                self.insert_key(key, id)?;

                if self.items[split_id].keys.first().unwrap()
                    > self.items[child.id].keys.first().unwrap()
                {
                    self.children[id].insert(child.index + 1, split_id);
                    Ok(())
                } else {
                    self.children[id].push(split_id);
                    Ok(())
                }
            } else {
                Ok(())
            }
        } else {
            self.insert_key(value, id)?;
            Ok(())
        }
    }

    fn split_item(&mut self, id: Id) -> Id {
        let new_id = self.create_item();

        let keys_len = self.items[id].keys.len();

        for _ in keys_len / 2..keys_len {
            let key = self.items[id].keys.pop().unwrap();
            self.items[new_id].keys.push(key);
        }

        if !self.items[new_id].keys.is_sorted() {
            self.items[new_id].keys.reverse();
        }

        if !self.children[id].is_empty() {
            let children_len = self.children[id].len();

            for _ in children_len.div_ceil(2)..children_len {
                let child_id = self.children[id].pop().unwrap();
                self.children[new_id].push(child_id);
            }

            self.children[new_id].reverse();
        }

        if let Some(leaf) = self.items[id].next_leaf {
            self.items[new_id].next_leaf = Some(leaf);
            self.items[id].next_leaf = Some(new_id);
        } else {
            self.items[id].next_leaf = Some(new_id);
        }

        new_id
    }

    fn create_item(&mut self) -> Id {
        let node = Node::new();
        let node_key = self.items.insert(node);

        self.children.insert(node_key, Vec::new());
        node_key
    }

    // --------------------
    // | DELETE FUNCTIONS |
    // --------------------

    pub fn delete(&mut self, value: i32) -> Result<(), TreeError> {
        if let Some(root_id) = self.root {
            self.delete_recursive(root_id, value)?;
            Ok(())
        } else {
            Err(TreeError::DeleteValue(value))
        }
    }

    fn delete_recursive(&mut self, id: Id, value: i32) -> Result<(), TreeError> {
        self.items[id].keys.retain(|x| *x != value);

        if let Some(child) = self.get_child(id, value) {
            // get left and right siblings

            if !self.children[child.id].is_empty() {
                self.delete_recursive(child.id, value)?;
            } else {
                self.items[child.id].keys.retain(|x| *x != value);
            }

            // if keys.len() is 0, steal from sibling

            let siblings = self.get_siblings(id, child);

            // if the sibling we stole from is empty, we merge

            // TODO: logic to steal from siblings or merge
        }
        Ok(())
    }

    fn get_siblings(&self, id: Id, child: Child) -> (Option<Id>, Option<Id>) {
        match child.position {
            Position::First => (None, Some(*self.children[id].index(child.index + 1))),
            Position::Last => (Some(*self.children[id].index(child.index - 1)), None),
            _ => (
                Some(*self.children[id].index(child.index - 1)),
                Some(*self.children[id].index(child.index + 1)),
            ),
        }
    }

    fn merge(&mut self, id: Id) -> Id {
        todo!()
    }

    // ------------------
    // | MISC FUNCTIONS |
    // ------------------

    fn get_child(&mut self, id: Id, value: i32) -> Option<Child> {
        if !self.children[id].is_empty() {
            if let Some(idx) = self.items[id].keys.iter().position(|x| *x > value) {
                let pos = match idx {
                    _ if idx == 0 => Position::First,
                    _ => Position::Index,
                };
                return Some(Child {
                    id: *self.children[id].index(idx),
                    index: idx,
                    position: pos,
                });
            } else if let Some(child_id) = self.children[id].last() {
                return Some(Child {
                    id: *child_id,
                    index: self.children[id].len() - 1,
                    position: Position::Last,
                });
            }
        }

        None
    }

    fn insert_key(&mut self, value: i32, id: Id) -> Result<(), TreeError> {
        if self.items[id].keys.contains(&value) {
            Err(TreeError::InsertValue(value))
        } else {
            match self.items[id].keys.iter().position(|x| *x > value) {
                Some(idx) => {
                    self.items[id].keys.insert(idx, value);
                    Ok(())
                }
                None => {
                    self.items[id].keys.push(value);
                    Ok(())
                }
            }
        }
    }

    fn push_key(&mut self, value: i32, id: Id) {
        self.items[id].keys.push(value);
    }

    // -------------------
    // | LEAF OPERATIONS |
    // -------------------

    fn get_first_leaf(&self, id: Id) -> Id {
        if !self.children[id].is_empty() {
            self.get_first_leaf(*self.children[id].first().unwrap())
        } else {
            id
        }
    }

    pub fn collect_leaves(&self) -> Vec<i32> {
        let mut data: Vec<i32> = Vec::new();
        if let Some(r) = self.root {
            let mut current_leaf = self.get_first_leaf(r);

            while let Some(next_leaf) = self.items[current_leaf].next_leaf {
                for k in &self.items[current_leaf].keys {
                    data.push(*k);
                }
                current_leaf = next_leaf;
            }

            for k in &self.items[current_leaf].keys {
                data.push(*k);
            }
        }
        data
    }
}

struct Child {
    id: Id,
    index: usize,
    position: Position,
}

#[derive(PartialEq)]
enum Position {
    First,
    Index,
    Last,
}

#[derive(Debug)]
struct Node {
    keys: Vec<i32>,
    next_leaf: Option<Id>,
}

impl Node {
    fn new() -> Self {
        Self {
            keys: Vec::new(),
            next_leaf: None,
        }
    }
}

#[derive(Error, Debug)]
pub enum TreeError {
    #[error("value {0:?} already exists in tree")]
    InsertValue(i32),

    #[error("failed to delete value {0:?}")]
    DeleteValue(i32),
}

// k
//
// #[derive(Error, Debug)]
// pub enum DeleteError {
//     #[error("failed to delete")]
//     None,
// }
