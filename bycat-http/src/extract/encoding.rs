use core::marker::PhantomData;
use core::task::{Poll, ready};

use bytes::Bytes;
use http::Response;
use pin_project_lite::pin_project;

use crate::body::{HttpBody, ToBytes, to_bytes};
use crate::error::BoxError;
use crate::{Error, FromRequest, IntoResponse};

pub trait Decoder<T> {
    type Error;
    type Output;
    fn decode(&self, data: &Bytes) -> Result<Self::Output, Self::Error>;
}

pub trait Encoder<T> {
    type Error;
    fn encode(&self, data: &T) -> Result<Bytes, Self::Error>;
}

pin_project! {
    pub struct DecodeFuture<B, T, D>
    where
        B: http_body::Body,
        D: Decoder<T>,
    {
        #[pin]
        inner: ToBytes<B>,
        decoder: D,
        ph: PhantomData<T>
    }
}

impl<B, T, D> Future for DecodeFuture<B, T, D>
where
    B: http_body::Body,
    B::Error: Into<BoxError>,
    D: Decoder<T>,
    D::Error: Into<BoxError>,
{
    type Output = Result<D::Output, Error>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let this = self.project();

        match ready!(this.inner.poll(cx)) {
            Ok(ret) => Poll::Ready(this.decoder.decode(&ret).map_err(Error::custom)),
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

macro_rules! encoding {
    ($mime: literal, $name: ident, $extract: ident, $error: ty, $from_bytes: expr, $to_bytes: expr) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name;

        impl<T> Encoder<T> for $name
        where
            T: serde::Serialize,
        {
            type Error = $error;

            fn encode(&self, data: &T) -> Result<Bytes, Self::Error> {
                let json = $to_bytes(data)?;
                Ok(Bytes::from(json))
            }
        }

        impl<T> Decoder<T> for $name
        where
            T: serde::de::DeserializeOwned,
        {
            type Error = $error;
            type Output = $extract<T>;

            fn decode(&self, data: &Bytes) -> Result<Self::Output, Self::Error> {
                Ok($extract($from_bytes(data)?))
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $extract<T>(pub T);

        impl<C, B, T> FromRequest<C, B> for $extract<T>
        where
            B: HttpBody,
            B::Error: Into<BoxError>,
            T: serde::de::DeserializeOwned,
        {
            type Future<'a>
                = DecodeFuture<B, T, $name>
            where
                C: 'a;

            fn from_request<'a>(parts: http::Request<B>, _state: &'a C) -> Self::Future<'a> {
                DecodeFuture {
                    inner: to_bytes(parts.into_body()),
                    decoder: $name,
                    ph: PhantomData,
                }
            }
        }

        impl<T, B> IntoResponse<B> for $extract<T>
        where
            T: serde::Serialize,
            B: From<Bytes>,
        {
            fn into_response(self) -> Response<B> {
                let bytes: Bytes = match $to_bytes(&self.0) {
                    Ok(bytes) => Bytes::from(bytes),
                    Err(err) => {
                        tracing::error!("Failed to encode {}: {}", stringify!($extract), err);
                        return Response::builder()
                            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                            .body(B::from(Bytes::from(err.to_string())))
                            .unwrap();
                    }
                };
                let resp = Response::builder()
                    .header(http::header::CONTENT_TYPE, $mime)
                    .header(http::header::CONTENT_LENGTH, bytes.len())
                    .body(B::from(bytes));

                resp.unwrap_or_else(|err| {
                    Response::builder()
                        .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .body(B::from(Bytes::from(err.to_string())))
                        .unwrap()
                })
            }
        }
    };
}

encoding!(
    "application/json",
    JsonEncoding,
    Json,
    serde_json::Error,
    serde_json::from_slice,
    serde_json::to_vec
);
