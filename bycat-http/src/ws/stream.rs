use super::handshake::without_handshake;
use crate::ws::compat::{AllowStd, ContextWaker, cvt};
use alloc::{
    pin::Pin,
    task::{Context, Poll, ready},
};
use futures::{
    AsyncRead, AsyncWrite,
    sink::{Sink, SinkExt},
    stream::{FusedStream, Stream},
};
use std::io::{Read, Write};
use tracing::{debug, trace};
use tungstenite::{
    error::Error as WsError,
    protocol::{CloseFrame, Message, Role, WebSocket, WebSocketConfig},
};

/// A wrapper around an underlying raw stream which implements the WebSocket
/// protocol.
///
/// A `WebSocketStream<S>` represents a handshake that has been completed
/// successfully and both the server and the client are ready for receiving
/// and sending data. Message from a `WebSocketStream<S>` are accessible
/// through the respective `Stream` and `Sink`. Check more information about
/// them in `futures-rs` crate documentation or have a look on the examples
/// and unit tests for this crate.
///
/// # Cancel safety
///
/// Reading messages is cancel-safe. `WebSocketStream` has no dedicated read
/// methods; messages arrive through its `Stream` implementation, and reading a
/// message via `StreamExt::next` follows that trait's cancel-safety: if the
/// `next()` future is dropped before it resolves (for example, as a branch of
/// `tokio::select!` that another branch completes first), no message is lost.
/// The next poll resumes from the same position in the stream.
///
/// The `Sink` side (sending) does not carry a documented cancel-safety
/// guarantee.
#[derive(Debug)]
pub struct WebSocketStream<S> {
    inner: WebSocket<AllowStd<S>>,
    closing: bool,
    ended: bool,
    /// Tungstenite is probably ready to receive more data.
    ///
    /// `false` once start_send hits `WouldBlock` errors.
    /// `true` initially and after `flush`ing.
    ready: bool,
}

impl<S> WebSocketStream<S> {
    /// Convert a raw socket into a WebSocketStream without performing a
    /// handshake.
    pub async fn from_raw_socket(stream: S, role: Role, config: Option<WebSocketConfig>) -> Self
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        without_handshake(stream, move |allow_std| {
            WebSocket::from_raw_socket(allow_std, role, config)
        })
        .await
    }

    /// Convert a raw socket into a WebSocketStream without performing a
    /// handshake.
    pub async fn from_partially_read(
        stream: S,
        part: Vec<u8>,
        role: Role,
        config: Option<WebSocketConfig>,
    ) -> Self
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        without_handshake(stream, move |allow_std| {
            WebSocket::from_partially_read(allow_std, part, role, config)
        })
        .await
    }

    pub(crate) fn new(ws: WebSocket<AllowStd<S>>) -> Self {
        Self {
            inner: ws,
            closing: false,
            ended: false,
            ready: true,
        }
    }

    fn with_context<F, R>(&mut self, ctx: Option<(ContextWaker, &mut Context<'_>)>, f: F) -> R
    where
        S: Unpin,
        F: FnOnce(&mut WebSocket<AllowStd<S>>) -> R,
        AllowStd<S>: Read + Write,
    {
        trace!("{}:{} WebSocketStream.with_context", file!(), line!());
        if let Some((kind, ctx)) = ctx {
            self.inner.get_mut().set_waker(kind, ctx.waker());
        }
        f(&mut self.inner)
    }

    /// Consumes the `WebSocketStream` and returns the underlying stream.
    pub fn into_inner(self) -> S {
        self.inner.into_inner().into_inner()
    }

    /// Returns a shared reference to the inner stream.
    pub fn get_ref(&self) -> &S
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        self.inner.get_ref().get_ref()
    }

    /// Returns a mutable reference to the inner stream.
    pub fn get_mut(&mut self) -> &mut S
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        self.inner.get_mut().get_mut()
    }

    /// Returns a reference to the configuration of the tungstenite stream.
    pub fn get_config(&self) -> &WebSocketConfig {
        self.inner.get_config()
    }

    /// Close the underlying web socket
    pub async fn close(&mut self, msg: Option<CloseFrame>) -> Result<(), WsError>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        self.send(Message::Close(msg)).await
    }
}

impl<T> Stream for WebSocketStream<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    type Item = Result<Message, WsError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        trace!("{}:{} Stream.poll_next", file!(), line!());

        // The connection has been closed or a critical error has occurred.
        // We have already returned the error to the user, the `Stream` is unusable,
        // so we assume that the stream has been "fused".
        if self.ended {
            return Poll::Ready(None);
        }

        match ready!(self.with_context(Some((ContextWaker::Read, cx)), |s| {
            trace!(
                "{}:{} Stream.with_context poll_next -> read()",
                file!(),
                line!()
            );
            cvt(s.read())
        })) {
            Ok(v) => Poll::Ready(Some(Ok(v))),
            Err(e) => {
                self.ended = true;
                if matches!(e, WsError::AlreadyClosed | WsError::ConnectionClosed) {
                    Poll::Ready(None)
                } else {
                    Poll::Ready(Some(Err(e)))
                }
            }
        }
    }
}

impl<T> FusedStream for WebSocketStream<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    fn is_terminated(&self) -> bool {
        self.ended
    }
}

impl<T> Sink<Message> for WebSocketStream<T>
where
    T: AsyncWrite + AsyncRead + Unpin,
{
    type Error = WsError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.ready {
            Poll::Ready(Ok(()))
        } else {
            // Currently blocked so try to flush the blockage away
            (*self)
                .with_context(Some((ContextWaker::Write, cx)), |s| cvt(s.flush()))
                .map(|r| {
                    self.ready = true;
                    r
                })
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        match (*self).with_context(None, |s| s.write(item)) {
            Ok(()) => {
                self.ready = true;
                Ok(())
            }
            Err(WsError::Io(err)) if err.kind() == std::io::ErrorKind::WouldBlock => {
                // the message was accepted and queued so not an error
                // but `poll_ready` will now start trying to flush the block
                self.ready = false;
                Ok(())
            }
            Err(e) => {
                self.ready = true;
                debug!("websocket start_send error: {}", e);
                Err(e)
            }
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        (*self)
            .with_context(Some((ContextWaker::Write, cx)), |s| cvt(s.flush()))
            .map(|r| {
                self.ready = true;
                match r {
                    // WebSocket connection has just been closed. Flushing completed, not an error.
                    Err(WsError::ConnectionClosed) => Ok(()),
                    other => other,
                }
            })
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.ready = true;
        let res = if self.closing {
            // After queueing it, we call `flush` to drive the close handshake to completion.
            (*self).with_context(Some((ContextWaker::Write, cx)), |s| s.flush())
        } else {
            (*self).with_context(Some((ContextWaker::Write, cx)), |s| s.close(None))
        };

        match res {
            Ok(()) => Poll::Ready(Ok(())),
            Err(WsError::ConnectionClosed) => Poll::Ready(Ok(())),
            Err(WsError::Io(err)) if err.kind() == std::io::ErrorKind::WouldBlock => {
                trace!("WouldBlock");
                self.closing = true;
                Poll::Pending
            }
            Err(err) => {
                debug!("websocket close error: {}", err);
                Poll::Ready(Err(err))
            }
        }
    }
}

// /// Get a domain from an URL.
// #[cfg(any(feature = "connect", feature = "native-tls", feature = "__rustls-tls"))]
// #[inline]
// fn domain(request: &tungstenite::handshake::client::Request) -> Result<String, WsError> {
//     match request.uri().host() {
//         // rustls expects IPv6 addresses without the surrounding [] brackets
//         #[cfg(feature = "__rustls-tls")]
//         Some(d) if d.starts_with('[') && d.ends_with(']') => Ok(d[1..d.len() - 1].to_string()),
//         Some(d) => Ok(d.to_string()),
//         None => Err(WsError::Url(tungstenite::error::UrlError::NoHostName)),
//     }
// }

// #[cfg(test)]
// mod tests {
//     #[cfg(feature = "connect")]
//     use crate::stream::MaybeTlsStream;
//     use crate::{compat::AllowStd, WebSocketStream};
//     use std::io::{Read, Write};
//     #[cfg(feature = "connect")]
//     use tokio::io::{AsyncReadExt, AsyncWriteExt};

//     fn is_read<T: Read>() {}
//     fn is_write<T: Write>() {}
//     #[cfg(feature = "connect")]
//     fn is_async_read<T: AsyncReadExt>() {}
//     #[cfg(feature = "connect")]
//     fn is_async_write<T: AsyncWriteExt>() {}
//     fn is_unpin<T: Unpin>() {}

//     #[test]
//     fn web_socket_stream_has_traits() {
//         is_read::<AllowStd<tokio::net::TcpStream>>();
//         is_write::<AllowStd<tokio::net::TcpStream>>();

//         #[cfg(feature = "connect")]
//         is_async_read::<MaybeTlsStream<tokio::net::TcpStream>>();
//         #[cfg(feature = "connect")]
//         is_async_write::<MaybeTlsStream<tokio::net::TcpStream>>();

//         is_unpin::<WebSocketStream<tokio::net::TcpStream>>();
//         #[cfg(feature = "connect")]
//         is_unpin::<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>>();
//     }
// }
