//! Borrowing in-order iterator, yielding `(&K, &V)` in **ascending** key order.

use crate::node::Node;

/// Iterator over a tree's entries in ascending key order.
///
/// Created by [`OrderStatisticTree::iter`](crate::OrderStatisticTree::iter) or by
/// iterating over `&tree`.
pub struct Iter<'a, K, V> {
    /// Ancestors whose left subtree has been walked but which have not yet been
    /// yielded. The top of the stack is the next entry to produce.
    stack: Vec<&'a Node<K, V>>,
    /// Subtree still to descend into (pushing left spine onto the stack).
    pending: Option<&'a Node<K, V>>,
}

impl<'a, K, V> Iter<'a, K, V> {
    pub(crate) fn new(root: Option<&'a Node<K, V>>) -> Self {
        Self {
            stack: Vec::new(),
            pending: root,
        }
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.pending {
            self.stack.push(node);
            self.pending = node.left.as_deref();
        }

        let node = self.stack.pop()?;
        self.pending = node.right.as_deref();
        Some((&node.key, &node.value))
    }
}
