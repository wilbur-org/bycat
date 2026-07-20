use bycat_package::Package;

use crate::{
    GeenieError, Item,
    item::{DynamicItem, ItemBox},
    result::ResultBuilder,
};

pub struct Context<'a, C, B> {
    pub(crate) files: &'a mut ResultBuilder<B>,
    pub(crate) questions: &'a mut Vec<Box<dyn DynamicItem<C, B>>>,
    pub(crate) ctx: &'a mut C,
}

impl<'a, C, B> Context<'a, C, B> {
    pub fn push<T>(&mut self, item: T) -> &mut Self
    where
        T: Item<C, B> + 'static,
    {
        self.questions.push(Box::new(ItemBox(item)));
        self
    }

    pub fn package(&mut self, file: Package<B>) -> Result<&mut Self, GeenieError> {
        self.files.push_file(file)?;
        Ok(self)
    }

    pub fn data_mut(&mut self) -> &mut C {
        self.ctx
    }

    pub fn data(&self) -> &C {
        self.ctx
    }
}

pub trait ContextLike<'a, C, B> {
    fn env(&self) -> &C;

    fn cwd(&self) -> &std::path::Path;

    fn push<T>(&mut self, item: T) -> &mut Self
    where
        T: Item<C, B> + 'a;
}
