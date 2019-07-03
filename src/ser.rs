//! Serialize a Rust data structure into UBJSON data.

use std::io::Write;

use byteorder::{BigEndian, WriteBytesExt};
use serde::ser::{self, Impossible, Serialize};

use crate::marker;
use crate::error::{Error, Result};

////////////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////////////

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

impl<'a, W: Write> ser::Serializer for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Dynamic<'a, W>;
    type SerializeTuple = Static<'a, W>;
    type SerializeTupleStruct = Static<'a, W>;
    type SerializeTupleVariant = Static<'a, W>;
    type SerializeMap = Dynamic<'a, W>;
    type SerializeStruct = Static<'a, W>;
    type SerializeStructVariant = Static<'a, W>;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.inner.write_u8(if v { marker::TRUE } else { marker::FALSE }).map_err(Error::Io)
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.inner.write_u8(marker::I8)?;
        self.inner.write_i8(v)?;
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        if (i16::from(i8::min_value()) <= v) && (v <= i16::from(i8::max_value())) {
            self.serialize_i8(v as i8)
        } else if (i16::from(u8::min_value()) <= v) && (v <= i16::from(u8::max_value())) {
            self.serialize_u8(v as u8)
        } else {
            self.inner.write_u8(marker::I16)?;
            self.inner.write_i16::<BigEndian>(v)?;
            Ok(())
        }
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        if (i32::from(i16::min_value()) <= v) && (v <= i32::from(i16::max_value())) {
            self.serialize_i16(v as i16)
        } else {
            self.inner.write_u8(marker::I32)?;
            self.inner.write_i32::<BigEndian>(v)?;
            Ok(())
        }
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        if (i64::from(i32::min_value()) <= v) && (v <= i64::from(i32::max_value())) {
            self.serialize_i32(v as i32)
        } else {
            self.inner.write_u8(marker::I64)?;
            self.inner.write_i64::<BigEndian>(v)?;
            Ok(())
        }
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.inner.write_u8(marker::U8)?;
        self.inner.write_u8(v)?;
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        if v <= u16::from(u8::max_value()) {
            self.serialize_u8(v as u8)
        } else if v <= i16::max_value() as u16 {
            self.serialize_i16(v as i16)
        } else {
            self.serialize_i32(i32::from(v))
        }
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        if v <= u32::from(u16::max_value()) {
            self.serialize_u16(v as u16)
        } else if v <= i32::max_value() as u32 {
            self.serialize_i32(v as i32)
        } else {
            self.serialize_i64(i64::from(v))
        }
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        if v <= u64::from(u32::max_value()) {
            self.serialize_u32(v as u32)
        } else if v <= i64::max_value() as u64 {
            self.serialize_i64(v as i64)
        } else {
            let v = v.to_string();
            self.inner.write_u8(marker::HI_PRECISION)?;
            self.serialize_u64(v.len() as u64)?;
            self.inner.write_all(v.as_bytes())?;
            Ok(())
        }
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.inner.write_u8(marker::F32)?;
        self.inner.write_f32::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.inner.write_u8(marker::F64)?;
        self.inner.write_f64::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        let v: u32 = v.into();
        if v <= 127 {
            self.inner.write_u8(marker::CHAR)?;
            self.inner.write_u8(v as u8)?;
            Ok(())
        } else {
            self.serialize_u32(v)
        }
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.inner.write_u8(marker::STRING)?;
        self.serialize_u64(v.len() as u64)?;
        self.inner.write_all(v.as_bytes())?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        let header = [marker::ARR_START, marker::TYPE, marker::U8, marker::LENGTH];
        self.inner.write_all(&header)?;
        self.serialize_u64(v.len() as u64)?;
        self.inner.write_all(v).map_err(Error::Io)?;
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.inner.write_u8(marker::NULL).map_err(Error::Io)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()>
        where T: Serialize
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_none()
    }

    fn serialize_unit_variant(self,
                              _name: &'static str,
                              variant_index: u32,
                              _variant: &'static str)
                              -> Result<()> {
        self.serialize_u32(variant_index)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<()>
        where T: Serialize
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(self,
                                            _name: &'static str,
                                            variant_index: u32,
                                            _variant: &'static str,
                                            value: &T)
                                            -> Result<()>
        where T: Serialize
    {
        let mut tup = self.serialize_tuple(2)?;
        ser::SerializeTuple::serialize_element(&mut tup, &variant_index)?;
        ser::SerializeTuple::serialize_element(&mut tup, value)?;
        ser::SerializeTuple::end(tup)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.inner.write_u8(marker::ARR_START)?;
        if let Some(len) = len {
            self.inner.write_u8(marker::LENGTH)?;
            len.serialize(&mut *self)?;
            Ok(Dynamic {
                   ser: self,
                   length_known: true,
            })
        } else {
            Ok(Dynamic {
                   ser: self,
                   length_known: false,
            })
        }
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        let header = [marker::ARR_START, marker::LENGTH];
        self.inner.write_all(&header)?;
        self.serialize_u64(len as u64)?;
        Ok(Static { ser: self })
    }

    fn serialize_tuple_struct(self,
                              _name: &'static str,
                              len: usize)
                              -> Result<Self::SerializeTupleStruct> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(self,
                               _name: &'static str,
                               variant_index: u32,
                               _variant: &'static str,
                               len: usize)
                               -> Result<Self::SerializeTupleVariant> {
        let mut tup = self.serialize_tuple(len + 1)?;
        ser::SerializeTuple::serialize_element(&mut tup, &variant_index)?;
        Ok(tup)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        self.inner.write_u8(marker::OBJ_START)?;
        if let Some(len) = len {
            self.inner.write_u8(marker::LENGTH)?;
            len.serialize(&mut *self)?;
            Ok(Dynamic {
                   ser: self,
                   length_known: true,
            })
        } else {
            Ok(Dynamic {
                   ser: self,
                   length_known: false,
            })
        }
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_tuple(len)
    }

    fn serialize_struct_variant(self,
                                name: &'static str,
                                variant_index: u32,
                                variant: &'static str,
                                len: usize)
                                -> Result<Self::SerializeStructVariant> {
        self.serialize_tuple_variant(name, variant_index, variant, len)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[doc(hidden)]
/// Serialization handler for compound types with non-optional length (i. e. len: usize).
pub struct Static<'a, W: 'a> {
    ser: &'a mut Serializer<W>,
}

impl<'a, W: 'a> ser::SerializeTuple for Static<'a, W>
    where W: Write
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: Serialize
    {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: 'a> ser::SerializeTupleStruct for Static<'a, W>
    where W: Write
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: Serialize
    {
        ser::SerializeTuple::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeTuple::end(self)
    }
}

impl<'a, W: 'a> ser::SerializeTupleVariant for Static<'a, W>
    where W: Write
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: Serialize
    {
        ser::SerializeTuple::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeTuple::end(self)
    }
}

impl<'a, W: 'a> ser::SerializeStruct for Static<'a, W>
    where W: Write
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
        where T: Serialize
    {
        ser::SerializeTuple::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeTuple::end(self)
    }
}

impl<'a, W: 'a> ser::SerializeStructVariant for Static<'a, W>
    where W: Write
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
        where T: Serialize
    {
        ser::SerializeTuple::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeTuple::end(self)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[doc(hidden)]
/// Serialization handler for compound types with optional length (i. e. len: Option<usize>).
pub struct Dynamic<'a, W: 'a> {
    ser: &'a mut Serializer<W>,
    length_known: bool,
}

impl<'a, W: 'a> ser::SerializeSeq for Dynamic<'a, W>
    where W: Write
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: Serialize
    {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<()> {
        if !self.length_known {
            self.ser.inner.write_u8(marker::ARR_END)?;
        }
        Ok(())
    }
}

impl<'a, W: 'a> ser::SerializeMap for Dynamic<'a, W>
    where W: Write
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
        where T: Serialize
    {
        key.serialize(MapKeySerializer { ser: &mut *self.ser })
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: Serialize
    {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<()> {
        if !self.length_known {
            self.ser.inner.write_u8(marker::OBJ_END)?;
        }
        Ok(())
    }
}

struct MapKeySerializer<'a, W: 'a> {
    ser: &'a mut Serializer<W>,
}

impl<'a, W> ser::Serializer for MapKeySerializer<'a, W>
    where W: Write
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Impossible<(), Error>;
    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_str(self, v: &str) -> Result<()> {
        v.len().serialize(&mut *self.ser)?;
        self.ser.inner.write_all(v.as_bytes())?;
        Ok(())
    }

    fn serialize_bool(self, _v: bool) -> Result<()> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_i8(self, _v: i8) -> Result<()> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_i16(self, _v: i16) -> Result<()> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_i32(self, _v: i32) -> Result<()> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_i64(self, _v: i64) -> Result<()> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_u8(self, _v: u8) -> Result<()> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_u16(self, _v: u16) -> Result<()> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_u32(self, _v: u32) -> Result<()> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_u64(self, _v: u64) -> Result<()> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_f32(self, _v: f32) -> Result<()> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_f64(self, _v: f64) -> Result<()> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_char(self, _v: char) -> Result<()> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<()> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_none(self) -> Result<()> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_some<T: ?Sized>(self, _v: &T) -> Result<()>
        where T: Serialize
    {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_unit(self) -> Result<()> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_unit_variant(self,
                              _name: &'static str,
                              _variant_index: u32,
                              _variant: &'static str)
                              -> Result<()> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, _value: &T) -> Result<()>
        where T: Serialize
    {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_newtype_variant<T: ?Sized>(self,
                                            _name: &'static str,
                                            _variant_index: u32,
                                            _variant: &'static str,
                                            _value: &T)
                                            -> Result<()>
        where T: Serialize
    {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_tuple_struct(self,
                              _name: &'static str,
                              _len: usize)
                              -> Result<Self::SerializeTupleStruct> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_tuple_variant(self,
                               _name: &'static str,
                               _variant_index: u32,
                               _variant: &'static str,
                               _len: usize)
                               -> Result<Self::SerializeTupleVariant> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_struct_variant(self,
                                _name: &'static str,
                                _variant_index: u32,
                                _variant: &'static str,
                                _len: usize)
                                -> Result<Self::SerializeStructVariant> {
        Err(Error::KeyMustBeAString)
    }
}
