use core::fmt;

use alloc::{sync::Arc, vec::Vec};

use crate::value::Value;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct List<V = Value> {
    items: Arc<Vec<V>>,
}

impl<V> Clone for List<V> {
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
        }
    }
}

impl<V> Default for List<V> {
    fn default() -> Self {
        List {
            items: Default::default(),
        }
    }
}

impl<V: fmt::Display> fmt::Display for List<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;

        for (idx, value) in self.items.iter().enumerate() {
            if idx > 0 {
                write!(f, ", ")?;
            }

            write!(f, "{value}")?;
        }

        write!(f, "]")?;
        Ok(())
    }
}

impl<V> core::ops::Deref for List<V> {
    type Target = [V];
    fn deref(&self) -> &Self::Target {
        self.items.as_slice()
    }
}

impl<V: Clone> core::ops::DerefMut for List<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.items).as_mut_slice()
    }
}

impl<V: Clone> List<V> {
    pub fn with_capacity(capcity: usize) -> List<V> {
        List {
            items: Arc::new(Vec::with_capacity(capcity)),
        }
    }

    pub fn push(&mut self, value: impl Into<V>) {
        Arc::make_mut(&mut self.items).push(value.into())
    }

    pub fn pop(&mut self) -> Option<V> {
        Arc::make_mut(&mut self.items).pop()
    }
}

impl<V> From<Vec<V>> for List<V> {
    fn from(value: Vec<V>) -> Self {
        List {
            items: Arc::new(value),
        }
    }
}

impl<V: Clone> From<List<V>> for Vec<V> {
    fn from(value: List<V>) -> Self {
        Arc::try_unwrap(value.items).unwrap_or_else(|err| (*err).clone())
    }
}

impl<V: Clone> IntoIterator for List<V> {
    type Item = V;
    type IntoIter = alloc::vec::IntoIter<V>;
    fn into_iter(self) -> Self::IntoIter {
        let entries: Vec<V> = self.into();
        entries.into_iter()
    }
}

impl<V> FromIterator<V> for List<V> {
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        List {
            items: Arc::new(Vec::from_iter(iter)),
        }
    }
}

impl<V: Clone> Extend<V> for List<V> {
    fn extend<T: IntoIterator<Item = V>>(&mut self, iter: T) {
        Arc::make_mut(&mut self.items).extend(iter)
    }
}
