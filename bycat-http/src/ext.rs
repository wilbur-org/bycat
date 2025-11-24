// use core::marker::PhantomData;

// use bycat_error::Error;
// use http::Request;

// use crate::FromRequestParts;

// pub trait RequestExt<B> {
//     type Extract<'a, C, T>: Future<Output = Result<T, Error>>
//     where
//         Self: 'a,
//         T: FromRequestParts<C>,
//         C: 'a;
//     fn extract<'a, C, T: FromRequestParts<C>>(&'a mut self, ctx: &'a C) -> Self::Extract<'a, C, T>;
// }

// struct RequestExtract<'a, T, B, C: 'a>
// where
//     T: FromRequestParts<C>,
// {
//     future: T::Future<'a>,
//     ctx: PhantomData<C>,
//     request: &'a mut Request<B>,
// }
