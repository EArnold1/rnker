// IMPORTANT: This AVL tree is a copy of an AVL tree implementation i made in another project https://github.com/EArnold1/bst,
// but this one has a size metadata that is used for rank-based operations(rank & select)

#[derive(Debug, Clone, PartialEq)]
pub struct Node<K, V> {
    pub key: K,
    pub value: V,
    height: isize, // 0-based, meaning leaf nodes have height 0
    size: usize,
    left: Option<Box<Node<K, V>>>,
    right: Option<Box<Node<K, V>>>,
}

impl<K, V> Node<K, V> {
    pub fn new(key: K, value: V) -> Self {
        Self {
            key,
            value,
            height: 0,
            size: 1,
            left: None,
            right: None,
        }
    }
}

pub struct AvlTree<K, V> {
    pub root: Option<Box<Node<K, V>>>,
}

impl<K: PartialEq + PartialOrd + Clone, V> Default for AvlTree<K, V> {
    fn default() -> Self {
        Self { root: None }
    }
}

impl<K: PartialEq + PartialOrd + Clone, V> AvlTree<K, V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: K, value: V) {
        Self::insert_node(&mut self.root, key, value);
    }

    fn ll_rotation(node: &mut Option<Box<Node<K, V>>>) {
        let mut old_root = node.take().unwrap();
        let mut new_root = old_root.left.take().unwrap();

        // Preserve the middle subtree
        old_root.left = new_root.right.take();

        // Put old root under new root
        new_root.right = Some(old_root);

        Self::update_height(new_root.right.as_mut().unwrap());
        Self::update_size(new_root.right.as_mut().unwrap());

        Self::update_height(&mut new_root);
        Self::update_size(&mut new_root);

        *node = Some(new_root);
    }

    fn lr_rotation(node: &mut Option<Box<Node<K, V>>>) {
        let mut old_root = node.take().unwrap();
        let mut left = old_root.left.take().unwrap();
        let mut new_root = left.right.take().unwrap();

        // preserve the subtrees of the new root
        left.right = new_root.left.take();
        old_root.left = new_root.right.take();

        new_root.left = Some(left);
        new_root.right = Some(old_root);

        Self::update_height(new_root.left.as_mut().unwrap());
        Self::update_size(new_root.left.as_mut().unwrap());

        Self::update_height(new_root.right.as_mut().unwrap());
        Self::update_size(new_root.right.as_mut().unwrap());

        Self::update_height(&mut new_root);
        Self::update_size(&mut new_root);

        *node = Some(new_root);
    }

    fn rr_rotation(node: &mut Option<Box<Node<K, V>>>) {
        let mut old_root = node.take().unwrap();
        let mut new_root = old_root.right.take().unwrap();

        // Preserve the middle subtree
        old_root.right = new_root.left.take();

        // Put old root under new root
        new_root.left = Some(old_root);

        Self::update_height(new_root.left.as_mut().unwrap());
        Self::update_size(new_root.left.as_mut().unwrap());

        Self::update_height(&mut new_root);
        Self::update_size(&mut new_root);

        *node = Some(new_root);
    }

    fn rl_rotation(node: &mut Option<Box<Node<K, V>>>) {
        let mut old_root = node.take().unwrap();
        let mut right = old_root.right.take().unwrap();
        let mut new_root = right.left.take().unwrap();

        // Preserve both middle subtrees
        old_root.right = new_root.left.take();
        right.left = new_root.right.take();

        new_root.left = Some(old_root);
        new_root.right = Some(right);

        Self::update_height(new_root.left.as_mut().unwrap());
        Self::update_size(new_root.left.as_mut().unwrap());

        Self::update_height(new_root.right.as_mut().unwrap());
        Self::update_size(new_root.right.as_mut().unwrap());

        Self::update_height(&mut new_root);
        Self::update_size(&mut new_root);

        *node = Some(new_root);
    }

    fn balance_node(node: &mut Option<Box<Node<K, V>>>) {
        if let Some(n) = node {
            // left-heavy
            if Self::bf(n) > 1 {
                if Self::bf(n.left.as_ref().unwrap()) >= 0 {
                    Self::ll_rotation(node);
                } else {
                    Self::lr_rotation(node);
                }
            } else if Self::bf(n) < -1 {
                // right-heavy
                if Self::bf(n.right.as_ref().unwrap()) <= 0 {
                    Self::rr_rotation(node);
                } else {
                    Self::rl_rotation(node);
                }
            }
        }
    }

    fn insert_node(node: &mut Option<Box<Node<K, V>>>, key: K, value: V) {
        match node {
            None => {
                *node = Some(Box::new(Node::new(key, value)));
            }

            Some(n) => {
                if n.key == key {
                    return;
                }

                if n.key > key {
                    Self::insert_node(&mut n.left, key, value);
                } else {
                    Self::insert_node(&mut n.right, key, value);
                }

                Self::update_height(n);

                Self::balance_node(node);
            }
        }
    }

    pub fn search(&self, key: K) -> Option<&Node<K, V>> {
        let mut curr = self.root.as_ref();

        while let Some(node) = curr {
            if node.key == key {
                return Some(node);
            }

            if node.key > key {
                // move left
                curr = node.left.as_ref();
            } else {
                // move right
                curr = node.right.as_ref();
            }
        }

        None
    }

    /// Calculates balanced factor of a node
    /// a balanced factor is between -1 and 1
    ///
    /// bf = height(left subtree) - height(right subtree)
    fn bf(node: &Node<K, V>) -> isize {
        Self::height(node.left.as_deref()) - Self::height(node.right.as_deref())
    }

    /// The height is 0-based,
    /// so a leaf node's height is 0
    fn height(node: Option<&Node<K, V>>) -> isize {
        // height of a None node is -1
        node.as_ref().map_or(-1, |node| node.height)
    }

    fn update_height(node: &mut Node<K, V>) {
        // h = max(left, right) + 1
        node.height = std::cmp::max(
            Self::height(node.left.as_deref()),
            Self::height(node.right.as_deref()),
        ) + 1;
    }

    pub fn pre_order_traversal(&self) -> Vec<(&K, &V)> {
        let mut result = Vec::new();
        Self::traversal_pre_order(&self.root, &mut result);
        result
    }

    pub fn root_node(&self) -> Option<&Node<K, V>> {
        self.root.as_deref()
    }

    /// Performs a pre-order traversal of the subtree rooted at the given node,
    /// appending the keys to the provided result vector.
    ///
    /// `RIGHT -> NODE(ROOT) -> LEFT`
    fn traversal_pre_order<'a>(
        node: &'a Option<Box<Node<K, V>>>,
        result: &mut Vec<(&'a K, &'a V)>,
    ) {
        if let Some(n) = node {
            Self::traversal_pre_order(&n.right, result);
            result.push((&n.key, &n.value));
            Self::traversal_pre_order(&n.left, result);
        }
    }

    pub fn delete(&mut self, key: K) {
        Self::delete_node(&mut self.root, key);
    }

    // recursive delete
    fn delete_node(node: &mut Option<Box<Node<K, V>>>, key: K) {
        let Some(n) = node else {
            return;
        };

        if n.key > key {
            Self::delete_node(&mut n.left, key);
        } else if n.key < key {
            Self::delete_node(&mut n.right, key);
        } else {
            // found the node to delete
            match (n.left.take(), n.right.take()) {
                // leaf node
                (None, None) => {
                    *node = None;
                    return;
                }

                // one child
                (Some(left), None) => {
                    *node = Some(left);
                    return;
                }
                (None, Some(right)) => {
                    *node = Some(right);
                    return;
                }

                // two children
                (Some(left), Some(right)) => {
                    // find the in-order successor: right subtree's leftmost node
                    let min_key = Self::min_value(&right);

                    n.key = min_key.clone();
                    n.left = Some(left);
                    n.right = Some(right);

                    // delete the successor from the right subtree
                    Self::delete_node(&mut n.right, min_key);
                }
            }
        }

        Self::update_height(n);
        Self::balance_node(node);
    }

    /// Finds the leftmost leaf node in the given subtree and returns its key.
    fn min_value(node: &Node<K, V>) -> K {
        let mut curr = node;
        while let Some(ref next) = curr.left {
            curr = next;
        }
        curr.key.clone()
    }

    fn size(node: Option<&Node<K, V>>) -> usize {
        if let Some(n) = node {
            1 + Self::size(n.left.as_deref()) + Self::size(n.right.as_deref())
        } else {
            0
        }
    }

    fn update_size(node: &mut Node<K, V>) {
        node.size = 1 + Self::size(node.left.as_deref()) + Self::size(node.right.as_deref());
    }

    /// Returns the rank of the given key in the AVL tree.
    /// If value is 0, the key is not present in the tree.
    pub fn rank(&self, key: K) -> usize {
        let mut node = self.root.as_deref();
        let mut rank = 0;

        while let Some(current) = node {
            let right_size = current.right.as_deref().map_or(0, |right| right.size);

            // when going right add nothing, when going left add the size of the right subtree plus one (for the current node)
            if current.key < key {
                node = current.right.as_deref();
            } else if current.key > key {
                rank += right_size + 1;
                node = current.left.as_deref();
            } else {
                rank += right_size + 1;
                break;
            }
        }

        rank
    }

    pub fn select(&self, key: usize) -> Option<(&K, &V)> {
        let mut root = self.root.as_deref();
        let mut k = key;

        while let Some(current) = root {
            let rank = current.right.as_deref().map_or(0, |right| right.size) + 1;

            if k == rank {
                return Some((&current.key, &current.value));
            } else if k < rank {
                root = current.right.as_deref();
            } else if k > rank {
                // we've seen rank number of nodes in the right subtree plus the current node
                // so subtract rank from k to continue searching in the left subtree
                k -= rank;
                root = current.left.as_deref();
            }
        }

        None
    }
}

// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[test]
//     fn test_bst_ordering() {
//         let mut tree = AvlTree::new();

//         tree.insert(20);
//         tree.insert(30);
//         tree.insert(10);

//         let mut root_node = Node::new(20);
//         root_node.height = 1;
//         root_node.left = Some(Box::new(Node::new(10)));
//         root_node.right = Some(Box::new(Node::new(30)));

//         assert_eq!(tree.root_node(), Some(&root_node));
//     }

//     // balanced factor check
//     // This test ensures that the balanced factor (BF) of every node in the AVL tree is within the allowed range (-1, 0, 1).
//     #[test]
//     fn test_absolute_balanced_factor() {
//         let mut tree = AvlTree::new();

//         tree.insert(20);
//         tree.insert(30);
//         tree.insert(10);

//         assert!(check_bf(tree.root.as_deref()));
//     }

//     #[test]
//     fn test_bst_nodes_balanced_factor() {
//         let mut tree = AvlTree::new();

//         tree.insert(20);
//         tree.insert(30);
//         tree.insert(10);

//         let root_node = tree.root_node().unwrap();
//         let left_child = root_node.left.as_deref().unwrap();
//         let right_child = root_node.right.as_deref().unwrap();

//         assert_eq!(AvlTree::bf(root_node), 0);
//         assert_eq!(AvlTree::bf(left_child), 0);
//         assert_eq!(AvlTree::bf(right_child), 0);
//     }

//     // height check
//     #[test]
//     fn test_root_height_after_insertions() {
//         let mut tree = AvlTree::new();

//         tree.insert(20);
//         tree.insert(40);
//         tree.insert(15);
//         tree.insert(25);
//         tree.insert(45);

//         assert_eq!(tree.root_node().unwrap().height, 2);
//     }

//     // in-order traversal check
//     #[test]
//     fn returns_in_order_traversal() {
//         let mut tree = AvlTree::new();

//         tree.insert(20);
//         tree.insert(30);
//         tree.insert(10);

//         assert_eq!(tree.in_order_traversal(), vec![10, 20, 30]);
//     }

//     // duplicate insertion check
//     #[test]
//     fn ignore_duplicate_insertion() {
//         let mut tree = AvlTree::new();

//         tree.insert(20);
//         tree.insert(20); // duplicate insertion

//         assert_eq!(tree.in_order_traversal(), vec![20]);
//     }

//     // rotation check
//     #[test]
//     fn test_rotation_balance_after_insert() {
//         let mut tree = AvlTree::new();

//         tree.insert(10);
//         tree.insert(20);
//         tree.insert(30); // should trigger a rotation

//         assert_eq!(tree.in_order_traversal(), vec![10, 20, 30]);
//         assert_eq!(tree.root_node().unwrap().key, 20);
//     }

//     #[test]
//     fn search_returns_matching_node() {
//         let mut tree = AvlTree::new();

//         tree.insert(10);
//         tree.insert(20);
//         tree.insert(30);
//         tree.insert(65);
//         tree.insert(15);

//         let expected = Node::new(15);

//         assert_eq!(tree.search(15), Some(&expected));
//     }

//     #[test]
//     fn search_returns_none_when_key_does_not_exist() {
//         let mut tree = AvlTree::new();

//         tree.insert(10);
//         tree.insert(20);
//         tree.insert(30);
//         tree.insert(65);
//         tree.insert(15);

//         assert_eq!(tree.search(99), None);
//     }

//     #[test]
//     fn delete_rebalances() {
//         let mut tree = AvlTree::new();
//         for k in [10, 20, 30, 40, 50, 25] {
//             tree.insert(k);
//         }
//         tree.delete(10);
//         assert!(check_bf(tree.root.as_deref())); // reuse the helper
//         assert_eq!(tree.in_order_traversal(), vec![20, 25, 30, 40, 50]);
//     }

//     fn check_bf(node: Option<&Node>) -> bool {
//         if let Some(n) = node {
//             let bf = AvlTree::bf(n);
//             if bf.abs() > 1 {
//                 return false;
//             }
//             return check_bf(n.left.as_deref()) && check_bf(n.right.as_deref());
//         }
//         true
//     }
// }
