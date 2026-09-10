# 2. The AVL tree — a self-balancing BST

Doc 1 ended with a problem: a BST built from sorted input degenerates into a
list, and every operation becomes `O(n)`.

An **AVL tree** (Adelson-Velsky and Landis, 1962) fixes this by enforcing one
extra rule on top of the BST ordering rule, and repairing that rule after every
insert and delete. The repair costs `O(log n)`, so all operations stay
`O(log n)`.

## Height, with a convention

**Height** of a node = the number of edges on the longest downward path to a leaf.

We define the height of an **empty spot** (a missing child) as `-1`. That single
convention removes every special case from the arithmetic:

```
height(node) = 1 + max(height(node.left), height(node.right))
```

- A leaf: `1 + max(-1, -1) = 0`. ✅
- A node with one leaf child: `1 + max(0, -1) = 1`. ✅

In the code this is `node::height` and `node::update_height` in `src/node.rs`.
Each node **stores** its height so reading it is `O(1)`; `update_height` recomputes
one node's height from its children's stored heights, also `O(1)`.

## The balance factor

For any node `N`:

```
balance_factor(N) = height(N.left) - height(N.right)
```

- `bf > 0` → **left-heavy** (left side is taller)
- `bf < 0` → **right-heavy**
- `bf == 0` → perfectly even

## The AVL invariant

> For **every** node `N`: `-1 <= balance_factor(N) <= 1`.

In words: at no node may one side be more than one level taller than the other.

This is enough to force the height of the whole tree to stay below about
`1.44 * log2(n)`. It is not perfectly balanced, but it is *balanced enough* — the
height is always `O(log n)`.

The list-shaped tree from doc 1 violates the invariant immediately: the root would
have `balance_factor = -1 - (n-2)`, wildly right-heavy.

## Restoring the invariant: rotations

A **rotation** is a local rearrangement of two or three nodes that changes the
tree's height distribution **without breaking the BST ordering rule**. It's the
only tool an AVL tree needs.

### Single rotation — the "straight line" cases

When a node is left-heavy *and* its tall child is also left-heavy (or even), the
three key nodes form a straight line `z – y – x` leaning left. One rotation
"pivots" around the middle node.

**LL case → rotate right around `z`:**

```
        z                        y
       / \                     /   \
      y   T4        ==>       x       z
     / \                     / \     / \
    x   T3                  T1 T2   T3 T4
   / \
  T1 T2
```

What moved:

1. `y` becomes the new subtree root; `z` becomes `y`'s right child.
2. `y`'s old right subtree `T3` — which holds keys *between* `y` and `z` — is
   handed to `z` as its new left child. That's the only subtree that changes
   parent.

Ordering still holds: `T1 < x < T2 < y < T3 < z < T4`, before and after.

Heights: the tall side `{x, T1, T2}` moved up a level, the short side `T4` moved
down a level. The subtree got shorter by one and even.

**RR case → rotate left around `z`:** the exact mirror image.

```
    z                            y
   / \                         /   \
  T1  y            ==>        z       x
     / \                     / \     / \
    T2  x                   T1 T2   T3 T4
       / \
      T3 T4
```

In the code these are `ll_rotation` and `rr_rotation` in `src/balance.rs`.

### Double rotation — the "zig-zag" cases

When a node is left-heavy but its tall child leans the *other* way (right), a
single rotation doesn't help — it just moves the lean from one side to the other.
You need two rotations.

**LR case:** `z` is left-heavy, but its left child `y` is right-heavy. The tall
grandchild is `x = y.right`.

```
      z                 z                    x
     / \               / \                 /   \
    y  T4    step 1    x  T4    step 2     y     z
   / \      ------>   / \      ------>    / \   / \
  T1  x     rotate   y  T3     rotate    T1 T2 T3 T4
     / \    y left  / \        z right
    T2 T3          T1 T2
```

- **Step 1:** rotate `y` left. Now `x` is directly above `y`, and we're back to a
  straight-line left-left shape.
- **Step 2:** rotate `z` right, exactly the LL case.

`x` ends up as the new root, with `y` taking `x`'s old left subtree `T2` and `z`
taking `x`'s old right subtree `T3`.

`src/balance.rs` does this in one pass rather than literally calling the two
single rotations — `lr_rotation` grabs `z`, `y`, `x`, hands `T2` to `y` and `T3`
to `z`, then makes `x` the root — but the result is identical.

**RL case:** the mirror image — `z` right-heavy, right child left-heavy,
`x = y.left` — handled by `rl_rotation`.

### Choosing the rotation

Given an unbalanced node `N`:

```
if balance_factor(N) > 1:          # left-heavy (bf is +2)
    if balance_factor(N.left) >= 0:  ll_rotation   # straight line
    else:                            lr_rotation   # zig-zag
elif balance_factor(N) < -1:       # right-heavy (bf is -2)
    if balance_factor(N.right) <= 0: rr_rotation   # straight line
    else:                            rl_rotation   # zig-zag
```

This is `rebalance` in `src/balance.rs`, verbatim. After one insert, `bf` can only
reach `±2`, never more, so this catches every case.

## Insert, the AVL way

```
insert(node, key):
    1. if node is empty: put a new leaf here, done.
    2. recurse into node.left or node.right (plain BST rule).
    3. on the way back up, at THIS node:
         a. update_height(node)      # a child may have grown
         b. rebalance(node)          # fix a ±2 balance factor if present
```

Steps 3a and 3b run at every node on the path from the new leaf back to the root.
For **insertion**, at most **one** rotation happens in the whole path — the first
rotation restores every height above it too.

### Worked example: insert `10, 20, 30, 40, 50, 25`

Insert `10`, `20`: fine.

Insert `30` → `10` is now right-heavy by 2 (`bf(10) = -2`), right child `20` is
right-heavy (`bf <= 0`) → **RR**:

```
  10                 20
    \              /    \
     20     ->    10     30
       \
        30
```

Insert `40`: goes right of `30`. All balance factors within `±1`. No rotation.

Insert `50` → `30` becomes right-heavy by 2, right child `40` right-heavy → **RR**
around `30`:

```
      20                    20
     /  \                  /   \
    10   30      ->       10    40
           \                   /  \
            40                30    50
              \
               50
```

Insert `25`: path is `25 < 30` — wait, `25 > 20` right, `25 < 40` left, `25 < 30`
left → new leaf left of `30`. Walking back up, `20` becomes right-heavy by 2
(`bf(20) = -2`); its right child `40` is **left**-heavy (`bf(40) = +1 > 0`) → the
zig-zag case → **RL**:

- `x = 30` (left child of `40`) becomes the new subtree root.
- `20` takes `x`'s old left subtree — which is `25` — as its new right child.
- `40` takes `x`'s old right subtree — empty here.

```
            30
          /    \
        20      40
       /  \       \
     10    25      50
```

In-order: `10, 20, 25, 30, 40, 50`. Height is `2`. Balanced.

(`src/tree.rs`'s test `remove_rebalances` starts from exactly this tree.)

## Delete, the AVL way

Deletion reuses the plain-BST deletion from doc 1 (three child-count cases), then
does the **same** `update_height` + `rebalance` on the way back up.

One difference from insert: a deletion **shortens** a subtree, and that can
unbalance several ancestors in turn. So unlike insert, delete may perform a
rotation at **more than one** level on the way up. The code doesn't special-case
this — it just calls `rebalance` at every level, which covers both insert and
delete.

## Cost

- The path from root to any leaf is `O(log n)` because the invariant caps the
  height.
- `update_height` and each rotation touch a constant number of nodes → `O(1)`.
- We do that work once per level → **`O(log n)` per insert or delete**.

Next: give each node a second piece of bookkeeping — its subtree size — and use it
to answer rank/select queries.
