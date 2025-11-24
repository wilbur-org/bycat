mod session;

use core::marker::PhantomData;

use alloc::{boxed::Box, string::ToString, sync::Arc};
use arc_swap::{ArcSwap, ArcSwapAny};
use bycat::{Middleware, Work};
use bycat_error::{BoxError, Error};
use bycat_value::Map;
use cookie::Cookie;
use futures::future::{BoxFuture, LocalBoxFuture};
use http::Request;
use uuid::Uuid;

use crate::cookies::CookieJar;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum State {
    Set(Uuid),
    Remove(Uuid),
    Init(Uuid),
    Noop,
}

impl State {
    pub fn id(&self) -> Option<Uuid> {
        match self {
            Self::Remove(id) => Some(*id),
            Self::Set(id) => Some(*id),
            Self::Init(id) => Some(*id),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionId(pub(crate) Arc<ArcSwap<State>>);

impl Default for SessionId {
    fn default() -> Self {
        SessionId(Arc::new(ArcSwapAny::new(State::Noop.into())))
    }
}

impl SessionId {
    pub fn new(id: Uuid) -> SessionId {
        SessionId(Arc::new(ArcSwapAny::new(State::Init(id).into())))
    }

    pub(crate) fn state(&self) -> State {
        **self.0.load()
    }

    fn remove(&self) {
        let state = self.state();
        if let Some(id) = state.id() {
            self.0.store(State::Remove(id).into());
        }
    }

    fn generate(&self) {
        self.0.store(State::Set(Uuid::new_v4()).into());
    }
}

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

    fn save<'a>(&'a self, id: &'a SessionId, session: &'a Session) -> Self::Save<'a>;
    fn load<'a>(&'a self, id: &'a SessionId) -> Self::Load<'a>;
    fn delete<'a>(&'a self, id: &'a SessionId) -> Self::Delete<'a>;
}

pub trait DynStore {
    fn save<'a>(&'a self, id: SessionId, session: &'a Map) -> BoxFuture<'a, Result<(), Error>>;
    fn load<'a>(&'a self, id: SessionId) -> BoxFuture<'a, Result<Map, Error>>;
    fn remove<'a>(&'a self, id: SessionId) -> BoxFuture<'a, Result<Map, Error>>;
}

pub type SessionStore = Arc<dyn DynStore>;

pub struct Sessions {
    store: Arc<dyn DynStore>,
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
    T::Error: Into<BoxError>,
{
    type Output = T::Output;

    type Error = Error;

    type Future<'a>
        = LocalBoxFuture<'a, Result<Self::Output, Self::Error>>
    where
        Self: 'a,
        C: 'a;

    fn call<'a>(&'a self, context: &'a C, mut req: Request<B>) -> Self::Future<'a> {
        Box::pin(async move {
            let cookies = CookieJar::from_request(&req)?;

            let id = if let Some(id) = req.extensions().get::<SessionId>() {
                id.clone()
            } else {
                let id = if let Some(id) = cookies.get("sess_id") {
                    let id = Uuid::parse_str(id.value()).map_err(bycat_error::Error::new)?;
                    SessionId::new(id)
                } else {
                    SessionId::default()
                };

                req.extensions_mut().insert(id.clone());

                id
            };

            let resp = self.work.call(context, req).await.map_err(Error::new)?;

            match id.state() {
                State::Set(uuid) => {
                    cookies.add(Cookie::new("sess_id", uuid.hyphenated().to_string()));
                }
                State::Remove(uuid) => {
                    cookies.remove(Cookie::new("sess_id", uuid.hyphenated().to_string()));
                }
                _ => {}
            }

            Ok(resp)
        })
    }
}
