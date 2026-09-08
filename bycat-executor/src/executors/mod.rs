//! Concrete executor adapters and boxed executor wrappers.

mod local;
mod send;

pub use local::LocalExecutor;
pub use send::SendExecutor;

#[cfg(feature = "tokio")]
mod tokio;

#[cfg(feature = "tokio")]
pub use tokio::{LocalTokioExecutor, TokioExecutor};

#[cfg(feature = "smol")]
mod smol;

#[cfg(feature = "smol")]
pub use smol::{SmolBlockingFuture, SmolExecutor};

#[cfg(feature = "compio")]
mod compio;

#[cfg(feature = "compio")]
pub use compio::CompioExecutor;
