use rnker::bst::avl_tree::AvlTree;

fn main() {
    let mut tree = AvlTree::<i32, &str>::new();

    for i in [
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
        tree.insert(i.0, i.1);
    }

    let rank = tree.rank(70);
    println!("rank of 70: {}", rank);

    let node = tree.select(4);
    println!("node with rank 4: {:?}", node);

    let search = tree.search(35);
    println!("search result for 35: {:?}", search);

    tree.delete(35);

    let search = tree.search(35);
    println!("search result for 35 after delete: {:?}", search);

    let pre_order = tree.pre_order_traversal();

    println!("pre-order traversal: {:?}", pre_order);
    println!("{:#?}", tree.root_node());
}
