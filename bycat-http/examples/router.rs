use bycat::{Work, middleware, prelude::WorkExt, work_fn};
use bycat_http::{
    WorkIntoResponseExt,
    cookies::Cookies,
    cors::Cors,
    extract::RequestBodyLimit,
    handler,
    router::{SendRouterBuilder, SendWork, UrlParams},
    session::{MemoryStore, Session, Sessions},
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> bycat_error::Result<()> {
    let mut router = SendRouterBuilder::new()
        .get(
            "/",
            handler(async || bycat_error::Result::Ok("Hello, world!")),
        )
        .get(
            "/:hello",
            handler(async |params: UrlParams| {
                bycat_error::Result::Ok(format!("Hello, {}", params.get("hello").unwrap()))
            }),
        )
        .middleware(middleware(|task: SendWork<_, _>| {
            work_fn(move |ctx: (), req| {
                let task = task.clone();
                async move {
                    println!("Hello, from middleware!");
                    task.call(&ctx, req).await
                }
            })
        }))
        .build();

    bycat_http::serve(("localhost", 3000), (), router)
        .await
        .unwrap();

    Ok(())
}
