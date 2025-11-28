use crate::{
    body::HttpBody,
    router::{RouteError, UrlParams},
};
use alloc::{collections::BTreeMap, string::String, vec::Vec};
use bycat::{Middleware, Work};
use core::{marker::PhantomData, task::Poll};
use http::{HeaderValue, Method, Request, Response, StatusCode, header::ALLOW};
use pin_project_lite::pin_project;
use routing::{Params, Segments, router::MethodFilter};
use std::fmt::Write as _;

#[derive(Debug, Clone)]
pub struct Entry<T> {
    pub handler: T,
    pub name: Option<String>,
}

pub struct Builder<T, M, C, B> {
    pub routes: routing::router::Router<Entry<T>>,
    pub middleware: Vec<M>,
    pub middleware_path: BTreeMap<String, Vec<M>>,
    context: PhantomData<fn() -> (C, B)>,
}

impl<T, M, C, B> Clone for Builder<T, M, C, B>
where
    T: Clone,
    M: Clone,
{
    fn clone(&self) -> Self {
        Self {
            routes: self.routes.clone(),
            middleware: self.middleware.clone(),
            middleware_path: self.middleware_path.clone(),
            context: PhantomData,
        }
    }
}

impl<T, M, C, B> Default for Builder<T, M, C, B> {
    fn default() -> Self {
        Builder {
            routes: routing::router::Router::new(),
            middleware: Vec::default(),
            middleware_path: Default::default(),
            context: PhantomData,
        }
    }
}

impl<T, M, C, B> Builder<T, M, C, B> {
    pub fn add_route(
        &mut self,
        method: MethodFilter,
        path: impl AsRef<str>,
        handler: T,
    ) -> Result<&mut Self, RouteError> {
        self.routes.route(
            method,
            path.as_ref(),
            Entry {
                handler,
                name: None,
            },
        )?;
        Ok(self)
    }

    pub fn middleware(&mut self, middleware: M) -> &mut Self {
        self.middleware.push(middleware);
        self
    }

    pub fn merge(&mut self, router: impl Into<Router<T, C, B>>) -> Result<&mut Self, RouteError> {
        self.routes.merge(router.into().routes)?;
        Ok(self)
    }

    pub fn mount(
        &mut self,
        path: &str,
        router: impl Into<Router<T, C, B>>,
    ) -> Result<&mut Self, RouteError> {
        self.routes.mount(path, router.into().routes)?;
        Ok(self)
    }
}

impl<T, M, C, B> Builder<T, M, C, B>
where
    T: Work<C, Request<B>>,
    M: Middleware<C, Request<B>, T, Work = T>,
{
    pub fn build(self) -> Router<T, C, B> {
        let routes = self.routes.map(|mut route, segments| {
            for m in self.middleware.iter().rev() {
                route.handler = m.wrap(route.handler);
            }

            let Some(segments) = segments else {
                return route;
            };

            for (p, ms) in self.middleware_path.iter() {
                if routing::match_path(&segments, &p, &mut ()) {
                    for m in ms.iter().rev() {
                        route.handler = m.wrap(route.handler);
                    }
                }
            }

            route
        });

        Router {
            routes,
            fallback: None,
            context: PhantomData,
        }
    }
}

impl<T, M, C, B> From<Builder<T, M, C, B>> for Router<T, C, B>
where
    T: Work<C, Request<B>>,
    M: Middleware<C, Request<B>, T, Work = T>,
{
    fn from(value: Builder<T, M, C, B>) -> Self {
        value.build()
    }
}

pub struct Router<T, C, B> {
    routes: routing::router::Router<Entry<T>>,
    fallback: Option<T>,
    context: PhantomData<fn() -> (C, B)>,
}

impl<T, C, B> Default for Router<T, C, B> {
    fn default() -> Self {
        Router {
            routes: routing::router::Router::new(),
            context: PhantomData,
            fallback: None,
        }
    }
}

impl<T, C, B> Router<T, C, B> {
    pub fn get_match<P: Params>(
        &self,
        method: MethodFilter,
        path: &str,
        params: &mut P,
    ) -> Option<&Entry<T>> {
        self.routes
            .match_route(path, method, params)
            .map(|(m, _)| m)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Segments<'_>, &routing::router::Route<Entry<T>>)> {
        self.routes.iter()
    }
}

impl<T, C, B> Work<C, Request<B>> for Router<T, C, B>
where
    T: Work<C, Request<B>, Output = Response<B>>,
    B: HttpBody,
{
    type Error = T::Error;
    type Output = Response<B>;

    type Future<'a>
        = RouterFuture<'a, T, C, B>
    where
        Self: 'a,
        C: 'a;

    fn call<'a>(&'a self, context: &'a C, req: Request<B>) -> Self::Future<'a> {
        RouterFuture {
            state: State::Init {
                context: Some(context),
                req: Some(req),
            },
            router: self,
        }
    }
}

pin_project! {
    #[project = StateProj]
    enum State<'a, T: 'a, C, B>
    where
        T: Work<C, Request<B>>
    {
        Init {
            context: Option<&'a C>,
            req: Option<Request<B>>
        },
        Future {
            #[pin]
            future: T::Future<'a>
        }
    }
}

pin_project! {
    pub struct RouterFuture<'a, T:'a, C, B>
    where
        T: Work<C, Request<B>>
    {
        router: &'a Router<T, C, B>,
        #[pin]
        state: State<'a, T, C, B>
    }
}

impl<'a, T, C, B> Future for RouterFuture<'a, T, C, B>
where
    T: Work<C, Request<B>, Output = Response<B>>,
    B: HttpBody,
{
    type Output = Result<Response<B>, T::Error>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            match this.state.as_mut().project() {
                StateProj::Init { context, req } => {
                    let mut req = req.take().unwrap();
                    let context = context.take().unwrap();
                    let mut params = UrlParams::default();
                    let found = if let Some(found) = this.router.get_match(
                        req.method().clone().into(),
                        req.uri().path(),
                        &mut params,
                    ) {
                        &found.handler
                    } else if req.method() == Method::OPTIONS {
                        let mut output = String::new();

                        for (idx, method) in this
                            .router
                            .routes
                            .match_routes(req.uri().path(), MethodFilter::all(), &mut ())
                            .map(|(_, method)| method)
                            .enumerate()
                        {
                            if idx > 0 {
                                output.push(',');
                            }

                            write!(&mut output, "{}", method).expect("Write");
                        }

                        let mut resp = Response::new(B::empty());
                        *resp.status_mut() = StatusCode::NO_CONTENT;
                        resp.headers_mut()
                            .insert(ALLOW, HeaderValue::from_str(&output).expect("HeaderValue"));
                        return Poll::Ready(Ok(resp));
                    } else if let Some(fallback) = &this.router.fallback {
                        fallback
                    } else {
                        let mut resp = Response::new(B::empty());
                        *resp.status_mut() = StatusCode::NOT_FOUND;
                        return Poll::Ready(Ok(resp));
                    };

                    req.extensions_mut().insert(params);
                    let future = found.call(context, req);
                    this.state.set(State::Future { future });
                }
                StateProj::Future { future } => return future.poll(cx),
            }
        }
    }
}
