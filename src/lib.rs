extern crate serde;
extern crate byteorder;

pub mod error;
pub mod ser;

pub use error::{Error, Result};
pub use ser::{to_vec, to_writer, Serializer};
