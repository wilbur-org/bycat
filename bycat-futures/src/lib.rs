#![no_std]

mod convert;
pub mod futures;
mod result;
pub mod stream;

pub use self::result::*;

#[cfg(test)]
mod tests;
