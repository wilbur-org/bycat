use core::marker::PhantomData;

use crate::router::RouteError;
use alloc::{collections::BTreeMap, string::String, vec::Vec};
use bycat::{Middleware, Work};
use http::Request;
use routing::{Params, Segments, router::MethodFilter};

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
    context: PhantomData<fn() -> (C, B)>,
}

impl<T, C, B> Default for Router<T, C, B> {
    fn default() -> Self {
        Router {
            routes: routing::router::Router::new(),
            context: PhantomData,
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
        self.routes.match_route(path, method, params)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Segments<'_>, &routing::router::Route<Entry<T>>)> {
        self.routes.iter()
    }
}
