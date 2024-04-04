use allocative::Allocative;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::iter::Enumerate;

#[derive(Clone, Debug, Allocative)]
pub struct IndexedEntry<V> {
    v: V,
    index: usize,
}

impl<V> IndexedEntry<V> {
    pub fn new(v: V, index: usize) -> IndexedEntry<V> {
        IndexedEntry { v, index }
    }
}

/// a CompactOrderedHashMap is an enum that provides a similar API to the std::collections::HashMap
/// but has four additional implementations that specialize for the case where fewer than five
/// entries are required, in order to improve CPU performance for read-intensive usage. it also
/// tracks an "index" value for each entry in order to maintain sort order. sort order is idempotent
/// for a key with multiple inserts.
/// there could be extra work done to:
/// - match the std::collections::HashMap API
/// - reduce cloning on insert
#[derive(Clone, Debug, Allocative)]
pub enum CompactOrderedHashMap<K: Hash + Ord + PartialEq + Clone, V> {
    OneEntry {
        k1: K,
        v1: V,
    },
    TwoEntries {
        k1: K,
        k2: K,
        v1: V,
        v2: V,
    },
    ThreeEntries {
        k1: K,
        k2: K,
        k3: K,
        v1: V,
        v2: V,
        v3: V,
    },
    FourEntries {
        k1: K,
        k2: K,
        k3: K,
        k4: K,
        v1: V,
        v2: V,
        v3: V,
        v4: V,
    },
    NEntries(HashMap<K, IndexedEntry<V>>),
}

type KeyIterator<'a, K> = Box<dyn Iterator<Item = &'a K> + 'a>;
type ValueIterator<'a, K, V> = Box<dyn Iterator<Item = (&'a K, &'a V)> + 'a>;
type IndexedFeatureIterator<'a, K, V> = Enumerate<Box<dyn Iterator<Item = (&'a K, &'a V)> + 'a>>;

impl<K: Hash + Ord + PartialEq + Clone, V: Clone> CompactOrderedHashMap<K, V> {
    /// creates an empty CompactOrderedHashMap
    pub fn empty() -> CompactOrderedHashMap<K, V> {
        CompactOrderedHashMap::NEntries(HashMap::new())
    }

    /// creates a CompactOrderedHashMap from a vector of values.
    ///
    /// # Arguments
    /// * `entries` - the entry pairs to put in the map, assumed sorted
    ///
    /// # Returns
    /// A CompactOrderedHashMap
    pub fn new(entries: Vec<(K, V)>) -> CompactOrderedHashMap<K, V> {
        use CompactOrderedHashMap as S;

        match &entries[..] {
            [] => S::empty(),
            [(key, value)] => S::OneEntry {
                k1: key.clone(),
                v1: value.clone(),
            },
            [(k1, v1), (k2, v2)] if unique_key_len(&[k1, k2]) == 2 => S::TwoEntries {
                k1: k1.clone(),
                k2: k2.clone(),
                v1: v1.clone(),
                v2: v2.clone(),
            },
            [(k1, v1), (k2, v2), (k3, v3)] if unique_key_len(&[k1, k2, k3]) == 3 => {
                S::ThreeEntries {
                    k1: k1.clone(),
                    k2: k2.clone(),
                    k3: k3.clone(),
                    v1: v1.clone(),
                    v2: v2.clone(),
                    v3: v3.clone(),
                }
            }
            [(k1, v1), (k2, v2), (k3, v3), (k4, v4)] if unique_key_len(&[k1, k2, k3, k4]) == 4 => {
                S::FourEntries {
                    k1: k1.clone(),
                    k2: k2.clone(),
                    k3: k3.clone(),
                    k4: k4.clone(),
                    v1: v1.clone(),
                    v2: v2.clone(),
                    v3: v3.clone(),
                    v4: v4.clone(),
                }
            }
            _ => {
                let indexed = entries
                    .into_iter()
                    .enumerate()
                    .map(|(index, (k, v))| {
                        let indexed_entry = IndexedEntry { v, index };
                        (k, indexed_entry)
                    })
                    .collect::<HashMap<_, _>>();
                S::NEntries(indexed)
            }
        }
    }

    /// report the size of the collection
    pub fn len(&self) -> usize {
        match self {
            CompactOrderedHashMap::OneEntry { .. } => 1,
            CompactOrderedHashMap::TwoEntries { .. } => 2,
            CompactOrderedHashMap::ThreeEntries { .. } => 3,
            CompactOrderedHashMap::FourEntries { .. } => 4,
            CompactOrderedHashMap::NEntries(f) => f.len(),
        }
    }

    /// report if the collection has no entries
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// wrapper around get that only returns whether the key is contained
    pub fn contains_key(&self, k: &K) -> bool {
        self.get(k).is_some()
    }

    pub fn keys(&self) -> KeyIterator<K> {
        match self {
            CompactOrderedHashMap::OneEntry { k1, v1: _ } => Box::new([k1].into_iter()),
            CompactOrderedHashMap::TwoEntries {
                k1,
                k2,
                v1: _,
                v2: _,
            } => Box::new([k1, k2].into_iter()),
            CompactOrderedHashMap::ThreeEntries {
                k1,
                k2,
                k3,
                v1: _,
                v2: _,
                v3: _,
            } => Box::new([k1, k2, k3].into_iter()),
            CompactOrderedHashMap::FourEntries {
                k1,
                k2,
                k3,
                k4,
                v1: _,
                v2: _,
                v3: _,
                v4: _,
            } => Box::new([k1, k2, k3, k4].into_iter()),
            CompactOrderedHashMap::NEntries(map) => {
                let keys = map.iter().sorted_by_key(|(_, v)| v.index).map(|(k, _)| k);
                Box::new(keys)
            }
        }
    }

    /// gets a value associated with the given key. navigates the
    /// potential implementations of CompactOrderedHashMap, where in the
    /// case of specialized instances, an equality scan occurs in if/else blocks.
    /// for larger instances, we divert to the underlying HashMap::get method.
    pub fn get(&self, k: &K) -> Option<&V> {
        match self {
            CompactOrderedHashMap::OneEntry { k1, v1 } => {
                if k1 == k {
                    Some(v1)
                } else {
                    None
                }
            }
            CompactOrderedHashMap::TwoEntries { k1, k2, v1, v2 } => {
                if k1 == k {
                    Some(v1)
                } else if k2 == k {
                    Some(v2)
                } else {
                    None
                }
            }
            CompactOrderedHashMap::ThreeEntries {
                k1,
                k2,
                k3,
                v1,
                v2,
                v3,
            } => {
                if k1 == k {
                    Some(v1)
                } else if k2 == k {
                    Some(v2)
                } else if k3 == k {
                    Some(v3)
                } else {
                    None
                }
            }
            CompactOrderedHashMap::FourEntries {
                k1,
                k2,
                k3,
                k4,
                v1,
                v2,
                v3,
                v4,
            } => {
                if k1 == k {
                    Some(v1)
                } else if k2 == k {
                    Some(v2)
                } else if k3 == k {
                    Some(v3)
                } else if k4 == k {
                    Some(v4)
                } else {
                    None
                }
            }
            CompactOrderedHashMap::NEntries(map) => map.get(k).map(|e| &e.v),
        }
    }

    /// inserts a element in the collection at some key. navigates the
    /// potential implementations of CompactOrderedHashMap, where in the
    /// case of specialized instances, an equality scan occurs in if/else blocks to
    /// test if replace logic should be used. otherwise, one more is added, which in the
    /// specialized case, involves instantiating the next-larger implementation of CompactOrderedHashMap
    /// and swapping memory with the old implementation.
    ///
    /// # Arguments
    /// * `k` - the key to insert at
    /// * `v` - the value to insert at the given key
    ///
    /// # Returns
    /// the previous value stored at this key, if exists
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        let mut v_insert = v.clone();
        match self {
            CompactOrderedHashMap::NEntries(_) if self.is_empty() => {
                let mut one = CompactOrderedHashMap::OneEntry { k1: k, v1: v };
                std::mem::swap(self, &mut one);
                None
            }
            CompactOrderedHashMap::OneEntry { k1, v1 } => {
                if k1 == &k {
                    let out = v1.clone();
                    std::mem::swap(v1, &mut v_insert);
                    Some(out)
                } else {
                    let mut two = CompactOrderedHashMap::TwoEntries::<K, V> {
                        k1: k1.clone(),
                        k2: k,
                        v1: v1.clone(),
                        v2: v_insert,
                    };
                    std::mem::swap(self, &mut two);
                    None
                }
            }
            CompactOrderedHashMap::TwoEntries { k1, k2, v1, v2 } => {
                if k1 == &k {
                    let out = v1.clone();
                    std::mem::swap(v1, &mut v_insert);
                    Some(out)
                } else if k2 == &k {
                    let out = v2.clone();
                    std::mem::swap(v2, &mut v_insert);
                    Some(out)
                } else {
                    let mut three = CompactOrderedHashMap::ThreeEntries::<K, V> {
                        k1: k1.clone(),
                        k2: k2.clone(),
                        k3: k,
                        v1: v1.clone(),
                        v2: v2.clone(),
                        v3: v_insert,
                    };
                    std::mem::swap(self, &mut three);
                    None
                }
            }
            CompactOrderedHashMap::ThreeEntries {
                k1,
                k2,
                k3,
                v1,
                v2,
                v3,
            } => {
                if k1 == &k {
                    let out = v1.clone();
                    std::mem::swap(v1, &mut v_insert);
                    Some(out)
                } else if k2 == &k {
                    let out = v2.clone();
                    std::mem::swap(v2, &mut v_insert);
                    Some(out)
                } else if k3 == &k {
                    let out = v3.clone();
                    std::mem::swap(v3, &mut v_insert);
                    Some(out)
                } else {
                    let mut four = CompactOrderedHashMap::FourEntries::<K, V> {
                        k1: k1.clone(),
                        k2: k2.clone(),
                        k3: k3.clone(),
                        k4: k,
                        v1: v1.clone(),
                        v2: v2.clone(),
                        v3: v3.clone(),
                        v4: v_insert,
                    };
                    std::mem::swap(self, &mut four);
                    None
                }
            }
            CompactOrderedHashMap::FourEntries {
                k1,
                k2,
                k3,
                k4,
                v1,
                v2,
                v3,
                v4,
            } => {
                if k1 == &k {
                    let out = v1.clone();
                    std::mem::swap(v1, &mut v_insert);
                    Some(out)
                } else if k2 == &k {
                    let out = v2.clone();
                    std::mem::swap(v2, &mut v_insert);
                    Some(out)
                } else if k3 == &k {
                    let out = v3.clone();
                    std::mem::swap(v3, &mut v_insert);
                    Some(out)
                } else if k4 == &k {
                    let out = v4.clone();
                    std::mem::swap(v4, &mut v_insert);
                    Some(out)
                } else {
                    let five: HashMap<K, IndexedEntry<V>> = HashMap::from([
                        (k1.clone(), IndexedEntry::new(v1.clone(), 0)),
                        (k2.clone(), IndexedEntry::new(v2.clone(), 1)),
                        (k3.clone(), IndexedEntry::new(v3.clone(), 2)),
                        (k4.clone(), IndexedEntry::new(v4.clone(), 3)),
                        (k, IndexedEntry::new(v, 4)),
                    ]);

                    std::mem::swap(self, &mut CompactOrderedHashMap::NEntries(five));
                    None
                }
            }
            CompactOrderedHashMap::NEntries(map) => {
                let index = map.get(&k).map(|e| e.index).unwrap_or(map.len() + 1);
                let result = map.insert(k, IndexedEntry::new(v, index));
                result.map(|r| r.v)
            }
        }
    }

    /// gets the (key, value) pair at some index. this method exists since
    /// ordering is guaranteed by this collection, but the performance
    /// is consistently O(n) as it requires a scan for all implementations.
    ///
    /// # Arguments
    /// * `index` - collection index to retrieve
    ///
    /// # Returns
    /// the key/value pair at the given index, if it exists
    pub fn get_pair(&self, index: usize) -> Option<(&K, &V)> {
        match self {
            CompactOrderedHashMap::OneEntry { k1, v1 } => {
                if index == 0 {
                    Some((k1, v1))
                } else {
                    None
                }
            }
            CompactOrderedHashMap::TwoEntries { k1, k2, v1, v2 } => {
                if index == 0 {
                    Some((k1, v1))
                } else if index == 1 {
                    Some((k2, v2))
                } else {
                    None
                }
            }
            CompactOrderedHashMap::ThreeEntries {
                k1,
                k2,
                k3,
                v1,
                v2,
                v3,
            } => {
                if index == 0 {
                    Some((k1, v1))
                } else if index == 1 {
                    Some((k2, v2))
                } else if index == 2 {
                    Some((k3, v3))
                } else {
                    None
                }
            }
            CompactOrderedHashMap::FourEntries {
                k1,
                k2,
                k3,
                k4,
                v1,
                v2,
                v3,
                v4,
            } => {
                if index == 0 {
                    Some((k1, v1))
                } else if index == 1 {
                    Some((k2, v2))
                } else if index == 2 {
                    Some((k3, v3))
                } else if index == 3 {
                    Some((k4, v4))
                } else {
                    None
                }
            }
            CompactOrderedHashMap::NEntries(indexed) => {
                if index > indexed.len() {
                    None
                } else {
                    indexed
                        .iter()
                        .find(|(_, f)| f.index == index)
                        .map(|(k, entry)| (k, &entry.v))
                }
            }
        }
    }

    /// retrieve the index of the value stored at the given key.
    ///
    /// # Arguments
    /// * `k` - the key to retrieve an index for
    ///
    /// # Returns
    /// The index of the entry, if the entry is stored in the collection.
    pub fn get_index(&self, k: &K) -> Option<usize> {
        match self {
            CompactOrderedHashMap::OneEntry { k1, .. } => {
                if k == k1 {
                    Some(0)
                } else {
                    None
                }
            }
            CompactOrderedHashMap::TwoEntries { k1, k2, .. } => {
                if k == k1 {
                    Some(0)
                } else if k == k2 {
                    Some(1)
                } else {
                    None
                }
            }
            CompactOrderedHashMap::ThreeEntries { k1, k2, k3, .. } => {
                if k == k1 {
                    Some(0)
                } else if k == k2 {
                    Some(1)
                } else if k == k3 {
                    Some(2)
                } else {
                    None
                }
            }
            CompactOrderedHashMap::FourEntries { k1, k2, k3, k4, .. } => {
                if k == k1 {
                    Some(0)
                } else if k == k2 {
                    Some(1)
                } else if k == k3 {
                    Some(2)
                } else if k == k4 {
                    Some(3)
                } else {
                    None
                }
            }
            CompactOrderedHashMap::NEntries(indexed) => indexed.get(k).map(|f| f.index),
        }
    }

    /// collects the hash map tuples and clones them so they can
    /// be used to build other collections
    pub fn to_vec(&self) -> Vec<(K, IndexedEntry<V>)> {
        self.iter()
            .enumerate()
            .map(|(idx, (k, v))| {
                (
                    k.clone(),
                    IndexedEntry {
                        index: idx,
                        v: v.clone(),
                    },
                )
            })
            .collect_vec()
    }

    /// iterates over the entries in this collection in their index ordering.
    pub fn iter(&self) -> ValueIterator<K, V> {
        let iter = CompactOrderedHashMapIter {
            iterable: self,
            index: 0,
        };
        Box::new(iter)
    }

    /// iterator that includes the IndexedEntry wrapper around each value
    pub fn indexed_iter(&self) -> IndexedFeatureIterator<K, V> {
        self.iter().enumerate()
    }
}

pub struct CompactOrderedHashMapIter<'a, K: Hash + Ord + PartialEq + Clone, V: Clone> {
    iterable: &'a CompactOrderedHashMap<K, V>,
    index: usize,
}

impl<'a, K: Hash + Ord + PartialEq + Clone, V: Clone> Iterator
    for CompactOrderedHashMapIter<'a, K, V>
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.iterable.len() {
            return None;
        }
        if let Some(tuple) = self.iterable.get_pair(self.index) {
            self.index += 1;
            Some(tuple)
        } else {
            None
        }
    }
}

impl<K: Hash + Ord + PartialEq + Clone, V: Clone> IntoIterator for CompactOrderedHashMap<K, V> {
    type Item = (K, IndexedEntry<V>);

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            CompactOrderedHashMap::OneEntry { k1: key, v1: value } => {
                vec![(key, IndexedEntry::new(value, 0))].into_iter()
            }
            CompactOrderedHashMap::TwoEntries { k1, k2, v1, v2 } => vec![
                (k1, IndexedEntry::new(v1, 0)),
                (k2, IndexedEntry::new(v2, 1)),
            ]
            .into_iter(),
            CompactOrderedHashMap::ThreeEntries {
                k1,
                k2,
                k3,
                v1,
                v2,
                v3,
            } => vec![
                (k1, IndexedEntry::new(v1, 0)),
                (k2, IndexedEntry::new(v2, 1)),
                (k3, IndexedEntry::new(v3, 2)),
            ]
            .into_iter(),
            CompactOrderedHashMap::FourEntries {
                k1,
                k2,
                k3,
                k4,
                v1,
                v2,
                v3,
                v4,
            } => vec![
                (k1, IndexedEntry::new(v1, 0)),
                (k2, IndexedEntry::new(v2, 1)),
                (k3, IndexedEntry::new(v3, 2)),
                (k4, IndexedEntry::new(v4, 3)),
            ]
            .into_iter(),
            CompactOrderedHashMap::NEntries(f) => f.into_iter().sorted_by_key(|(_, f)| f.index),
        }
    }
}

impl<K: Hash + Ord + PartialEq + Clone, V: Clone> From<Vec<(K, V)>>
    for CompactOrderedHashMap<K, V>
{
    fn from(value: Vec<(K, V)>) -> Self {
        CompactOrderedHashMap::new(value)
    }
}

impl<K: Hash + Ord + PartialEq + Clone, V: Clone> FromIterator<(K, V)>
    for CompactOrderedHashMap<K, V>
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut map: CompactOrderedHashMap<K, V> = CompactOrderedHashMap::empty();
        for (k, v) in iter {
            let _ = map.insert(k, v);
        }
        map
    }
}

/// helper function that counts the number of unique entries in a slice
/// by testing for equality in a HashSet
fn unique_key_len<K: Hash + PartialEq + Eq>(entries: &[K]) -> usize {
    entries.iter().collect::<HashSet<_>>().len()
}

#[cfg(test)]
mod test {
    use super::CompactOrderedHashMap;

    #[derive(Clone, PartialEq, Eq, Debug)]
    struct TestValue {
        field: String,
    }

    #[test]
    fn test_inserts() {
        let mut map: CompactOrderedHashMap<String, TestValue> = CompactOrderedHashMap::empty();
        match &map {
            CompactOrderedHashMap::NEntries(empty) => {
                assert_eq!(empty.len(), 0)
            }
            _ => panic!("expected NEntries type before insert"),
        }
        let k1 = String::from("choo choo");
        let v1 = TestValue {
            field: String::from("chugga"),
        };
        let k2 = String::from("blah bloo");
        let v2 = TestValue {
            field: String::from("fooey"),
        };
        let k3 = String::from("skip");
        let v3 = TestValue {
            field: String::from("noggin"),
        };
        let k4 = String::from("chinook");
        let v4 = TestValue {
            field: String::from("fleas"),
        };
        let k5 = String::from("topographic");
        let v5 = TestValue {
            field: String::from("population"),
        };
        let insert_1 = map.insert(k1.clone(), v1.clone());
        match &map {
            CompactOrderedHashMap::OneEntry { k1: _, v1: _ } => {}
            _ => panic!("expected OneEntry type after insert"),
        }
        let insert_2 = map.insert(k2.clone(), v2.clone());
        match &map {
            CompactOrderedHashMap::TwoEntries {
                k1: _,
                k2: _,
                v1: _,
                v2: _,
            } => {}
            _ => panic!("expected TwoEntries type after insert"),
        }
        let insert_3 = map.insert(k3.clone(), v3.clone());
        match &map {
            CompactOrderedHashMap::ThreeEntries {
                k1: _,
                k2: _,
                k3: _,
                v1: _,
                v2: _,
                v3: _,
            } => {}
            _ => panic!("expected ThreeEntries type after insert"),
        }
        let insert_4 = map.insert(k4.clone(), v4.clone());
        match &map {
            CompactOrderedHashMap::FourEntries {
                k1: _,
                k2: _,
                k3: _,
                k4: _,
                v1: _,
                v2: _,
                v3: _,
                v4: _,
            } => {}
            _ => panic!("expected FourEntries type after insert"),
        }
        let insert_5 = map.insert(k5.clone(), v5.clone());
        match &map {
            CompactOrderedHashMap::NEntries(_) => {}
            _ => panic!("expected NEntries type after insert"),
        }
        let r1 = map.get(&k1);
        let i1 = map.get_index(&k1);
        let r2 = map.get(&k2);
        let i2 = map.get_index(&k2);
        let r3 = map.get(&k3);
        let i3 = map.get_index(&k3);
        let r4 = map.get(&k4);
        let i4 = map.get_index(&k4);
        let r5 = map.get(&k5);
        let i5 = map.get_index(&k5);

        // no keys were overwritten
        assert!(insert_1.is_none());
        assert!(insert_2.is_none());
        assert!(insert_3.is_none());
        assert!(insert_4.is_none());
        assert!(insert_5.is_none());
        // values and stored indices are as expected
        assert_eq!(Some(&v1), r1);
        assert_eq!(Some(0), i1);
        assert_eq!(Some(&v2), r2);
        assert_eq!(Some(1), i2);
        assert_eq!(Some(&v3), r3);
        assert_eq!(Some(2), i3);
        assert_eq!(Some(&v4), r4);
        assert_eq!(Some(3), i4);
        assert_eq!(Some(&v5), r5);
        assert_eq!(Some(4), i5);

        // test that ordering is correct
        let expected_values_sorted = vec![&v1, &v2, &v3, &v4, &v5];
        for ((_stored_k, stored_v), expected_v) in
            map.iter().zip(expected_values_sorted.into_iter())
        {
            assert_eq!(
                stored_v.field, expected_v.field,
                "stored values do not match, could be due to ordering logic"
            );
        }
    }

    #[test]
    fn test_replace_value_at_key() {
        let mut map: CompactOrderedHashMap<String, TestValue> = CompactOrderedHashMap::empty();
        let k1 = String::from("choo choo");
        let v1 = TestValue {
            field: String::from("chugga"),
        };
        let v2 = TestValue {
            field: String::from("fooey"),
        };
        let insert_1 = map.insert(k1.clone(), v1.clone());
        let insert_2 = map.insert(k1.clone(), v2.clone());
        let stored = map.get(&k1);
        assert_eq!(None, insert_1);
        assert_eq!(Some(&v1), insert_2.as_ref());
        assert_eq!(Some(&v2), stored);
    }
}
