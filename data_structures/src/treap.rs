use std::mem;
use std::ops::{Add, Sub};
use std::vec::Vec;
use std::cmp::Ordering;
use rand::{Rng, XorShiftRng};

/// A struct representing an internal node of a treap.
struct Node<T: Ord, U> {
    key: T,
    value: U,
    priority: u32,
    left: Tree<T, U>,
    right: Tree<T, U>,
}

impl<T: Ord, U> Node<T, U> {
    #[inline]
    fn is_heap_property_violated(&self, child: &Tree<T, U>) -> bool {
        match *child {
            None => false,
            Some(ref child_node) => self.priority < child_node.priority,
        }
    }

    #[inline]
    fn rotate_left(&mut self) {
        let right = self.right.take();
        if let Some(mut old_node) = right {
            mem::swap(self, &mut old_node);
            old_node.right = self.left.take();
            self.left = Some(old_node);
        }
    }

    #[inline]
    fn rotate_right(&mut self) {
        let left = self.left.take();
        if let Some(mut old_node) = left {
            mem::swap(self, &mut old_node);
            old_node.left = self.right.take();
            self.right = Some(old_node);
        }
    }
}

type Tree<T, U> = Option<Box<Node<T, U>>>;

/// An ordered map implemented by a treap.
///
/// A treap is a tree that satisfies both the binary search
/// tree property and a heap property. Each node has a key, a value, and a priority. The key of any
/// node is greather than all keys in its left subtree and less than all keys occuring in its right
/// subtree. The priority of a node is greater than the priority of all nodes in its subtrees. By
/// randomly generating priorities, the expected height of the tree is proportional to the
/// logarithm of the number of keys.
///
/// # Examples
/// ```
/// use data_structures::Treap;
///
/// let mut t = Treap::new();
/// t.insert(0, 1);
/// t.insert(3, 4);
///
/// assert_eq!(t.get(&0), Some(&1));
/// assert_eq!(t.get(&1), None);
/// assert_eq!(t.size(), 2);
///
/// assert_eq!(t.min(), Some(&0));
/// assert_eq!(t.ceil(&2), Some(&3));
///
/// *t.get_mut(&0).unwrap() = 2;
/// assert_eq!(t.remove(&0), Some((0, 2)));
/// assert_eq!(t.remove(&1), None);
/// ```
pub struct Treap<T: Ord, U> {
    root: Tree<T, U>,
    rng: XorShiftRng,
    size: usize,
}

impl<T: Ord, U> Treap<T, U> {
    /// Constructs a new, empty `Treap<T, U>`
    ///
    /// # Examples
    /// ```
    /// use data_structures::Treap;
    ///
    /// let mut t: Treap<u32, u32> = Treap::new();
    /// ```
    pub fn new() -> Self {
        Treap {
            root: None,
            rng: XorShiftRng::new_unseeded(),
            size: 0,
        }
    }

    fn merge(l_tree: &mut Tree<T, U>, r_tree: Tree<T, U>) {
        match (l_tree.take(), r_tree) {
            (Some(mut l_node), Some(mut r_node)) => {
                if l_node.priority > r_node.priority {
                    Self::merge(&mut l_node.right, Some(r_node));
                    *l_tree = Some(l_node);
                } else {
                    let mut new_tree = Some(l_node);
                    Self::merge(&mut new_tree, r_node.left.take());
                    r_node.left = new_tree;
                    *l_tree = Some(r_node);
                }
            },
            (new_tree, None) | (None, new_tree) => *l_tree = new_tree,
        }
    }

    fn split(tree: &mut Tree<T, U>, k: &T) -> (Tree<T, U>, Tree<T, U>) {
        match tree.take() {
            Some(mut node) => {
                let mut ret;
                if node.key < *k {
                    ret = Self::split(&mut node.right, k);
                    *tree = Some(node);
                } else if node.key > *k {
                    let mut res = Self::split(&mut node.left, k);
                    *tree = node.left.take();
                    node.left = res.1;
                    ret = (res.0, Some(node));
                } else {
                    *tree = node.left.take();
                    let right = node.right.take();
                    ret = (Some(node), right);
                }
                ret
            },
            None => (None, None),
        }
    }


    fn tree_insert(tree: &mut Tree<T, U>, new_node: Node<T, U>) -> Option<(T, U)> {
        if let Some(ref mut node) = *tree {
            let mut ret;
            match new_node.key.cmp(&node.key) {
                Ordering::Less => {
                    ret = Self::tree_insert(&mut node.left, new_node);
                    if node.is_heap_property_violated(&node.left) {
                        node.rotate_right();
                    }
                },
                Ordering::Greater => {
                    ret = Self::tree_insert(&mut node.right, new_node);
                    if node.is_heap_property_violated(&node.right) {
                        node.rotate_left();
                    }
                },
                Ordering::Equal => {
                    let &mut Node { ref mut key, ref mut value, .. } = &mut **node;
                    ret = Some((mem::replace(key, new_node.key), mem::replace(value, new_node.value)));
                },
            }
            ret
        } else {
            *tree = Some(Box::new(new_node));
            None
        }
    }

    /// Inserts a key-value pair into the treap. If the key already exists in the treap, it will
    /// return and replace the old key-value pair.
    ///
    /// # Examples
    /// ```
    /// use data_structures::Treap;
    ///
    /// let mut t = Treap::new();
    /// assert_eq!(t.insert(1, 1), None);
    /// assert_eq!(t.get(&1), Some(&1));
    /// assert_eq!(t.insert(1, 2), Some((1, 1)));
    /// assert_eq!(t.get(&1), Some(&2));
    /// ```
    pub fn insert(&mut self, key: T, value: U) -> Option<(T, U)> {
        let &mut Treap { ref mut root, ref mut rng, ref mut size } = self;
        let new_node = Node {
            key: key,
            value: value,
            priority: rng.next_u32(),
            left: None,
            right: None,
        };
        let ret = Self::tree_insert(root, new_node);
        if ret.is_none() {
            *size += 1;
        }
        ret
    }

    /// Removes a key-value pair from the treap. If the key exists in the treap, it will return
    /// the associated key-value pair. Otherwise it will return `None`.
    ///
    /// # Examples
    /// ```
    /// use data_structures::Treap;
    ///
    /// let mut t = Treap::new();
    /// t.insert(1, 1);
    /// assert_eq!(t.remove(&1), Some((1, 1)));
    /// assert_eq!(t.remove(&1), None);
    /// ```
    pub fn remove(&mut self, key: &T) -> Option<(T, U)> {
        let &mut Treap { ref mut root, ref mut size, .. } = self;
        let (old_node_opt, r_tree) = Self::split(root, key);
        Self::merge(root, r_tree);
        match old_node_opt {
            Some(old_node) => {
                let unboxed_old_node = *old_node;
                let Node { key, value, .. } = unboxed_old_node;
                *size -= 1;
                Some((key, value))
            },
            None => None,
        }
    }

    fn tree_contains(tree: &Tree<T, U>, key: &T) -> bool {
        match *tree {
            Some(ref node) => {
                if key == &node.key {
                    true
                } else if key < &node.key {
                    Self::tree_contains(&node.left, key)
                } else {
                    Self::tree_contains(&node.right, key)
                }
            },
            None => false,
        }
    }

    /// Checks if a key exists in the treap.
    ///
    /// # Examples
    /// ```
    /// use data_structures::Treap;
    ///
    /// let mut t = Treap::new();
    /// t.insert(1, 1);
    /// assert_eq!(t.contains(&0), false);
    /// assert_eq!(t.contains(&1), true);
    /// ```
    pub fn contains(&self, key: &T) -> bool {
        let &Treap { ref root, .. } = self;
        Self::tree_contains(root, key)
    }

    fn tree_get<'a>(tree: &'a Tree<T, U>, key: &T) -> Option<&'a U> {
        match *tree {
            Some(ref node) => {
                if key == &node.key {
                    Some(&node.value)
                } else if key < &node.key {
                    Self::tree_get(&node.left, key)
                } else {
                    Self::tree_get(&node.right, key)
                }
            },
            None => None,
        }
    }

    /// Returns an immutable reference to the value associated with a particular key. It will
    /// return `None` if the key does not exist in the treap.
    ///
    /// # Examples
    /// ```
    /// use data_structures::Treap;
    ///
    /// let mut t = Treap::new();
    /// t.insert(1, 1);
    /// assert_eq!(t.get(&0), None);
    /// assert_eq!(t.get(&1), Some(&1));
    /// ```
    pub fn get(&self, key: &T) -> Option<&U> {
        let &Treap { ref root, .. } = self;
        Self::tree_get(root, key)
    }

    fn tree_get_mut<'a>(tree: &'a mut Tree<T, U>, key: &T) -> Option<&'a mut U> {
        match *tree {
            Some(ref mut node) => {
                if key == &node.key {
                    Some(&mut node.value)
                } else if key < &node.key {
                    Self::tree_get_mut(&mut node.left, key)
                } else {
                    Self::tree_get_mut(&mut node.right, key)
                }
            },
            None => None,
        }
    }

    /// Returns a mutable reference to the value associated with a particular key. Returns `None`
    /// if such a key does not exist.
    ///
    /// # Examples
    /// ```
    /// use data_structures::Treap;
    ///
    /// let mut t = Treap::new();
    /// t.insert(1, 1);
    /// *t.get_mut(&1).unwrap() = 2;
    /// assert_eq!(t.get(&1), Some(&2));
    /// ```
    pub fn get_mut(&mut self, key: &T) -> Option<&mut U> {
        let &mut Treap { ref mut root, .. } = self;
        Self::tree_get_mut(root, key)
    }

    /// Returns the size of the treap.
    ///
    /// # Examples
    /// ```
    /// use data_structures::Treap;
    ///
    /// let mut t = Treap::new();
    /// t.insert(1, 1);
    /// assert_eq!(t.size(), 1);
    /// ```
    pub fn size(&self) -> usize {
        let &Treap { ref size, .. } = self;
        *size
    }

    fn tree_ceil<'a>(tree: &'a Tree<T, U>, key: &T) -> Option<&'a T> {
        match *tree {
            Some(ref node) => {
                if &node.key == key {
                    Some(&node.key)
                } else if &node.key < key {
                    Self::tree_ceil(&node.right, key)
                } else {
                    let res = Self::tree_ceil(&node.left, key);
                    if res.is_some() {
                        res
                    } else {
                        Some(&node.key)
                    }
                }
            },
            None => None,
        }
    }

    /// Returns a key in the treap that is greater than or equal to a particular key. Returns
    /// `None` if such a key does not exist.
    ///
    /// # Examples
    /// ```
    /// use data_structures::Treap;
    ///
    /// let mut t = Treap::new();
    /// t.insert(1, 1);
    /// assert_eq!(t.ceil(&0), Some(&1));
    /// assert_eq!(t.ceil(&2), None);
    /// ```
    pub fn ceil(&self, key: &T) -> Option<&T> {
        let &Treap { ref root, .. } = self;
        Self::tree_ceil(root, key)
    }

    fn tree_floor<'a>(tree: &'a Tree<T, U>, key: &T) -> Option<&'a T> {
        match *tree {
            Some(ref node) => {
                if &node.key == key {
                    Some(&node.key)
                } else if &node.key > key {
                    Self::tree_floor(&node.left, key)
                } else {
                    let res = Self::tree_floor(&node.right, key);
                    if res.is_some() {
                        res
                    } else {
                        Some(&node.key)
                    }
                }
            },
            None => None,
        }
    }

    /// Returns a key in the treap that is less than or equal to a particular key. Returns
    /// `None` if such a key does not exist.
    ///
    /// # Examples
    /// ```
    /// use data_structures::Treap;
    ///
    /// let mut t = Treap::new();
    /// t.insert(1, 1);
    /// assert_eq!(t.floor(&0), None);
    /// assert_eq!(t.floor(&2), Some(&1));
    /// ```
    pub fn floor(&self, key: &T) -> Option<&T> {
        let &Treap { ref root, .. } = self;
        Self::tree_floor(root, key)
    }

    fn tree_min(tree: &Tree<T, U>) -> Option<&T> {
        if let Some(ref node) = *tree {
            let mut curr = node;
            while let Some(ref left_node) = curr.left {
                curr = left_node;
            }
            Some(&curr.key)
        } else {
            None
        }
    }

    /// Returns the minimum key of the treap. Returns `None` if the treap is empty.
    ///
    /// # Examples
    /// ```
    /// use data_structures::Treap;
    ///
    /// let mut t = Treap::new();
    /// t.insert(1, 1);
    /// t.insert(3, 3);
    /// assert_eq!(t.min(), Some(&1));
    /// ```
    pub fn min(&self) -> Option<&T> {
        let &Treap { ref root, .. } = self;
        Self::tree_min(root)
    }

    fn tree_max(tree: &Tree<T, U>) -> Option<&T> {
        match *tree {
            Some(ref node) => {
                if node.right.is_some() {
                    Self::tree_max(&node.right)
                } else {
                    Some(&node.key)
                }
            },
            None => None,
        }
    }

    /// Returns the maximum key of the treap. Returns `None` if the treap is empty.
    ///
    /// # Examples
    /// ```
    /// use data_structures::Treap;
    ///
    /// let mut t = Treap::new();
    /// t.insert(1, 1);
    /// t.insert(3, 3);
    /// assert_eq!(t.max(), Some(&3));
    /// ```
    pub fn max(&self) -> Option<&T> {
        let &Treap { ref root, .. } = self;
        Self::tree_max(root)
    }

    fn tree_union(left_tree: Tree<T, U>, right_tree: Tree<T, U>, mut swapped: bool) -> (Tree<T, U>, usize) {
        match (left_tree, right_tree) {
            (Some(mut left_node), Some(mut right_node)) => {
                if left_node.priority < right_node.priority {
                    mem::swap(&mut left_node, &mut right_node);
                    swapped = !swapped;
                }
                let mut dups = 0;
                {
                    let &mut Node {
                        left: ref mut left_subtree,
                        right: ref mut right_subtree,
                        ref mut key,
                        ref mut value,
                        ..
                    } = &mut *left_node;
                    let mut right_left_subtree = Some(right_node);
                    let (duplicate_opt, right_right_subtree) = Self::split(&mut right_left_subtree, key);
                    let (new_left_subtree, left_dups) = Self::tree_union(left_subtree.take(), right_left_subtree, swapped);
                    let (new_right_subtree, right_dups) = Self::tree_union(right_subtree.take(), right_right_subtree, swapped);
                    dups += left_dups + right_dups;
                    *left_subtree = new_left_subtree;
                    *right_subtree = new_right_subtree;
                    if let Some(duplicate_node) = duplicate_opt {
                        if swapped {
                            *value = duplicate_node.value;
                        }
                        dups += 1;
                    }
                }
                (Some(left_node), dups)
            },
            (None, right_tree) => (right_tree, 0),
            (left_tree, None) => (left_tree, 0),
        }
    }

    /// Returns the union of two treaps. If there is a key that is found in both `left` and
    /// `right`, the union will contain the value associated with the key in `left`. The `+`
    /// operator is implemented to take the union of two treaps.
    ///
    /// # Examples
    /// ```
    /// use data_structures::Treap;
    ///
    /// let mut n = Treap::new();
    /// n.insert(1, 1);
    /// n.insert(2, 2);
    ///
    /// let mut m = Treap::new();
    /// m.insert(2, 3);
    /// m.insert(3, 3);
    ///
    /// let union = Treap::union(n, m);
    /// assert_eq!(
    ///     union.into_iter().collect::<Vec<(&u32, &u32)>>(),
    ///     vec![(&1, &1), (&2, &2), (&3, &3)],
    /// );
    /// ```
    pub fn union(left: Self, right: Self) -> Self {
        let Treap { root: left_tree, rng, size: left_size } = left;
        let Treap { root: right_tree, size: right_size, .. } = right;
        let (root, dups) = Self::tree_union(left_tree, right_tree, false);
        Treap { root, rng, size: left_size + right_size - dups }
    }

    fn tree_inter(left_tree: Tree<T, U>, right_tree: Tree<T, U>, mut swapped: bool) -> (Tree<T, U>, usize) {
        if let (Some(mut left_node), Some(mut right_node)) = (left_tree, right_tree) {
            let mut dups = 0;
            {
                if left_node.priority < right_node.priority {
                    mem::swap(&mut left_node, &mut right_node);
                    swapped = !swapped;
                }
                let &mut Node {
                    left: ref mut left_subtree,
                    right: ref mut right_subtree,
                    ref mut key,
                    ref mut value,
                    ..
                } = &mut *left_node;
                let mut right_left_subtree = Some(right_node);
                let (duplicate_opt, right_right_subtree) = Self::split(&mut right_left_subtree, key);
                let (new_left_subtree, left_dups) = Self::tree_inter(left_subtree.take(), right_left_subtree, swapped);
                let (new_right_subtree, right_dups) = Self::tree_inter(right_subtree.take(), right_right_subtree, swapped);
                dups += left_dups + right_dups;
                *left_subtree = new_left_subtree;
                *right_subtree = new_right_subtree;
                match duplicate_opt {
                    Some(duplicate_node) => {
                        if swapped {
                            *value = duplicate_node.value;
                        }
                        dups += 1;
                    },
                    None => {
                        Self::merge(left_subtree, right_subtree.take());
                        return (left_subtree.take(), dups);
                    },
                }
            }
            (Some(left_node), dups)
        } else {
            (None, 0)
        }
    }

    /// Returns the intersection of two treaps. If there is a key that is found in both `left` and
    /// `right`, the union will contain the value associated with the key in `left`.
    ///
    /// # Examples
    /// ```
    /// use data_structures::Treap;
    ///
    /// let mut n = Treap::new();
    /// n.insert(1, 1);
    /// n.insert(2, 2);
    ///
    /// let mut m = Treap::new();
    /// m.insert(2, 3);
    /// m.insert(3, 3);
    ///
    /// let inter = Treap::inter(n, m);
    /// assert_eq!(
    ///     inter.into_iter().collect::<Vec<(&u32, &u32)>>(),
    ///     vec![(&2, &2)],
    /// );
    /// ```
    pub fn inter(left: Self, right: Self) -> Self {
        let Treap { root: left_tree, rng, .. } = left;
        let Treap { root: right_tree, .. } = right;
        let (root, dups) = Self::tree_inter(left_tree, right_tree, false);
        Treap { root, rng, size: dups }
    }

    fn tree_subtract(left_tree: Tree<T, U>, right_tree: Tree<T, U>, mut swapped: bool) -> (Tree<T, U>, usize) {
        match (left_tree, right_tree) {
            (Some(mut left_node), Some(mut right_node)) => {
                let mut dups = 0;
                {
                    if left_node.priority < right_node.priority {
                        mem::swap(&mut left_node, &mut right_node);
                        swapped = !swapped;
                    }
                    let &mut Node {
                        left: ref mut left_subtree,
                        right: ref mut right_subtree,
                        ref mut key,
                        ..
                    } = &mut *left_node;
                    let mut right_left_subtree = Some(right_node);
                    let (duplicate_opt, right_right_subtree) = Self::split(&mut right_left_subtree, key);
                    let (new_left_subtree, left_dups) = Self::tree_subtract(left_subtree.take(), right_left_subtree, swapped);
                    let (new_right_subtree, right_dups) = Self::tree_subtract(right_subtree.take(), right_right_subtree, swapped);
                    dups += left_dups + right_dups;
                    *left_subtree = new_left_subtree;
                    *right_subtree = new_right_subtree;
                    if duplicate_opt.is_some() || swapped {
                        Self::merge(left_subtree, right_subtree.take());
                        return (left_subtree.take(), dups + 1);
                    }
                }
                (Some(left_node), dups)
            },
            (left_tree, right_tree) => {
                if swapped {
                    (right_tree, 0)
                } else {
                    (left_tree, 0)
                }
            },
        }
    }

    /// Returns `left` subtracted by `right`. The returned treap will contain all entries that do
    /// not have a key in `right`. The `-` operator is implemented to take the difference of two
    /// treaps.
    ///
    /// # Examples
    /// ```
    /// use data_structures::Treap;
    ///
    /// let mut n = Treap::new();
    /// n.insert(1, 1);
    /// n.insert(2, 2);
    ///
    /// let mut m = Treap::new();
    /// m.insert(2, 3);
    /// m.insert(3, 3);
    ///
    /// let subtract = Treap::subtract(n, m);
    /// assert_eq!(
    ///     subtract.into_iter().collect::<Vec<(&u32, &u32)>>(),
    ///     vec![(&1, &1)],
    /// );
    /// ```
    pub fn subtract(left: Self, right: Self) -> Self {
        let Treap { root: left_tree, rng, size } = left;
        let Treap { root: right_tree, .. } = right;
        let (root, dups) = Self::tree_subtract(left_tree, right_tree, false);
        Treap { root, rng, size: size - dups }
    }

    /// Returns an iterator over the treap. The iterator will yield key-value pairs using in-order
    /// traversal.
    ///
    /// # Examples
    /// ```
    /// use data_structures::Treap;
    ///
    /// let mut t = Treap::new();
    /// t.insert(1, 1);
    /// t.insert(3, 3);
    ///
    /// let mut iterator = t.iter();
    /// assert_eq!(iterator.next(), Some((&1, &1)));
    /// assert_eq!(iterator.next(), Some((&3, &3)));
    /// assert_eq!(iterator.next(), None);
    /// ```
    pub fn iter(&self) -> TreapIterator<T, U> {
        let &Treap { ref root, .. } = self;
        TreapIterator {
            current: root,
            stack: Vec::new(),
        }
    }
}

impl<'a, T: 'a + Ord, U: 'a> IntoIterator for &'a Treap<T, U> {
    type Item = (&'a T, &'a U);
    type IntoIter = TreapIterator<'a, T, U>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator for `Treap<T, U>`
///
/// This iterator traverses the elements of a treap in-order.
pub struct TreapIterator<'a, T: 'a + Ord, U: 'a> {
    current: &'a Tree<T, U>,
    stack: Vec<&'a Node<T, U>>,
}

impl<'a, T: 'a + Ord, U: 'a> Iterator for TreapIterator<'a, T, U> {
    type Item = (&'a T, &'a U);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(ref node) = *self.current {
            self.stack.push(node);
            self.current = &node.left;
        }
        self.stack.pop().map(|node| {
            let &Node {
                ref key,
                ref value,
                ref right,
                ..
            } = node;
            self.current = right;
            (key, value)
        })
    }
}

impl<T: Ord, U> Default for Treap<T, U> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord, U> Add for Treap<T, U> {
    type Output = Treap<T, U>;

    fn add(self, other: Treap<T, U>) -> Treap<T, U> {
        Treap::union(self, other)
    }
}

impl<T: Ord, U> Sub for Treap<T, U> {
    type Output = Treap<T, U>;

    fn sub(self, other: Treap<T, U>) -> Treap<T, U> {
        Treap::subtract(self, other)
    }
}

#[cfg(test)]
mod tests {
    use super::Treap;

    #[test]
    fn test_size_empty() {
        let tree: Treap<u32, u32> = Treap::new();
        assert_eq!(tree.size(), 0);
    }

    #[test]
    fn test_min_max_empty() {
        let tree: Treap<u32, u32> = Treap::new();
        assert_eq!(tree.min(), None);
        assert_eq!(tree.max(), None);
    }

    #[test]
    fn test_insert() {
        let mut tree = Treap::new();
        tree.insert(1, 1);
        assert!(tree.contains(&1));
        assert_eq!(tree.get(&1), Some(&1));
    }

    #[test]
    fn test_insert_replace() {
        let mut tree = Treap::new();
        let ret_1 = tree.insert(1, 1);
        let ret_2 = tree.insert(1, 3);
        assert_eq!(tree.get(&1), Some(&3));
        assert_eq!(ret_1, None);
        assert_eq!(ret_2, Some((1, 1)));
    }

    #[test]
    fn test_remove() {
        let mut tree = Treap::new();
        tree.insert(1, 1);
        let ret = tree.remove(&1);
        assert!(!tree.contains(&1));
        assert_eq!(ret, Some((1, 1)));
    }

    #[test]
    fn test_min_max() {
        let mut tree = Treap::new();
        tree.insert(1, 1);
        tree.insert(3, 3);
        tree.insert(5, 5);

        assert_eq!(tree.min(), Some(&1));
        assert_eq!(tree.max(), Some(&5));
    }

    #[test]
    fn test_get_mut() {
        let mut tree = Treap::new();
        tree.insert(1, 1);
        {
            let value = tree.get_mut(&1);
            *value.unwrap() = 3;
        }
        assert_eq!(tree.get(&1), Some(&3));
    }

    #[test]
    fn test_floor_ceil() {
        let mut tree = Treap::new();
        tree.insert(1, 1);
        tree.insert(3, 3);
        tree.insert(5, 5);

        assert_eq!(tree.floor(&0), None);
        assert_eq!(tree.floor(&2), Some(&1));
        assert_eq!(tree.floor(&4), Some(&3));
        assert_eq!(tree.floor(&6), Some(&5));

        assert_eq!(tree.ceil(&0), Some(&1));
        assert_eq!(tree.ceil(&2), Some(&3));
        assert_eq!(tree.ceil(&4), Some(&5));
        assert_eq!(tree.ceil(&6), None);
    }

    #[test]
    fn test_union() {
        let mut n = Treap::new();
        n.insert(1, 1);
        n.insert(2, 2);
        n.insert(3, 3);

        let mut m = Treap::new();
        m.insert(3, 5);
        m.insert(4, 4);
        m.insert(5, 5);

        let union = n + m;

        assert_eq!(
            union.into_iter().collect::<Vec<(&u32, &u32)>>(),
            vec![(&1, &1), (&2, &2), (&3, &3), (&4, &4), (&5, &5)],
        );
        assert_eq!(union.size(), 5);
    }

    #[test]
    fn test_inter() {
        let mut n = Treap::new();
        n.insert(1, 1);
        n.insert(2, 2);
        n.insert(3, 3);

        let mut m = Treap::new();
        m.insert(3, 5);
        m.insert(4, 4);
        m.insert(5, 5);

        let inter = Treap::inter(n, m);

        assert_eq!(
            inter.into_iter().collect::<Vec<(&u32, &u32)>>(),
            vec![(&3, &3)],
        );
        assert_eq!(inter.size(), 1);
    }

    #[test]
    fn test_subtract() {
        let mut n = Treap::new();
        n.insert(1, 1);
        n.insert(2, 2);
        n.insert(3, 3);

        let mut m = Treap::new();
        m.insert(3, 5);
        m.insert(4, 4);
        m.insert(5, 5);

        let sub = n - m;

        assert_eq!(
            sub.into_iter().collect::<Vec<(&u32, &u32)>>(),
            vec![(&1, &1), (&2, &2)],
        );
        assert_eq!(sub.size(), 2);
    }

    #[test]
    fn test_iter() {
        let mut tree = Treap::new();
        tree.insert(1, 2);
        tree.insert(5, 6);
        tree.insert(3, 4);

        assert_eq!(
            tree.into_iter().collect::<Vec<(&u32, &u32)>>(),
            vec![(&1, &2), (&3, &4), (&5, &6)]
        );
    }
}
