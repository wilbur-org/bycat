use futures::{Sink, Stream};
use http::HeaderValue;
use hyper::upgrade::Upgraded;
use pin_project_lite::pin_project;
use tungstenite::{Error as WsError, Message};

use crate::{serve::FuturesIo, ws::stream::WebSocketStream};

pin_project! {
    pub struct WebSocket {
        #[pin]
        pub(crate) socket: WebSocketStream<FuturesIo<Upgraded>>,
        pub(crate) protocol: Option<HeaderValue>,
    }
}

impl Stream for WebSocket {
    type Item = Result<Message, WsError>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        this.socket.poll_next(cx)
    }
}

impl Sink<Message> for WebSocket {
    type Error = WsError;

    fn poll_ready(
        self: alloc::pin::Pin<&mut Self>,
        cx: &mut alloc::task::Context<'_>,
    ) -> alloc::task::Poll<Result<(), Self::Error>> {
        self.project().socket.poll_ready(cx)
    }

    fn start_send(self: alloc::pin::Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        self.project().socket.start_send(item)
    }

    fn poll_flush(
        self: alloc::pin::Pin<&mut Self>,
        cx: &mut alloc::task::Context<'_>,
    ) -> alloc::task::Poll<Result<(), Self::Error>> {
        self.project().socket.poll_flush(cx)
    }

    fn poll_close(
        self: alloc::pin::Pin<&mut Self>,
        cx: &mut alloc::task::Context<'_>,
    ) -> alloc::task::Poll<Result<(), Self::Error>> {
        self.project().socket.poll_close(cx)
    }
}
