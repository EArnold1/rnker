# 4. The Rust implementation

This maps the ideas from docs 1–3 onto the code in `src/`.

## Module layout

| file | holds | visibility |
|------|-------|------------|
| `src/lib.rs` | crate docs, module declarations, the two `pub use` re-exports | — |
| `src/node.rs` | `Node<K, V>` and the `height` / `size` / `balance_factor` helpers | `pub(crate)` |
| `src/balance.rs` | the four rotations + `rebalance` | `pub(crate)` |
| `src/tree.rs` | `OrderStatisticTree<K, V>` (the public type) + the recursive insert/remove core | mixed |
| `src/iter.rs` | `Iter<'a, K, V>`, the ascending iterator | `pub` |

Only two names leave the crate: `OrderStatisticTree` and `Iter`. Everything else
(`Node`, rotations, the recursive helpers) is `pub(crate)` or private — callers
can't build a malformed tree.

## The node

```rust
pub(crate) struct Node<K, V> {
    pub(crate) key: K,
    pub(crate) value: V,
    pub(crate) height: i32,     // doc 2 — leaf = 0, updated bottom-up
    pub(crate) size: usize,     // doc 3 — leaf = 1, updated bottom-up
    pub(crate) left: Option<Box<Node<K, V>>>,
    pub(crate) right: Option<Box<Node<K, V>>>,
}
```

Why each piece:

- **`Option<...>`** — a child may be absent. `None` is "no child".
- **`Box<...>`** — a `Node` contains `Node`s, so its size would be infinite without
  a pointer indirection. `Box` is a pointer to heap-allocated data; the compiler
  can size `Box<Node>` (it's just a pointer).
- **`height: i32`** — signed on purpose. The empty-spot convention is `-1`, and
  `balance_factor` is a difference that can be negative.
- **`size: usize`** — a count; it can't be negative and it indexes things.

## `Link` — "the slot that holds a node"

```rust
type Link<K, V> = Option<Box<Node<K, V>>>;
```

This alias is the single most important idea in the implementation. Nearly every
function takes `&mut Link<K, V>` — a mutable reference to the **slot** — rather
than `&mut Node<K, V>` — a reference to the node itself.

Why: rotations and deletion need to **replace** the node that lives in a slot with
a different node. If a function only had `&mut Node`, it could change the node's
fields but never swap in a whole new node. With `&mut Link` it can:

```rust
let mut old_root = link.take().unwrap();   // slot is now None, we own the node
// ... rearrange ...
*link = Some(new_root);                    // slot now holds a different node
```

`.take()` pulls the `Some(Box<Node>)` out and leaves `None` behind, handing us
ownership of the boxed node so we can freely move its children around.

## Insert

```rust
fn insert_node<K: Ord, V>(link: &mut Link<K, V>, key: K, value: V) -> Option<V> {
    let Some(node) = link.as_mut() else {
        *link = Some(Node::boxed(key, value));   // empty slot -> new leaf
        return None;
    };

    let replaced = match key.cmp(&node.key) {
        Ordering::Less    => insert_node(&mut node.left,  key, value),
        Ordering::Greater => insert_node(&mut node.right, key, value),
        Ordering::Equal   => return Some(std::mem::replace(&mut node.value, value)),
    };

    node::refresh(link.as_mut().unwrap());   // update_height + update_size
    balance::rebalance(link);                // fix a ±2 balance factor
    replaced
}
```

Line by line against doc 2's pseudo-code:

1. **Empty slot** → drop a new leaf in and return (no old value).
2. **`Equal`** → the key exists. Swap the value in place with
   `std::mem::replace` (which returns the old value), and `return` early — the
   tree's *shape* didn't change, so there's nothing to refresh or rebalance.
3. **`Less` / `Greater`** → recurse into the child slot. The recursion does the
   leaf insertion and all the refresh/rebalance work *below* this node.
4. Back from the recursion: `refresh` this node (a child may have grown taller or
   gained a descendant), then `rebalance` it.

### The `link.as_mut().unwrap()` re-borrow

Notice we take `node` with `link.as_mut()`, use it inside the `match`, then in the
last two lines go back to `link` directly. We can't keep `node` alive across
`balance::rebalance(link)`, because `rebalance` needs `&mut link` and `node` is
already borrowing from `link` — two mutable borrows of the same thing.

So the pattern is: use the `node` borrow for the comparison and recursion, let it
end, then re-acquire access through `link` (`link.as_mut().unwrap()` — we know
it's `Some`, we're inside the `Some` arm). This "borrow, drop, re-borrow" shuffle
shows up throughout `tree.rs` and `balance.rs`; it's the price of the `&mut Link`
pattern, and the compiler enforces it.

## Rotations

`src/balance.rs`. Each rotation is the doc-2 picture turned into `.take()` /
assignment:

```rust
fn ll_rotation<K, V>(node: &mut Link<K, V>) {
    let mut old_root = node.take().unwrap();          // z
    let mut new_root = old_root.left.take().unwrap(); // y

    old_root.left = new_root.right.take();  // T3 moves from y.right to z.left
    node::refresh(&mut old_root);           // z's children changed -> refresh z FIRST

    new_root.right = Some(old_root);        // z becomes y's right child
    node::refresh(&mut new_root);           // now y's children are final -> refresh y

    *node = Some(new_root);                 // slot now holds y
}
```

**Refresh order matters:** always refresh the node that ends up *lower* first, so
that when you refresh the node above it, the lower node's `height`/`size` are
already correct. Here: `old_root` (ends up lower) before `new_root`.

`lr_rotation` / `rl_rotation` do the same for the zig-zag cases — grab `z`, `y`,
`x`, redistribute `x`'s two subtrees to `y` and `z`, refresh `y` and `z`, then
refresh `x` and drop it in the slot.

`rebalance` is doc 2's decision table, unchanged.

## Delete

Three functions work together.

### `remove_node` — find the target, then repair upward

```rust
fn remove_node<K: Ord + Clone, V>(link: &mut Link<K, V>, key: &K) -> Option<V> {
    let ordering = key.cmp(&link.as_deref()?.key);   // `?` -> None if slot empty

    let removed = match ordering {
        Ordering::Less    => remove_node(&mut link.as_mut().unwrap().left,  key),
        Ordering::Greater => remove_node(&mut link.as_mut().unwrap().right, key),
        Ordering::Equal   => Some(remove_root(link)),
    };

    if let Some(node) = link.as_mut() {   // the slot may now be empty
        node::refresh(node);
        balance::rebalance(link);
    }
    removed
}
```

Same "borrow, drop, re-borrow" dance: compute `ordering` first (a short borrow),
then act. The `if let Some(node)` guard matters — after `remove_root`, this slot
might be `None` (we deleted a leaf), and there's nothing to refresh.

Recall from doc 2: deletion can unbalance several ancestors, so `rebalance` runs
at *every* level on the way up, and may rotate more than once.

### `remove_root` — unlink the node sitting in this slot

```rust
fn remove_root<K: Ord + Clone, V>(link: &mut Link<K, V>) -> V {
    let mut root = *link.take().expect("remove_root on an empty slot");

    match (root.left.take(), root.right.take()) {
        (None, None)                        => root.value,            // leaf
        (Some(child), None) | (None, Some(child)) => {                // one child
            *link = Some(child);
            root.value
        }
        (Some(left), Some(right)) => {                                // two children
            let mut right = Some(right);
            let (succ_key, succ_value) = take_min(&mut right);        // in-order successor
            let removed = std::mem::replace(&mut root.value, succ_value);
            root.key   = succ_key;
            root.left  = Some(left);
            root.right = right;
            *link = Some(Box::new(root));
            removed
        }
    }
}
```

- `*link.take().unwrap()` — `.take()` gives `Box<Node>`, the `*` moves the `Node`
  out of the box so we can pull fields out of it individually.
- The **two-children** case is doc 1's successor trick. `take_min` on the right
  subtree removes *and returns* the smallest entry there; we overwrite this node's
  `key` and `value` with it. Overwriting **both** fields matters — an earlier
  version copied only the key, leaving the wrong value attached.
- Not rebalanced here: `remove_node` (the caller) does that as the recursion
  unwinds.

### `take_min` — remove the leftmost entry

```rust
fn take_min<K: Ord + Clone, V>(link: &mut Link<K, V>) -> (K, V) {
    let has_left = link.as_deref().expect("empty subtree").left.is_some();

    if has_left {
        let entry = take_min(&mut link.as_mut().unwrap().left);
        node::refresh(link.as_mut().unwrap());
        balance::rebalance(link);
        entry
    } else {
        let node = *link.take().unwrap();   // this IS the minimum
        *link = node.right;                 // its right child (if any) slides up
        (node.key, node.value)
    }
}
```

Keep going left until a node has no left child — that node holds the smallest key.
Replace it with its right child. Refresh and rebalance on the way back up, just
like a normal delete (because that's what this is — a delete of the minimum).

## Trait bounds

| bound | where | why |
|-------|-------|-----|
| `K: Ord` | `insert`, `get`, `rank`, `select`, iteration builders | every step compares the search key against a node key; a BST needs a **total** order (`Ord`), not the partial order of `PartialOrd` — see the note below |
| `K: Ord + Clone` | `remove` only | the two-children case copies the successor's key into the node being kept; `take_min` returns an owned `K` |
| `V` | no bounds | values are just carried around |

### `Ord` vs `PartialEq + PartialOrd`

`PartialOrd` allows *incomparable* pairs — `a.partial_cmp(&b)` can return `None`
(the classic case is `f64` and `NaN`). If a comparison against a node key ever came
back `None`, the search would take the wrong branch and quietly corrupt the tree's
invariant with no error. `Ord::cmp` always returns exactly one of
`Less` / `Equal` / `Greater`, which both guarantees the total order a BST depends
on and lets each step be a single `match key.cmp(&node.key)`. `Ord` implies
`Eq + PartialOrd + PartialEq`, so nothing is lost.

## The iterator

`src/iter.rs` is an **iterative** in-order traversal (doc 1) — recursion would
need to borrow the whole tree through a call stack we don't control.

```rust
pub struct Iter<'a, K, V> {
    stack: Vec<&'a Node<K, V>>,     // ancestors not yet yielded
    pending: Option<&'a Node<K, V>>, // subtree whose left spine we still must push
}
```

Each `next()`:

1. walk `pending` all the way left, pushing every node onto `stack`;
2. pop one node — that's the next key in ascending order — and yield it;
3. set `pending` to that node's right child, so the next call explores it.

`&tree` also works as an iterator (`impl IntoIterator for &OrderStatisticTree`),
so `for (k, v) in &tree` is idiomatic.

## Tests

- **`src/tree.rs`, `mod tests`** — white-box. Has access to the private `root` and
  to `assert_valid()`, a `#[cfg(test)]` helper that recursively checks: BST
  ordering, `|balance_factor| <= 1` at every node, and every stored `height` /
  `size` equal to a freshly recomputed value. The stress test runs 500 random
  insert/remove ops and calls `assert_valid()` after each one.
- **`tests/order_statistic.rs`** — black-box. Only the public API. Checks the
  descending-rank semantics, that `select` inverts `rank` for every key, boundary
  cases (`select(0)`, `select(len + 1)`), behaviour after removals, and `String`
  keys.
- **doc-tests** — the examples in `src/lib.rs` and on `OrderStatisticTree` are
  compiled and run by `cargo test`.

Run everything: `cargo test`. Lint: `cargo clippy --all-targets -- -D warnings`.
See the demo: `cargo run --example demo`.
