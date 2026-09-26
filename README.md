# rosu-render

Rust wrapper for the [API and websocket](https://ordr.issou.best/docs/) of [`o!rdr`] to render [`osu!`] replays.

## Usage

[`examples/full.rs`](https://github.com/MaxOhn/rosu-render/blob/main/examples/full.rs) is a complete example: it commissions a render and follows it until it is done, with a supervisor task that keeps the websocket connected and forwards the events of the render.

The short version:

```rust
use rosu_render::{
    model::{RenderSkinOption, Verification},
    OrdrClient,
};

#[tokio::main]
async fn main() {
    // In production, use your key as verification instead.
    let client = OrdrClient::builder()
        .verification(Verification::DevModeSuccess)
        .build();

    let skin = RenderSkinOption::default();
    let render = client
        .render_with_replay_file(&replay_file, "your_name", &skin)
        .await
        .expect("Failed to commission render");
    println!("Render {} is queued", render.render_id);
    // ...
}
```

Listen for the render's events through the websocket, for example as done in [`examples/full.rs`](https://github.com/MaxOhn/rosu-render/blob/main/examples/full.rs).

## Features

### Crypto provider

Using [`rustls`] for TLS requires configuring a crypto provider via crate
features or manually installing a global default. The default is `rustls-ring`.

#### `rustls-ring`

The `rustls-ring` feature will enable the use of [`ring`] as the crypto
provider. This is recommended for platform compatibility.

#### `rustls-aws_lc_rs`

The `rustls-aws_lc_rs` feature will enable the use of [`aws-lc-rs`] as the
crypto provider. This is recommended for performance and on widely used
platforms.

#### Manual installation

If none of the other crypto providers are enabled, a custom one must be
installed by the application using [`CryptoProvider::install_default`].

### TLS

`rosu-render` has features to enable HTTPS connectivity with [`hyper`]. These
features are mutually exclusive. `rustls-webpki-roots` is enabled by default.

#### `native-tls`

The `native-tls` feature uses a HTTPS connector provided by [`hyper-tls`].

#### `rustls-native-roots`

The `rustls-native-roots` feature uses a HTTPS connector provided by [`hyper-rustls`], which uses
[`rustls`] as the TLS backend, and enables its `native-tokio` feature, which uses [`rustls-native-certs`]
for root certificates. This requires configuring a crypto provider.

#### `rustls-platform-verifier`

The `rustls-platform-verifier` feature uses a HTTPS connector provided by [`hyper-rustls`], which uses
[`rustls`] as the TLS backend, and enables its [`rustls-platform-verifier`] feature, which uses
[`rustls-platform-verifier`] for certificate validation. This requires configuring a crypto provider.

#### `rustls-webpki-roots`

The `rustls-webpki-roots` feature uses a HTTPS connector provided by [`hyper-rustls`], which uses
[`rustls`] as the TLS backend, and enables its `webpki-tokio` feature, which uses [`webpki-roots`]
for root certificates. This requires configuring a crypto provider.

This should be preferred over `rustls-native-roots` in Docker containers based on `scratch`.

This is enabled by default.

### Trust-DNS

The `hickory` feature enables [`hyper-hickory`], which replaces the default
`GaiResolver` in [`hyper`]. [`hyper-hickory`] instead provides a fully async
DNS resolver on the application level.

[`o!rdr`]: https://ordr.issou.best/
[`osu!`]: https://osu.ppy.sh/home

[`CryptoProvider::install_default`]: https://docs.rs/rustls/latest/rustls/crypto/struct.CryptoProvider.html#method.install_default
[`aws-lc-rs`]: https://crates.io/crates/aws-lc-rs
[`hyper`]: https://crates.io/crates/hyper
[`hyper-hickory`]: https://crates.io/crates/hyper-hickory
[`hyper-rustls`]: https://crates.io/crates/hyper-rustls
[`hyper-tls`]: https://crates.io/crates/hyper-tls
[`ring`]: https://crates.io/crates/ring
[`rustls`]: https://crates.io/crates/rustls
[`rustls-native-certs`]: https://crates.io/crates/rustls-native-certs
[`rustls-platform-verifier`]: https://crates.io/crates/rustls-platform-verifier
[`webpki-roots`]: https://crates.io/crates/webpki-roots
