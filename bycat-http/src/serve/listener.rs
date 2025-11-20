use alloc::io;

/// Types that can listen for connections.
pub trait Listener {
    /// The listener's IO type.
    type Io: Socket + Unpin;

    /// The listener's address type.
    type Addr;

    /// Accept a new incoming connection to this listener.
    ///
    /// If the underlying accept call can return an error, this function must
    /// take care of logging and retrying.
    fn accept(&mut self) -> impl Future<Output = (Self::Io, Self::Addr)> + Send;

    /// Returns the local address that this listener is bound to.
    fn local_addr(&self) -> io::Result<Self::Addr>;
}

pub trait Socket: hyper::rt::Read + hyper::rt::Write {}

impl<T> Socket for T where T: hyper::rt::Read + hyper::rt::Write {}

#[cfg(feature = "serve-tokio")]
impl Listener for tokio::net::TcpListener {
    type Io = hyper_util::rt::TokioIo<tokio::net::TcpStream>;
    type Addr = alloc::net::SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            match Self::accept(self).await {
                Ok((socket, addr)) => return (hyper_util::rt::TokioIo::new(socket), addr),
                Err(e) => handle_accept_error(e).await,
            }
        }
    }

    #[inline]
    fn local_addr(&self) -> io::Result<Self::Addr> {
        Self::local_addr(self)
    }
}

#[cfg(all(feature = "serve-tokio", unix))]
impl Listener for tokio::net::UnixListener {
    type Io = hyper_util::rt::TokioIo<tokio::net::UnixStream>;
    type Addr = tokio::net::unix::SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            match Self::accept(self).await {
                Ok((socket, addr)) => return (hyper_util::rt::TokioIo::new(socket), addr),
                Err(e) => handle_accept_error(e).await,
            }
        }
    }

    #[inline]
    fn local_addr(&self) -> io::Result<Self::Addr> {
        Self::local_addr(self)
    }
}

#[cfg(feature = "serve-tokio")]
async fn handle_accept_error(e: io::Error) {
    if is_connection_error(&e) {
        return;
    }

    // [From `hyper::Server` in 0.14](https://github.com/hyperium/hyper/blob/v0.14.27/src/server/tcp.rs#L186)
    //
    // > A possible scenario is that the process has hit the max open files
    // > allowed, and so trying to accept a new connection will fail with
    // > `EMFILE`. In some cases, it's preferable to just wait for some time, if
    // > the application will likely close some files (or connections), and try
    // > to accept the connection again. If this option is `true`, the error
    // > will be logged at the `error` level, since it is still a big deal,
    // > and then the listener will sleep for 1 second.
    //
    tracing::error!("accept error: {e}");
    tokio::time::sleep(alloc::time::Duration::from_secs(1)).await;
}

#[cfg(feature = "serve-tokio")]
fn is_connection_error(e: &io::Error) -> bool {
    matches!(
        e.kind(),
        io::ErrorKind::ConnectionRefused
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::ConnectionReset
    )
}

#[cfg(feature = "serve-tokio")]
#[derive(Debug, Clone, Default)]
pub struct LocalTokioExecutor;

#[cfg(feature = "serve-tokio")]
impl<T> hyper::rt::Executor<T> for LocalTokioExecutor
where
    T: Future + 'static,
{
    fn execute(&self, fut: T) {
        tokio::task::spawn_local(fut);
    }
}

#[cfg(feature = "serve-tokio")]
pub use hyper_util::rt::TokioExecutor;

#[cfg(feature = "serve-smol")]
#[derive(Debug, Clone, Default)]
pub struct SmolExecutor;

#[cfg(feature = "serve-smol")]
impl<T> hyper::rt::Executor<T> for SmolExecutor
where
    T: Future + Send + 'static,
    T::Output: Send,
{
    fn execute(&self, fut: T) {
        smol::spawn(fut).detach();
    }
}

// #[cfg(feature = "serve-smol")]
// pub use hyper_util::rt::TokioExecutor;

#[cfg(feature = "serve-smol")]
impl Listener for smol::net::TcpListener {
    type Io = crate::futures::FuturesIo<smol::net::TcpStream>;

    type Addr = std::net::SocketAddr;

    fn accept(&mut self) -> impl Future<Output = (Self::Io, Self::Addr)> + Send {
        async move {
            loop {
                match Self::accept(self).await {
                    Ok((socket, addr)) => return (crate::futures::FuturesIo::new(socket), addr),
                    Err(e) => handle_smol_accept_error(e).await,
                }
            }
        }
    }

    fn local_addr(&self) -> io::Result<Self::Addr> {
        Self::local_addr(self)
    }
}

#[cfg(feature = "serve-smol")]
async fn handle_smol_accept_error(e: io::Error) {
    if is_connection_error(&e) {
        return;
    }

    // [From `hyper::Server` in 0.14](https://github.com/hyperium/hyper/blob/v0.14.27/src/server/tcp.rs#L186)
    //
    // > A possible scenario is that the process has hit the max open files
    // > allowed, and so trying to accept a new connection will fail with
    // > `EMFILE`. In some cases, it's preferable to just wait for some time, if
    // > the application will likely close some files (or connections), and try
    // > to accept the connection again. If this option is `true`, the error
    // > will be logged at the `error` level, since it is still a big deal,
    // > and then the listener will sleep for 1 second.
    //
    tracing::error!("accept error: {e}");
    smol::Timer::after(std::time::Duration::from_secs(1)).await;
}
