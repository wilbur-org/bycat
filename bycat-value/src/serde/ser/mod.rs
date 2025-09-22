mod bytes;
mod error;
mod list;
mod map;
mod number;
mod serializer;
mod string;
mod time;
mod value;

pub trait HasSerializer {
    type Serializer: serde::ser::Serializer<Ok = Self> + Default;
}

pub use self::{error::SerializerError, serializer::to_value};
