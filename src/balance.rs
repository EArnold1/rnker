//! AVL rebalancing: the four rotations and the choice of which one to apply.
//!
//! Each function takes the `Option<Box<Node>>` slot holding the subtree root so
//! it can swap in a new root, then refreshes each touched node's cached
//! `height`/`size` bottom-up via [`node::refresh`].

use crate::node::{self, Node};

type Link<K, V> = Option<Box<Node<K, V>>>;

/// LL case: single right rotation; the left child becomes the new root.
fn ll_rotation<K, V>(node: &mut Link<K, V>) {
    let mut old_root = node.take().unwrap();
    let mut new_root = old_root.left.take().unwrap();

    old_root.left = new_root.right.take();
    node::refresh(&mut old_root);

    new_root.right = Some(old_root);
    node::refresh(&mut new_root);

    *node = Some(new_root);
}

/// RR case: single left rotation; the right child becomes the new root.
fn rr_rotation<K, V>(node: &mut Link<K, V>) {
    let mut old_root = node.take().unwrap();
    let mut new_root = old_root.right.take().unwrap();

    old_root.right = new_root.left.take();
    node::refresh(&mut old_root);

    new_root.left = Some(old_root);
    node::refresh(&mut new_root);

    *node = Some(new_root);
}

/// LR case: the left child's right child becomes the new root.
/// Same result as `rr_rotation` on the left child then `ll_rotation` on the node.
fn lr_rotation<K, V>(node: &mut Link<K, V>) {
    let mut z = node.take().unwrap();
    let mut y = z.left.take().unwrap();
    let mut x = y.right.take().unwrap(); // the new root

    y.right = x.left.take();
    z.left = x.right.take();
    node::refresh(&mut y);
    node::refresh(&mut z);

    x.left = Some(y);
    x.right = Some(z);
    node::refresh(&mut x);

    *node = Some(x);
}

/// RL case: mirror image of [`lr_rotation`].
fn rl_rotation<K, V>(node: &mut Link<K, V>) {
    let mut z = node.take().unwrap();
    let mut y = z.right.take().unwrap();
    let mut x = y.left.take().unwrap(); // the new root

    z.right = x.left.take();
    y.left = x.right.take();
    node::refresh(&mut z);
    node::refresh(&mut y);

    x.left = Some(z);
    x.right = Some(y);
    node::refresh(&mut x);

    *node = Some(x);
}

/// Rebalance the node after its own cached fields have been refreshed. Both child
/// subtrees are assumed valid, so at most one rotation is needed.
pub(crate) fn rebalance<K, V>(node: &mut Link<K, V>) {
    let Some(n) = node.as_deref() else { return };
    let bf = node::balance_factor(n);

    if bf > 1 {
        // left-heavy: left child's balance factor picks LL vs LR
        let left = n.left.as_deref().unwrap();
        if node::balance_factor(left) >= 0 {
            ll_rotation(node);
        } else {
            lr_rotation(node);
        }
    } else if bf < -1 {
        // right-heavy: right child's balance factor picks RR vs RL
        let right = n.right.as_deref().unwrap();
        if node::balance_factor(right) <= 0 {
            rr_rotation(node);
        } else {
            rl_rotation(node);
        }
    }
}
