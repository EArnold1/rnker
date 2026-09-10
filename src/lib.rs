//! `rnker` — an **order statistic tree** (a.k.a. ranking tree).
//!
//! A self-balancing (AVL) binary search tree that, on top of the usual ordered-map
//! operations, answers two order-statistic queries in `O(log n)` by keeping a
//! subtree-size count on every node:
//!
//! - [`rank`](OrderStatisticTree::rank) — given a key, which position does it hold?
//! - [`select`](OrderStatisticTree::select) — given a position, which key is there?
//!
//! Positions are **descending and 1-based**: the largest key has rank `1`.
//!
//! # Example
//!
//! ```
//! use rnker::OrderStatisticTree;
//!
//! let mut tree: OrderStatisticTree<i32, &str> = OrderStatisticTree::new();
//! for (key, value) in [(45, "John"), (40, "Alice"), (30, "Bob"), (60, "Frank")] {
//!     tree.insert(key, value);
//! }
//!
//! assert_eq!(tree.rank(&60), Some(1)); // largest key
//! assert_eq!(tree.select(1), Some((&60, &"Frank")));
//! assert_eq!(tree.rank(&99), None); // absent
//!
//! assert_eq!(tree.remove(&30), Some("Bob"));
//! assert_eq!(tree.len(), 3);
//! ```

mod balance;
mod iter;
mod node;
mod tree;

pub use iter::Iter;
pub use tree::OrderStatisticTree;
