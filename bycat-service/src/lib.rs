#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod and;
pub mod map;
pub mod map_err;
mod matcher;
mod middleware;
mod middleware_fn;
mod service;
mod service_ext;
mod service_fn;
pub mod split;
pub mod then;
#[cfg(feature = "tower")]
mod tower;
mod util;
pub mod when;
pub use self::{
    matcher::Matcher, middleware::*, middleware_fn::*, service::*, service_fn::*, util::*,
    when::when,
};

#[cfg(feature = "tower")]
pub use self::tower::{Tower, TowerFuture};

pub mod prelude {
    pub use super::service_ext::*;
}
