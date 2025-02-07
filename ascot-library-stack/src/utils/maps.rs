use core::hash::Hash;

use heapless::{FnvIndexMap, IndexMapIter};

use serde::{Deserialize, Serialize};

/// A map of elements for internal storage.
#[derive(Debug, Clone)]
pub struct Map<K: Eq + Hash, V, const N: usize>(FnvIndexMap<K, V, N>);

/// A serializable map of elements.
#[derive(Debug, Clone, Serialize)]
pub struct SerialMap<K: Eq + Hash, V, const N: usize>(FnvIndexMap<K, V, N>);

/// A serializable and deserializable map of elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputMap<K: Eq + Hash, V, const N: usize>(FnvIndexMap<K, V, N>);

macro_rules! from_map {
    ($for:ident) => {
        impl<K, V, K1, V1, const N: usize> From<Map<K1, V1, N>> for $for<K, V, N>
        where
            K: Clone + Copy + Eq + Hash + From<K1>,
            V: Clone + Copy + Eq + From<V1>,
            K1: Clone + Copy + Eq + Hash,
            V1: Clone + Copy + Eq,
        {
            fn from(map: Map<K1, V1, N>) -> Self {
                let mut new_map = Self::new();
                for (key, value) in map.iter() {
                    let _ = new_map
                        .0
                        .insert(K::from(key.clone()), V::from(value.clone()));
                }
                new_map
            }
        }
    };
}

macro_rules! map_implementation {
    ($impl:ident) => {
        impl<'a, K, V, const N: usize> IntoIterator for &'a $impl<K, V, N>
        where
            K: Clone + Copy + Eq + Hash,
            V: Clone + Copy
        {
            type Item = (&'a K, &'a V);
            type IntoIter = IndexMapIter<'a, K, V>;

            fn into_iter(self) -> Self::IntoIter {
                self.iter()
            }
        }

        impl<K, V, const N: usize> Default for $impl<K, V, N>
        where
            K: Clone + Copy + Eq + Hash,
            V: Clone + Copy
        {
            fn default() -> Self {
                Self::new()
            }
        }

        impl<K, V, const N: usize> $impl<K, V, N>
        where
            K: Clone + Copy + Eq + Hash,
            V: Clone + Copy
        {
            #[doc = concat!("Creates a [`", stringify!($impl), "`].")]
            #[must_use]
            #[inline]
            pub fn new() -> Self {
                Self(FnvIndexMap::new())
            }

            #[doc = concat!("Initializes a [`", stringify!($impl), "`] with a determined element.")]
            #[must_use]
            #[inline]
            pub fn init(key: K, value: V) -> Self {
                Self::new().insert(key, value)
            }

            #[doc = concat!("Inserts an element to a [`", stringify!($impl), "`].")]
            #[must_use]
            #[inline]
            pub fn insert(mut self, key: K, value: V) -> Self {
                let _ = self.0.insert(key, value);
                self
            }

            #[doc = concat!("Adds an element to a [`", stringify!($impl), "`].")]
            #[inline]
            pub fn add(&mut self, key: K, value: V) {
                let _ = self.0.insert(key, value);
            }

            #[doc = concat!("Checks whether the [`", stringify!($impl), "`] is empty.")]
            #[must_use]
            #[inline]
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            #[doc = concat!("Returns the [`", stringify!($impl), "`] length.")]
            #[must_use]
            #[inline]
            pub fn len(&self) -> usize {
                self.0.len()
            }

            #[doc = concat!("Checks whether the [`", stringify!($impl), "`] contains the given key.")]
            #[inline]
            pub fn contains_key(&self, key: &K) -> bool {
                self.0.contains_key(key)
            }

            #[doc = concat!("Returns an iterator over the [`", stringify!($impl), "`].")]
            #[doc = ""]
            #[doc = "**It iterates in the insertion order.**"]
            #[must_use]
            #[inline]
            pub fn iter(&self) -> IndexMapIter<'_, K, V> {
                self.0.iter()
            }

            #[doc = concat!("Initializes [`", stringify!($impl), "`] with a list of `(key, value)`.")]
            #[inline]
            pub fn init_with_elements(input_elements: &[(K, V)]) -> Self {
                let mut elements = Self::new();
                for (key, value) in input_elements.iter() {
                    elements.add(*key, *value);
                }
                elements
            }
        }
    };
}

// Map implementation.
map_implementation!(Map);

// Serial map implementation.
map_implementation!(SerialMap);

// Output map implementation.
map_implementation!(OutputMap);

// Convert from map into serial map.
from_map!(SerialMap);
// Convert from map into output map.
from_map!(OutputMap);
