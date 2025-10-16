struct Node<T> {
    is_leaf: bool,
    keys: Vec<T>,
    children: Vec<Node<T>>,
    order: usize,
}

impl<T> Node<T> {
    fn new(is_leaf: bool, order: usize) -> Self {
        Self {
            is_leaf,
            keys: Vec::new(),
            children: Vec::new(),
            order,
        }
    }
}
