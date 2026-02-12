use alloc::{string::ToString, sync::Arc};
use core::{borrow::Borrow, fmt};

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct String(Arc<str>);

impl String {
    pub fn new(std: alloc::string::String) -> String {
        String(Arc::from(std))
    }

    pub fn as_str(&self) -> &str {
        &*self.0
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
        String(Arc::from(value.as_str()))
    }
}

impl From<Arc<str>> for String {
    fn from(value: Arc<str>) -> Self {
        String(value)
    }
}

impl From<String> for Arc<str> {
    fn from(value: String) -> Self {
        value.0
    }
}

impl From<String> for alloc::string::String {
    fn from(value: String) -> Self {
        value.to_string()
    }
}

impl From<alloc::string::String> for String {
    fn from(value: alloc::string::String) -> Self {
        String::new(value)
    }
}

impl PartialEq<str> for String {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<'a> PartialEq<&'a str> for String {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<alloc::string::String> for String {
    fn eq(&self, other: &alloc::string::String) -> bool {
        self.as_str() == other
    }
}
