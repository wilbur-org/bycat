use core::borrow::Borrow;

use alloc::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct String(Arc<alloc::string::String>);

impl String {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Borrow<str> for String {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}
