#![no_std]

mod convert;
pub mod futures;
mod result;
pub mod stream;

pub use self::{convert::TupleFuture, result::*};

#[cfg(test)]
mod tests;
