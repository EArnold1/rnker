# 3. The order statistic tree — AVL + subtree size

An AVL tree gives us fast `insert` / `search` / `delete` and sorted iteration.
Now we want two more operations, also fast:

- **`rank(key)`** — "what position does this key hold in sorted order?"
- **`select(i)`** — "which key is at position `i`?"

These are called **order statistic** queries. `rank` and `select` are inverses of
each other.

With just a plain AVL tree, answering `rank` means counting nodes — `O(n)`. We
want `O(log n)`. The fix is one extra number per node.

## The extra field: subtree size

Every node stores:

```
size = the number of nodes in the subtree rooted at this node
     = 1 (itself) + size(left child) + size(right child)
```

A leaf has `size = 1`. An empty spot has `size = 0` (same convention style as
`height = -1`).

In the code: `Node.size`, plus `node::size` / `node::update_size` in
`src/node.rs`. Like height, it's **stored**, so reading it is `O(1)`, and
`update_size` recomputes one node from its children's stored sizes — also `O(1)`:

```
update_size(node) = 1 + size(node.left) + size(node.right)
```

### Keeping it correct

`size` must be refreshed at **exactly the same places** as `height`:

- on the way back up during `insert` and `delete`, at every node on the path;
- inside every rotation, for every node whose children changed.

`src/node.rs` bundles the two together as `node::refresh`, and `src/balance.rs`
calls `refresh` after every rotation. Miss one spot and `rank`/`select` silently
return wrong answers — the tree still looks fine, the counts are just stale.

`len()` falls out for free: it's `size(root)`, read in `O(1)`.

## A worked tree

We'll use this tree for every example below:

```
              30                    sizes:
            /    \                    30 -> 6
          20      40                  20 -> 3   (10, 20, 25)
         /  \       \                 40 -> 2   (40, 50)
       10    25      50               10 -> 1
                                      25 -> 1
                                      50 -> 1
```

Keys in **ascending** order:  `10, 20, 25, 30, 40, 50`
Keys in **descending** order: `50, 40, 30, 25, 20, 10`

## `rnker` uses descending rank

> **Rank 1 is the largest key.** Rank `n` is the smallest.

| key | 50 | 40 | 30 | 25 | 20 | 10 |
|-----|----|----|----|----|----|----|
| rank| 1  | 2  | 3  | 4  | 5  | 6  |

This is a design choice, not a rule of the data structure (it comes from the
earlier project this grew out of, whose traversal visited **right → node → left**).
Ascending rank works identically with the left/right comparisons swapped; and you
can always convert: `ascending_rank = len - descending_rank + 1`.

## `rank(key)` — key → position

Walk from the root, keeping a running count `rank` of keys already known to come
*before* `key` in descending order (i.e. keys known to be larger).

At each node, let `R = size(node.right)` (how many keys are bigger than
`node.key` *within this subtree*):

| comparison | meaning | action |
|---|---|---|
| `key > node.key` | `key` is bigger than everything here and to the left; the answer is to the right | go right, add nothing |
| `key < node.key` | `node` itself and its whole right subtree are all bigger than `key`, so they come first | `rank += R + 1`, go left |
| `key == node.key` | found it; everything counted so far, plus `node` and its right subtree, sits above it | **return `rank + R + 1`** |
| ran off the bottom | `key` isn't in the tree | **return `None`** |

### Example: `rank(40)`

```
start: rank = 0, node = 30
  40 > 30            -> go right                     rank = 0
node = 40
  40 == 40          -> return 0 + size(50-subtree=1) + 1  = 2
```

`40` has rank **2**. ✅

### Example: `rank(20)`

```
start: rank = 0, node = 30
  20 < 30            -> rank += size({40,50}=2) + 1  -> rank = 3;  go left
node = 20
  20 == 20          -> return 3 + size({25}=1) + 1   = 5
```

`20` has rank **5**. ✅

### Example: `rank(35)` (absent)

```
rank = 0, node = 30
  35 > 30  -> go right
node = 40
  35 < 40  -> rank += size({50}=1)+1 -> rank = 2;  go left
node = 40.left = empty  -> return None
```

The earlier version of this code returned the half-computed `2` here instead of
`None` — a bug. The rule is: only a matched key produces a number.

This is `OrderStatisticTree::rank` in `src/tree.rs`.

## `select(i)` — position → key

The inverse walk. Carry `remaining`, the rank we're still looking for, expressed
**relative to the current subtree**.

At each node, let `here = size(node.right) + 1`. That is `node`'s own descending
rank *within its own subtree*: all the bigger keys (`node.right`) come first, then
`node`.

| comparison | meaning | action |
|---|---|---|
| `remaining == here` | `node` is exactly the one we want | **return `node`** |
| `remaining < here` | the target is among the bigger keys | go right, `remaining` unchanged |
| `remaining > here` | skip `node` and its whole right subtree (`here` keys) | `remaining -= here`, go left |

(`rnker` first rejects `i == 0` or `i > len()` up front, so the walk can't fall
off the bottom.)

### Example: `select(2)`

```
remaining = 2, node = 30
  here = size({40,50}=2) + 1 = 3
  2 < 3  -> go right
node = 40
  here = size({50}=1) + 1 = 2
  2 == 2 -> return 40
```

Position 2 is `40`. ✅ (Consistent with `rank(40) == 2`.)

### Example: `select(6)`

```
remaining = 6, node = 30
  here = 3;  6 > 3  -> remaining = 3;  go left
node = 20
  here = size({25}=1) + 1 = 2;  3 > 2  -> remaining = 1;  go left
node = 10
  here = size(empty) + 1 = 1;  1 == 1  -> return 10
```

Position 6 is `10`, the smallest key. ✅

This is `OrderStatisticTree::select` in `src/tree.rs`.

## Why this is `O(log n)`

Both walks take one step per level and do `O(1)` work per step (a comparison and a
stored-`size` read). The AVL invariant keeps the number of levels at `O(log n)`.
So `rank` and `select` are `O(log n)` — the same as `search`.

## What you can build from these

- **count of keys greater than `x`**: `rank(x) - 1` (or a near-miss walk if `x`
  might be absent)
- **k-th largest**: `select(k)`
- **median**: `select((len + 1) / 2)`
- **percentile / quantile**: `select(len * p)`
- **"is `a` ranked above `b`?"**: `rank(a) < rank(b)`

All in `O(log n)`, on a structure that still supports `O(log n)` insert and
delete — which a sorted array cannot.
