use std::path::PathBuf;

use bycat::Work;
use bycat_error::Error;
use bycat_package::Package;
use relative_path::RelativePathBuf;

use super::Body;

pub struct FsWork {
    root: PathBuf,
}

impl FsWork {
    pub fn new(root: impl Into<PathBuf>) -> FsWork {
        FsWork { root: root.into() }
    }
}

impl<C> Work<C, RelativePathBuf> for FsWork {
    type Output = Package<Body>;

    type Error = Error;

    type Future<'a>
        = core::future::Ready<Result<Package<Body>, Error>>
    where
        Self: 'a,
        C: 'a;

    fn call<'a>(&'a self, _ctx: &'a C, path: RelativePathBuf) -> Self::Future<'a> {
        let full_path = path.to_logical_path(&self.root);
        let mime = mime_guess::from_path(&full_path).first_or_octet_stream();
        core::future::ready(Ok(Package::new(path, mime, Body::Path(full_path))))
    }
}
