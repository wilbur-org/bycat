use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use routing::Params;

#[derive(Debug, Clone, Default)]
pub struct UrlParams {
    pub(crate) inner: BTreeMap<Arc<str>, Arc<str>>,
    pub(crate) order: Vec<Arc<str>>,
}

impl UrlParams {
    pub fn get(&self, name: &str) -> Option<&Arc<str>> {
        self.inner.get(name)
    }

    pub fn get_at(&self, idx: usize) -> Option<&Arc<str>> {
        self.order.get(idx)
    }
}

impl Params for UrlParams {
    fn set(&mut self, key: alloc::borrow::Cow<'_, str>, value: alloc::borrow::Cow<'_, str>) {
        let value: Arc<str> = Arc::from(value.as_ref());
        self.order.push(value.clone());
        self.inner.insert(Arc::from(key.as_ref()), value);
    }
}
