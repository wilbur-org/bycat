use bycat_executor::TokioExecutor;
use bycat_http::{
    Html,
    body::Body,
    error::Result,
    handler,
    router::{SendRouter, SendRouterBuilder},
    ws::{self, WebSocket},
};
use bycat_task::Work;
use futures::{SinkExt, StreamExt};
use http::Request;
use hyper::{body::Incoming, service::service_fn};
use tokio::{net::TcpListener, task::LocalSet};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let router: SendRouter<(), Body> = SendRouterBuilder::new()
        .with_get("/", Html(include_str!("./ws.html")))?
        .with_get(
            "/ws",
            handler(async |upgrade: ws::WebSocketUpgrade| {
                let (resp, future) = upgrade
                    .on_failed_upgrade(|err| {
                        eprintln!("{err}");
                    })
                    .on_upgrade(async |stream: WebSocket| {
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

    let local_set = LocalSet::new();

    local_set
        .run_until(async move {
            bycat_http::serve::Tokio::new(router)
                .upgradable(true)
                .serve_local((), ("localhost", 3000))
                .await
        })
        .await?;

    // bycat_http::serve(("localhost", 3000), (), router)
    //     .await
    //     .unwrap();

    Ok(())
}
