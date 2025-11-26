use crate::session::SessionId;
use alloc::{boxed::Box, collections::HashMap, sync::Arc};
use bycat_error::Error;
use bycat_value::Map;
use futures::future::BoxFuture;
use parking_lot::RwLock;
use uuid::Uuid;

pub trait Store {
    fn save<'a>(
        &'a self,
        id: SessionId,
        session: &'a Map,
    ) -> impl Future<Output = Result<(), Error>> + Send;
    fn load<'a>(&'a self, id: SessionId) -> impl Future<Output = Result<Map, Error>> + Send;
    fn remove<'a>(&'a self, id: SessionId) -> impl Future<Output = Result<(), Error>> + Send;
}

#[derive(Default)]
pub struct MemoryStore {
    tree: RwLock<HashMap<Uuid, Map>>,
}

impl Store for MemoryStore {
    fn save<'a>(
        &'a self,
        id: SessionId,
        session: &'a Map,
    ) -> impl Future<Output = Result<(), Error>> + Send {
        async move {
            if let Some(id) = id.id() {
                self.tree.write().insert(id, session.clone());
            }
            Ok(())
        }
    }

    fn load<'a>(&'a self, id: SessionId) -> impl Future<Output = Result<Map, Error>> + Send {
        async move {
            if let Some(id) = id.id() {
                Ok(self.tree.read().get(&id).cloned().unwrap_or_default())
            } else {
                Ok(Map::default())
            }
        }
    }

    fn remove<'a>(&'a self, id: SessionId) -> impl Future<Output = Result<(), Error>> + Send {
        async move {
            if let Some(id) = id.id() {
                self.tree.write().remove(&id);
            }
            Ok(())
        }
    }
}

pub(crate) trait DynStore: Send + Sync {
    fn save<'a>(&'a self, id: SessionId, session: &'a Map) -> BoxFuture<'a, Result<(), Error>>;
    fn load<'a>(&'a self, id: SessionId) -> BoxFuture<'a, Result<Map, Error>>;
    fn remove<'a>(&'a self, id: SessionId) -> BoxFuture<'a, Result<(), Error>>;
}

pub(crate) type SessionStore = Arc<dyn DynStore>;

pub(crate) struct DynStoreImpl<T>(pub T);

impl<T> DynStore for DynStoreImpl<T>
where
    T: Store + Send + Sync,
{
    fn save<'a>(&'a self, id: SessionId, session: &'a Map) -> BoxFuture<'a, Result<(), Error>> {
        Box::pin(async move { self.0.save(id, session).await })
    }

    fn load<'a>(&'a self, id: SessionId) -> BoxFuture<'a, Result<Map, Error>> {
        Box::pin(async move { self.0.load(id).await })
    }

    fn remove<'a>(&'a self, id: SessionId) -> BoxFuture<'a, Result<(), Error>> {
        Box::pin(async move { self.0.remove(id).await })
    }
}
