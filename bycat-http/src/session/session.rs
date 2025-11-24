use core::{
    mem::transmute,
    task::{Poll, ready},
};

use crate::{
    FromRequest, FromRequestParts,
    session::{SessionId, SessionStore},
};
use alloc::sync::Arc;
use arc_swap::{ArcSwap, ArcSwapAny};
use bycat_container::{Extensible, ReadableContainer};
use bycat_error::Error;
use bycat_value::{Map, Value};
use futures::future::BoxFuture;
use http::request::Parts;
use pin_project_lite::pin_project;
use uuid::Uuid;

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub(crate) enum State {
//     Set(Uuid),
//     Remove(Uuid),
//     Init(Uuid),
//     Noop,
// }

// impl State {
//     pub fn id(&self) -> Option<Uuid> {
//         match self {
//             Self::Remove(id) => Some(*id),
//             Self::Set(id) => Some(*id),
//             Self::Init(id) => Some(*id),
//             _ => None,
//         }
//     }
// }

// #[derive(Debug, Clone)]
// pub struct SessionId(pub(crate) Arc<ArcSwap<State>>);

// impl Default for SessionId {
//     fn default() -> Self {
//         SessionId(Arc::new(ArcSwapAny::new(State::Noop.into())))
//     }
// }

// impl SessionId {
//     pub fn new(id: Uuid) -> SessionId {
//         SessionId(Arc::new(ArcSwapAny::new(State::Init(id).into())))
//     }

//     pub(crate) fn state(&self) -> State {
//         **self.0.load()
//     }

//     fn remove(&self) {
//         let state = self.state();
//         if let Some(id) = state.id() {
//             self.0.store(State::Remove(id).into());
//         }
//     }

//     fn generate(&self) {
//         self.0.store(State::Set(Uuid::new_v4()).into());
//     }
// }

pub struct Session {
    id: SessionId,
    store: SessionStore,
    value: Map,
}

impl Session {
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.value.get(key)
    }

    pub fn set(&mut self, key: &str, value: Value) -> Option<Value> {
        self.value.insert(key, value)
    }

    pub async fn load(&mut self) {
        if let Ok(ret) = self.store.load(self.id.clone()).await {
            self.value = ret;
        }
    }

    pub fn remove(&mut self, name: &str) {
        self.value.remove(name);
    }

    pub async fn regenerate_id(&mut self) {
        self.store.remove(self.id.clone()).await;
        self.id.generate();
        self.save().await;
    }

    pub async fn save(&mut self) {
        if self.id.state().id().is_none() {
            self.id.generate();
        }

        self.store.save(self.id.clone(), &self.value).await;
    }

    pub async fn delete(&mut self) {
        self.store.remove(self.id.clone()).await;
        self.id.remove();
    }

    // pub fn iter(&self) -> Iter<'_, vaerdi::String, Value> {
    //     self.value.iter()
    // }
}

impl<C: Extensible> FromRequestParts<C> for Session {
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
          store: &'a SessionStore
      },
  }
}

pin_project! {
  pub struct SessionFuture<'a, C> {
    #[pin]
    state: SessionFutureState<'a, C>
  }
}

impl<'a, C: Extensible> Future for SessionFuture<'a, C> {
    type Output = Result<Session, Error>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            match this.state.as_mut().project() {
                SessionFutureStateProj::Init { state, parts } => {
                    let Some(store) = (*state).get::<SessionStore>() else {
                        return Poll::Ready(Err(Error::new("session store not found")));
                    };

                    let Some(id) = parts.extensions.get::<SessionId>() else {
                        return Poll::Ready(Err(Error::new("session not found")));
                    };

                    let future = store.load(id.clone());

                    let id = id.clone();

                    this.state
                        .set(SessionFutureState::Future { future, id, store });

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
