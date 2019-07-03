pub mod error;
mod marker;
pub mod ser;

pub use error::{Error, Result};
pub use ser::{to_vec, to_writer, Serializer};
