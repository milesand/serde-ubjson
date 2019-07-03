pub mod ser;
pub mod error;

pub use ser::{to_vec, to_writer, Serializer};
pub use error::{Error, Result};