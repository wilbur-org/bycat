use core::{
    borrow::Borrow,
    fmt::{self, Write},
};

use alloc::{collections::btree_map::BTreeMap, sync::Arc};

use crate::{string::String, value::Value};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Map<K = String, V = Value> {
    pub(crate) entries: Arc<BTreeMap<K, V>>,
}

impl<K, V> Clone for Map<K, V> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
        }
    }
}

impl<K, V> Default for Map<K, V> {
    fn default() -> Self {
        Map {
            entries: Default::default(),
        }
    }
}

impl<K, V> Map<K, V> {
    pub fn with_capacity(_len: usize) -> Map<K, V> {
        Map::default()
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: ?Sized,
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        self.entries.get(key)
    }

    #[inline]
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        Q: ?Sized,
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        self.entries.contains_key(key)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> alloc::collections::btree_map::Iter<'_, K, V> {
        self.entries.iter()
    }
}

impl<K, V> Map<K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        Q: ?Sized,
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        Arc::make_mut(&mut self.entries).get_mut(key)
    }

    pub fn insert(&mut self, key: impl Into<K>, value: impl Into<V>) -> Option<V> {
        let entries = Arc::make_mut(&mut self.entries);
        entries.insert(key.into(), value.into())
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        Q: ?Sized,
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        let entries = Arc::make_mut(&mut self.entries);
        entries.remove(key)
    }

    #[inline]
    pub fn entry<S>(&mut self, key: S) -> alloc::collections::btree_map::Entry<'_, K, V>
    where
        S: Into<K>,
    {
        Arc::make_mut(&mut self.entries).entry(key.into())
    }

    #[inline]
    pub fn iter_mut(&mut self) -> alloc::collections::btree_map::IterMut<'_, K, V> {
        Arc::make_mut(&mut self.entries).iter_mut()
    }
}

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('{')?;

        for (idx, (k, v)) in self.entries.iter().enumerate() {
            if idx > 0 {
                writeln!(f, ", ")?;
            }
            write!(f, "{k}: {v}")?;
        }

        f.write_char('}')
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for Map<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Map {
            entries: Arc::new(BTreeMap::from_iter(iter)),
        }
    }
}

impl<K: Clone + Ord, V: Clone> Extend<(K, V)> for Map<K, V> {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        let this = Arc::make_mut(&mut self.entries);
        this.extend(iter)
    }
}

impl<K: Clone, V: Clone> IntoIterator for Map<K, V> {
    type Item = (K, V);
    type IntoIter = alloc::collections::btree_map::IntoIter<K, V>;
    fn into_iter(self) -> Self::IntoIter {
        let entries: BTreeMap<K, V> = self.into();
        entries.into_iter()
    }
}

impl<'a, K, V> IntoIterator for &'a Map<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = alloc::collections::btree_map::Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}

impl<K: Clone, V: Clone> From<Map<K, V>> for BTreeMap<K, V> {
    fn from(value: Map<K, V>) -> Self {
        Arc::try_unwrap(value.entries).unwrap_or_else(|err| (*err).clone())
    }
}
impl<K, V> From<BTreeMap<K, V>> for Map<K, V> {
    fn from(value: BTreeMap<K, V>) -> Self {
        Map {
            entries: Arc::new(value),
        }
    }
}

impl<K, V> From<Arc<BTreeMap<K, V>>> for Map<K, V> {
    fn from(value: Arc<BTreeMap<K, V>>) -> Self {
        Map { entries: value }
    }
}

impl<K, V> From<Map<K, V>> for Arc<BTreeMap<K, V>> {
    fn from(value: Map<K, V>) -> Self {
        value.entries
    }
}
