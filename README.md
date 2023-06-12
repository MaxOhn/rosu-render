# rosu-render

Rust wrapper for the [API](https://ordr.issou.best/docs/) of [o!rdr](https://ordr.issou.best/) to render [osu!](https://osu.ppy.sh/home) replays.

## Usage

```rust
use rosu_render::{
    model::{RenderSkinOption, Verification},
    Ordr,
};
use std::future::ready;

let ordr = Ordr::builder()
    .verification(Verification::Key("my_key".into()))
    .with_websocket(true) // defaults to true so this line can be omitted
    .on_render_progress(|progress| {
        Box::pin(ready(println!(
            "{id}: {text}",
            id = progress.render_id,
            text = progress.progress
        )))
    })
    .on_render_done(|done| {
        Box::pin(ready(println!(
            "{id}: URL={url}",
            id = done.render_id,
            url = done.video_url
        )))
    })
    .on_render_failed(|failed| {
        Box::pin(ready(println!(
            "{id}: Failed: {msg}",
            id = failed.render_id,
            msg = failed.error_msg
        )))
    })
    .build()
    .await
    .expect("Failed to connect websocket");

let render_list = ordr.render_list().page_size(2).await.expect("Failed to get render list");
println!("{render_list:#?}");

let skin_list = ordr.skin_list().page_size(3).page(2).await.expect("Failed to get skin list");
println!("{skin_list:#?}");

let server_list_count = ordr.server_online_count().await.expect("Failed to get server list count");
println!("Server list count: {server_list_count}");

let replay_file = tokio::fs::read("./4165361249.osr").await.expect("Failed to get replay file");
let skin = RenderSkinOption::new("YUGEN", true);
let render = ordr
    .render_with_replay_file(&replay_file, "your_name", &skin)
    .await
    .expect("Failed to commission replay render");
println!("{render:#?}");

// Now the websocket will receive events for your commissioned replay render

tokio::signal::ctrl_c().await.unwrap();

// To disconnect the websocket properly before dropping the client, call this method
ordr.disconnect().await.unwrap();

println!("Shutting down");
```

## Status

Although this library is fully functional, it is lacking tests, CI, and more examples. It's also missing a few optional features like flexible TLS connectors for socket.io; it currently always uses `rustls-webpki-roots`.

Since the underlying socket.io wrapper is only a fork that's not pushed to crates.io, this library cannot be published and must currently be imported via 
```toml
rosu_render = { git = "https://github.com/MaxOhn/rosu-render" }
```
