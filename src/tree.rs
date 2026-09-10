//! The public [`OrderStatisticTree`] type and its recursive insert/remove core.

use std::cmp::Ordering;

use crate::balance;
use crate::iter::Iter;
use crate::node::{self, Node};

type Link<K, V> = Option<Box<Node<K, V>>>;

/// A self-balancing (AVL) binary search tree that also answers order-statistic
/// queries — [`rank`](Self::rank) and [`select`](Self::select) — in `O(log n)`.
///
/// It behaves like an ordered map: keys are unique, [`insert`](Self::insert)
/// replaces an existing value, and [`get`](Self::get) / [`remove`](Self::remove)
/// take the key by reference.
///
/// # Ordering of ranks
///
/// Ranks are **descending and 1-based**: the largest key has rank `1`, the
/// smallest has rank [`len`](Self::len). `select` is the inverse of `rank`.
///
/// ```
/// use rnker::OrderStatisticTree;
///
/// let tree: OrderStatisticTree<i32, ()> =
///     [10, 20, 30].into_iter().map(|k| (k, ())).collect();
///
/// assert_eq!(tree.rank(&30), Some(1));
/// assert_eq!(tree.rank(&10), Some(3));
/// assert_eq!(tree.select(2).map(|(k, _)| *k), Some(20));
/// ```
#[derive(Debug, Clone)]
pub struct OrderStatisticTree<K, V> {
    root: Link<K, V>,
}

impl<K, V> Default for OrderStatisticTree<K, V> {
    fn default() -> Self {
        Self { root: None }
    }
}

impl<K, V> OrderStatisticTree<K, V> {
    /// Creates an empty tree.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of entries. `O(1)` — read from the root's cached subtree size.
    pub fn len(&self) -> usize {
        node::size(self.root.as_deref())
    }

    /// Returns `true` if the tree holds no entries.
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Removes all entries.
    pub fn clear(&mut self) {
        self.root = None;
    }

    /// Iterates over `(&K, &V)` in ascending key order.
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter::new(self.root.as_deref())
    }
}

impl<K: Ord, V> OrderStatisticTree<K, V> {
    /// Inserts a key/value pair, returning the previous value for `key` if it was
    /// already present.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        insert_node(&mut self.root, key, value)
    }

    /// Returns a reference to the value for `key`.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.get_key_value(key).map(|(_, value)| value)
    }

    /// Returns the stored key and its value.
    pub fn get_key_value(&self, key: &K) -> Option<(&K, &V)> {
        let mut cursor = self.root.as_deref();
        while let Some(node) = cursor {
            cursor = match key.cmp(&node.key) {
                Ordering::Less => node.left.as_deref(),
                Ordering::Greater => node.right.as_deref(),
                Ordering::Equal => return Some((&node.key, &node.value)),
            };
        }
        None
    }

    /// Returns `true` if `key` is present.
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Returns the 1-based descending rank of `key`, or `None` if it is absent.
    ///
    /// The largest key has rank `1`.
    pub fn rank(&self, key: &K) -> Option<usize> {
        let mut cursor = self.root.as_deref();
        let mut rank = 0;
        while let Some(node) = cursor {
            let right = node::size(node.right.as_deref());
            match key.cmp(&node.key) {
                // Smaller keys sit further down the descending order: skip this
                // node and its (larger-keyed) right subtree entirely.
                Ordering::Greater => cursor = node.right.as_deref(),
                Ordering::Less => {
                    rank += right + 1;
                    cursor = node.left.as_deref();
                }
                Ordering::Equal => return Some(rank + right + 1),
            }
        }
        None
    }

    /// Returns the key/value pair at the given 1-based descending `rank`, or
    /// `None` if `rank` is `0` or greater than [`len`](Self::len).
    pub fn select(&self, rank: usize) -> Option<(&K, &V)> {
        if rank == 0 || rank > self.len() {
            return None;
        }

        let mut cursor = self.root.as_deref();
        let mut remaining = rank;
        while let Some(node) = cursor {
            let here = node::size(node.right.as_deref()) + 1;
            match remaining.cmp(&here) {
                Ordering::Equal => return Some((&node.key, &node.value)),
                Ordering::Less => cursor = node.right.as_deref(),
                Ordering::Greater => {
                    remaining -= here;
                    cursor = node.left.as_deref();
                }
            }
        }
        None
    }
}

impl<K: Ord + Clone, V> OrderStatisticTree<K, V> {
    /// Removes `key`, returning its value if it was present.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        remove_node(&mut self.root, key)
    }
}

impl<'a, K, V> IntoIterator for &'a OrderStatisticTree<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<K: Ord, V> Extend<(K, V)> for OrderStatisticTree<K, V> {
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        for (key, value) in iter {
            self.insert(key, value);
        }
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for OrderStatisticTree<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut tree = Self::new();
        tree.extend(iter);
        tree
    }
}

/// Inserts into the subtree at `link`, refreshing cached fields and rebalancing
/// on the way back up. Returns the replaced value, if any.
fn insert_node<K: Ord, V>(link: &mut Link<K, V>, key: K, value: V) -> Option<V> {
    let Some(node) = link.as_mut() else {
        *link = Some(Node::boxed(key, value));
        return None;
    };

    let replaced = match key.cmp(&node.key) {
        Ordering::Less => insert_node(&mut node.left, key, value),
        Ordering::Greater => insert_node(&mut node.right, key, value),
        // Key already here: swap the value, tree shape is unchanged.
        Ordering::Equal => return Some(std::mem::replace(&mut node.value, value)),
    };

    node::refresh(link.as_mut().unwrap());
    balance::rebalance(link);
    replaced
}

/// Removes `key` from the subtree at `link`, refreshing cached fields and
/// rebalancing on the way back up.
fn remove_node<K: Ord + Clone, V>(link: &mut Link<K, V>, key: &K) -> Option<V> {
    let ordering = key.cmp(&link.as_deref()?.key);

    let removed = match ordering {
        Ordering::Less => remove_node(&mut link.as_mut().unwrap().left, key),
        Ordering::Greater => remove_node(&mut link.as_mut().unwrap().right, key),
        Ordering::Equal => Some(remove_root(link)),
    };

    if let Some(node) = link.as_mut() {
        node::refresh(node);
        balance::rebalance(link);
    }
    removed
}

/// Removes the node that *is* `link`, rewiring its children into the slot.
/// Does not rebalance — the caller does that as it unwinds.
fn remove_root<K: Ord + Clone, V>(link: &mut Link<K, V>) -> V {
    let mut root = *link.take().expect("remove_root on an empty slot");

    match (root.left.take(), root.right.take()) {
        (None, None) => root.value,
        (Some(child), None) | (None, Some(child)) => {
            *link = Some(child);
            root.value
        }
        (Some(left), Some(right)) => {
            // Replace this node's key/value with its in-order successor
            // (the smallest key in the right subtree), then drop the successor.
            let mut right = Some(right);
            let (succ_key, succ_value) = take_min(&mut right);

            let removed = std::mem::replace(&mut root.value, succ_value);
            root.key = succ_key;
            root.left = Some(left);
            root.right = right;
            *link = Some(Box::new(root));
            removed
        }
    }
}

/// Removes and returns the smallest entry of a non-empty subtree, refreshing and
/// rebalancing the nodes above it.
fn take_min<K: Ord + Clone, V>(link: &mut Link<K, V>) -> (K, V) {
    let has_left = link
        .as_deref()
        .expect("take_min on an empty subtree")
        .left
        .is_some();

    if has_left {
        let entry = take_min(&mut link.as_mut().unwrap().left);
        node::refresh(link.as_mut().unwrap());
        balance::rebalance(link);
        entry
    } else {
        let node = *link.take().unwrap();
        *link = node.right;
        (node.key, node.value)
    }
}

#[cfg(test)]
impl<K: Ord + std::fmt::Debug, V> OrderStatisticTree<K, V> {
    /// Asserts the AVL invariant, BST ordering, and that every cached `height`
    /// and `size` matches a freshly recomputed value. Test-only.
    pub(crate) fn assert_valid(&self) {
        fn check<K: Ord + std::fmt::Debug, V>(
            link: &Link<K, V>,
            lower: Option<&K>,
            upper: Option<&K>,
        ) -> (i32, usize) {
            let Some(node) = link.as_deref() else {
                return (-1, 0);
            };
            if let Some(lo) = lower {
                assert!(node.key > *lo, "BST order violated at {:?}", node.key);
            }
            if let Some(hi) = upper {
                assert!(node.key < *hi, "BST order violated at {:?}", node.key);
            }

            let (lh, ls) = check(&node.left, lower, Some(&node.key));
            let (rh, rs) = check(&node.right, Some(&node.key), upper);

            assert!(
                (lh - rh).abs() <= 1,
                "AVL balance violated at {:?}",
                node.key
            );
            let height = lh.max(rh) + 1;
            let size = ls + rs + 1;
            assert_eq!(node.height, height, "stale height at {:?}", node.key);
            assert_eq!(node.size, size, "stale size at {:?}", node.key);
            (height, size)
        }

        check(&self.root, None, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a tree from `keys`, using each key as its own value, asserting the
    /// invariants after every insert.
    fn tree_of(keys: &[i32]) -> OrderStatisticTree<i32, i32> {
        let mut tree = OrderStatisticTree::new();
        for &k in keys {
            tree.insert(k, k);
            tree.assert_valid();
        }
        tree
    }

    #[test]
    fn insert_keeps_avl_invariant_for_sorted_input() {
        // Ascending then descending input is the classic way to force every
        // rotation kind (LL/RR on the runs, LR/RL at the turn).
        let tree = tree_of(&[1, 2, 3, 4, 5, 6, 7, 20, 19, 18, 17, 16, 15]);
        assert_eq!(tree.len(), 13);
    }

    #[test]
    fn insert_triggers_each_rotation() {
        // LL: 30, 20, 10  ->  root 20
        assert_eq!(tree_of(&[30, 20, 10]).root.as_ref().unwrap().key, 20);
        // RR: 10, 20, 30  ->  root 20
        assert_eq!(tree_of(&[10, 20, 30]).root.as_ref().unwrap().key, 20);
        // LR: 30, 10, 20  ->  root 20
        assert_eq!(tree_of(&[30, 10, 20]).root.as_ref().unwrap().key, 20);
        // RL: 10, 30, 20  ->  root 20
        assert_eq!(tree_of(&[10, 30, 20]).root.as_ref().unwrap().key, 20);
    }

    #[test]
    fn insert_replaces_value_and_keeps_size() {
        let mut tree = OrderStatisticTree::new();
        assert_eq!(tree.insert(1, "a"), None);
        assert_eq!(tree.insert(1, "b"), Some("a"));
        assert_eq!(tree.len(), 1);
        assert_eq!(tree.get(&1), Some(&"b"));
    }

    #[test]
    fn remove_covers_all_three_child_cases() {
        // 20(root) -> 10, 40 ; 40 -> 30, 50
        let mut tree = tree_of(&[20, 10, 40, 30, 50]);

        assert_eq!(tree.remove(&10), Some(10)); // leaf
        tree.assert_valid();

        assert_eq!(tree.remove(&40), Some(40)); // two children -> promote 50
        tree.assert_valid();

        assert_eq!(tree.remove(&50), Some(50)); // one child after the promotion
        tree.assert_valid();

        assert_eq!(tree.remove(&999), None);
        assert_eq!(keys(&tree), vec![20, 30]);
    }

    #[test]
    fn remove_rebalances() {
        let mut tree = tree_of(&[10, 20, 30, 40, 50, 25]);
        tree.remove(&10);
        tree.assert_valid();
        assert_eq!(keys(&tree), vec![20, 25, 30, 40, 50]);
    }

    #[test]
    fn stress_insert_remove_keeps_invariants() {
        let mut tree = OrderStatisticTree::new();
        // Deterministic pseudo-random-ish sequence.
        let mut x: u64 = 1;
        let mut inserted = Vec::new();
        for _ in 0..500 {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
            let k = (x >> 33) as i32 % 200;
            if tree.get(&k).is_some() {
                tree.remove(&k);
                inserted.retain(|&v| v != k);
            } else {
                tree.insert(k, k);
                inserted.push(k);
            }
            tree.assert_valid();
        }
        inserted.sort_unstable();
        assert_eq!(keys(&tree), inserted);
    }

    fn keys(tree: &OrderStatisticTree<i32, i32>) -> Vec<i32> {
        tree.iter().map(|(k, _)| *k).collect()
    }
}
