#[derive(Debug)]
pub enum WebsocketError {
    ConnectionNotUpgradable,
    InvalidWebSocketVersionHeader,
    MethodNotConnect,
    WebSocketKeyHeaderMissing,
    InvalidUpgradeHeader,
    InvalidConnectionHeader,
    MethodNotGet,
}

impl core::fmt::Display for WebsocketError {
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl core::error::Error for WebsocketError {}

impl From<WebsocketError> for crate::Error {
    fn from(value: WebsocketError) -> Self {
        crate::Error::custom(value)
    }
}
