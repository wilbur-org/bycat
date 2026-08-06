use bycat::{prelude::WorkExt, work_fn};
use bycat_http::{
    FromRequest, WorkIntoResponseExt,
    body::Body,
    cookies::Cookies,
    cors::Cors,
    extract::RequestBodyLimit,
    handler,
    session::{MemoryStore, Session, Sessions},
    ws::{self, WebSocket},
};
use futures::{SinkExt, StreamExt};
use http::{Request, Response, header::CONTENT_TYPE};

#[tokio::main(flavor = "current_thread")]
async fn main() -> bycat_http::error::Result<()> {
    bycat_http::serve(
        ("localhost", 3000),
        (),
        work_fn(|ctx: (), req: Request<_>| async move {
            if req.uri().path() == "/ws" {
                let upgrade = ws::WebSocketUpgrade::from_request(req, &ctx).await?;
                let (resp, future) = upgrade.on_upgrade(async |stream: WebSocket| {
                    println!("Socket connected");
                    let (mut write, mut read) = stream.split();

                    write
                        .send(ws::Message::Text("Hello from server".into()))
                        .await
                        .expect("Failed to send message");

                    while let Some(msg) = read.next().await {
                        let msg = msg.expect("Failed to read message");
                        write.send(msg).await.expect("Failed to send message");
                    }
                });

                tokio::spawn(future);

                bycat_error::Result::Ok(resp)
            } else {
                Ok(Response::builder()
                    .status(200)
                    .header(CONTENT_TYPE, "text/html")
                    .body(Body::from(include_str!("./ws.html")))
                    .unwrap())
            }
        })
        .into_response(),
    )
    .await
    .unwrap();

    Ok(())
}
