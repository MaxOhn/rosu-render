use tokio::net::TcpStream;
use tokio_tungstenite::{
    tungstenite::protocol::WebSocketConfig, Connector, MaybeTlsStream, WebSocketStream,
};
use url::Url;

use super::error::EngineIoError;

const WEBSOCKET_CONFIG: WebSocketConfig = WebSocketConfig {
    accept_unmasked_frames: false,
    max_frame_size: None,
    max_message_size: None,
    max_send_queue: None,
};

pub(super) type Connection = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Clone)]
pub(super) struct TlsContainer {
    #[allow(unused)]
    tls: Option<r#impl::TlsConnector>,
}

impl TlsContainer {
    pub(super) fn new() -> Result<Self, EngineIoError> {
        r#impl::new()
    }

    pub(super) async fn connect(&self, url: &Url) -> Result<Connection, EngineIoError> {
        r#impl::connect(url, WEBSOCKET_CONFIG, self).await
    }

    #[allow(unused)]
    pub(super) fn connector(&self) -> Option<Connector> {
        r#impl::connector(self)
    }
}

#[cfg(any(feature = "rustls-native-roots", feature = "rustls-webpki-roots"))]
mod r#impl {
    //! Rustls

    use rustls_tls::ClientConfig;
    use std::sync::Arc;
    use tokio_tungstenite::{tungstenite::protocol::WebSocketConfig, Connector};
    use url::Url;

    use crate::websocket::engineio::error::EngineIoError;

    use super::{Connection, TlsContainer};

    pub(super) type TlsConnector = Arc<ClientConfig>;

    #[cfg(any(feature = "rustls-native-roots", feature = "rustls-webpki-roots"))]
    pub(super) fn new() -> Result<TlsContainer, EngineIoError> {
        let mut roots = rustls_tls::RootCertStore::empty();

        #[cfg(feature = "rustls-native-roots")]
        {
            let certs = rustls_native_certs::load_native_certs()
                .map_err(|err| EngineIoError::LoadingTls(Box::new(err)))?;

            for cert in certs {
                roots
                    .add(&rustls_tls::Certificate(cert.0))
                    .map_err(|err| EngineIoError::LoadingTls(Box::new(err)))?;
            }
        }

        #[cfg(feature = "rustls-webpki-roots")]
        {
            roots.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
                rustls_tls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                    ta.subject,
                    ta.spki,
                    ta.name_constraints,
                )
            }));
        };

        let config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(roots)
            .with_no_client_auth();

        Ok(TlsContainer {
            tls: Some(Arc::new(config)),
        })
    }

    pub(super) async fn connect(
        url: &Url,
        config: WebSocketConfig,
        tls: &TlsContainer,
    ) -> Result<Connection, EngineIoError> {
        let (stream, _) = tokio_tungstenite::connect_async_tls_with_config(
            url,
            Some(config),
            false,
            tls.connector(),
        )
        .await
        .map_err(EngineIoError::Reconnect)?;

        Ok(stream)
    }

    pub(super) fn connector(container: &TlsContainer) -> Option<Connector> {
        container
            .tls
            .as_ref()
            .map(|tls| Connector::Rustls(Arc::clone(tls)))
    }
}

#[cfg(all(
    feature = "native",
    not(any(feature = "rustls-native-roots", feature = "rustls-webpki-roots"))
))]
mod r#impl {
    //! Native TLS

    pub(super) use native_tls::TlsConnector;
    use tokio_tungstenite::{tungstenite::protocol::WebSocketConfig, Connector};
    use url::Url;

    use super::{Connection, TlsContainer};

    use crate::websocket::engineio::error::EngineIoError;

    pub(super) fn new() -> Result<TlsContainer, EngineIoError> {
        let native_connector =
            TlsConnector::new().map_err(|err| EngineIoError::LoadingTls(Box::new(err)))?;

        Ok(TlsContainer {
            tls: Some(native_connector),
        })
    }

    pub(super) async fn connect(
        url: &Url,
        config: WebSocketConfig,
        tls: &TlsContainer,
    ) -> Result<Connection, EngineIoError> {
        let (stream, _) = tokio_tungstenite::connect_async_tls_with_config(
            url,
            Some(config),
            false,
            tls.connector(),
        )
        .await
        .map_err(EngineIoError::Reconnect)?;

        Ok(stream)
    }

    pub(super) fn connector(container: &TlsContainer) -> Option<Connector> {
        container
            .tls
            .as_ref()
            .map(|tls| Connector::NativeTls(tls.clone()))
    }
}

#[cfg(not(any(
    feature = "native",
    feature = "rustls-native-roots",
    feature = "rustls-webpki-roots"
)))]
mod r#impl {
    //! Plain connections with no TLS.

    pub(super) type TlsConnector = ();
    use tokio_tungstenite::{tungstenite::protocol::WebSocketConfig, Connector};
    use url::Url;

    use crate::websocket::engineio::EngineIoError;

    use super::{Connection, TlsContainer};

    pub(super) fn new() -> Result<TlsContainer, EngineIoError> {
        Ok(TlsContainer { tls: None })
    }

    pub(super) async fn connect(
        url: &Url,
        config: WebSocketConfig,
        _tls: &TlsContainer,
    ) -> Result<Connection, EngineIoError> {
        let (stream, _) = tokio_tungstenite::connect_async_with_config(url, Some(config), false)
            .await
            .map_err(EngineIoError::Reconnect)?;

        Ok(stream)
    }

    pub(super) fn connector(_: &TlsContainer) -> Option<Connector> {
        None
    }
}
