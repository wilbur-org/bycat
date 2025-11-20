use bycat::{Middleware, Work};
use bycat_error::Error;
use core::task::{Poll, ready};
use http::{Request, Response};
use pin_project_lite::pin_project;

use crate::{IntoResponse, cookies::CookieJar};

#[derive(Debug, Default)]
pub struct Cookies;

impl<C, B, H> Middleware<C, Request<B>, H> for Cookies
where
    H: Work<C, Request<B>>,
    H::Error: Into<Error>,
    H::Output: IntoResponse<B>,
    <H::Output as IntoResponse<B>>::Error: Into<Error>,
{
    type Work = CookieWork<H>;

    fn wrap(&self, handler: H) -> Self::Work {
        CookieWork { handler }
    }
}

pub struct CookieWork<H> {
    handler: H,
}

impl<H, C, B> Work<C, Request<B>> for CookieWork<H>
where
    H: Work<C, Request<B>>,
    H::Error: Into<Error>,
    H::Output: IntoResponse<B>,
    <H::Output as IntoResponse<B>>::Error: Into<Error>,
{
    type Output = Response<B>;

    type Error = Error;

    type Future<'a>
        = CookieWorkFuture<'a, H, C, B>
    where
        Self: 'a,
        C: 'a;

    fn call<'a>(&'a self, context: &'a C, mut req: Request<B>) -> Self::Future<'a> {
        let cookie_jar = CookieJar::from_headers(req.headers());
        req.extensions_mut().insert(cookie_jar.clone());
        CookieWorkFuture {
            future: self.handler.call(context, req),
            cookie_jar,
        }
    }
}

pin_project! {
    pub struct CookieWorkFuture<'a, H: 'a, C: 'a, B>
    where
        H: Work<C, Request<B>>,
    {
        #[pin]
        future: H::Future<'a>,
        cookie_jar: CookieJar
    }
}

impl<'a, H: 'a, C: 'a, B> Future for CookieWorkFuture<'a, H, C, B>
where
    H: Work<C, Request<B>>,
    H::Error: Into<Error>,
    H::Output: IntoResponse<B>,
    <H::Output as IntoResponse<B>>::Error: Into<Error>,
{
    type Output = Result<Response<B>, Error>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let this = self.project();
        match ready!(this.future.poll(cx)) {
            Ok(ret) => match ret.into_response() {
                Ok(mut ret) => {
                    this.cookie_jar.apply(ret.headers_mut());
                    Poll::Ready(Ok(ret))
                }
                Err(err) => Poll::Ready(Err(err.into())),
            },
            Err(err) => Poll::Ready(Err(err.into())),
        }
    }
}
