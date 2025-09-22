use alloc::string::ToString;
use core::fmt;

#[derive(Debug)]
pub enum SerializerError {
    Custom(alloc::string::String),
}

impl fmt::Display for SerializerError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SerializerError::Custom(ref s) => fmt.write_str(s),
        }
    }
}

impl core::error::Error for SerializerError {}

impl serde::ser::Error for SerializerError {
    fn custom<T: fmt::Display>(msg: T) -> SerializerError {
        SerializerError::Custom(msg.to_string())
    }
}
