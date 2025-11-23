use core::marker::PhantomData;

use alloc::sync::Arc;
use bycat::{Middleware, Work};
use bycat_error::Error;
use futures::future::{BoxFuture, LocalBoxFuture};
use http::Request;

pub struct SessionId;

pub struct Session {}

pub trait Store {
    type Save<'a>
    where
        Self: 'a;
    type Load<'a>
    where
        Self: 'a;
    type Delete<'a>
    where
        Self: 'a;

    fn save<'a>(&'a self, id: SessionId, session: &'a Session) -> Self::Save<'a>;
    fn load<'a>(&'a self, id: SessionId) -> Self::Load<'a>;
    fn delete<'a>(&'a self, id: SessionId) -> Self::Delete<'a>;
}

pub trait DynStore {
    fn save<'a>(&'a self, id: SessionId, session: &'a Session) -> BoxFuture<'a, Result<(), Error>>;
    fn load<'a>(&'a self, id: SessionId) -> BoxFuture<'a, Result<Session, Error>>;
}

pub struct Sessions {
    store: Arc<dyn DynStore>,
}

impl<C, B, T> Middleware<C, Request<B>, T> for Sessions
where
    T: Work<C, Request<B>>,
{
    type Work = SessionsWork<C, B, T>;

    fn wrap(&self, handler: T) -> Self::Work {
        SessionsWork {
            work: handler,
            req: PhantomData,
        }
    }
}

pub struct SessionsWork<C, B, T> {
    work: T,
    req: PhantomData<(C, B)>,
}

impl<C, B, T> Work<C, Request<B>> for SessionsWork<C, B, T>
where
    T: Work<C, Request<B>>,
{
    type Output = T::Output;

    type Error = T::Error;

    type Future<'a>
        = LocalBoxFuture<'a, Result<Self::Output, Self::Error>>
    where
        Self: 'a,
        C: 'a;

    fn call<'a>(&'a self, context: &'a C, req: Request<B>) -> Self::Future<'a> {
        if req.extensions().get::<Session>().is_some() {
            self.work.call(context, req)
        } else {
          let cookies = Cook
            todo!()
        }
    }
}
