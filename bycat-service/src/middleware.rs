use crate::Service;

pub trait Middleware<C, B, H> {
    type Work: Service<C, B>;

    fn wrap(&self, handle: H) -> Self::Work;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Passthrough;

impl<B, C, H> Middleware<C, B, H> for Passthrough
where
    H: Service<C, B>,
{
    type Work = H;
    fn wrap(&self, handle: H) -> Self::Work {
        handle
    }
}
