//! Executor abstractions and runtime adapters for `bycat`.
//!
//! This crate provides a small set of traits for spawning futures and blocking
//! work, plus concrete adapters for popular async runtimes. It is `no_std` and
//! only requires `alloc`.
//!
//! # Features
//!
//! - `tokio` (default): adapters for the Tokio runtime.
//! - `smol`: adapters for the `smol` runtime.
//! - `compio`: adapters for the `compio` runtime.
//! - `hyper`: implements `hyper::rt::Executor` for the enabled runtime adapters.

#![no_std]

extern crate alloc;

mod executors;
mod traits;
mod types;

pub use executors::*;
pub use traits::*;
pub use types::*;
