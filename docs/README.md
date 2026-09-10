# `rnker` — how it works, from the ground up

This folder explains the data structure behind `rnker` in layers. Each layer adds
one idea on top of the previous one:

| #   | Document                                                   | Idea it adds                                                                                                                              |
| --- | ---------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | [`01-binary-search-tree.md`](01-binary-search-tree.md)     | Ordered storage in a tree, and why an unbalanced tree is slow.                                                                            |
| 2   | [`02-avl-tree.md`](02-avl-tree.md)                         | Keeping the tree short with **rotations**, so every operation is `O(log n)`.                                                              |
| 3   | [`03-order-statistic-tree.md`](03-order-statistic-tree.md) | Storing a **subtree size** on every node, so we can ask _"what is the rank of this key?"_ and _"which key has this rank?"_ in `O(log n)`. |
| 4   | [`04-rust-implementation.md`](04-rust-implementation.md)   | How the three ideas above map onto the actual Rust in `src/`.                                                                             |

Read them in order the first time. Each one is self-contained enough to revisit
on its own later.

## Summary

A **binary search tree** keeps keys in sorted order so lookups take time
proportional to the tree's height. An **AVL tree** is a binary search tree that
rebalances itself after every insert and delete so its height stays around
`log2(n)`. An **order statistic tree** is an AVL tree where each node also records
how many nodes are in its subtree; that single extra number lets you convert
between a key and its position ("rank") in either direction without walking the
whole tree.

## Vocabulary

- **Node** — one stored entry: a key, a value, and links to up to two children.
- **Root** — the topmost node. The whole tree hangs off it.
- **Leaf** — a node with no children.
- **Subtree** — a node together with everything hanging below it.
- **Height** — the number of steps on the longest path from a node down to a leaf.
  A leaf has height `0`; an empty spot has height `-1` (a convention that makes the
  arithmetic clean — see doc 2).
- **`n`** — the number of nodes in the tree.
