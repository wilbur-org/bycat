use core::{borrow::Borrow, fmt};

use alloc::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct String(Arc<alloc::string::String>);

impl String {
    pub fn new(std: alloc::string::String) -> String {
        String(Arc::new(std))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Borrow<str> for String {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for String {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl core::ops::Deref for String {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl From<Arc<alloc::string::String>> for String {
    fn from(value: Arc<alloc::string::String>) -> Self {
        String(value)
    }
}

impl From<String> for Arc<alloc::string::String> {
    fn from(value: String) -> Self {
        value.0
    }
}

impl From<String> for alloc::string::String {
    fn from(value: String) -> Self {
        Arc::try_unwrap(value.0).unwrap_or_else(|err| (*err).clone())
    }
}

impl From<alloc::string::String> for String {
    fn from(value: alloc::string::String) -> Self {
        String::new(value)
    }
}
