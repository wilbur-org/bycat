use bycat_package::{Content, IntoPackage, Package};
#[cfg(feature = "fs")]
use futures::StreamExt;
use relative_path::RelativePathBuf;
use spurgt::{Spurgt, core::BoxError};

use crate::{GeenieError, Item};

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct File {
//     pub path: RelativePathBuf,
//     pub content: Vec<u8>,
// }

// impl File {
//     pub fn new(path: impl Into<RelativePathBuf>, content: impl Into<Vec<u8>>) -> File {
//         File {
//             path: path.into(),
//             content: content.into(),
//         }
//     }

//     #[cfg(feature = "fs")]
//     pub async fn write_to(&self, path: &std::path::Path, force: bool) -> Result<(), GeenieError> {
//         let file_path = self.path.to_logical_path(&path);
//         if async_fs::metadata(&file_path).await.is_ok() && !force {
//             return Err(GeenieError::exists(self.path.clone()));
//         }
//         if let Some(parent) = file_path.parent() {
//             async_fs::create_dir_all(parent).await?;
//         }
//         async_fs::write(file_path, &self.content).await?;

//         Ok(())
//     }
// }

// impl<E, C, > Item<E, C> for File {
//     fn process<'a>(
//         self,
//         mut ctx: crate::Context<'a, E, C>,
//         _env: &'a mut Spurgt<E>,
//     ) -> impl std::future::Future<Output = Result<(), GeenieError>> + 'a {
//         async move {
//             ctx.file(self)?;
//             Ok(())
//         }
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageItem<P> {
    file: P,
}

impl<P> PackageItem<P> {
    pub fn new(file: P) -> PackageItem<P> {
        PackageItem { file }
    }
}

impl<C, B, P> Item<C, B> for PackageItem<P>
where
    P: IntoPackage<B> + 'static,
    P::Error: Into<BoxError>,
{
    fn process<'a>(
        self,
        mut ctx: crate::Context<'a, C, B>,
    ) -> impl std::future::Future<Output = Result<(), GeenieError>> + 'a {
        async move {
            let package = self
                .file
                .into_package()
                .await
                .map_err(GeenieError::backend)?;
            ctx.package(package)?;
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileList<T: Content> {
    pub(crate) files: Vec<Package<T>>,
}

impl<T: Content> FileList<T> {
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

    pub fn push(&mut self, file: Package<T>) {
        self.files.push(file);
    }
}

impl<C, B: Content> Item<C, B> for FileList<B> {
    fn process<'a>(
        self,
        mut ctx: crate::Context<'a, C, B>,
    ) -> impl std::future::Future<Output = Result<(), GeenieError>> + 'a {
        async move {
            for file in self.files {
                ctx.package(file)?;
            }
            Ok(())
        }
    }
}

impl<T: Content> IntoIterator for FileList<T> {
    type IntoIter = std::vec::IntoIter<Package<T>>;
    type Item = Package<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.files.into_iter()
    }
}

impl<'a, T: Content> IntoIterator for &'a FileList<T> {
    type IntoIter = std::slice::Iter<'a, Package<T>>;
    type Item = &'a Package<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.files.iter()
    }
}

impl<T: Content> From<Vec<Package<T>>> for FileList<T> {
    fn from(files: Vec<Package<T>>) -> Self {
        FileList { files }
    }
}

impl<T: Content> From<FileList<T>> for Vec<Package<T>> {
    fn from(files: FileList<T>) -> Self {
        files.files
    }
}
