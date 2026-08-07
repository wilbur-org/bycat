use crate::Error;
use crate::{
    FromRequestParts,
    body::HttpBody,
    util::{header_contains, header_eq},
    ws::{
        callback::Callback, error::WebsocketError, get_stream::GetWebSocketStream,
        websocket::WebSocket,
    },
};
use alloc::{
    borrow::Cow,
    collections::BTreeSet,
    task::{Poll, ready},
};
use bytes::Bytes;
use http::Request;
use http::{HeaderValue, Method, Response, StatusCode, Version, header, request::Parts};
use hyper::upgrade::{OnUpgrade, Upgraded};
use pin_project_lite::pin_project;
use sha1::{Digest, Sha1};
use tungstenite::protocol::{Role, WebSocketConfig};

/// What to do when a connection upgrade fails.
///
/// See [`WebSocketUpgrade::on_failed_upgrade`] for more details.
pub trait OnFailedUpgrade: Send + 'static {
    /// Call the callback.
    fn call(self, error: hyper::Error);
}

impl<F> OnFailedUpgrade for F
where
    F: FnOnce(hyper::Error) + Send + 'static,
{
    fn call(self, error: hyper::Error) {
        self(error)
    }
}

/// The default `OnFailedUpgrade` used by `WebSocketUpgrade`.
///
/// It simply ignores the error.
#[non_exhaustive]
#[derive(Debug)]
pub struct DefaultOnFailedUpgrade;

impl OnFailedUpgrade for DefaultOnFailedUpgrade {
    #[inline]
    fn call(self, _error: hyper::Error) {}
}

pub struct WebSocketUpgrade<F = DefaultOnFailedUpgrade> {
    config: WebSocketConfig,
    /// The chosen protocol sent in the `Sec-WebSocket-Protocol` header of the response.
    protocol: Option<HeaderValue>,
    /// `None` if HTTP/2+ WebSockets are used.
    sec_websocket_key: Option<HeaderValue>,
    on_upgrade: hyper::upgrade::OnUpgrade,
    on_failed_upgrade: F,
    sec_websocket_protocol: BTreeSet<HeaderValue>,
}

impl<F> WebSocketUpgrade<F> {
    pub fn protocols<I>(mut self, protocols: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Cow<'static, str>>,
    {
        self.protocol = protocols
            .into_iter()
            .map(Into::into)
            .find(|proto| {
                // FIXME: When https://github.com/hyperium/http/pull/814
                //        is merged + released, we can look use
                //        `contains(proto.as_bytes())` without converting
                //        to `HeaderValue` first.
                let Ok(proto) = HeaderValue::from_str(proto) else {
                    return false;
                };
                self.sec_websocket_protocol.contains(&proto)
            })
            .map(|protocol| match protocol {
                Cow::Owned(s) => HeaderValue::from_str(&s).unwrap(),
                Cow::Borrowed(s) => HeaderValue::from_static(s),
            });

        self
    }

    /// Return the WebSocket subprotocols requested by the client.
    ///
    /// # Examples
    ///
    /// If the client sends the following HTTP header in the WebSocket upgrade request:
    ///
    /// ```txt
    /// Sec-WebSocket-Protocol: soap, wamp
    /// ```
    ///
    /// this method returns an iterator yielding `"soap"` and `"wamp"`.
    pub fn requested_protocols(&self) -> impl Iterator<Item = &HeaderValue> {
        self.sec_websocket_protocol.iter()
    }

    /// Set the chosen WebSocket subprotocol.
    ///
    /// Another method, [`protocols()`][Self::protocols], also sets the chosen WebSocket
    /// subprotocol. If both methods are called, only the latter call takes effect.
    ///
    /// # Notes
    ///
    /// - The chosen protocol is echoed back in the WebSocket upgrade
    ///   response as required by RFC 6455. Some browsers may reject a
    ///   value that was not present in the client's request.
    pub fn set_selected_protocol(&mut self, protocol: HeaderValue) {
        self.protocol = Some(protocol);
    }

    /// Return the selected WebSocket subprotocol, if one has been chosen.
    ///
    /// If [`protocols()`][Self::protocols] selects a matching protocol, or
    /// [`set_selected_protocol()`][Self::set_selected_protocol] has been called, the return
    /// value will be `Some` containing the selected protocol. Otherwise, it will be `None`.
    pub fn selected_protocol(&self) -> Option<&HeaderValue> {
        self.protocol.as_ref()
    }

    /// Provide a callback to call if upgrading the connection fails.
    ///
    /// The connection upgrade is performed in a background task. If that fails this callback
    /// will be called.
    ///
    /// By default any errors will be silently ignored.
    ///
    /// # Example
    ///
    /// ```
    /// use axum::{
    ///     extract::{WebSocketUpgrade},
    ///     response::Response,
    /// };
    ///
    /// async fn handler(ws: WebSocketUpgrade) -> Response {
    ///     ws.on_failed_upgrade(|error| {
    ///         report_error(error);
    ///     })
    ///     .on_upgrade(|socket| async { /* ... */ })
    /// }
    /// #
    /// # fn report_error(_: axum::Error) {}
    /// ```
    pub fn on_failed_upgrade<C>(self, callback: C) -> WebSocketUpgrade<C>
    where
        C: OnFailedUpgrade,
    {
        WebSocketUpgrade {
            config: self.config,
            protocol: self.protocol,
            sec_websocket_key: self.sec_websocket_key,
            on_upgrade: self.on_upgrade,
            on_failed_upgrade: callback,
            sec_websocket_protocol: self.sec_websocket_protocol,
        }
    }

    /// Finalize upgrading the connection and call the provided callback with
    /// the stream.
    #[must_use = "to set up the WebSocket connection, this response must be returned"]
    pub fn on_upgrade<C, B>(self, callback: C) -> (Response<B>, WebSocketHandlerFuture<C, F>)
    where
        C: Callback,
        F: OnFailedUpgrade,
        B: HttpBody,
    {
        let on_upgrade = self.on_upgrade;
        let config = self.config;
        let on_failed_upgrade = self.on_failed_upgrade;

        let protocol = self.protocol.clone();

        let future = WebSocketHandlerFuture {
            protocol,
            func: Some(callback),
            config: Some(config),
            state: UpgradeFutureState::Upgrade { future: on_upgrade },
            on_failed_upgrade: Some(on_failed_upgrade),
        };

        let mut response = if let Some(sec_websocket_key) = &self.sec_websocket_key {
            // If `sec_websocket_key` was `Some`, we are using HTTP/1.1.

            #[allow(clippy::declare_interior_mutable_const)]
            const UPGRADE: HeaderValue = HeaderValue::from_static("upgrade");
            #[allow(clippy::declare_interior_mutable_const)]
            const WEBSOCKET: HeaderValue = HeaderValue::from_static("websocket");

            Response::builder()
                .status(StatusCode::SWITCHING_PROTOCOLS)
                .header(header::CONNECTION, UPGRADE)
                .header(header::UPGRADE, WEBSOCKET)
                .header(
                    header::SEC_WEBSOCKET_ACCEPT,
                    sign(sec_websocket_key.as_bytes()),
                )
                .body(B::empty())
                .unwrap()
        } else {
            // Otherwise, we are HTTP/2+. As established in RFC 9113 section 8.5, we just respond
            // with a 2XX with an empty body:
            // <https://datatracker.ietf.org/doc/html/rfc9113#name-the-connect-method>.
            Response::new(B::empty())
        };

        if let Some(protocol) = self.protocol {
            response
                .headers_mut()
                .insert(header::SEC_WEBSOCKET_PROTOCOL, protocol);
        }

        (response, future)
    }
}

impl WebSocketUpgrade {
    pub fn from_request_parts(parts: &mut Parts) -> Result<Self, Error> {
        let sec_websocket_key = if parts.version <= Version::HTTP_11 {
            if parts.method != Method::GET {
                return Err(WebsocketError::MethodNotGet.into());
            }

            if !header_contains(&parts.headers, header::CONNECTION, "upgrade") {
                return Err(WebsocketError::InvalidConnectionHeader.into());
            }

            if !header_eq(&parts.headers, header::UPGRADE, "websocket") {
                return Err(WebsocketError::InvalidUpgradeHeader.into());
            }

            Some(
                parts
                    .headers
                    .get(header::SEC_WEBSOCKET_KEY)
                    .ok_or(WebsocketError::WebSocketKeyHeaderMissing)?
                    .clone(),
            )
        } else {
            if parts.method != Method::CONNECT {
                return Err(WebsocketError::MethodNotConnect.into());
            }

            // if this feature flag is disabled, we won’t be receiving an HTTP/2 request to begin
            // with.
            #[cfg(feature = "http2")]
            if parts
                .extensions
                .get::<hyper::ext::Protocol>()
                .map_or(true, |p| p.as_str() != "websocket")
            {
                return Err(InvalidProtocolPseudoheader.into());
            }

            None
        };

        if !header_eq(&parts.headers, header::SEC_WEBSOCKET_VERSION, "13") {
            return Err(WebsocketError::InvalidWebSocketVersionHeader.into());
        }

        let on_upgrade = parts
            .extensions
            .remove::<hyper::upgrade::OnUpgrade>()
            .ok_or(WebsocketError::ConnectionNotUpgradable)?;

        let sec_websocket_protocol = parts
            .headers
            .get_all(header::SEC_WEBSOCKET_PROTOCOL)
            .iter()
            .flat_map(|val| val.as_bytes().split(|&b| b == b','))
            .map(|proto| {
                HeaderValue::from_bytes(proto.trim_ascii())
                    .expect("substring of HeaderValue is valid HeaderValue")
            })
            .collect();

        Ok(Self {
            config: Default::default(),
            protocol: None,
            sec_websocket_key,
            on_upgrade,
            sec_websocket_protocol,
            on_failed_upgrade: DefaultOnFailedUpgrade,
        })
    }
}

impl<S> FromRequestParts<S> for WebSocketUpgrade<DefaultOnFailedUpgrade>
where
    S: Send + Sync,
{
    type Future<'a>
        = core::future::Ready<Result<Self, Error>>
    where
        S: 'a;

    fn from_request_parts<'a>(parts: &'a mut Parts, _state: &'a S) -> Self::Future<'a> {
        core::future::ready(Self::from_request_parts(parts))
    }
}

fn sign(key: &[u8]) -> HeaderValue {
    use base64::engine::Engine as _;

    let mut sha1 = Sha1::default();
    sha1.update(key);
    sha1.update(&b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11"[..]);
    let b64 = Bytes::from(base64::engine::general_purpose::STANDARD.encode(sha1.finalize()));
    HeaderValue::from_maybe_shared(b64).expect("base64 is a valid value")
}

pin_project! {
    #[project = UpgradeFutureProj]
    enum UpgradeFutureState<F> {
        Upgrade { #[pin] future: OnUpgrade },
        Stream {
            #[pin]
            future: GetWebSocketStream<Upgraded>
        },
        Call { #[pin] future: F },
    }
}

pin_project! {

    pub struct WebSocketHandlerFuture<T: Callback, F> {
        func: Option<T>,
        protocol: Option<HeaderValue>,
        config: Option<WebSocketConfig>,
        #[pin]
        state: UpgradeFutureState<T::Future>,
        on_failed_upgrade: Option<F>,
    }
}

impl<T: Callback, F: OnFailedUpgrade> Future for WebSocketHandlerFuture<T, F> {
    type Output = ();

    fn poll(
        mut self: alloc::pin::Pin<&mut Self>,
        cx: &mut alloc::task::Context<'_>,
    ) -> alloc::task::Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            match this.state.as_mut().project() {
                UpgradeFutureProj::Upgrade { future } => {
                    //
                    let ret = match ready!(future.poll(cx)) {
                        Ok(ret) => ret,
                        Err(err) => {
                            if let Some(failed_upgrade) = this.on_failed_upgrade.take() {
                                failed_upgrade.call(err);
                            }
                            return Poll::Ready(());
                        }
                    };

                    this.state.set(UpgradeFutureState::Stream {
                        future: GetWebSocketStream::new(ret, Role::Server, this.config.take()),
                    });
                }
                UpgradeFutureProj::Stream { future } => {
                    let stream = ready!(future.poll(cx));
                    let protocol = this.protocol.take();

                    let socket = WebSocket {
                        socket: stream,
                        protocol,
                    };

                    let func = this.func.take().expect("callback");

                    this.state.set(UpgradeFutureState::Call {
                        future: func.call(socket),
                    });
                }
                UpgradeFutureProj::Call { future } => return future.poll(cx),
            }
        }
    }
}
