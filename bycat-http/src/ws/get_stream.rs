use alloc::task::Poll;
use tungstenite::{
    WebSocket,
    protocol::{Role, WebSocketConfig},
};

use crate::{
    serve::FuturesIo,
    ws::{compat::AllowStd, stream::WebSocketStream},
};

pub struct GetWebSocketStream<S> {
    role: Role,
    config: Option<WebSocketConfig>,
    stream: Option<S>,
}

impl<S> GetWebSocketStream<S> {
    pub fn new(stream: S, role: Role, config: Option<WebSocketConfig>) -> GetWebSocketStream<S> {
        GetWebSocketStream {
            role,
            config,
            stream: Some(stream),
        }
    }
}

impl<S: Unpin> Future for GetWebSocketStream<S> {
    type Output = WebSocketStream<FuturesIo<S>>;

    fn poll(
        self: alloc::pin::Pin<&mut Self>,
        cx: &mut alloc::task::Context<'_>,
    ) -> alloc::task::Poll<Self::Output> {
        let this = self.get_mut();

        let inner_stream = this.stream.take().expect("stream");
        let config = this.config.take();

        let stream = AllowStd::new(FuturesIo::new(inner_stream), cx.waker());

        let socket = WebSocket::from_raw_socket(stream, this.role, config);

        Poll::Ready(WebSocketStream::new(socket))
    }
}
