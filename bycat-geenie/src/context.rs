use crate::{
    File, GeenieError, Item,
    command::{Command, CommandBox},
    item::{DynamicItem, ItemBox},
    result::ResultBuilder,
};

pub struct Context<'a, E, C> {
    pub(crate) files: &'a mut ResultBuilder<E>,
    pub(crate) questions: &'a mut Vec<Box<dyn DynamicItem<E, C>>>,
    pub(crate) ctx: &'a mut C,
}

impl<'a, E, C> Context<'a, E, C> {
    pub fn push<T>(&mut self, item: T) -> &mut Self
    where
        T: Item<E, C> + 'static,
    {
        self.questions.push(Box::new(ItemBox(item)));
        self
    }

    pub fn file(&mut self, file: impl Into<File>) -> Result<&mut Self, GeenieError> {
        self.files.push_file(file.into())?;
        Ok(self)
    }

    pub fn command<T>(&mut self, command: T) -> &mut Self
    where
        T: Command<E> + 'static,
    {
        self.files.push_command(Box::new(CommandBox(command)));
        self
    }

    pub fn data_mut(&mut self) -> &mut C {
        self.ctx
    }

    pub fn data(&self) -> &C {
        self.ctx
    }
}

pub trait ContextLike<'a, E, C> {
    fn env(&self) -> &E;

    fn cwd(&self) -> &std::path::Path;

    fn push<T>(&mut self, item: T) -> &mut Self
    where
        T: Item<E, C> + 'a;
}
