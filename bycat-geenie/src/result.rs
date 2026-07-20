use std::collections::BTreeSet;

use bycat_package::{Content, Package};
use relative_path::RelativePathBuf;
use spurgt::Spurgt;

use crate::FileList;
use crate::{GeenieError, Item};

pub(crate) struct ResultBuilder<B> {
    pub(crate) files: Vec<Package<B>>,
    seen: BTreeSet<RelativePathBuf>,
}

impl<B> Default for ResultBuilder<B> {
    fn default() -> Self {
        Self {
            files: Default::default(),
            seen: Default::default(),
        }
    }
}

impl<B> ResultBuilder<B> {
    pub fn push_file(&mut self, file: Package<B>) -> Result<(), GeenieError> {
        if self.seen.contains(file.path()) {
            return Err(GeenieError::duplicate(file.path().to_relative_path_buf()));
        }

        self.seen.insert(file.path().to_relative_path_buf());
        self.files.push(file);

        Ok(())
    }

    // pub fn push_command(&mut self, command: Box<dyn DynamicCommand<E>>) {
    //     self.commands.push(command);
    // }

    pub fn build(self) -> GeenieResult<B> {
        GeenieResult { files: self.files }
    }
}

impl<C, B> Item<C, B> for ResultBuilder<B>
where
    B: Content + 'static,
{
    fn process<'a>(
        self,
        mut ctx: crate::Context<'a, C, B>,
    ) -> impl std::future::Future<Output = Result<(), GeenieError>> + 'a {
        async move {
            ctx.push(FileList::from(self.files));

            Ok(())
        }
    }
}

pub struct GeenieResult<B> {
    pub files: Vec<Package<B>>,
}

impl<B> GeenieResult<B> {
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

impl<C, B> Item<C, B> for GeenieResult<B>
where
    B: Content + 'static,
{
    fn process<'a>(
        self,
        mut ctx: crate::Context<'a, C, B>,
    ) -> impl std::future::Future<Output = Result<(), GeenieError>> + 'a {
        async move {
            ctx.push(FileList { files: self.files });

            Ok(())
        }
    }
}
