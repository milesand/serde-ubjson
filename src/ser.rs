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
    where W: Write,
          T: Serialize
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
            .map_err(|err| Error::Io(err))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        if (i8::min_value() as i16 <= v) && (v <= i8::max_value() as i16) {
            self.serialize_i8(v as i8)
        } else {
            self.inner
                .write_u8(b'I')
                .and_then(|_| self.inner.write_i16::<BigEndian>(v))
                .map_err(|err| Error::Io(err))
        }
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        if (i16::min_value() as i32 <= v) && (v <= i16::max_value() as i32) {
            self.serialize_i16(v as i16)
        } else {
            self.inner
                .write_u8(b'l')
                .and_then(|_| self.inner.write_i32::<BigEndian>(v))
                .map_err(|err| Error::Io(err))
        }
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        if (i32::min_value() as i64 <= v) && (v <= i32::max_value() as i64) {
            self.serialize_i32(v as i32)
        } else {
            self.inner
                .write_u8(b'L')
                .and_then(|_| self.inner.write_i64::<BigEndian>(v))
                .map_err(|err| Error::io(err))
        }
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.inner
            .write_u8(b'U')
            .and_then(|_| self.inner.write_u8(v))
            .map_err(|err| Error::Io(err))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        if v <= u8::max_value() as u16 {
            self.serialize_u8(v as u8)
        } else if v <= i16::max_value() as u16 {
            self.serialize_i16(v as i16)
        } else {
            self.serialize_i32(v as i32)
        }
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        if v <= u16::max_value() as u32 {
            self.serialize_u16(v as u16)
        } else if v <= i32::max_value() as u32 {
            self.serialize_i32(v as i32)
        } else {
            self.serialize_i64(v as i64)
        }
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        if v <= u32::max_value() as u64 {
            self.serialize_u32(v as u32)
        } else if v <= i64::max_value() as u64 {
            self.serialize_i64(v as i64)
        } else {
            let v = v.to_string();
            self.inner
                .write_u8(b'H')
                .map_err(|err| Error::Io(err))
                .and_then(|_| self.serialize_u64(v.len()))
                .and_then(|_| self.inner.write(v.as_bytes()).map_err(|err| Error::Io(err)))
        }
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.inner
            .write_u8(b'd')
            .and_then(|_| self.inner.write_f32::<BigEndian>(v))
            .map_err(|err| Error::Io(err))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.inner
            .write_u8(b'D')
            .and_then(|_| self.inner.write_f64::<BigEndian>(v))
            .map_err(|err| Error::Io(err))
    }

    fn serialize_char(self, v: char) -> Result<()> {
        let v: u32 = v.into();
        if v <= u8::max_value() as u32 {
            self.inner
                .write_u8(b'C')
                .and_then(|_| self.inner.write_u8(v as u8))
                .map_err(|err| Error::Io(err))
        } else {
            self.serialize_u32(v)
        }
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.inner
            .write_u8(b'S')
            .map_err(|err| Error::Io(err))
            .and_then(|_| self.serialize_u64(v.len() as u64))
            .and_then(|_| {
                          self.inner
                              .write(v.as_bytes())
                              .map(|_| ())
                              .map_err(|err| Error::Io(err))
                      })
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.inner
            .write(b"[$U")
            .map(|_| ())
            .map_err(|err| Error::Io(err))
            .and_then(|_| self.serialize_u64(v.len() as u64))
            .and_then(|_| {
                          self.inner
                              .write(v)
                              .map(|_| ())
                              .map_err(|err| Error::Io(err))
                      })
    }

    fn serialize_none(self) -> Result<()> {
        self.inner.write_u8(b'Z').map_err(|err| Error::Io(err))
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()>
        where T: Serialize
    {
        unimplemented!()
    }

    fn serialize_unit(self) -> Result<()> {
        unimplemented!()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
        unimplemented!()
    }

    fn serialize_unit_variant(self,
                              name: &'static str,
                              variant_index: u32,
                              variant: &'static str)
                              -> Result<()> {
        unimplemented!()
    }

    fn serialize_newtype_struct<T: ?Sized>(self, name: &'static str, value: &T) -> Result<()>
        where T: Serialize
    {
        unimplemented!()
    }

    fn serialize_newtype_variant<T: ?Sized>(self,
                                            name: &'static str,
                                            variant_index: u32,
                                            variant: &'static str,
                                            value: &T)
                                            -> Result<()>
        where T: Serialize
    {
        unimplemented!()
    }
}
