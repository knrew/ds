#![allow(unused)]

use rand::{rng, seq::SliceRandom, Rng};
use std::collections::BTreeSet;

use ds::avl_tree_set::AVLTreeSet;

fn main() {
    let mut rng = rng();

    println!("AVLTreeSet");

    // let mut tree = AVLTreeSet::new();
    // for _ in 0..100 {
    //     tree.insert(rng.random_range(0..100));
    // }
    // println!("{:?}", tree);
    // println!("{}", tree.visualize());

    let mut avl = AVLTreeSet::new();
    for _ in 0..10000 {
        let x = rng.random_range(-100..=100);

        if rng.random_ratio(2, 3) {
            avl.insert(x);
        } else {
            avl.remove(&x);
        }
    }
    println!("{}", avl.visualize());
}
