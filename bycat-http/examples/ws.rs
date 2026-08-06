use bycat_http::{
    Html,
    error::Result,
    handler,
    router::SendRouterBuilder,
    ws::{self, WebSocket},
};
use futures::{SinkExt, StreamExt};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let router = SendRouterBuilder::new()
        .with_get("/", Html(include_str!("./ws.html")))?
        .with_get(
            "/ws",
            handler(async |upgrade: ws::WebSocketUpgrade| {
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

                resp
            }),
        )?
        .build();

    bycat_http::serve(("localhost", 3000), (), router)
        .await
        .unwrap();

    Ok(())
}
