#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, nonstandard_style, rust_2018_idioms, unused)]
#![allow(
    clippy::module_name_repetitions,
    clippy::explicit_iter_loop,
    clippy::similar_names,
    clippy::missing_errors_doc,
    clippy::struct_excessive_bools,
    clippy::cast_possible_truncation
)]

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
