//! Serialize a Rust data structure into UBJSON data.

use std::io::Write;

use byteorder::{BigEndian, ByteOrder, WriteBytesExt};
use serde::ser::{self, Serialize};

use error::{Error, Result};

/// Serialize the given value as a UBJSON byte vector.
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
    where T: Serialize
{
    let mut serializer = Serializer::new(Vec::new());
    value.serialize(&mut serializer)?;
    Ok(serializer.into_inner())
}

/// Serialize the given value as UBJSON into the IO stream.
pub fn to_writer<T, W>(writer: W, value: &T) -> Result<()>
    where W: Write, T: Serialize
{
    let mut serializer = Serializer::new(writer);
    value.serialize(&mut serializer)?;
    Ok(())
}

/// Structure for serializing Rust values into UBJSON.
pub struct Serializer<W> {
    inner: W,
}

impl<W> Serializer<W>
    where W: Write
{
    /// Creates a new UBJSON serializer.
    pub fn new(writer: W) -> Self {
        Serializer { inner: writer }
    }

    /// Consumes the serializer and returns the writer it wrapped.
    fn into_inner(self) -> W {
        self.inner
    }
}

impl<'a, W> ser::Serializer for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.inner
            .write_u8(match v {
                          true => b'T',
                          false => b'F',
                      })
            .map_err(|err| Error::Io(err))
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.inner
            .write_u8(b'i')
            .and_then(|_| self.inner.write_i8(v))
            .map_err(|err| Error::io(err))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.inner
            .write_u8(b'I')
            .and_then(|_| self.inner.write_i16::<BigEndian>(v))
            .map_err(|err| Error::io(err))
    }
    
    fn serialize_i32(self, v: i32) -> Result<()> {
        self.inner
            .write_u8(b'l')
            .and_then(|_| self.inner.write_i32::<BigEndian>(v))
            .map_err(|err| Error::io(err))
    }
    
    fn serialize_i64(self, v: i64) -> Result<()> {
        self.inner
            .write_u8(b'L')
            .and_then(|_| self.inner.write_i64::<BigEndian>(v))
            .map_err(|err| Error::io(err))
    }
            
    fn serialize_u8(self, v: u8) -> Result<()> {
        self.inner
            .write_u8(b'U')
            .and_then(|_| self.inner.write_u8(v))
            .map_err(|err| Error::io(err))
    }
    
    fn serialize_u16(self, v: u16) -> Result<()> {
        unimplemented!()
    }
    
    fn serialize_u32(self, v: u32) -> Result<()> {
        unimplemented!()
    }
    
    fn serialize_u64(self, v: u64) -> Result<()> {
        unimplemented!()
    }
}
