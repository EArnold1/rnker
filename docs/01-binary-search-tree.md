# 1. The binary search tree

## The goal

We want a container that keeps key/value pairs and supports three operations
quickly:

- **insert** a new key,
- **search** for a key,
- **delete** a key,

while always being able to walk the keys in **sorted order**.

A sorted array gives fast search (binary search) but slow insert/delete (you have
to shift elements). A linked list gives fast insert/delete but slow search. A
binary search tree is an attempt to get both.

## The shape

Every node has:

- a **key** and a **value**,
- an optional **left child**,
- an optional **right child**.

## The one rule

> For any node `N`: every key in `N`'s **left** subtree is **less than** `N.key`,
> and every key in `N`'s **right** subtree is **greater than** `N.key`.

This is the **BST ordering property**. It must hold at *every* node, not just the
root.

Here is a valid BST holding `10, 20, 30, 40, 50`:

```
        30
       /  \
     20    40
    /        \
  10          50
```

Check the rule at `30`: left subtree is `{10, 20}`, all `< 30`; right subtree is
`{40, 50}`, all `> 30`. Check it at `20`: left is `{10}` (`< 20`), right is empty.
And so on.

## Searching

To find a key, start at the root and compare:

- equal? found it.
- target is smaller? go **left**.
- target is larger? go **right**.
- ran off the bottom (no child that way)? the key isn't in the tree.

Searching for `10` in the tree above: `10 < 30` go left → `10 < 20` go left →
`10 == 10` found. Three comparisons.

Each step throws away one subtree, so the number of comparisons is at most the
**height** of the tree plus one.

## Inserting

Insertion is a search that didn't find the key: you walk down exactly as above,
and when you would step off the bottom, you put the new node there instead.

Inserting `25` into the tree above: `25 < 30` left → `25 > 20` right → right child
of `20` is empty, so `25` becomes it:

```
        30
       /  \
     20    40
    /  \     \
  10   25     50
```

The new node is always a leaf. It always lands in the one spot where the ordering
rule still holds.

## Deleting

Deletion is the fiddly one. Find the node, then handle three cases by how many
children it has:

**No children (a leaf).** Just remove it.

```
delete 10:            30                30
                     /  \      ->       /  \
                   20    40           20    40
                  /  \     \            \     \
                10   25     50          25     50
```

**One child.** Replace the node with that child (the child subtree slides up).

```
delete 40:            30                30
                     /  \      ->       /  \
                   20    40           20    50
                     \     \            \
                     25     50          25
```

**Two children.** You can't just remove it — both subtrees need a parent. The
trick: find the node's **in-order successor** (the smallest key in its right
subtree), copy that key/value into the node being "deleted", then delete the
successor from the right subtree. The successor always has at most one child (no
left child, or it wouldn't be the smallest), so deleting *it* is one of the two
easy cases.

```
delete 30 (successor is 40, the leftmost node of the right subtree):

        30                    40
       /  \                  /  \
     20    50      ->      20    50
       \   /                 \
       25 40                 25
```

## Why sorted order is easy

An **in-order traversal** — recurse left, visit the node, recurse right — visits
keys smallest to largest, because the ordering rule says exactly that: everything
left is smaller, everything right is larger.

```
in_order(20, 25, ..):   left({10}) , 20 , right({25})  ->  10, 20, 25
in_order(whole tree):   10, 20, 25, 30, 40, 50
```

`rnker`'s iterator (`src/iter.rs`) is this traversal, done with an explicit stack
instead of recursion.

## The problem: height is not guaranteed

Every operation above costs "about the height of the tree". For a nicely bushy
tree of `n` nodes, the height is about `log2(n)` — 20 for a million nodes. Great.

But the height depends on **insertion order**. Insert `10, 20, 30, 40, 50` in that
order and you get:

```
10
  \
   20
     \
      30
        \
         40
           \
            50
```

This is a linked list wearing a tree costume. Height is `n - 1`. Search is now
`O(n)`. Sorted or nearly-sorted input — extremely common in practice — produces
exactly this worst case.

The fix is to actively reshape the tree as we go, so the height can never drift
far from `log2(n)`. That is the next document.
