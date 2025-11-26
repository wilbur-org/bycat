use bycat::prelude::WorkExt;
use bycat_http::{
    WorkIntoResponseExt,
    extract::RequestBodyLimit,
    handler,
    session::{MemoryStore, Session, Sessions},
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> bycat_error::Result<()> {
    bycat_http::serve(
        ("localhost", 3000),
        (),
        handler(|session: Session| async move {
            //

            "Hello, World!"
        })
        .wrap(RequestBodyLimit(1024))
        .into_response()
        .wrap(Sessions::new(MemoryStore::default())),
    )
    .await
    .unwrap();

    Ok(())
}
