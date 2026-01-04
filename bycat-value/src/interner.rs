use alloc::sync::Arc;
use core::hash::Hash;

use crate::String;

#[cfg(not(feature = "hash"))]
type Set<T> = alloc::collections::btree_set::BTreeSet<T>;

// #[cfg(feature = "hash")]
// type Set<T> = hashbrown::HashSet<T>;

// #[cfg(not(feature = "std"))]
pub type Lock<T> = spin::RwLock<T>;

// #[cfg(feature = "std")]
// pub type Lock<T> = parking_lot::Mutex<T>;

#[derive(Debug, Default, Clone)]
pub struct Interner {
    files: Arc<Lock<Set<String>>>,
}

impl Interner {
    pub fn get_or_intern<S>(&self, string: S) -> String
    where
        S: Into<String> + Hash + AsRef<str> + ?Sized,
    {
        if let Some(found) = self.files.read().get(string.as_ref()).cloned() {
            found
        } else {
            let atom = string.into();
            self.files.write().insert(atom.clone());
            atom
        }
    }

    pub fn clear(&self) {
        self.files.write().clear();
    }

    pub fn len(&self) -> usize {
        self.files.read().len()
    }

    pub fn total_bytes(&self) -> usize {
        self.files
            .read()
            .iter()
            .fold(0, |p, m| p + m.as_bytes().len())
    }
}
