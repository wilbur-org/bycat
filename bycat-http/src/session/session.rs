use crate::{FromRequestParts, session::store::SessionStore};
use alloc::sync::Arc;
use arc_swap::{ArcSwap, ArcSwapAny};
use bycat_error::Error;
use bycat_value::{Map, Value};
use core::{
    mem::transmute,
    task::{Poll, ready},
};
use futures::future::BoxFuture;
use http::request::Parts;
use pin_project_lite::pin_project;
use uuid::Uuid;

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
pub struct SessionId(Arc<ArcSwap<State>>);

impl Default for SessionId {
    fn default() -> Self {
        SessionId(Arc::new(ArcSwapAny::new(State::Noop.into())))
    }
}

impl SessionId {
    pub fn new(id: Uuid) -> SessionId {
        SessionId(Arc::new(ArcSwapAny::new(State::Init(id).into())))
    }

    pub fn id(&self) -> Option<Uuid> {
        self.0.load().id()
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

pub struct Session {
    id: SessionId,
    store: SessionStore,
    value: Map,
}

impl Session {
    pub fn get<T: TryFrom<Value>>(&self, key: &str) -> Result<Option<T>, Error>
    where
        T::Error: core::error::Error + Send + Sync + 'static,
    {
        match self.value.get(key) {
            Some(ret) => T::try_from(ret.clone()).map(Some).map_err(Error::new),
            None => Ok(None),
        }
    }

    pub fn get_raw(&self, key: &str) -> Option<&Value> {
        self.value.get(key)
    }

    pub fn set<V: Into<Value>>(&mut self, key: &str, value: V) -> Option<Value> {
        self.value.insert(key, value)
    }

    pub fn remove(&mut self, name: &str) {
        self.value.remove(name);
    }

    pub async fn regenerate_id(&mut self) -> Result<(), Error> {
        self.store.remove(self.id.clone()).await?;
        self.id.generate();
        self.save().await?;
        Ok(())
    }

    pub async fn save(&mut self) -> Result<(), Error> {
        if self.id.state().id().is_none() {
            self.id.generate();
        }

        self.store.save(self.id.clone(), &self.value).await?;

        Ok(())
    }

    pub async fn load(&mut self) {
        if let Ok(ret) = self.store.load(self.id.clone()).await {
            self.value = ret;
        }
    }

    pub async fn delete(&mut self) -> Result<(), Error> {
        self.store.remove(self.id.clone()).await?;
        self.id.remove();
        Ok(())
    }
}

impl<C> FromRequestParts<C> for Session {
    type Future<'a>
        = SessionFuture<'a, C>
    where
        C: 'a;
    fn from_request_parts<'a>(
        parts: &'a mut http::request::Parts,
        state: &'a C,
    ) -> Self::Future<'a> {
        SessionFuture {
            state: SessionFutureState::Init { state, parts },
        }
    }
}

pin_project! {
  #[project = SessionFutureStateProj]
  enum SessionFutureState<'a, C> {
      Init {
          state: &'a C,
          parts: &'a mut Parts,
      },
      Future {
          #[pin]
          future: BoxFuture<'a, Result<Map, Error>>,
          id: SessionId,
          store: SessionStore
      },
  }
}

pin_project! {
  pub struct SessionFuture<'a, C> {
    #[pin]
    state: SessionFutureState<'a, C>
  }
}

impl<'a, C> Future for SessionFuture<'a, C> {
    type Output = Result<Session, Error>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            match this.state.as_mut().project() {
                SessionFutureStateProj::Init { parts, .. } => {
                    let Some(store) = parts.extensions.get::<SessionStore>() else {
                        return Poll::Ready(Err(Error::new("session store not found")));
                    };

                    let Some(id) = parts.extensions.get::<SessionId>() else {
                        return Poll::Ready(Err(Error::new("session not found")));
                    };

                    let future = store.load(id.clone());

                    let id = id.clone();
                    let store = store.clone();

                    unsafe {
                        // SAFTY: We own Store
                        let state = transmute::<SessionFutureState<'_, C>, SessionFutureState<'a, C>>(
                            SessionFutureState::Future { future, id, store },
                        );
                        this.state.set(state);
                    }
                    continue;
                }
                SessionFutureStateProj::Future { future, id, store } => {
                    //
                    match ready!(future.poll(cx)) {
                        Ok(value) => {
                            return Poll::Ready(Ok(Session {
                                id: id.clone(),
                                store: store.clone(),
                                value,
                            }));
                        }
                        Err(err) => return Poll::Ready(Err(err)),
                    }
                }
            }
        }
    }
}
