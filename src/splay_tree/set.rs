use crate::splay_tree::map::{SplayMap, SplayMapIntoIter, SplayMapIter};
use std::borrow::Borrow;

/// An ordered map implemented using splay tree.
///
/// An splay tree is a self-adjusting binary tree with an additional property that recently accessed
/// items are quick to access again. After each operation, the item that was accessed is "splayed"
/// to the root of the tree.
///
/// # Examples
///
/// ```
/// use extended_collections::splay_tree::SplaySet;
///
/// let mut set = SplaySet::new();
/// set.insert(0);
/// set.insert(3);
///
/// assert_eq!(set.len(), 2);
///
/// assert_eq!(set.min(), Some(&0));
/// assert_eq!(set.ceil(&2), Some(&3));
///
/// assert_eq!(set.remove(&0), Some(0));
/// assert_eq!(set.remove(&1), None);
/// ```
pub struct SplaySet<T> {
    map: SplayMap<T, ()>,
}

impl<T> SplaySet<T> {
    /// Constructs a new, empty `SplaySet<T>`
    ///
    /// # Examples
    ///
    /// ```
    /// use extended_collections::splay_tree::SplaySet;
    ///
    /// let set: SplaySet<u32> = SplaySet::new();
    /// ```
    pub fn new() -> Self {
        SplaySet {
            map: SplayMap::new(),
        }
    }

    /// Inserts a key into the set. If the key already exists in the set, it will return and
    /// replace the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use extended_collections::splay_tree::SplaySet;
    ///
    /// let mut set = SplaySet::new();
    /// assert_eq!(set.insert(1), None);
    /// assert!(set.contains(&1));
    /// assert_eq!(set.insert(1), Some(1));
    /// ```
    pub fn insert(&mut self, key: T) -> Option<T>
    where
        T: Ord,
    {
        self.map.insert(key, ()).map(|pair| pair.0)
    }

    /// Removes a key from the set. If the key exists in the set, it will return the associated
    /// key. Otherwise it will return `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use extended_collections::splay_tree::SplaySet;
    ///
    /// let mut set = SplaySet::new();
    /// set.insert(1);
    /// assert_eq!(set.remove(&1), Some(1));
    /// assert_eq!(set.remove(&1), None);
    /// ```
    pub fn remove(&mut self, key: &T) -> Option<T>
    where
        T: Ord,
    {
        self.map.remove(key).map(|pair| pair.0)
    }

    /// Checks if a key exists in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use extended_collections::splay_tree::SplaySet;
    ///
    /// let mut set = SplaySet::new();
    /// set.insert(1);
    /// assert!(!set.contains(&0));
    /// assert!(set.contains(&1));
    /// ```
    pub fn contains<V>(&self, key: &V) -> bool
    where
        T: Borrow<V>,
        V: Ord + ?Sized,
    {
        self.map.contains_key(key)
    }

    /// Returns the number of elements in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use extended_collections::splay_tree::SplaySet;
    ///
    /// let mut set = SplaySet::new();
    /// set.insert(1);
    /// assert_eq!(set.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the set is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use extended_collections::splay_tree::SplaySet;
    ///
    /// let set: SplaySet<u32> = SplaySet::new();
    /// assert!(set.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Clears the set, removing all values.
    ///
    /// # Examples
    ///
    /// ```
    /// use extended_collections::splay_tree::SplaySet;
    ///
    /// let mut set = SplaySet::new();
    /// set.insert(1);
    /// set.insert(2);
    /// set.clear();
    /// assert_eq!(set.is_empty(), true);
    /// ```
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Returns a key in the set that is less than or equal to a particular key. Returns `None` if
    /// such a key does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use extended_collections::splay_tree::SplaySet;
    ///
    /// let mut set = SplaySet::new();
    /// set.insert(1);
    /// assert_eq!(set.floor(&0), None);
    /// assert_eq!(set.floor(&2), Some(&1));
    /// ```
    pub fn floor<V>(&self, key: &V) -> Option<&T>
    where
        T: Borrow<V>,
        V: Ord + ?Sized,
    {
        self.map.floor(key)
    }

    /// Returns a key in the set that is greater than or equal to a particular key. Returns `None`
    /// if such a key does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use extended_collections::splay_tree::SplaySet;
    ///
    /// let mut set = SplaySet::new();
    /// set.insert(1);
    /// assert_eq!(set.ceil(&0), Some(&1));
    /// assert_eq!(set.ceil(&2), None);
    /// ```
    pub fn ceil<V>(&self, key: &V) -> Option<&T>
    where
        T: Borrow<V>,
        V: Ord + ?Sized,
    {
        self.map.ceil(key)
    }

    /// Returns the minimum key of the set. Returns `None` if the set is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use extended_collections::splay_tree::SplaySet;
    ///
    /// let mut set = SplaySet::new();
    /// set.insert(1);
    /// set.insert(3);
    /// assert_eq!(set.min(), Some(&1));
    /// ```
    pub fn min(&self) -> Option<&T>
    where
        T: Ord,
    {
        self.map.min()
    }

    /// Returns the maximum key of the set. Returns `None` if the set is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use extended_collections::splay_tree::SplaySet;
    ///
    /// let mut set = SplaySet::new();
    /// set.insert(1);
    /// set.insert(3);
    /// assert_eq!(set.max(), Some(&3));
    /// ```
    pub fn max(&self) -> Option<&T>
    where
        T: Ord,
    {
        self.map.max()
    }

    /// Returns an iterator over the set. The iterator will yield keys using in-order traversal.
    ///
    /// # Examples
    ///
    /// ```
    /// use extended_collections::splay_tree::SplaySet;
    ///
    /// let mut set = SplaySet::new();
    /// set.insert(1);
    /// set.insert(3);
    ///
    /// let mut iterator = set.iter();
    /// assert_eq!(iterator.next(), Some(&1));
    /// assert_eq!(iterator.next(), Some(&3));
    /// assert_eq!(iterator.next(), None);
    /// ```
    pub fn iter(&self) -> SplaySetIter<'_, T> {
        SplaySetIter {
            map_iter: self.map.iter(),
        }
    }
}

impl<T> IntoIterator for SplaySet<T> {
    type IntoIter = SplaySetIntoIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            map_iter: self.map.into_iter(),
        }
    }
}

impl<'a, T> IntoIterator for &'a SplaySet<T>
where
    T: 'a,
{
    type IntoIter = SplaySetIter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An owning iterator for `SplaySet<T>`.
///
/// This iterator traverses the elements of the set in-order and yields owned keys.
pub struct SplaySetIntoIter<T> {
    map_iter: SplayMapIntoIter<T, ()>,
}

impl<T> Iterator for SplaySetIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.map_iter.next().map(|pair| pair.0)
    }
}

/// An iterator for `SplaySet<T>`.
///
/// This iterator traverses the elements of the set in-order and yields immutable references.
pub struct SplaySetIter<'a, T> {
    map_iter: SplayMapIter<'a, T, ()>,
}

impl<'a, T> Iterator for SplaySetIter<'a, T>
where
    T: 'a,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.map_iter.next().map(|pair| pair.0)
    }
}

impl<T> Default for SplaySet<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::SplaySet;

    #[test]
    fn test_len_empty() {
        let set: SplaySet<u32> = SplaySet::new();
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn test_is_empty() {
        let set: SplaySet<u32> = SplaySet::new();
        assert!(set.is_empty());
    }

    #[test]
    fn test_min_max_empty() {
        let set: SplaySet<u32> = SplaySet::new();
        assert_eq!(set.min(), None);
        assert_eq!(set.max(), None);
    }

    #[test]
    fn test_insert() {
        let mut set = SplaySet::new();
        assert_eq!(set.insert(1), None);
        assert!(set.contains(&1));
    }

    #[test]
    fn test_insert_replace() {
        let mut set = SplaySet::new();
        assert_eq!(set.insert(1), None);
        assert_eq!(set.insert(1), Some(1));
    }

    #[test]
    fn test_remove() {
        let mut set = SplaySet::new();
        set.insert(1);
        assert_eq!(set.remove(&1), Some(1));
        assert!(!set.contains(&1));
    }

    #[test]
    fn test_min_max() {
        let mut set = SplaySet::new();
        set.insert(1);
        set.insert(3);
        set.insert(5);

        assert_eq!(set.min(), Some(&1));
        assert_eq!(set.max(), Some(&5));
    }

    #[test]
    fn test_floor_ceil() {
        let mut set = SplaySet::new();
        set.insert(1);
        set.insert(3);
        set.insert(5);

        assert_eq!(set.floor(&0), None);
        assert_eq!(set.floor(&2), Some(&1));
        assert_eq!(set.floor(&4), Some(&3));
        assert_eq!(set.floor(&6), Some(&5));

        assert_eq!(set.ceil(&0), Some(&1));
        assert_eq!(set.ceil(&2), Some(&3));
        assert_eq!(set.ceil(&4), Some(&5));
        assert_eq!(set.ceil(&6), None);
    }

    #[test]
    fn test_into_iter() {
        let mut set = SplaySet::new();
        set.insert(1);
        set.insert(5);
        set.insert(3);

        assert_eq!(set.into_iter().collect::<Vec<u32>>(), vec![1, 3, 5]);
    }

    #[test]
    fn test_iter() {
        let mut set = SplaySet::new();
        set.insert(1);
        set.insert(5);
        set.insert(3);

        assert_eq!(set.iter().collect::<Vec<&u32>>(), vec![&1, &3, &5]);
    }
}
