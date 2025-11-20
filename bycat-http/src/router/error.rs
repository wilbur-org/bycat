use bycat_error::Error;
use core::fmt;

#[derive(Debug)]
pub enum RouteError {
    Parse(routing::ParseError),
    Route(routing::router::RouteError),
}

impl fmt::Display for RouteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RouteError::Parse(err) => write!(f, "Parse error: {}", err),
            RouteError::Route(err) => write!(f, "Route error: {}", err),
        }
    }
}

impl core::error::Error for RouteError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            RouteError::Parse(err) => Some(err),
            RouteError::Route(err) => Some(err),
        }
    }
}

impl From<routing::ParseError> for RouteError {
    fn from(value: routing::ParseError) -> Self {
        RouteError::Parse(value)
    }
}

impl From<routing::router::RouteError> for RouteError {
    fn from(value: routing::router::RouteError) -> Self {
        RouteError::Route(value)
    }
}

impl From<RouteError> for Error {
    fn from(value: RouteError) -> Self {
        Error::new(value)
    }
}
