mod session;
mod store;

use crate::{
    cookies::CookieJar,
    session::{
        session::State,
        store::{DynStoreImpl, SessionStore},
    },
};
use alloc::{borrow::Cow, println, string::ToString, sync::Arc};
use bycat::{Middleware, Work};
use bycat_error::{BoxError, Error};
use cookie::Cookie;
use core::{
    marker::PhantomData,
    task::{Poll, ready},
};
use http::Request;
use pin_project_lite::pin_project;
use uuid::Uuid;

pub use self::{
    session::{Session, SessionId},
    store::{MemoryStore, Store},
};

#[derive(Clone)]
pub struct Sessions {
    store: SessionStore,
    cookie_name: Cow<'static, str>,
    cookie_key: cookie::Key,
}

impl Sessions {
    pub fn new<S>(store: S) -> Sessions
    where
        S: Store + Send + Sync + 'static,
    {
        Sessions {
            store: Arc::new(DynStoreImpl(store)),
            cookie_name: Cow::Borrowed("sess_id"),
            cookie_key: cookie::Key::generate(),
        }
    }

    pub fn cookie_name<T>(mut self, cookie: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        self.cookie_name = cookie.into();
        self
    }
}

impl<C, B, T> Middleware<C, Request<B>, T> for Sessions
where
    T: Work<C, Request<B>>,
    T::Error: Into<BoxError>,
{
    type Work = SessionsWork<C, B, T>;

    fn wrap(&self, handler: T) -> Self::Work {
        SessionsWork {
            work: handler,
            store: self.store.clone(),
            cookie_name: self.cookie_name.clone(),
            cookie_key: self.cookie_key.clone(),
            req: PhantomData,
        }
    }
}

pub struct SessionsWork<C, B, T> {
    work: T,
    store: SessionStore,
    cookie_name: Cow<'static, str>,
    cookie_key: cookie::Key,
    req: PhantomData<(C, B)>,
}

impl<C, B, T: Clone> Clone for SessionsWork<C, B, T> {
    fn clone(&self) -> Self {
        Self {
            work: self.work.clone(),
            store: self.store.clone(),
            cookie_name: self.cookie_name.clone(),
            cookie_key: self.cookie_key.clone(),
            req: self.req.clone(),
        }
    }
}

impl<C, B, T> Work<C, Request<B>> for SessionsWork<C, B, T>
where
    T: Work<C, Request<B>>,
    T::Error: Into<BoxError>,
{
    type Output = T::Output;

    type Error = Error;

    type Future<'a>
        = SessionWorkFuture<'a, C, B, T>
    where
        Self: 'a,
        C: 'a;

    fn call<'a>(&'a self, context: &'a C, req: Request<B>) -> Self::Future<'a> {
        SessionWorkFuture {
            state: SessionWorkFutureState::Init {
                req: Some(req),
                context,
                work: &self.work,
            },
            cookie_name: &self.cookie_name,
            cookie_key: &self.cookie_key,
            store: &self.store,
        }
    }
}

pin_project! {
    #[project = SessionFutureStateProj]
    enum SessionWorkFutureState<'a, C, B, T: 'a>
    where
        T: Work<C, Request<B>>
    {
        Init {
            req: Option<Request<B>>,
            context: &'a C,
            work: &'a T
        },
        Future {
            #[pin]
            future: T::Future<'a>,
            cookies: CookieJar,
            id: SessionId
        }
    }
}

pin_project! {
    pub struct SessionWorkFuture<'a, C, B, T>
    where
        T: Work<C, Request<B>>
    {
        #[pin]
        state: SessionWorkFutureState<'a, C, B, T>,
        cookie_name: &'a Cow<'static,str>,
        cookie_key: &'a cookie::Key,
        store: &'a SessionStore,

    }
}

impl<'a, C, B, T> Future for SessionWorkFuture<'a, C, B, T>
where
    T: Work<C, Request<B>>,
    T::Error: Into<BoxError>,
{
    type Output = Result<T::Output, Error>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            match this.state.as_mut().project() {
                SessionFutureStateProj::Init { req, context, work } => {
                    let mut req = req.take().unwrap();

                    let cookies = CookieJar::from_request(&req)?;
                    let id = if let Some(id) = req.extensions().get::<SessionId>() {
                        id.clone()
                    } else {
                        let id = if let Some(id) =
                            cookies.signed(&this.cookie_key).get(&this.cookie_name)
                        {
                            let id =
                                Uuid::parse_str(id.value()).map_err(bycat_error::Error::new)?;
                            SessionId::new(id)
                        } else {
                            SessionId::default()
                        };

                        req.extensions_mut().insert(id.clone());

                        id
                    };

                    req.extensions_mut().insert(this.store.clone());

                    let future = work.call(*context, req);

                    this.state.set(SessionWorkFutureState::Future {
                        future,
                        cookies,
                        id,
                    });
                }
                SessionFutureStateProj::Future {
                    future,
                    cookies,
                    id,
                } => match ready!(future.poll(cx)) {
                    Ok(ret) => {
                        match id.state() {
                            State::Set(uuid) => {
                                let mut cookie = Cookie::new(
                                    this.cookie_name.clone(),
                                    uuid.hyphenated().to_string(),
                                );

                                cookie.set_secure(true);
                                cookie.set_http_only(true);

                                cookies.signed(&this.cookie_key).add(cookie);
                            }
                            State::Remove(uuid) => {
                                cookies.signed(&this.cookie_key).remove(Cookie::new(
                                    this.cookie_name.clone(),
                                    uuid.hyphenated().to_string(),
                                ));
                            }
                            _ => {}
                        }

                        return Poll::Ready(Ok(ret));
                    }
                    Err(err) => return Poll::Ready(Err(Error::new(err))),
                },
            }
        }
    }
}
