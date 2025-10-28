use bptree::*;
mod bptree;

fn main() -> Result<(), TreeError> {
    println!("Hello, world!");

    let mut tree = Tree::new(4);

    for i in 1..=10 {
        tree.insert(i)?;
    }

    tree.delete(8)?;

    println!("{tree:#?}");

    Ok(())
}
