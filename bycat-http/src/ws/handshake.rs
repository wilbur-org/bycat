use alloc::{
    io::{Read, Write},
    pin::Pin,
    task::{Context, Poll},
};
use futures::{AsyncRead, AsyncWrite};
use tracing::trace;
use tungstenite::WebSocket;

use crate::ws::{compat::AllowStd, stream::WebSocketStream};

pub(crate) async fn without_handshake<F, S>(stream: S, f: F) -> WebSocketStream<S>
where
    F: FnOnce(AllowStd<S>) -> WebSocket<AllowStd<S>> + Unpin,
    S: AsyncRead + AsyncWrite + Unpin,
{
    let start = SkippedHandshakeFuture(Some(SkippedHandshakeFutureInner { f, stream }));

    let ws = start.await;

    WebSocketStream::new(ws)
}

struct SkippedHandshakeFuture<F, S>(Option<SkippedHandshakeFutureInner<F, S>>);
struct SkippedHandshakeFutureInner<F, S> {
    f: F,
    stream: S,
}

impl<F, S> Future for SkippedHandshakeFuture<F, S>
where
    F: FnOnce(AllowStd<S>) -> WebSocket<AllowStd<S>> + Unpin,
    S: Unpin,
    AllowStd<S>: Read + Write,
{
    type Output = WebSocket<AllowStd<S>>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let inner = self
            .get_mut()
            .0
            .take()
            .expect("future polled after completion");
        trace!("Setting context when skipping handshake");
        let stream = AllowStd::new(inner.stream, ctx.waker());

        Poll::Ready((inner.f)(stream))
    }
}
