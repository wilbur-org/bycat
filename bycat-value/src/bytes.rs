use core::borrow::{Borrow, BorrowMut};

use alloc::{sync::Arc, vec::Vec};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bytes(Arc<Vec<u8>>);

impl Bytes {
    pub fn new(data: Vec<u8>) -> Self {
        Bytes(Arc::new(data))
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        Bytes(Arc::new(slice.to_vec()))
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        Arc::make_mut(&mut self.0).as_mut()
    }
}

impl core::ops::Deref for Bytes {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl core::ops::DerefMut for Bytes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl AsMut<[u8]> for Bytes {
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_slice_mut()
    }
}

impl Borrow<[u8]> for Bytes {
    fn borrow(&self) -> &[u8] {
        self.as_slice()
    }
}

impl BorrowMut<[u8]> for Bytes {
    fn borrow_mut(&mut self) -> &mut [u8] {
        self.as_slice_mut()
    }
}

impl From<Arc<Vec<u8>>> for Bytes {
    fn from(value: Arc<Vec<u8>>) -> Self {
        Bytes(value)
    }
}

impl From<Bytes> for Vec<u8> {
    fn from(value: Bytes) -> Self {
        Arc::try_unwrap(value.0).unwrap_or_else(|err| (*err).clone())
    }
}
