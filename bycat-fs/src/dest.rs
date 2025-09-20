use bycat::Work;
use bycat_error::Error;
use bycat_package::Package;
use futures::future::BoxFuture;
use mime::Mime;
use pin_project_lite::pin_project;
use std::{
    path::{Path, PathBuf},
    task::{Poll, ready},
};
use tokio::io::AsyncWriteExt;

use crate::Body;

#[derive(Debug, Clone)]
pub struct FsDest {
    path: std::path::PathBuf,
}

impl FsDest {
    pub fn new(path: impl Into<PathBuf>) -> FsDest {
        FsDest { path: path.into() }
    }
}

impl<C, T> Work<C, Package<T>> for FsDest
where
    T: Into<Body>,
    for<'a> T: 'a,
{
    type Output = Package<Body>;

    type Error = Error;

    type Future<'a>
        = FsDestFuture<'a>
    where
        C: 'a;

    fn call<'a>(&'a self, _ctx: &'a C, req: Package<T>) -> Self::Future<'a> {
        let package = req.map_sync(|m| m.into());

        FsDestFuture {
            state: FsDestFutureState::Ensure {
                future: Box::pin(async move {
                    if !tokio::fs::try_exists(&self.path)
                        .await
                        .map_err(Error::new)?
                    {
                        tokio::fs::create_dir_all(&self.path)
                            .await
                            .map_err(Error::new)?
                    }

                    Ok(package)
                }),
            },
            root: &self.path,
        }
    }
}

pin_project! {
    #[project = StateProj]
    enum FsDestFutureState<'a> {
        Ensure {
            #[pin]
            future: BoxFuture<'a, Result<Package<Body>, Error>>,
        },
        Write {
            #[pin]
            future: BoxFuture<'a, Result<Package<Body>, Error>>,
        },
    }

}

pin_project! {
    pub struct FsDestFuture<'a> {
        #[pin]
        state: FsDestFutureState<'a>,
        root: &'a Path,
    }
}

impl<'a> Future for FsDestFuture<'a> {
    type Output = Result<Package<Body>, Error>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            match this.state.as_mut().project() {
                StateProj::Ensure { future } => {
                    let mut ret = match ready!(future.poll(cx)) {
                        Ok(ret) => ret,
                        Err(err) => return Poll::Ready(Err(err)),
                    };

                    let file_path = ret.path().to_logical_path(this.root);

                    let future = Box::pin(async move {
                        ret.content_mut().write_to(&file_path).await?;
                        Ok(ret)
                    });

                    this.state.set(FsDestFutureState::Write { future });
                }
                StateProj::Write { future } => return future.poll(cx),
            }
        }
    }
}

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
