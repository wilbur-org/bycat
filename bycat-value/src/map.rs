use core::borrow::Borrow;

use alloc::{collections::btree_map::BTreeMap, sync::Arc};

use crate::{string::String, value::Value};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Map<K = String, V = Value> {
    entries: Arc<BTreeMap<K, V>>,
}

impl<K, V> Default for Map<K, V> {
    fn default() -> Self {
        Map {
            entries: Default::default(),
        }
    }
}

impl<K, V> Map<K, V> {
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: ?Sized,
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        self.entries.get(key)
    }
}

impl<K, V> Map<K, V>
where
    K: Clone + Ord,
    V: Clone,
{
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
}
