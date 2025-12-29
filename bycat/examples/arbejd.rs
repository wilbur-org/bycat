use std::{convert::Infallible, fmt::Display};

use bycat::{Middleware, Work, prelude::*, when, work_fn};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let test = when(
        |v: &u32| *v == 42,
        work_fn(|ctx: i32, req: u32| async move {
            Result::<_, Infallible>::Ok(format!("Hello, {req}: {ctx}"))
        }),
    )
    .map_err(|_err| "Fejlede");

    // let test = work_fn(|ctx: i32, req: u32| async move {
    //     Result::<_, Infallible>::Ok(format!("Hello, {req}: {ctx}"))
    // });

    let handler = test.pipe(work_fn(
        |_ctx, req| async move { Ok(format!("Hello {req}")) },
    ));

    let out = handler.call(&100, 42).await; //tokio::spawn(async move { handler.call(&100, 42).await }).await;

    println!("{:?}", out);
}
