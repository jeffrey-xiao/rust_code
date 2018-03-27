use cuckoo::fingerprint_vec::FingerprintVec;
use rand::{Rng, XorShiftRng};
use siphasher::sip::SipHasher;
use std::cmp;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

const DEFAULT_FINGERPRINT_BIT_COUNT: usize = 8;
const DEFAULT_ENTRIES_PER_INDEX: usize = 4;
const DEFAULT_MAX_KICKS: usize = 512;

/// A space-efficient probabilistic data structure to test for membership in a set. Cuckoo filters
/// also provide the flexibility to remove items.
///
/// A cuckoo filter is based on cuckoo hashing and is essentially a cuckoo hash table storing
/// each keys' fingerprint. Cuckoo filters can be highly compact and serve as an improvement over
/// variations of tradition Bloom filters that support deletion (E.G. counting Bloom filters).
///
/// # Examples
/// ```
/// use data_structures::cuckoo::CuckooFilter;
///
/// let mut filter = CuckooFilter::new(100);
///
/// assert!(!filter.contains(&"foo"));
/// filter.insert(&"foo");
/// assert!(filter.contains(&"foo"));
///
/// filter.remove(&"foo");
/// assert!(!filter.contains(&"foo"));
///
/// assert_eq!(filter.len(), 4);
/// assert_eq!(filter.bucket_len(), 25);
/// assert_eq!(filter.fingerprint_bit_count(), 8);
/// ```
pub struct CuckooFilter<T: Hash> {
    max_kicks: usize,
    fingerprint_buckets: Vec<FingerprintVec>,
    extra_items: Vec<(u64, usize)>,
    hashers: [SipHasher; 2],
    _marker: PhantomData<T>,
}

impl<T: Hash> CuckooFilter<T> {
    fn get_hashers() -> [SipHasher; 2] {
        let mut rng = XorShiftRng::new_unseeded();
        [
            SipHasher::new_with_keys(rng.next_u64(), rng.next_u64()),
            SipHasher::new_with_keys(rng.next_u64(), rng.next_u64()),
        ]
    }

    /// Constructs a new, empty `CuckooFilter<T>` with an estimated max capacity of `item_count`.
    /// By defauly, the cuckoo filter will have 8 bits per item fingerprint, 4 entries per index,
    /// and a maximum of 512 item displacements before terminating the insertion process. The
    /// cuckoo filter will have an estimated maximum false positive probability of 3%.
    ///
    /// # Panics
    /// Panics if `item_count` is 0.
    ///
    /// # Examples
    /// ```
    /// use data_structures::cuckoo::CuckooFilter;
    ///
    /// let filter: CuckooFilter<u32> = CuckooFilter::new(100);
    /// ```
    pub fn new(item_count: usize) -> Self {
        assert!(item_count > 0);
        let bucket_len = (item_count + DEFAULT_ENTRIES_PER_INDEX - 1) / DEFAULT_ENTRIES_PER_INDEX;
        CuckooFilter {
            max_kicks: DEFAULT_MAX_KICKS,
            fingerprint_buckets: vec![FingerprintVec::new(
                DEFAULT_FINGERPRINT_BIT_COUNT,
                bucket_len,
            ); DEFAULT_ENTRIES_PER_INDEX],
            extra_items: Vec::new(),
            hashers: Self::get_hashers(),
            _marker: PhantomData,
        }
    }

    /// Constructs a new, empty `CuckooFilter<T>` with an estimated max capacity of `item_count`, a
    /// fingerprint bit count of `fingerprint_bit_count`, `entries_per_index` entries per index,
    /// and a maximum of 512 item displacements before terminating the insertion process. This
    /// method provides no guarantees on the false positive probability of the cuckoo filter.
    ///
    /// # Panics
    /// Panics if `item_count` is 0, if `fingerprint_bit_count` less than 1 or greater than 64, or
    /// if `entries_per_index` is 0.
    ///
    /// # Examples
    /// ```
    /// use data_structures::cuckoo::CuckooFilter;
    ///
    /// let filter: CuckooFilter<u32> = CuckooFilter::from_parameters(100, 16, 8);
    /// ```
    pub fn from_parameters(item_count: usize, fingerprint_bit_count: usize, entries_per_index: usize) -> Self {
        assert!(
            item_count > 0 &&
            fingerprint_bit_count > 1 &&
            fingerprint_bit_count <= 64 &&
            entries_per_index > 0
        );
        let bucket_len = (item_count + entries_per_index - 1) / entries_per_index;
        CuckooFilter {
            max_kicks: DEFAULT_MAX_KICKS,
            fingerprint_buckets: vec![FingerprintVec::new(
                fingerprint_bit_count,
                bucket_len,
            ); entries_per_index],
            extra_items: Vec::new(),
            hashers: Self::get_hashers(),
            _marker: PhantomData,
        }
    }

    /// Constructs a new, empty `CuckooFilter<T>` with an estimated max capacity of `item_count`,
    /// an estimated maximum false positive probability of `fpp`, a fingerprint bit count of
    /// `fingerprint_bit_count`, and a maximum of 512 item displacements before terminating the
    /// insertion process.
    ///
    /// # Panics
    /// Panics if `item_count` is 0 or if `entries_per_index` is 0.
    ///
    /// # Examples
    /// ```
    /// use data_structures::cuckoo::CuckooFilter;
    ///
    /// let filter: CuckooFilter<u32> = CuckooFilter::from_entries_per_index(100, 0.01, 4);
    /// ```
    pub fn from_entries_per_index(item_count: usize, fpp: f64, entries_per_index: usize) -> Self {
        assert!(item_count > 0 && entries_per_index > 0);
        let power = 2.0 / (1.0 - (1.0 - fpp).powf(1.0 / (2.0 * entries_per_index as f64)));
        let fingerprint_bit_count = power.log2().ceil() as usize;
        let bucket_len = (item_count + entries_per_index - 1) / entries_per_index;
        CuckooFilter {
            max_kicks: DEFAULT_MAX_KICKS,
            fingerprint_buckets: vec![FingerprintVec::new(
                fingerprint_bit_count,
                bucket_len,
            ); entries_per_index],
            extra_items: Vec::new(),
            hashers: Self::get_hashers(),
            _marker: PhantomData,
        }
    }

    /// Constructs a new, empty `CuckooFilter<T>` with an estimated max capacity of `item_count`,
    /// an estimated maximum false positive probability of `fpp`, `entries_per_index` entries per
    /// index, and a maximum of 512 item displacements before terminating the insertion process.
    ///
    /// # Panics
    /// Panics if `item_count` is 0, if `fingerprint_bit_count` is less than 1 or greater than 64,
    /// or if it is impossible to achieve the given maximum false positive probability.
    ///
    /// # Examples
    /// ```
    /// use data_structures::cuckoo::CuckooFilter;
    ///
    /// let filter: CuckooFilter<u32> = CuckooFilter::from_fingerprint_bit_count(100, 0.01, 10);
    /// ```
    pub fn from_fingerprint_bit_count(item_count: usize, fpp: f64, fingerprint_bit_count: usize) -> Self {
        assert!(item_count > 0 && fingerprint_bit_count > 1 && fingerprint_bit_count <= 64);
        let fingerprints_count = 2.0f64.powi(fingerprint_bit_count as i32);
        let single_fpp = (fingerprints_count - 2.0) / (fingerprints_count - 1.0);
        let entries_per_index = ((1.0 - fpp).log(single_fpp) / 2.0).floor() as usize;
        assert!(entries_per_index > 0);
        let bucket_len = (item_count + entries_per_index - 1) / entries_per_index;
        CuckooFilter {
            max_kicks: DEFAULT_MAX_KICKS,
            fingerprint_buckets: vec![FingerprintVec::new(
                fingerprint_bit_count,
                bucket_len,
            ); entries_per_index],
            extra_items: Vec::new(),
            hashers: Self::get_hashers(),
            _marker: PhantomData,
        }
    }

    fn get_hashes(&self, item: &T) -> [u64; 2] {
        let mut ret = [0; 2];
        for (index, hash) in ret.iter_mut().enumerate() {
            let mut sip = self.hashers[index];
            item.hash(&mut sip);
            *hash = sip.finish();
        }
        ret
    }

    fn get_fingerprint(raw_fingerprint: u64) -> Vec<u8> {
        (0..8).map(|index| ((raw_fingerprint >> (index * 8)) & (0xFF)) as u8).collect()
    }

    fn get_raw_fingerprint(fingerprint: &[u8]) -> u64 {
        let mut ret = 0u64;
        for (index, byte) in fingerprint.iter().enumerate() {
            ret |= (u64::from(*byte)) << (index * 8)
        }
        ret
    }

    fn get_fingerprint_and_indexes(&self, mut hashes: [u64; 2]) -> (Vec<u8>, usize, usize) {
        let trailing_zeros = 64 - self.fingerprint_bit_count();
        let mut raw_fingerprint = hashes[0] << trailing_zeros >> trailing_zeros;
        let mut fingerprint = Self::get_fingerprint(raw_fingerprint);

        // rehash when fingerprint is all 0s
        while raw_fingerprint == 0 {
            let mut sip = self.hashers[0];
            hashes[0].hash(&mut sip);
            hashes[0] = sip.finish();
            raw_fingerprint = hashes[0] << trailing_zeros >> trailing_zeros;
            fingerprint = Self::get_fingerprint(raw_fingerprint);
        }

        let index_1 = hashes[1] as usize % self.bucket_len();
        let index_2 = (index_1 ^ raw_fingerprint as usize) % self.bucket_len();
        (fingerprint, index_1, index_2)
    }

    /// Inserts an element into the bloom filter.
    ///
    /// # Examples
    /// ```
    /// use data_structures::cuckoo::CuckooFilter;
    ///
    /// let mut filter = CuckooFilter::new(100);
    /// filter.insert(&"foo");
    /// ```
    pub fn insert(&mut self, item: &T) {
        let (mut fingerprint, index_1, index_2) = self.get_fingerprint_and_indexes(self.get_hashes(item));
        if !self.contains_fingerprint(&fingerprint, index_1, index_2) {
            if self.insert_fingerprint(fingerprint.as_slice(), index_1) || self.insert_fingerprint(fingerprint.as_slice(), index_2) {
                return;
            }

            // have to kick out an entry
            let mut rng = XorShiftRng::new_unseeded();
            let mut index = if rng.gen::<bool>() { index_1 } else { index_2 };
            let mut prev_index = index;

            for _ in 0..self.max_kicks {
                let bucket_index = rng.gen_range(0, self.fingerprint_buckets.len());
                let new_fingerprint = self.fingerprint_buckets[bucket_index].get(index);
                self.fingerprint_buckets[bucket_index].set(index, fingerprint.as_slice());
                fingerprint = new_fingerprint;
                prev_index = index;
                index = (prev_index ^ Self::get_raw_fingerprint(&fingerprint) as usize) % self.bucket_len();
                if self.insert_fingerprint(fingerprint.as_slice(), index) {
                    return;
                }
            }

            self.extra_items.push((Self::get_raw_fingerprint(&fingerprint), cmp::min(prev_index, index)));
        }
    }

    fn insert_fingerprint(&mut self, fingerprint: &[u8], index: usize) -> bool {
        for bucket in &mut self.fingerprint_buckets {
            if bucket.get(index).iter().all(|byte| *byte == 0) {
                bucket.set(index, fingerprint);
                return true;
            }
        }
        false
    }

    /// Removes an element from the cuckoo filter.
    ///
    /// # Examples
    /// ```
    /// use data_structures::cuckoo::CuckooFilter;
    ///
    /// let mut filter = CuckooFilter::new(100);
    ///
    /// filter.insert(&"foo");
    /// assert!(filter.contains(&"foo"));
    ///
    /// filter.remove(&"foo");
    /// assert!(!filter.contains(&"foo"));
    /// ```
    pub fn remove(&mut self, item: &T) {
        let (fingerprint, index_1, index_2) = self.get_fingerprint_and_indexes(self.get_hashes(item));
        self.remove_fingerprint(&fingerprint, index_1, index_2)
    }

    fn remove_fingerprint(&mut self, fingerprint: &[u8], index_1: usize, index_2: usize) {
        let raw_fingerprint = Self::get_raw_fingerprint(fingerprint);
        let min_index = cmp::min(index_1, index_2);
        if let Some(index) = self.extra_items.iter().position(|item| *item == (raw_fingerprint, min_index)) {
            self.extra_items.swap_remove(index);
        }
        for bucket in &mut self.fingerprint_buckets {
            if Self::get_raw_fingerprint(&bucket.get(index_1)) == raw_fingerprint {
                bucket.set(index_1, Self::get_fingerprint(0).as_slice());
            }
            if Self::get_raw_fingerprint(&bucket.get(index_2)) == raw_fingerprint {
                bucket.set(index_2, Self::get_fingerprint(0).as_slice());
            }
        }
    }

    /// Checks if an element is possibly in the bloom filter.
    ///
    /// # Examples
    /// ```
    /// use data_structures::cuckoo::CuckooFilter;
    ///
    /// let mut filter = CuckooFilter::new(100);
    ///
    /// filter.insert(&"foo");
    /// assert!(filter.contains(&"foo"));
    /// ```
    pub fn contains(&self, item: &T) -> bool {
        let (fingerprint, index_1, index_2) = self.get_fingerprint_and_indexes(self.get_hashes(item));
        self.contains_fingerprint(&fingerprint, index_1, index_2)
    }

    fn contains_fingerprint(&self, fingerprint: &[u8], index_1: usize, index_2: usize) -> bool {
        let raw_fingerprint = Self::get_raw_fingerprint(fingerprint);
        let min_index = cmp::min(index_1, index_2);
        if self.extra_items.contains(&(raw_fingerprint, min_index)) {
            return true;
        }
        self.fingerprint_buckets.iter().any(|fingerprint_vec| {
            fingerprint_vec.get(index_1).iter().zip(fingerprint).all(|pair| pair.0 == pair.1) ||
            fingerprint_vec.get(index_2).iter().zip(fingerprint).all(|pair| pair.0 == pair.1)
        })
    }

    /// Clears the cuckoo filter, removing all elements.
    ///
    /// # Examples
    /// ```
    /// use data_structures::cuckoo::CuckooFilter;
    ///
    /// let mut filter = CuckooFilter::new(100);
    ///
    /// filter.insert(&"foo");
    /// filter.clear();
    ///
    /// assert!(!filter.contains(&"foo"));
    /// ```
    pub fn clear(&mut self) {
        let bucket_len = self.bucket_len();
        for buckets in &mut self.fingerprint_buckets {
            for index in 0..bucket_len {
                buckets.set(index, Self::get_fingerprint(0).as_slice());
            }
        }
    }

    /// Returns the number of entries per index in the cuckoo filter.
    ///
    /// # Examples
    /// ```
    /// use data_structures::cuckoo::CuckooFilter;
    ///
    /// let filter: CuckooFilter<u32> = CuckooFilter::new(100);
    ///
    /// assert_eq!(filter.len(), 4);
    /// ```
    pub fn len(&self) -> usize {
        self.fingerprint_buckets.len()
    }

    /// Returns `true` if the cuckoo filter is empty.
    ///
    /// # Examples
    /// ```
    /// use data_structures::cuckoo::CuckooFilter;
    ///
    /// let filter: CuckooFilter<u32> = CuckooFilter::new(100);
    ///
    /// assert!(!filter.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.fingerprint_buckets.is_empty()
    }

    /// Returns the len of each bucket in the cuckoo filter.
    ///
    /// # Examples
    /// ```
    /// use data_structures::cuckoo::CuckooFilter;
    ///
    /// let filter: CuckooFilter<u32> = CuckooFilter::new(100);
    ///
    /// assert_eq!(filter.bucket_len(), 25);
    /// ```
    pub fn bucket_len(&self) -> usize {
        match self.fingerprint_buckets.first() {
            Some(bucket) => bucket.len(),
            _ => unreachable!(),
        }
    }

    /// Returns the number of items that could not be inserted into the CuckooFilter.
    ///
    /// # Examples
    /// ```
    /// use data_structures::cuckoo::CuckooFilter;
    ///
    /// let mut filter = CuckooFilter::from_parameters(1, 8, 1);
    ///
    /// filter.insert(&"foo");
    /// filter.insert(&"foobar");
    /// assert_eq!(filter.extra_items_len(), 1);
    /// ```
    pub fn extra_items_len(&self) -> usize {
        self.extra_items.len()
    }

    /// Returns `true` if there are any items that could not be inserted into the cuckoo filter.
    ///
    /// # Examples
    /// ```
    /// use data_structures::cuckoo::CuckooFilter;
    ///
    /// let mut filter = CuckooFilter::from_parameters(1, 8, 1);
    ///
    /// filter.insert(&"foo");
    /// filter.insert(&"foobar");
    /// assert!(filter.is_nearly_full());
    /// ```
    pub fn is_nearly_full(&self) -> bool {
        !self.extra_items.is_empty()
    }

    /// Returns the number of bits in each item fingerprint.
    ///
    /// # Examples
    /// ```
    /// use data_structures::cuckoo::CuckooFilter;
    ///
    /// let filter: CuckooFilter<u32> = CuckooFilter::new(100);
    ///
    /// assert_eq!(filter.fingerprint_bit_count(), 8);
    /// ```
    pub fn fingerprint_bit_count(&self) -> usize {
        match self.fingerprint_buckets.first() {
            Some(bucket) => bucket.fingerprint_bit_count(),
            _ => unreachable!(),
        }
    }

    /// Returns the estimated false positive probability of the cuckoo filter. This value will
    /// increase as more items are added.
    ///
    /// # Examples
    /// ```
    /// use data_structures::cuckoo::CuckooFilter;
    ///
    /// let mut filter = CuckooFilter::new(100);
    /// assert!(filter.estimate_fpp() < 1e-6);
    ///
    /// filter.insert(&0);
    /// assert!((filter.estimate_fpp() - 0.000628487) < 1e-6);
    pub fn estimate_fpp(&self) -> f64 {
        let fingerprints_count = 2.0f64.powi(self.fingerprint_bit_count() as i32);
        let single_fpp = (fingerprints_count - 2.0) / (fingerprints_count - 1.0);
        let occupied_len: usize = self.fingerprint_buckets.iter().map(|bucket| bucket.occupied_len()).sum();
        let occupied_ratio = occupied_len as f64 / (self.len() * self.bucket_len()) as f64;
        return 1.0 - single_fpp.powf(2.0 * self.len() as f64 * occupied_ratio);
    }
}

#[cfg(test)]
mod tests {
    use super::CuckooFilter;

    #[test]
    fn test_get_fingerprint() {
        let fingerprint = CuckooFilter::<u32>::get_fingerprint(0x7FBFDFEFF7FBFDFE);
        assert_eq!(CuckooFilter::<u32>::get_raw_fingerprint(&fingerprint), 0x7FBFDFEFF7FBFDFE);
    }

    #[test]
    fn test_get_raw_fingerprint() {
        let fingerprint = vec![0xFF, 0xFF];
        assert_eq!(CuckooFilter::<u32>::get_raw_fingerprint(&fingerprint), 0xFFFF);
    }

    #[test]
    fn test_new() {
        let filter: CuckooFilter<u32> = CuckooFilter::new(100);
        assert_eq!(filter.len(), 4);
        assert!(!filter.is_empty());
        assert_eq!(filter.bucket_len(), 25);
        assert_eq!(filter.fingerprint_bit_count(), 8);
    }

    #[test]
    fn test_from_parameters() {
        let filter: CuckooFilter<u32> = CuckooFilter::from_parameters(100, 16, 8);
        assert_eq!(filter.len(), 8);
        assert!(!filter.is_empty());
        assert_eq!(filter.bucket_len(), 13);
        assert_eq!(filter.fingerprint_bit_count(), 16);
    }

    #[test]
    fn test_from_entries_per_index() {
        let filter: CuckooFilter<u32> = CuckooFilter::from_entries_per_index(100, 0.01, 4);
        assert_eq!(filter.len(), 4);
        assert!(!filter.is_empty());
        assert_eq!(filter.bucket_len(), 25);
        assert_eq!(filter.fingerprint_bit_count(), 11);
    }

    #[test]
    fn test_from_fingerprint_bit_count() {
        let filter: CuckooFilter<u32> = CuckooFilter::from_fingerprint_bit_count(100, 0.01, 10);
        assert_eq!(filter.len(), 5);
        assert!(!filter.is_empty());
        assert_eq!(filter.bucket_len(), 20);
        assert_eq!(filter.fingerprint_bit_count(), 10);
    }

    #[test]
    fn test_insert() {
        let mut filter = CuckooFilter::new(100);
        filter.insert(&"foo");
        assert!(filter.contains(&"foo"));
    }

    #[test]
    fn test_insert_existing_item() {
        let mut filter = CuckooFilter::new(100);
        filter.insert(&"foo");
        filter.insert(&"foo");
        assert!(filter.contains(&"foo"));
    }

    #[test]
    fn test_insert_extra_items() {
        let mut filter = CuckooFilter::from_parameters(1, 8, 1);

        filter.insert(&"foo");
        filter.insert(&"foobar");

        assert_eq!(filter.extra_items.len(), 1);
        assert!(filter.is_nearly_full());

        assert!(filter.contains(&"foo"));
        assert!(filter.contains(&"foobar"));
    }

    #[test]
    fn test_remove() {
        let mut filter = CuckooFilter::new(100);
        filter.insert(&"foo");
        filter.remove(&"foo");
        assert!(!filter.contains(&"foo"));
    }

    #[test]
    fn test_remove_extra_items() {
        let mut filter = CuckooFilter::from_parameters(1, 8, 1);

        filter.insert(&"foo");
        filter.insert(&"foobar");

        filter.remove(&"foo");
        filter.remove(&"foobar");

        assert!(!filter.contains(&"foo"));
        assert!(!filter.contains(&"foobar"));
    }
    #[test]
    fn test_remove_both_indexes() {
        let mut filter = CuckooFilter::from_parameters(2, 8, 1);

        filter.insert(&"baz");
        filter.insert(&"qux");
        filter.insert(&"foobar");
        filter.insert(&"barfoo");

        filter.remove(&"baz");
        filter.remove(&"qux");
        filter.remove(&"foobar");
        filter.remove(&"barfoo");

        assert!(!filter.contains(&"baz"));
        assert!(!filter.contains(&"qux"));
        assert!(!filter.contains(&"foobar"));
        assert!(!filter.contains(&"barfoo"));
    }

    #[test]
    fn test_estimate_fpp() {
        let mut filter = CuckooFilter::new(100);
        assert!(filter.estimate_fpp() < 1e-6);
       
        filter.insert(&0);
        println!("{}", filter.estimate_fpp());
        assert!((filter.estimate_fpp() - 0.000628487) < 1e-6);
    }
}
