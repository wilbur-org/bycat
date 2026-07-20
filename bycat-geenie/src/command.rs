use std::{future::Future, path::Path, pin::Pin};

use spurgt::Spurgt;

use crate::{GeenieError, Item};

pub trait Command<E> {
    fn run<'a>(
        &'a self,
        env: &'a mut Spurgt<E>,
        path: &'a Path,
    ) -> impl Future<Output = Result<(), GeenieError>> + 'a;
}

pub trait DynamicCommand<E> {
    fn run<'a>(
        &'a self,
        env: &'a mut Spurgt<E>,
        path: &'a Path,
    ) -> Pin<Box<dyn Future<Output = Result<(), GeenieError>> + 'a>>;
}

pub struct CommandBox<T>(pub T);

impl<E, T> DynamicCommand<E> for CommandBox<T>
where
    T: Command<E> + 'static,
{
    fn run<'a>(
        &'a self,
        env: &'a mut Spurgt<E>,
        path: &'a Path,
    ) -> Pin<Box<dyn Future<Output = Result<(), GeenieError>> + 'a>> {
        Box::pin(async move { self.0.run(env, path).await })
    }
}

pub struct CommandList<E> {
    cmds: Vec<Box<dyn DynamicCommand<E>>>,
}

impl<E> CommandList<E> {
    pub async fn run_in(&self, env: &mut Spurgt<E>, path: &Path) -> Result<(), GeenieError> {
        for cmd in &self.cmds {
            cmd.run(env, path).await?;
        }
        Ok(())
    }
}

impl<E> From<Vec<Box<dyn DynamicCommand<E>>>> for CommandList<E> {
    fn from(value: Vec<Box<dyn DynamicCommand<E>>>) -> Self {
        CommandList { cmds: value }
    }
}

impl<E> IntoIterator for CommandList<E> {
    type IntoIter = std::vec::IntoIter<Box<dyn DynamicCommand<E>>>;
    type Item = Box<dyn DynamicCommand<E>>;

    fn into_iter(self) -> Self::IntoIter {
        self.cmds.into_iter()
    }
}

impl<'a, E> IntoIterator for &'a CommandList<E> {
    type IntoIter = std::slice::Iter<'a, Box<dyn DynamicCommand<E>>>;
    type Item = &'a Box<dyn DynamicCommand<E>>;

    fn into_iter(self) -> Self::IntoIter {
        self.cmds.iter()
    }
}

impl<E, C> Item<E, C> for CommandList<E> {
    fn process<'a>(
        self,
        ctx: crate::Context<'a, E, C>,
        _env: &'a mut Spurgt<E>,
    ) -> impl Future<Output = Result<(), GeenieError>> + 'a {
        async move {
            for cmd in self {
                ctx.files.push_command(cmd);
            }

            Ok(())
        }
    }
}

pub struct CommandItem<T>(pub T);

impl<E, C, T> Item<E, C> for CommandItem<T>
where
    T: Command<E> + 'static,
{
    fn process<'a>(
        self,
        mut ctx: crate::Context<'a, E, C>,
        _env: &'a mut Spurgt<E>,
    ) -> impl Future<Output = Result<(), GeenieError>> + 'a {
        async move {
            ctx.command(self.0);
            Ok(())
        }
    }
}
