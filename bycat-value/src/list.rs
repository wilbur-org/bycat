use core::fmt;

use alloc::{sync::Arc, vec::Vec};

use crate::value::Value;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct List {
    items: Arc<Vec<Value>>,
}

impl fmt::Display for List {
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

impl core::ops::Deref for List {
    type Target = [Value];
    fn deref(&self) -> &Self::Target {
        self.items.as_slice()
    }
}

impl core::ops::DerefMut for List {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.items).as_mut_slice()
    }
}

impl List {
    pub fn push(&mut self, value: impl Into<Value>) {
        Arc::make_mut(&mut self.items).push(value.into())
    }

    pub fn pop(&mut self) -> Option<Value> {
        Arc::make_mut(&mut self.items).pop()
    }
}

impl From<Vec<Value>> for List {
    fn from(value: Vec<Value>) -> Self {
        List {
            items: Arc::new(value),
        }
    }
}

impl From<List> for Vec<Value> {
    fn from(value: List) -> Self {
        Arc::try_unwrap(value.items).unwrap_or_else(|err| (*err).clone())
    }
}

impl IntoIterator for List {
    type Item = Value;
    type IntoIter = alloc::vec::IntoIter<Value>;
    fn into_iter(self) -> Self::IntoIter {
        let entries: Vec<Value> = self.into();
        entries.into_iter()
    }
}
