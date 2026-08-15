use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use bytes::Bytes;
use core::pin::Pin;
use core::task::Poll;
use http_body_util::combinators::BoxBody;
use hyper::body::Incoming;
use pin_project_lite::pin_project;

use crate::{Error, error::BoxError};

pub trait HttpBody: http_body::Body + Sized {
    fn empty() -> Self;

    fn from_bytes(bytes: Bytes) -> Self;

    // fn from_streaming<B>(inner: B) -> Self
    // where
    //     B: http_body::Body + Send + Sync + 'static,
    //     B::Error: core::error::Error + Send + Sync + 'static,
    //     B::Data: Into<Bytes>;
}

pub trait FromStreaming<S>: HttpBody {
    fn from_streaming(inner: S) -> Self;
}

pub fn to_bytes<T: http_body::Body>(body: T) -> ToBytes<T>
where
    T::Error: Into<Box<dyn core::error::Error + Send + Sync>>,
{
    use http_body_util::BodyExt;

    ToBytes {
        inner: BodyExt::collect(body),
    }
}

enum Inner {
    Reusable(Bytes),
    Streaming(BoxBody<Bytes, Error>),
    Incoming(Incoming),
}

pub struct Body {
    inner: Inner,
}

impl Body {
    pub fn empty() -> Body {
        Body {
            inner: Inner::Reusable(Bytes::new()),
        }
    }

    pub fn from_streaming<B>(inner: B) -> Body
    where
        B: http_body::Body + Send + Sync + 'static,
        B::Error: Into<Error>,
        B::Data: Into<Bytes>,
    {
        use http_body_util::BodyExt;

        let boxed = inner
            .map_frame(|f| f.map_data(Into::into))
            .map_err(Into::into)
            .boxed();

        Body {
            inner: Inner::Streaming(boxed),
        }
    }
}

impl<S> FromStreaming<S> for Body
where
    S: http_body::Body + Send + Sync + 'static,
    S::Error: Into<Error>,
    S::Data: Into<Bytes>,
{
    fn from_streaming(inner: S) -> Self {
        Body::from_streaming(inner)
    }
}

impl http_body::Body for Body {
    type Data = bytes::Bytes;

    type Error = Error;

    fn poll_frame(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        match self.inner {
            Inner::Reusable(ref mut bytes) => {
                let out = bytes.split_off(0);
                if out.is_empty() {
                    Poll::Ready(None)
                } else {
                    Poll::Ready(Some(Ok(http_body::Frame::data(out))))
                }
            }
            Inner::Incoming(ref mut incoming) => {
                let incoming = Pin::new(incoming);
                match incoming.poll_frame(cx) {
                    Poll::Ready(Some(Ok(data))) => Poll::Ready(Some(Ok(data))),
                    Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(Error::custom(err)))),
                    Poll::Ready(None) => Poll::Ready(None),
                    Poll::Pending => Poll::Pending,
                }
            }
            Inner::Streaming(ref mut body) => {
                Poll::Ready(core::task::ready!(Pin::new(body).poll_frame(cx)))
            }
        }
    }

    fn size_hint(&self) -> http_body::SizeHint {
        match self.inner {
            Inner::Reusable(ref bytes) => http_body::SizeHint::with_exact(bytes.len() as u64),
            Inner::Streaming(ref body) => body.size_hint(),
            Inner::Incoming(ref i) => i.size_hint(),
        }
    }

    fn is_end_stream(&self) -> bool {
        match self.inner {
            Inner::Reusable(ref bytes) => bytes.is_empty(),
            Inner::Streaming(ref body) => body.is_end_stream(),
            Inner::Incoming(ref i) => i.is_end_stream(),
        }
    }
}

impl HttpBody for Body {
    fn empty() -> Self {
        Body {
            inner: Inner::Reusable(Bytes::new()),
        }
    }

    fn from_bytes(bytes: Bytes) -> Self {
        Body {
            inner: Inner::Reusable(bytes),
        }
    }

    // fn from_streaming<B>(inner: B) -> Self
    // where
    //     B: http_body::Body + Send + Sync + 'static,
    //     B::Error: core::error::Error + Send + Sync + 'static,
    //     B::Data: Into<Bytes>,
    // {
    //     use http_body_util::BodyExt;

    //     let boxed = inner
    //         .map_frame(|f| f.map_data(Into::into))
    //         .map_err(Error::custom)
    //         .boxed();

    //     Body {
    //         inner: Inner::Streaming(boxed),
    //     }
    // }
}

impl<'a> From<&'a str> for Body {
    fn from(value: &'a str) -> Self {
        value.as_bytes().to_vec().into()
    }
}

impl From<String> for Body {
    fn from(value: String) -> Self {
        value.into_bytes().into()
    }
}

impl From<Vec<u8>> for Body {
    fn from(value: Vec<u8>) -> Self {
        Body {
            inner: Inner::Reusable(value.into()),
        }
    }
}

impl From<Bytes> for Body {
    fn from(value: Bytes) -> Self {
        Body {
            inner: Inner::Reusable(value),
        }
    }
}

impl From<Incoming> for Body {
    fn from(value: Incoming) -> Self {
        Body {
            inner: Inner::Incoming(value),
        }
    }
}

pin_project! {
    pub struct ToBytes<T>
    where
        T: http_body::Body,
    {
        #[pin]
        inner: http_body_util::combinators::Collect<T>,
    }
}

impl<T> Future for ToBytes<T>
where
    T: http_body::Body,
    T::Error: Into<BoxError>,
{
    type Output = Result<Bytes, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.inner.poll(cx) {
            Poll::Ready(Ok(buf)) => Poll::Ready(Ok(buf.to_bytes())),
            Poll::Ready(Err(err)) => Poll::Ready(Err(Error::custom(err))),
            Poll::Pending => Poll::Pending,
        }
    }
}
