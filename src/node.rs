//! The internal tree node plus the `O(1)` height/size bookkeeping helpers.
//!
//! Every structural change must leave `height` and `size` consistent with the
//! node's children. The helpers here only ever read the children's *cached*
//! fields, so refreshing one node is `O(1)`; callers refresh bottom-up.

/// A single node of the tree.
///
/// * `height` is 0-based — a leaf has height `0`, an absent child has height `-1`.
/// * `size` is the number of nodes in the subtree rooted here — a leaf has size `1`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Node<K, V> {
    pub(crate) key: K,
    pub(crate) value: V,
    pub(crate) height: i32,
    pub(crate) size: usize,
    pub(crate) left: Option<Box<Node<K, V>>>,
    pub(crate) right: Option<Box<Node<K, V>>>,
}

impl<K, V> Node<K, V> {
    pub(crate) fn new(key: K, value: V) -> Self {
        Self {
            key,
            value,
            height: 0,
            size: 1,
            left: None,
            right: None,
        }
    }

    /// A fresh leaf node, already boxed for insertion into a child slot.
    pub(crate) fn boxed(key: K, value: V) -> Box<Self> {
        Box::new(Self::new(key, value))
    }
}

/// Cached height of an optional node; `-1` when absent (so a leaf computes to
/// `max(-1, -1) + 1 == 0`).
pub(crate) fn height<K, V>(node: Option<&Node<K, V>>) -> i32 {
    node.map_or(-1, |n| n.height)
}

/// Cached subtree size of an optional node; `0` when absent.
pub(crate) fn size<K, V>(node: Option<&Node<K, V>>) -> usize {
    node.map_or(0, |n| n.size)
}

/// Balance factor `height(left) - height(right)`; in a valid AVL tree it stays
/// within `-1..=1`.
pub(crate) fn balance_factor<K, V>(node: &Node<K, V>) -> i32 {
    height(node.left.as_deref()) - height(node.right.as_deref())
}

/// Recompute `node.height` from its children's cached heights.
pub(crate) fn update_height<K, V>(node: &mut Node<K, V>) {
    node.height = height(node.left.as_deref()).max(height(node.right.as_deref())) + 1;
}

/// Recompute `node.size` from its children's cached sizes.
pub(crate) fn update_size<K, V>(node: &mut Node<K, V>) {
    node.size = 1 + size(node.left.as_deref()) + size(node.right.as_deref());
}

/// Refresh both cached fields of `node`. Call this on every node on the path back
/// up to the root after an insert or remove.
pub(crate) fn refresh<K, V>(node: &mut Node<K, V>) {
    update_height(node);
    update_size(node);
}
