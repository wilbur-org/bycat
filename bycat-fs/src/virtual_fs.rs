use bycat_package::Package;
use bycat_source::Source;
use relative_path::RelativePath;

pub trait VirtualFS {
    type Body;
    type Error;
    type Walk: Source<(), Item = Package<Self::Body>, Error = Self::Error>;
    type List: Source<(), Item = Package<Self::Body>, Error = Self::Error>;
    type Exists<'a>: Future<Output = Result<bool, Self::Error>>
    where
        Self: 'a;

    type Read<'a>: Future<Output = Result<Package<Self::Body>, Self::Error>>
    where
        Self: 'a;
    type Write<'a>: Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    fn walk(&self) -> Self::Walk;

    fn exists<'a>(&self, path: impl AsRef<RelativePath>) -> Self::Exists<'a>;

    fn list(&self, path: impl AsRef<RelativePath>) -> Self::List;

    fn read<'a>(&'a self, path: impl AsRef<RelativePath>) -> Self::Read<'a>;

    fn write<'a>(&'a self, package: Package<Self::Body>) -> Self::Write<'a>;
}
