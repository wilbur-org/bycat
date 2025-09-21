use alloc::{sync::Arc, vec::Vec};

use crate::value::Value;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct List {
    items: Arc<Vec<Value>>,
}
