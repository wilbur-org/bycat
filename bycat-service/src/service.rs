use crate::shutdown::Shutdown;

pub trait Service {
    type Error;
    type Future<'a>: Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;
    fn serve<'a>(&'a self, shutdown: &'a Shutdown) -> Self::Future<'a>;
}
