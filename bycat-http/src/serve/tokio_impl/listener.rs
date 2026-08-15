use crate::serve::Listener;
use std::io;
impl Listener for tokio::net::TcpListener {
    type Io = hyper_util::rt::TokioIo<tokio::net::TcpStream>;
    type Addr = alloc::net::SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        Box::pin(async move {
            loop {
                match Self::accept(self).await {
                    Ok((socket, addr)) => return (hyper_util::rt::TokioIo::new(socket), addr),
                    Err(e) => handle_accept_error(e).await,
                }
            }
        })
        .await
    }

    #[inline]
    fn local_addr(&self) -> io::Result<Self::Addr> {
        Self::local_addr(self)
    }
}

#[cfg(unix)]
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

fn is_connection_error(e: &io::Error) -> bool {
    matches!(
        e.kind(),
        io::ErrorKind::ConnectionRefused
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::ConnectionReset
    )
}
