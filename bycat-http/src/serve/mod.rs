// mod builder;
mod connection;
mod futures;
mod listener;
mod server;

use self::connection::Connection;
pub use self::{futures::FuturesIo, listener::*, server::*};

pub use bycat_server::Shutdown;
pub use hyper::rt::Executor;

#[cfg(feature = "serve-tokio")]
mod tokio_impl;
#[cfg(feature = "serve-tokio")]
pub use tokio_impl::Tokio;

#[cfg(feature = "serve-compio")]
mod compio_impl;
#[cfg(feature = "serve-compio")]
pub use compio_impl::Compio;
