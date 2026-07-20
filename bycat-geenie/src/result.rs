use std::collections::BTreeSet;

use relative_path::RelativePathBuf;
use spurgt::Spurgt;

use crate::command::DynamicCommand;
use crate::{command::CommandList, FileList};
use crate::{File, GeenieError, Item};

pub(crate) struct ResultBuilder<E> {
    pub(crate) files: Vec<File>,
    seen: BTreeSet<RelativePathBuf>,
    pub(crate) commands: Vec<Box<dyn DynamicCommand<E>>>,
}

impl<E> Default for ResultBuilder<E> {
    fn default() -> Self {
        Self {
            files: Default::default(),
            seen: Default::default(),
            commands: Default::default(),
        }
    }
}

impl<E> ResultBuilder<E> {
    pub fn push_file(&mut self, file: File) -> Result<(), GeenieError> {
        if self.seen.contains(&file.path) {
            return Err(GeenieError::duplicate(file.path.clone()));
        }

        self.seen.insert(file.path.clone());
        self.files.push(file);

        Ok(())
    }

    pub fn push_command(&mut self, command: Box<dyn DynamicCommand<E>>) {
        self.commands.push(command);
    }

    pub fn build(self, env: Spurgt<E>) -> GeenieResult<E> {
        GeenieResult {
            files: FileList { files: self.files },
            commands: self.commands.into(),
            env,
        }
    }
}

impl<E: 'static, C> Item<E, C> for ResultBuilder<E> {
    fn process<'a>(
        self,
        mut ctx: crate::Context<'a, E, C>,
        _env: &'a mut Spurgt<E>,
    ) -> impl std::future::Future<Output = Result<(), GeenieError>> + 'a {
        async move {
            ctx.push(FileList::from(self.files))
                .push(CommandList::from(self.commands));

            Ok(())
        }
    }
}

pub struct GeenieResult<E> {
    pub env: Spurgt<E>,
    pub files: FileList,
    pub commands: CommandList<E>,
}

impl<E> GeenieResult<E> {
    #[cfg(feature = "fs")]
    pub async fn write_to(
        &mut self,
        path: impl AsRef<std::path::Path>,
        force: bool,
    ) -> Result<(), GeenieError> {
        self.files.write_to(path.as_ref(), force).await?;
        self.commands.run_in(&mut self.env, path.as_ref()).await?;

        Ok(())
    }
}

impl<E: 'static, C> Item<E, C> for GeenieResult<E> {
    fn process<'a>(
        self,
        mut ctx: crate::Context<'a, E, C>,
        _env: &'a mut Spurgt<E>,
    ) -> impl std::future::Future<Output = Result<(), GeenieError>> + 'a {
        async move {
            ctx.push(self.files).push(self.commands);

            Ok(())
        }
    }
}
