# rnker

An order statistic tree (ranking tree): a balanced binary search tree that
supports `rank` and `select` queries in O(log n).

```rust
use rnker::OrderStatisticTree;

let mut tree: OrderStatisticTree<i32, &str> = OrderStatisticTree::new();
for (k, v) in [(45, "John"), (40, "Alice"), (30, "Bob"), (60, "Frank")] {
    tree.insert(k, v);
}

assert_eq!(tree.rank(&60), Some(1));            // ranks are descending: largest key is rank 1
assert_eq!(tree.select(1), Some((&60, &"Frank")));
assert_eq!(tree.remove(&30), Some("Bob"));
```

- `cargo test` — unit + integration + doc tests
- `cargo run --example demo` — a small walkthrough

## Documentation

A from-the-ground-up explanation of the data structure lives in [`docs/`](docs/):

1. [`docs/01-binary-search-tree.md`](docs/01-binary-search-tree.md) — ordered tree storage, and why an unbalanced tree is slow
2. [`docs/02-avl-tree.md`](docs/02-avl-tree.md) — balance factor and the four rotations
3. [`docs/03-order-statistic-tree.md`](docs/03-order-statistic-tree.md) — subtree-size metadata, `rank`, and `select`
4. [`docs/04-rust-implementation.md`](docs/04-rust-implementation.md) — how it all maps onto the code in `src/`
