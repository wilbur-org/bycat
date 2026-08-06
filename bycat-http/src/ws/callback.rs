use crate::ws::websocket::WebSocket;

pub trait Callback {
    type Future: Future<Output = ()>;

    fn call(self, socket: WebSocket) -> Self::Future;
}

impl<T, U> Callback for T
where
    T: FnOnce(WebSocket) -> U,
    U: Future<Output = ()>,
{
    type Future = U;

    fn call(self, socket: WebSocket) -> Self::Future {
        (self)(socket)
    }
}
