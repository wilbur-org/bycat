use core::task::{Poll, ready};

use alloc::{path::PathBuf, vec::Vec};
use bycat::Work;
use bycat_fs::{Fs, VirtualFS};
use bycat_package::Package;
use http::{Request, Response};
use pin_project_lite::pin_project;
use relative_path::RelativePath;

enum AssetSource {
    File(PathBuf),
    Dir(PathBuf),
}

pub struct Assets<T> {
    fs: T,
}

impl<T, C, B> Work<C, Request<B>> for Assets<T>
where
    T: VirtualFS,
{
    type Output = Package<T::Body>;

    type Error = T::Error;

    type Future<'a>
        = AssetFuture<'a, T, B>
    where
        Self: 'a,
        C: 'a;

    fn call<'a>(&'a self, _context: &'a C, req: Request<B>) -> Self::Future<'a> {
        AssetFuture {
            fs: &self.fs,
            state: AssetFutureState::Exists {
                future: self.fs.exists(req.uri().path()),
            },
            req,
        }
    }
}

pin_project! {
    #[project = AssetFutureProj]
    pub enum AssetFutureState<'a, T: 'a>
    where
        T: VirtualFS
    {
        Exists {
            #[pin]
            future: T::Exists<'a>
        },
        Read {
            #[pin]
            future: T::Read<'a>
        }
    }
}

pin_project! {
    pub struct AssetFuture<'a, T: 'a, B> where T: VirtualFS {
        #[pin]
        state: AssetFutureState<'a, T>,
        fs: &'a T,
        req: Request<B>
    }
}

impl<'a, T, B> Future for AssetFuture<'a, T, B>
where
    T: VirtualFS,
{
    type Output = Result<Package<T::Body>, T::Error>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();
            match this.state.as_mut().project() {
                AssetFutureProj::Exists { future } => {
                    let ret = match ready!(future.poll(cx)) {
                        Ok(ret) => ret,
                        Err(err) => return Poll::Ready(Err(err)),
                    };

                    if ret {
                        this.state.set(AssetFutureState::Read {
                            future: this.fs.read(this.req.uri().path()),
                        });
                    } else {
                        todo!()
                    }
                }
                AssetFutureProj::Read { future } => return future.poll(cx),
            }
        }
    }
}
