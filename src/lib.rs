extern crate serde;
extern crate byteorder;

mod error;
mod ser;

pub use error::{Error, Result};
pub use ser::{to_vec, Serializer};
