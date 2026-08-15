use alloc::io;

use crate::serve::Listener;

impl Listener for compio::net::TcpListener {
    type Io = cyper_core::HyperStream<compio::net::TcpStream>;

    type Addr = core::net::SocketAddr;

    fn accept(&mut self) -> impl Future<Output = (Self::Io, Self::Addr)> + Send {
        // async move {
        //     let (socket, addr) = compio::net::TcpListener::accept(&self).await?;
        //     Ok((cyper_core::HyperStream::new_plain(socket), addr))
        // }
        async move {
            loop {
                match compio::net::TcpListener::accept(&self).await {
                    Ok((socket, addr)) => {
                        return (cyper_core::HyperStream::new_plain(socket), addr);
                    }
                    Err(e) => handle_accept_error(e).await,
                }
            }
        }
    }

    fn local_addr(&self) -> io::Result<Self::Addr> {
        compio::net::TcpListener::local_addr(&self)
    }
}

impl Listener for compio::net::UnixListener {
    type Io = cyper_core::HyperStream<compio::net::UnixStream>;

    type Addr = socket2::SockAddr;

    type Error = std::io::Error;

    fn accept(&mut self) -> impl Future<Output = Result<(Self::Io, Self::Addr), Self::Error>> {
        async move {
            let (socket, addr) = compio::net::UnixListener::accept(&self).await?;

            Ok((cyper_core::HyperStream::new_plain(socket), addr))
        }
    }

    fn local_addr(&self) -> Option<Self::Addr> {
        compio::net::UnixListener::local_addr(&self).ok()
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
    compio::time::sleep(alloc::time::Duration::from_secs(1)).await;
}

fn is_connection_error(e: &io::Error) -> bool {
    matches!(
        e.kind(),
        io::ErrorKind::ConnectionRefused
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::ConnectionReset
    )
}
