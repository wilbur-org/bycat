use core::{
    convert::Infallible,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use alloc::string::String;
use bycat::Work;
use bycat_error::{BoxError, Error};
use bycat_futures::IntoResult;
use http::{HeaderValue, Request, Response};
use pin_project_lite::pin_project;

pub trait IntoResponse<B> {
    type Error;
    fn into_response(self) -> Result<Response<B>, Self::Error>;
}

impl<B> IntoResponse<B> for Response<B> {
    type Error = Infallible;
    fn into_response(self) -> Result<Response<B>, Self::Error> {
        Ok(self)
    }
}

impl<T, E, B> IntoResponse<B> for Result<T, E>
where
    T: IntoResponse<B>,
    T::Error: core::error::Error + Send + Sync + 'static,
    E: IntoResponse<B>,
    E::Error: core::error::Error + Send + Sync + 'static,
{
    type Error = bycat_error::Error;
    fn into_response(self) -> Result<Response<B>, Self::Error> {
        match self {
            Self::Ok(ret) => ret.into_response().map_err(Error::new),
            Self::Err(err) => err.into_response().map_err(Error::new),
        }
    }
}

impl<'a, B> IntoResponse<B> for &'a str
where
    B: From<&'a str>,
{
    type Error = Infallible;
    fn into_response(self) -> Result<Response<B>, Self::Error> {
        let mut resp = Response::new(B::from(self));
        resp.headers_mut().insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain"),
        );
        Ok(resp)
    }
}

impl<B> IntoResponse<B> for String
where
    B: From<String>,
{
    type Error = Infallible;
    fn into_response(self) -> Result<Response<B>, Self::Error> {
        let mut resp = Response::new(B::from(self));
        resp.headers_mut().insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain"),
        );
        Ok(resp)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Html<T>(pub T);

impl<T, B> IntoResponse<B> for Html<T>
where
    B: From<T>,
{
    type Error = Infallible;
    fn into_response(self) -> Result<Response<B>, Self::Error> {
        let mut resp = Response::new(B::from(self.0));
        resp.headers_mut().insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("text/html"),
        );
        Ok(resp)
    }
}

pub trait WorkIntoResponseExt<C, B>: Work<C, Request<B>> {
    fn into_response(self) -> RouteHandler<Self>
    where
        Self: Sized,
        Self::Output: IntoResponse<B>,
        Self::Error: Into<Error>,
    {
        RouteHandler { work: self }
    }
}

impl<T, C, B> WorkIntoResponseExt<C, B> for T where T: Work<C, Request<B>> {}

#[derive(Debug, Clone, Copy)]
pub struct RouteHandler<T> {
    work: T,
}

impl<T, C, B> Work<C, Request<B>> for RouteHandler<T>
where
    T: Work<C, Request<B>>,
    T::Output: IntoResponse<B>,
    <T::Output as IntoResponse<B>>::Error: Into<BoxError>,
    T::Error: Into<Error>,
{
    type Error = Error;
    type Output = Response<B>;
    type Future<'a>
        = RouteHandlerFuture<T::Future<'a>, B>
    where
        Self: 'a,
        C: 'a;

    fn call<'a>(&'a self, context: &'a C, req: Request<B>) -> Self::Future<'a> {
        RouteHandlerFuture {
            future: self.work.call(context, req),
            body: PhantomData,
        }
    }
}

pin_project! {
    pub struct RouteHandlerFuture<T, B> {
        #[pin]
        future: T,
        body: PhantomData<B>
    }
}

impl<T, B> Future for RouteHandlerFuture<T, B>
where
    T: Future,
    T::Output: IntoResult,
    <T::Output as IntoResult>::Error: Into<Error>,
    <T::Output as IntoResult>::Output: IntoResponse<B>,
    <<T::Output as IntoResult>::Output as IntoResponse<B>>::Error: Into<BoxError>,
{
    type Output = Result<Response<B>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.future.poll(cx).map(|result| {
            result
                .into_result()
                .map_err(Into::into)
                .and_then(|resp| resp.into_response().map_err(Error::new))
        })
    }
}
