# rosu-render

Rust wrapper for the [API and websocket](https://ordr.issou.best/docs/) of [`o!rdr`] to render [`osu!`] replays.

## Usage

```rust
use rosu_render::{
    model::{RenderSkinOption, Verification},
    OrdrClient, OrdrWebsocket,
};

#[tokio::main]
async fn main() {
    // In production, use your key as verification or omit verification entirely.
    let client = OrdrClient::builder().verification(Verification::DevModeSuccess).build();
    let mut websocket = OrdrWebsocket::connect().await.expect("Failed to connect websocket");

    // The channel lets us notify the websocket when to disconnect
    let (disconnect_tx, mut disconnect_rx) = tokio::sync::oneshot::channel::<()>();

    // Handle websocket events in a different task
    let websocket_handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                event_res = websocket.next_event() => {
                    match event_res {
                        Ok(event) => println!("{event:?}"),
                        Err(err) => println!("Websocket error: {err:?}"),
                    }
                },
                _ = &mut disconnect_rx => {
                    println!("Received disconnect notification");
                    websocket.disconnect().await.expect("Failed to disconnect gracefully");

                    return;
                }
            }
        }
    });

    // Requesting from the API

    let render_list = client
        .render_list()
        .page_size(2)
        .await
        .expect("Failed to get render list");
    println!("{render_list:#?}");

    let skin_list = client
        .skin_list()
        .page_size(3)
        .page(2)
        .await
        .expect("Failed to get skin list");
    println!("{skin_list:#?}");

    let server_list_count = client
        .server_online_count()
        .await
        .expect("Failed to get server list count");
    println!("{server_list_count:?}");

    let replay_file = tokio::fs::read("./assets/2283307549.osr")
        .await
        .expect("Failed to get replay file");
    let skin = RenderSkinOption::default();
    let render = client
        .render_with_replay_file(&replay_file, "your_name", &skin)
        .await
        .expect("Failed to commission replay render");
    println!("{render:#?}");

    // Now the websocket will receive events for your commissioned replay render

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // Notify the websocket to disconnect
    let _ = disconnect_tx.send(());
    websocket_handle.await.expect("websocket worker panicked");

    println!("Shutting down");
}
```

## Features

* `native`: platform's native TLS implementation via [`native-tls`]
* `rustls-native-roots`: [`rustls`] using native root certificates
* `rustls-webpki-roots` (*default*): [`rustls`] using [`webpki-roots`] for root certificates

[`o!rdr`]: https://ordr.issou.best/
[`osu`]: https://osu.ppy.sh/home
[`native-tls`]: https://crates.io/crates/native-tls
[`rustls`]: https://crates.io/crates/rustls
[`webpki-roots`]: https://crates.io/crates/webpki-roots