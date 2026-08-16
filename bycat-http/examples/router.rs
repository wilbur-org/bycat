use bycat_http::{
    error::Result,
    handler,
    router::{SendRouterBuilder, SendWork, UrlParams},
};
use bycat_service::{Service, middleware, service_fn};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let router = SendRouterBuilder::new()
        .with_get("/", handler(async || "Hello, world!"))?
        .with_get(
            "/:hello",
            handler(async |params: UrlParams| format!("Hello, {}", params.get("hello").unwrap())),
        )?
        .with_middleware(middleware(|task: SendWork<_, _>| {
            service_fn(move |ctx: (), req| {
                let task = task.clone();
                async move {
                    println!("Hello, from middleware!");
                    task.call(&ctx, req).await
                }
            })
        }))
        .build();

    bycat_http::serve::Tokio::new(router)
        .serve((), ("localhost", 3000))
        .await
        .unwrap();

    Ok(())
}
