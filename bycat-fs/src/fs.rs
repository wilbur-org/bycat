use std::{
    path::{Path, PathBuf},
    task::{Poll, ready},
};

use bycat::{Matcher, Work};
use bycat_error::Error;
use bycat_package::{IntoPackage, Package};
use bycat_source::Source;
use futures::future::BoxFuture;
use pin_project_lite::pin_project;
use relative_path::RelativePath;

use crate::{Body, ReadDir, ResolvedPath, WalkDir, WalkDirStream};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fs {
    root: PathBuf,
}

impl Fs {
    pub fn new(path: impl Into<PathBuf>) -> Fs {
        Fs { root: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.root
    }

    pub fn find<T: Matcher<ResolvedPath> + Send + Sync + 'static>(&self, matcher: T) -> WalkDir {
        let resolver = WalkDir::new(self.root.to_path_buf()).pattern(matcher);
        resolver
    }

    pub fn list(&self, path: impl AsRef<RelativePath>) -> ReadDir {
        let resolver = ReadDir::new(path.as_ref().to_logical_path(&self.root));
        resolver
    }

    pub async fn read(&self, path: impl AsRef<RelativePath>) -> Result<Package<Body>, Error> {
        let path = ResolvedPath::new(
            self.root.to_path_buf(),
            path.as_ref().to_relative_path_buf(),
        );
        path.into_package().await
    }

    pub async fn write(&self, file: Package<Body>) -> Result<(), Error> {
        self.call(&(), file).await?;
        Ok(())
    }
}

impl<C> Source<C> for Fs {
    type Item = Package<Body>;

    type Error = bycat_error::Error;

    type Stream<'a>
        = WalkDirStream
    where
        Self: 'a,
        C: 'a;

    fn create_stream<'a>(self, ctx: &'a C) -> Self::Stream<'a> {
        WalkDir::new(self.root.clone()).create_stream(ctx)
    }
}

// Work

impl<C, T> Work<C, Package<T>> for Fs
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
                    if !tokio::fs::try_exists(&self.root)
                        .await
                        .map_err(Error::new)?
                    {
                        tokio::fs::create_dir_all(&self.root)
                            .await
                            .map_err(Error::new)?
                    }

                    Ok(package)
                }),
            },
            root: &self.root,
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
