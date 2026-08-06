use alloc::sync::Arc;
use bycat::{Middleware, Work};
use futures::future::BoxFuture;
use http::Request;
use routing::router::MethodFilter;

use crate::{
    Error, IntoResponse,
    body::HttpBody,
    router::{Builder, RouteError, Router, RouterFuture},
};

pub struct SendRouterBuilder<C, B> {
    builder: Builder<SendWork<C, B>, SendMiddleware<C, B>, C, B>,
}

impl<C: Send + Sync, B: Send + 'static> SendRouterBuilder<C, B> {
    pub fn new() -> Self {
        Self {
            builder: Builder::default(),
        }
    }

    pub fn get<T>(&mut self, path: &str, worker: T) -> Result<&mut Self, RouteError>
    where
        T: Work<C, Request<B>> + Send + Sync + 'static,
        T::Error: Into<Error>,
        for<'a> T::Future<'a>: Send + 'a,
        T::Output: IntoResponse<B>,
    {
        self.route(MethodFilter::GET, path, worker)
    }

    pub fn post<T>(&mut self, path: &str, worker: T) -> Result<&mut Self, RouteError>
    where
        T: Work<C, Request<B>> + Send + Sync + 'static,
        T::Error: Into<Error>,
        for<'a> T::Future<'a>: Send + 'a,
        T::Output: IntoResponse<B>,
    {
        self.route(MethodFilter::POST, path, worker)
    }

    pub fn put<T>(&mut self, path: &str, worker: T) -> Result<&mut Self, RouteError>
    where
        T: Work<C, Request<B>> + Send + Sync + 'static,
        T::Error: Into<Error>,
        for<'a> T::Future<'a>: Send + 'a,
        T::Output: IntoResponse<B>,
    {
        self.route(MethodFilter::PUT, path, worker)
    }

    pub fn patch<T>(&mut self, path: &str, worker: T) -> Result<&mut Self, RouteError>
    where
        T: Work<C, Request<B>> + Send + Sync + 'static,
        T::Error: Into<Error>,
        for<'a> T::Future<'a>: Send + 'a,
        T::Output: IntoResponse<B>,
    {
        self.route(MethodFilter::PATCH, path, worker)
    }

    pub fn delete<T>(&mut self, path: &str, worker: T) -> Result<&mut Self, RouteError>
    where
        T: Work<C, Request<B>> + Send + Sync + 'static,
        T::Error: Into<Error>,
        for<'a> T::Future<'a>: Send + 'a,
        T::Output: IntoResponse<B>,
    {
        self.route(MethodFilter::DELETE, path, worker)
    }

    pub fn route<T>(
        &mut self,
        method: MethodFilter,
        path: &str,
        worker: T,
    ) -> Result<&mut Self, RouteError>
    where
        T: Work<C, Request<B>> + Send + Sync + 'static,
        T::Error: Into<Error>,
        for<'a> T::Future<'a>: Send + 'a,
        T::Output: IntoResponse<B>,
    {
        let send_worker = SendWork::new(worker);
        self.builder.add_route(method, path, send_worker)?;
        Ok(self)
    }

    pub fn middleware<T>(&mut self, middleware: T) -> &mut Self
    where
        T: Middleware<C, Request<B>, SendWork<C, B>> + Send + Sync + 'static,
        T::Work: Work<C, Request<B>> + Send + Sync + 'static,
        <T::Work as Work<C, Request<B>>>::Error: Into<Error>,
        <T::Work as Work<C, Request<B>>>::Output: IntoResponse<B>,
        for<'a> <T::Work as Work<C, Request<B>>>::Future<'a>: Send + 'a,
    {
        let send_middleware = SendMiddleware::new(middleware);
        self.builder.middleware(send_middleware);
        self
    }

    pub fn mount<T>(&mut self, path: &str, router: T) -> Result<&mut Self, RouteError>
    where
        T: Into<Router<SendWork<C, B>, C, B>>,
    {
        self.builder.mount(path, router.into())?;
        Ok(self)
    }

    pub fn merge<T>(&mut self, router: T) -> Result<&mut Self, RouteError>
    where
        T: Into<Router<SendWork<C, B>, C, B>>,
    {
        self.builder.merge(router.into())?;
        Ok(self)
    }

    pub fn with_get<T>(mut self, path: &str, worker: T) -> Result<Self, RouteError>
    where
        T: Work<C, Request<B>> + Send + Sync + 'static,
        T::Error: Into<Error>,
        for<'a> T::Future<'a>: Send + 'a,
        T::Output: IntoResponse<B>,
    {
        self.get(path, worker)?;
        Ok(self)
    }

    pub fn with_post<T>(mut self, path: &str, worker: T) -> Result<Self, RouteError>
    where
        T: Work<C, Request<B>> + Send + Sync + 'static,
        T::Error: Into<Error>,
        for<'a> T::Future<'a>: Send + 'a,
        T::Output: IntoResponse<B>,
    {
        self.post(path, worker)?;
        Ok(self)
    }

    pub fn with_put<T>(mut self, path: &str, worker: T) -> Result<Self, RouteError>
    where
        T: Work<C, Request<B>> + Send + Sync + 'static,
        T::Error: Into<Error>,
        for<'a> T::Future<'a>: Send + 'a,
        T::Output: IntoResponse<B>,
    {
        self.put(path, worker)?;
        Ok(self)
    }

    pub fn with_patch<T>(mut self, path: &str, worker: T) -> Result<Self, RouteError>
    where
        T: Work<C, Request<B>> + Send + Sync + 'static,
        T::Error: Into<Error>,
        for<'a> T::Future<'a>: Send + 'a,
        T::Output: IntoResponse<B>,
    {
        self.patch(path, worker)?;
        Ok(self)
    }

    pub fn with_delete<T>(mut self, path: &str, worker: T) -> Result<Self, RouteError>
    where
        T: Work<C, Request<B>> + Send + Sync + 'static,
        T::Error: Into<Error>,
        for<'a> T::Future<'a>: Send + 'a,
        T::Output: IntoResponse<B>,
    {
        self.delete(path, worker)?;
        Ok(self)
    }

    pub fn with_middleware<T>(mut self, middleware: T) -> Self
    where
        T: Middleware<C, Request<B>, SendWork<C, B>> + Send + Sync + 'static,
        T::Work: Work<C, Request<B>> + Send + Sync + 'static,
        <T::Work as Work<C, Request<B>>>::Error: Into<Error>,
        <T::Work as Work<C, Request<B>>>::Output: IntoResponse<B>,
        for<'a> <T::Work as Work<C, Request<B>>>::Future<'a>: Send + 'a,
    {
        self.middleware(middleware);
        self
    }

    pub fn build(self) -> SendRouter<C, B> {
        SendRouter {
            router: Arc::new(self.builder.build()),
        }
    }
}

impl<C: Send + Sync, B: Send + 'static> From<SendRouterBuilder<C, B>> for SendRouter<C, B> {
    fn from(builder: SendRouterBuilder<C, B>) -> Self {
        builder.build()
    }
}

impl<C: Send + Sync, B: Send + 'static> From<SendRouterBuilder<C, B>>
    for Router<SendWork<C, B>, C, B>
{
    fn from(builder: SendRouterBuilder<C, B>) -> Self {
        builder.builder.build()
    }
}

impl<C: Send + Sync, B: Send + 'static> From<SendRouter<C, B>> for Router<SendWork<C, B>, C, B> {
    fn from(builder: SendRouter<C, B>) -> Self {
        builder.router.as_ref().clone()
    }
}

pub struct SendRouter<C, B> {
    router: Arc<Router<SendWork<C, B>, C, B>>,
}

impl<C, B> Clone for SendRouter<C, B> {
    fn clone(&self) -> Self {
        Self {
            router: self.router.clone(),
        }
    }
}

impl<C, B: HttpBody> Work<C, Request<B>> for SendRouter<C, B> {
    type Error = Error;
    type Output = http::Response<B>;
    type Future<'a>
        = RouterFuture<'a, SendWork<C, B>, C, B>
    where
        Self: 'a,
        C: 'a;

    fn call<'a>(&'a self, context: &'a C, req: Request<B>) -> Self::Future<'a> {
        self.router.call(context, req)
    }
}

pub struct SendWork<C, B> {
    inner: Arc<dyn Worker<C, B> + Send + Sync>,
}

impl<C, B> Clone for SendWork<C, B> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<C: Send + Sync, B: Send + 'static> SendWork<C, B> {
    pub fn new<T>(worker: T) -> Self
    where
        T: Work<C, Request<B>> + Send + Sync + 'static,
        T::Error: Into<Error>,
        for<'a> T::Future<'a>: Send + 'a,
        T::Output: IntoResponse<B>,
    {
        struct Wrapper<T>(T);

        impl<C: Send + Sync, B: Send + 'static, T> Worker<C, B> for Wrapper<T>
        where
            T: Work<C, Request<B>> + Send + Sync + 'static,
            T::Error: Into<Error>,
            for<'a> T::Future<'a>: Send + 'a,
            T::Output: IntoResponse<B>,
        {
            fn call<'a>(
                &'a self,
                context: &'a C,
                req: http::Request<B>,
            ) -> BoxFuture<'a, Result<http::Response<B>, Error>> {
                Box::pin(async move {
                    let fut = self.0.call(context, req);

                    let res = fut.await.map_err(Into::into)?;
                    Ok(res.into_response())
                })
            }
        }

        Self {
            inner: Arc::from(Wrapper(worker)) as Arc<dyn Worker<C, B> + Send + Sync>,
        }
    }
}

impl<C, B> Work<C, Request<B>> for SendWork<C, B> {
    type Error = Error;
    type Output = http::Response<B>;
    type Future<'a>
        = BoxFuture<'a, Result<Self::Output, Self::Error>>
    where
        Self: 'a,
        C: 'a;

    fn call<'a>(&'a self, context: &'a C, req: Request<B>) -> Self::Future<'a> {
        self.inner.call(context, req)
    }
}

trait Worker<C, B> {
    fn call<'a>(
        &'a self,
        context: &'a C,
        req: http::Request<B>,
    ) -> BoxFuture<'a, Result<http::Response<B>, Error>>;
}

trait Middler<C, B> {
    fn wrap(&self, future: SendWork<C, B>) -> SendWork<C, B>;
}

struct SendMiddleware<C, B> {
    inner: Arc<dyn Middler<C, B> + Send + Sync>,
}

impl<C: Send + Sync, B: Send + 'static> SendMiddleware<C, B> {
    pub fn new<T>(middleware: T) -> Self
    where
        T: Middleware<C, Request<B>, SendWork<C, B>> + Send + Sync + 'static,
        T::Work: Work<C, Request<B>> + Send + Sync + 'static,
        <T::Work as Work<C, Request<B>>>::Output: IntoResponse<B>,
        <T::Work as Work<C, Request<B>>>::Error: Into<Error>,
        for<'a> <T::Work as Work<C, Request<B>>>::Future<'a>: Send + 'a,
    {
        struct Wrapper<T>(T);

        impl<C: Send + Sync, B: Send + 'static, T> Middler<C, B> for Wrapper<T>
        where
            T: Middleware<C, Request<B>, SendWork<C, B>> + Send + Sync + 'static,
            T::Work: Work<C, Request<B>> + Send + Sync + 'static,
            <T::Work as Work<C, Request<B>>>::Error: Into<Error>,
            <T::Work as Work<C, Request<B>>>::Output: IntoResponse<B>,
            for<'a> <T::Work as Work<C, Request<B>>>::Future<'a>: Send + 'a,
        {
            fn wrap(&self, future: SendWork<C, B>) -> SendWork<C, B> {
                let task = self.0.wrap(future);
                SendWork::new(task)
            }
        }

        Self {
            inner: Arc::from(Wrapper(middleware)) as Arc<dyn Middler<C, B> + Send + Sync>,
        }
    }
}

impl<C, B> Middleware<C, Request<B>, SendWork<C, B>> for SendMiddleware<C, B> {
    type Work = SendWork<C, B>;
    fn wrap(&self, future: SendWork<C, B>) -> SendWork<C, B> {
        self.inner.wrap(future)
    }
}
