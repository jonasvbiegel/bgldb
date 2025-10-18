mod bptree;

use std::time::Instant;

use crate::bptree::BPTree;
use rand::prelude::*;

fn main() {
    //     funny benckmark
    //     --------------------------------
    //     let mut rng = rand::rng();
    //
    //     let mut vec: Vec<u32> = Vec::new();
    //
    //     println!("inserting into vec continually sorting");
    //     std::thread::sleep(std::time::Duration::from_millis(3000));
    //     let mut now = Instant::now();
    //     for _ in 0..100_000 {
    //         let i = rng.random::<u32>();
    //         insert_sorted(&mut vec, i);
    //         print!(".");
    //     }
    //     let vec_time = now.elapsed().as_millis() as f32 / 1000.0;
    //     println!();
    //
    //     let mut bptree_i32 = BPTree::<u32>::new(4);
    //
    //     println!("inserting into b+ tree");
    //     std::thread::sleep(std::time::Duration::from_millis(3000));
    //     now = Instant::now();
    //     for _ in 0..100_000 {
    //         let i = rng.random::<u32>();
    //         bptree_i32.insert(i);
    //         print!(".");
    //     }
    //     let tree_time = now.elapsed().as_millis() as f32 / 1000.0;
    //
    //     println!("{vec_time} {tree_time}");

    let mut bptree = BPTree::<i32>::new(4);
    for i in 1..=7 {
        bptree.insert(i);
    }
    println!("{bptree:#?}")
}

fn insert_sorted(vector: &mut Vec<u32>, n: u32) {
    if let Some(i) = vector.iter().position(|x| *x >= n) {
        {
            vector.insert(i, n);
        }
    } else {
        vector.push(n);
    }
}
