extern crate extended_collections;

use extended_collections::lsm::{Tree, SizeTieredStrategy};
use std::path::PathBuf;

fn main() {
    let sts = SizeTieredStrategy::new(
        4,
        50,
        0.5,
        1.5,
        100,
    );

    let mut tree = Tree::new(PathBuf::from("test_db"), sts).unwrap();

    for i in 0..100 {
        tree.insert(i, i).unwrap();
    }

    for i in 0..100 {
        println!("Finding {}", i);
        assert_eq!(tree.get(&i).unwrap(), Some(i));
    }
}
