#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
pub mod fs;

mod virtual_fs;

pub use mime::{self, Mime};
#[cfg(feature = "std")]
pub use mime_guess;

pub use self::virtual_fs::VirtualFS;

#[cfg(feature = "std")]
pub use self::fs::{Body, FileResolver, Fs, ReadDir, WalkDir};
