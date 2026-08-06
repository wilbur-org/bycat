use bycat::{Work, middleware, work_fn};
use bycat_http::{
    error::Result,
    handler,
    router::{SendRouterBuilder, SendWork, UrlParams},
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let router = SendRouterBuilder::new()
        .with_get("/", handler(async || "Hello, world!"))?
        .with_get(
            "/:hello",
            handler(async |params: UrlParams| format!("Hello, {}", params.get("hello").unwrap())),
        )?
        .with_middleware(middleware(|task: SendWork<_, _>| {
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
