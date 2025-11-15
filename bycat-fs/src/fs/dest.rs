use bycat::Work;
use bycat_error::Error;
use bycat_package::Package;
use futures::future::BoxFuture;
use mime::Mime;
use std::{boxed::Box, path::PathBuf, vec::Vec};
use tokio::io::AsyncWriteExt;

use super::Body;

pub trait Filter: Send + Sync {
    fn append(&self, pkg: &Package<Body>) -> bool;
}

impl Filter for Mime {
    fn append(&self, pkg: &Package<Body>) -> bool {
        pkg.mime() == self
    }
}

pub struct KravlDestination {
    root: PathBuf,
    append: Vec<Box<dyn Filter>>,
}

impl KravlDestination {
    pub fn new(path: impl Into<PathBuf>) -> KravlDestination {
        KravlDestination {
            root: path.into(),
            append: Default::default(),
        }
    }

    pub fn append_when<T>(mut self, filter: T) -> Self
    where
        T: Filter + 'static,
    {
        self.append.push(Box::new(filter));
        self
    }
}

impl KravlDestination {
    fn append(&self, pkg: &Package<Body>) -> bool {
        for filter in &self.append {
            if filter.append(pkg) {
                return true;
            }
        }
        false
    }
}

impl<C> Work<C, Package<Body>> for KravlDestination {
    type Output = Package<Body>;
    type Error = Error;
    type Future<'a>
        = BoxFuture<'a, Result<Self::Output, Error>>
    where
        Self: 'a,
        C: 'a;

    fn call<'a>(&'a self, _ctx: &'a C, mut req: Package<Body>) -> Self::Future<'a> {
        Box::pin(async move {
            let path = req.path().to_logical_path(&self.root);

            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await.ok();
            }

            if self.append(&req) {
                let mut file = tokio::fs::OpenOptions::default()
                    .append(true)
                    .create(true)
                    .open(&path)
                    .await
                    .map_err(Error::new)?;
                let bytes = req.replace_content(Body::Empty).bytes().await?;
                file.write_all(&bytes).await.map_err(Error::new)?;
                file.write_all(b"\n").await.map_err(Error::new)?;
            } else {
                let mut file = tokio::fs::OpenOptions::default()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open(&path)
                    .await
                    .map_err(Error::new)?;
                let bytes = req.replace_content(Body::Empty).bytes().await?;
                file.write_all(&bytes).await.map_err(Error::new)?;
            }

            Ok(req)
        })
    }
}
