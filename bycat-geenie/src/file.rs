#[cfg(feature = "fs")]
use futures::StreamExt;
use relative_path::RelativePathBuf;
use spurgt::Spurgt;

use crate::{GeenieError, Item};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub path: RelativePathBuf,
    pub content: Vec<u8>,
}

impl File {
    pub fn new(path: impl Into<RelativePathBuf>, content: impl Into<Vec<u8>>) -> File {
        File {
            path: path.into(),
            content: content.into(),
        }
    }

    #[cfg(feature = "fs")]
    pub async fn write_to(&self, path: &std::path::Path, force: bool) -> Result<(), GeenieError> {
        let file_path = self.path.to_logical_path(&path);
        if async_fs::metadata(&file_path).await.is_ok() && !force {
            return Err(GeenieError::exists(self.path.clone()));
        }
        if let Some(parent) = file_path.parent() {
            async_fs::create_dir_all(parent).await?;
        }
        async_fs::write(file_path, &self.content).await?;

        Ok(())
    }
}

impl<E, C> Item<E, C> for File {
    fn process<'a>(
        self,
        mut ctx: crate::Context<'a, E, C>,
        _env: &'a mut Spurgt<E>,
    ) -> impl std::future::Future<Output = Result<(), GeenieError>> + 'a {
        async move {
            ctx.file(self)?;
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileList {
    pub(crate) files: Vec<File>,
}

impl FileList {
    #[cfg(feature = "fs")]
    pub async fn write_to(
        &self,
        path: impl AsRef<std::path::Path>,
        force: bool,
    ) -> Result<(), GeenieError> {
        let path = path.as_ref();
        for files in self.files.chunks(10) {
            let mut futures = futures::stream::FuturesUnordered::new();

            for file in files {
                futures.push(async move { file.write_to(path, force).await });
            }

            while let Some(next) = futures.next().await {
                match next {
                    Ok(e) => {
                        let _ = e;
                    }
                    Err(err) => {
                        if err.is_io() {
                            return Err(err);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn push(&mut self, file: File) {
        self.files.push(file);
    }
}

impl<E, C> Item<E, C> for FileList {
    fn process<'a>(
        self,
        mut ctx: crate::Context<'a, E, C>,
        _env: &'a mut Spurgt<E>,
    ) -> impl std::future::Future<Output = Result<(), GeenieError>> + 'a {
        async move {
            for file in self.files {
                ctx.file(file)?;
            }
            Ok(())
        }
    }
}

impl IntoIterator for FileList {
    type IntoIter = std::vec::IntoIter<File>;
    type Item = File;

    fn into_iter(self) -> Self::IntoIter {
        self.files.into_iter()
    }
}

impl<'a> IntoIterator for &'a FileList {
    type IntoIter = std::slice::Iter<'a, File>;
    type Item = &'a File;

    fn into_iter(self) -> Self::IntoIter {
        self.files.iter()
    }
}

impl From<Vec<File>> for FileList {
    fn from(files: Vec<File>) -> Self {
        FileList { files }
    }
}

impl From<FileList> for Vec<File> {
    fn from(files: FileList) -> Self {
        files.files
    }
}
