use crate::shutdown::Shutdown;
use bycat_error::Error;

pub trait Service {
    type Future<'a>: Future<Output = Result<(), Error>>
    where
        Self: 'a;
    fn serve<'a>(&'a self, shutdown: &'a Shutdown) -> Self::Future<'a>;
}
