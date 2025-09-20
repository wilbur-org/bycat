use std::{
    path::PathBuf,
    task::{Poll, ready},
};

use crate::{
    Body,
    resolver::{FileResolver, IntoResolverStream, ResolvedPath},
};
use bycat::Matcher;
use bycat_error::Error;
use bycat_package::Package;
use bycat_source::Source;
use futures::Stream;
use pin_project_lite::pin_project;
use relative_path::RelativePathBuf;

pub struct FsSource {
    root: FileResolver,
}

impl Default for FsSource {
    fn default() -> Self {
        FsSource::new(std::env::current_dir().unwrap())
    }
}

impl FsSource {
    pub fn new(root: PathBuf) -> FsSource {
        FsSource {
            root: FileResolver::new(root),
        }
    }

    pub fn pattern<T: Matcher<ResolvedPath> + Send + Sync + 'static>(self, pattern: T) -> Self {
        Self {
            root: self.root.pattern(pattern),
        }
    }
}

impl<C> Source<C> for FsSource {
    type Item = Package<Body>;

    type Error = Error;

    type Stream<'a>
        = FsSourceStream
    where
        Self: 'a,
        C: 'a;

    fn create_stream<'a>(self, _ctx: &'a C) -> Self::Stream<'a> {
        FsSourceStream {
            stream: self.root.into_find(),
        }
    }
}

pin_project! {

pub struct FsSourceStream {
    #[pin]
    stream: IntoResolverStream,
}
}

impl Stream for FsSourceStream {
    type Item = Result<Package<Body>, Error>;
    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let next = match ready!(self.project().stream.poll_next(cx)) {
            Some(Ok(ret)) => ret,
            Some(Err(err)) => return Poll::Ready(Some(Err(Error::new(err)))),
            None => return Poll::Ready(None),
        };

        let full_path = next.full_path();
        let mime = mime_guess::from_path(&full_path).first_or_octet_stream();

        Poll::Ready(Some(Ok(Package::new(
            next.path().clone(),
            mime,
            Body::Path(full_path),
        ))))
    }
}
