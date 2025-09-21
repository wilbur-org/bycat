use alloc::boxed::Box;
use async_trait::async_trait;
use bytes::{BufMut, Bytes, BytesMut};
use core::{
    convert::Infallible,
    task::{Poll, ready},
};
use futures::{TryStream, TryStreamExt};
use pin_project_lite::pin_project;

#[async_trait]
pub trait Content {
    type Error;
    async fn bytes(&mut self) -> Result<Bytes, Self::Error>;
}

#[async_trait]
impl Content for Bytes {
    type Error = Infallible;
    async fn bytes(&mut self) -> Result<Bytes, Self::Error> {
        Ok(self.clone())
    }
}

enum StreamContentState<T> {
    Stream(T),
    Bytes(Bytes),
}

pub struct StreamContent<T> {
    state: StreamContentState<T>,
}

impl<T> StreamContent<T> {
    pub fn new(stream: T) -> StreamContent<T> {
        StreamContent {
            state: StreamContentState::Stream(stream),
        }
    }
}

impl<T> From<Bytes> for StreamContent<T> {
    fn from(value: Bytes) -> Self {
        StreamContent {
            state: StreamContentState::Bytes(value),
        }
    }
}

#[async_trait]
impl<T> Content for StreamContent<T>
where
    T: TryStream<Ok = Bytes> + Send + Unpin,
{
    type Error = T::Error;
    async fn bytes(&mut self) -> Result<Bytes, Self::Error> {
        match &mut self.state {
            StreamContentState::Bytes(bs) => Ok(bs.clone()),
            StreamContentState::Stream(stream) => {
                let mut bytes = BytesMut::new();
                while let Some(next) = stream.try_next().await? {
                    bytes.put(next);
                }

                let bytes = bytes.freeze();

                self.state = StreamContentState::Bytes(bytes.clone());
                Ok(bytes)
            }
        }
    }
}

pin_project! {
pub struct CollectBytes<S> {
    #[pin]
    stream: S,
    buffer: Option<BytesMut>,
}

}

impl<S> CollectBytes<S> {
    pub fn new(stream: S) -> CollectBytes<S> {
        CollectBytes {
            stream,
            buffer: Some(BytesMut::new()),
        }
    }
}

impl<S> Future for CollectBytes<S>
where
    S: TryStream<Ok = Bytes>,
{
    type Output = Result<Bytes, S::Error>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        loop {
            let this = self.as_mut().project();

            let ret = match ready!(this.stream.try_poll_next(cx)) {
                Some(Ok(ret)) => ret,
                Some(Err(err)) => return Poll::Ready(Err(err)),
                None => return Poll::Ready(Ok(this.buffer.take().unwrap().freeze())),
            };

            this.buffer.as_mut().unwrap().put(ret);
        }
    }
}
