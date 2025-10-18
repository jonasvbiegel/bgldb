mod bptree;

use crate::bptree::BPTree;

fn main() {
    let mut bptree = BPTree::<i32>::new(4);

    for i in 1..=10 {
        bptree.insert(i);
    }

    println!("---------------------------------------------");
    println!("after inserting the tree looks like this");
    println!("{bptree:#?}");
}
