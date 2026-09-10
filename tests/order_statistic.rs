//! Black-box tests of the public API — ordering, rank/select semantics, iteration.

use rnker::OrderStatisticTree;

fn sample() -> OrderStatisticTree<i32, &'static str> {
    [
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
    ]
    .into_iter()
    .collect()
}

#[test]
fn iter_yields_ascending_keys() {
    let tree = sample();
    let keys: Vec<_> = tree.iter().map(|(k, _)| *k).collect();
    assert_eq!(keys, vec![20, 30, 35, 40, 41, 42, 45, 46, 50, 60, 70]);
}

#[test]
fn rank_is_descending_and_one_based() {
    let tree = sample();
    assert_eq!(tree.rank(&70), Some(1)); // largest
    assert_eq!(tree.rank(&60), Some(2));
    assert_eq!(tree.rank(&20), Some(11)); // smallest == len
    assert_eq!(tree.rank(&99), None); // absent
    assert_eq!(tree.rank(&25), None); // absent, mid-range
}

#[test]
fn select_is_the_inverse_of_rank() {
    let tree = sample();
    for (key, _) in &tree {
        let rank = tree.rank(key).unwrap();
        assert_eq!(tree.select(rank).map(|(k, _)| k), Some(key));
    }
}

#[test]
fn select_out_of_range_is_none() {
    let tree = sample();
    assert_eq!(tree.select(0), None);
    assert_eq!(tree.select(tree.len() + 1), None);
}

#[test]
fn select_reads_top_to_bottom() {
    let tree = sample();
    assert_eq!(tree.select(1), Some((&70, &"Jack")));
    assert_eq!(tree.select(4), Some((&46, &"David")));
    assert_eq!(tree.select(11), Some((&20, &"Hannah")));
}

#[test]
fn rank_select_hold_after_removal() {
    let mut tree = sample();
    tree.remove(&35);
    tree.remove(&70);

    assert_eq!(tree.len(), 9);
    assert_eq!(tree.rank(&60), Some(1)); // 70 is gone, 60 is now largest
    assert_eq!(tree.select(1), Some((&60, &"Frank")));
    assert_eq!(tree.rank(&35), None);

    for (key, _) in &tree {
        assert_eq!(
            tree.select(tree.rank(key).unwrap()).map(|(k, _)| k),
            Some(key)
        );
    }
}

#[test]
fn empty_tree_answers_sensibly() {
    let tree: OrderStatisticTree<i32, ()> = OrderStatisticTree::new();
    assert!(tree.is_empty());
    assert_eq!(tree.len(), 0);
    assert_eq!(tree.rank(&1), None);
    assert_eq!(tree.select(1), None);
    assert_eq!(tree.iter().next(), None);
}

#[test]
fn works_with_string_keys() {
    let mut tree: OrderStatisticTree<String, i32> = OrderStatisticTree::new();
    for (i, name) in ["delta", "alpha", "charlie", "bravo"].iter().enumerate() {
        tree.insert(name.to_string(), i as i32);
    }
    let keys: Vec<_> = tree.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(keys, vec!["alpha", "bravo", "charlie", "delta"]);
    assert_eq!(tree.rank(&"delta".to_string()), Some(1));
    assert_eq!(tree.select(1).map(|(k, _)| k.as_str()), Some("delta"));
}
