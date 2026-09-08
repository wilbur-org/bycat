use core::pin::Pin;

use crate::{BlockingSpawner, Executor, HasBlockingSpawner, HasSpawner, Spawner};

/// Executor adapter that spawns tasks on the `smol` runtime.
///
/// Detached tasks are spawned with [`smol::spawn`] and run on the global
/// `smol` executor. Futures must be `Send` and `'static`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SmolExecutor;

impl<T> Executor<T> for SmolExecutor
where
    T: core::future::Future + Send + 'static,
    T::Output: Send + 'static,
{
    fn spawn(&self, work: T) {
        smol::spawn(work).detach();
    }
}

#[cfg(feature = "hyper")]
impl<T> hyper::rt::Executor<T> for SmolExecutor
where
    T: core::future::Future + Send + 'static,
    T::Output: Send + 'static,
{
    fn execute(&self, work: T) {
        smol::spawn(work).detach();
    }
}

impl Spawner<'static> for SmolExecutor {
    fn spawn<T>(&self, work: T)
    where
        T: core::future::Future<Output = ()> + Send + 'static,
    {
        smol::spawn(work).detach();
    }
}

impl BlockingSpawner for SmolExecutor {
    type Error = core::convert::Infallible;
    type Future<R> = SmolBlockingFuture<R>;
    fn spawn_blocking<T, R>(&self, work: T) -> Self::Future<R>
    where
        R: Send + 'static,
        T: FnOnce() -> R + Send + 'static,
    {
        SmolBlockingFuture(smol::unblock(work))
    }
}

/// Future returned by [`SmolExecutor::spawn_blocking`].
///
/// Wraps a [`smol::Task`] so that it resolves to `Result<R, Infallible>`,
/// matching the [`BlockingSpawner`] contract.
#[derive(Debug)]
pub struct SmolBlockingFuture<R>(smol::Task<R>);

impl<R> core::future::Future for SmolBlockingFuture<R> {
    type Output = Result<R, core::convert::Infallible>;
    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx).map(Ok)
    }
}

impl HasBlockingSpawner for SmolExecutor {
    type Spawner = Self;
    fn blocking_spawner(&self) -> &Self::Spawner {
        self
    }
}

impl HasSpawner<'static> for SmolExecutor {
    type Spawner = Self;
    fn spawner(&self) -> &Self::Spawner {
        self
    }
}
