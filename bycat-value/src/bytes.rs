use alloc::{sync::Arc, vec::Vec};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bytes(Arc<Vec<u8>>);
