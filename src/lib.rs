#![doc = include_str!("../README.md")]

mod routing;
mod util;

pub mod client;
pub mod model;
pub mod request;

#[cfg(any(
    feature = "native",
    feature = "rustls-native-roots",
    feature = "rustls-webpki-roots"
))]
pub mod websocket;

#[macro_use]
extern crate tracing;

pub use self::client::{error::ClientError, OrdrClient};

#[cfg(any(
    feature = "native",
    feature = "rustls-native-roots",
    feature = "rustls-webpki-roots"
))]
pub use self::websocket::{error::WebsocketError, OrdrWebsocket};
