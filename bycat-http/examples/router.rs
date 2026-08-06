use bycat::prelude::WorkExt;
use bycat_http::{
    WorkIntoResponseExt,
    cookies::Cookies,
    cors::Cors,
    extract::RequestBodyLimit,
    handler,
    router::SendRouterBuilder,
    session::{MemoryStore, Session, Sessions},
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> bycat_error::Result<()> {
    let mut router = SendRouterBuilder::new()
        .get(
            "/",
            handler(async || bycat_error::Result::Ok("Hello, world!")),
        )
        .build();

    bycat_http::serve(("localhost", 3000), (), router)
        .await
        .unwrap();

    Ok(())
}
