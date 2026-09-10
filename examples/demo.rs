//! Run with `cargo run --example demo`.

use rnker::OrderStatisticTree;

fn main() {
    let mut tree: OrderStatisticTree<i32, &str> = OrderStatisticTree::new();

    for (key, value) in [
        (45, "John"),
        (40, "Alice"),
        (30, "Bob"),
        (41, "Eve"),
        (35, "Charlie"),
        (46, "David"),
        (60, "Frank"),
        (50, "Grace"),
        (20, "Hannah"),
        (42, "Ivy"),
        (70, "Jack"),
    ] {
        tree.insert(key, value);
    }

    println!("rank of 70:         {:?}", tree.rank(&70));
    println!("key with rank 4:    {:?}", tree.select(4));
    println!("lookup of 35:       {:?}", tree.get(&35));

    println!("removed 35:         {:?}", tree.remove(&35));
    println!("lookup of 35 now:   {:?}", tree.get(&35));

    let ascending: Vec<_> = tree.iter().map(|(key, _)| *key).collect();
    println!("keys (ascending):   {ascending:?}");
    println!("len:                {}", tree.len());
}
