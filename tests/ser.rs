extern crate serde;
extern crate serde_bytes;
extern crate serde_ubjson;

use serde::Serialize;
use serde_ubjson::Serializer;

macro_rules! test_cases {
    ($(($in:expr, $out:expr)),*$(,)?) => {
        let mut buf: Vec<u8> = Vec::new();
        $(
            buf.clear();
            if let Err(err) = $in.serialize(&mut Serializer::new(&mut buf)) {
                panic!("Serializing {:?} failed with error: {}", $in, err);
            }
            if $out.as_ref() != buf.as_slice() {
                panic!("Wrong result on serializing {:?}: expected {:?}, got {:?}",
                       $in, $out, buf.as_slice());
            }
        )*
    }
}

#[test]
fn serialize_bool() {
    test_cases! {
        (true,  b"T"),
        (false, b"F"),
    }
}

#[test]
fn serialize_i8() {
    test_cases! {
        (i8::min_value(), b"i\x80"),
        (0i8,             b"i\x00"),
        (i8::max_value(), b"i\x7f"),
    }
}

#[test]
fn serialize_i16() {
    test_cases! {
        (i16::min_value(),               b"I\x80\x00"),
        (i16::from(i8::min_value()) - 1, b"I\xff\x7f"),
        (i16::from(i8::min_value()),     b"i\x80"),
        (0i16,                           b"i\x00"),
        (i16::from(i8::max_value()),     b"i\x7f"),
        (i16::from(i8::max_value())+ 1,  b"U\x80"),
        (i16::from(u8::max_value()),     b"U\xff"),
        (i16::from(u8::max_value()) + 1, b"I\x01\x00"),
        (i16::max_value(),               b"I\x7f\xff"),
    }
}

#[allow(clippy::cyclomatic_complexity)]
#[test]
fn serialize_i32() {
    test_cases! {
        (i32::min_value(),                b"l\x80\x00\x00\x00"),
        (i32::from(i16::min_value()) - 1, b"l\xff\xff\x7f\xff"),
        (i32::from(i16::min_value()),     b"I\x80\x00"),
        (i32::from(i8::min_value()) - 1,  b"I\xff\x7f"),
        (i32::from(i8::min_value()),      b"i\x80"),
        (0i32,                            b"i\x00"),
        (i32::from(i8::max_value()),      b"i\x7f"),
        (i32::from(i8::max_value()) + 1,  b"U\x80"),
        (i32::from(u8::max_value()),      b"U\xff"),
        (i32::from(u8::max_value()) + 1,  b"I\x01\x00"),
        (i32::from(i16::max_value()),     b"I\x7f\xff"),
        (i32::from(i16::max_value()) + 1, b"l\x00\x00\x80\x00"),
        (i32::max_value(),                b"l\x7f\xff\xff\xff"),
    }
}

#[allow(clippy::cyclomatic_complexity)]
#[test]
fn serialize_i64() {
    test_cases! {
        (i64::min_value(),                b"L\x80\x00\x00\x00\x00\x00\x00\x00"),
        (i64::from(i32::min_value()) - 1, b"L\xff\xff\xff\xff\x7f\xff\xff\xff"),
        (i64::from(i32::min_value()),     b"l\x80\x00\x00\x00"),
        (i64::from(i16::min_value()) - 1, b"l\xff\xff\x7f\xff"),
        (i64::from(i16::min_value()),     b"I\x80\x00"),
        (i64::from(i8::min_value()) - 1,  b"I\xff\x7f"),
        (i64::from(i8::min_value()),      b"i\x80"),
        (0i64,                            b"i\x00"),
        (i64::from(i8::max_value()),      b"i\x7f"),
        (i64::from(i8::max_value()) + 1,  b"U\x80"),
        (i64::from(u8::max_value()),      b"U\xff"),
        (i64::from(u8::max_value()) + 1,  b"I\x01\x00"),
        (i64::from(i16::max_value()),     b"I\x7f\xff"),
        (i64::from(i16::max_value()) + 1, b"l\x00\x00\x80\x00"),
        (i64::from(i32::max_value()),     b"l\x7f\xff\xff\xff"),
        (i64::from(i32::max_value()) + 1, b"L\x00\x00\x00\x00\x80\x00\x00\x00"),
        (i64::max_value(),                b"L\x7f\xff\xff\xff\xff\xff\xff\xff"),
    }
}

#[test]
fn serialize_u8() {
    test_cases! {
        (u8::min_value(), b"U\x00"),
        (u8::max_value(), b"U\xff"),
    }
}

#[test]
fn serialize_u16() {
    test_cases! {
        (0u16, b"U\x00"),
        (u16::from(u8::max_value()),      b"U\xff"),
        (u16::from(u8::max_value()) + 1,  b"I\x01\x00"),
        (i16::max_value() as u16,         b"I\x7f\xff"),
        (i16::max_value() as u16 + 1,     b"l\x00\x00\x80\x00"),
        (u16::max_value(),                b"l\x00\x00\xff\xff"),
    }
}

#[test]
fn serialize_u32() {
    test_cases! {
        (0u32,                            b"U\x00"),
        (u32::from(u8::max_value()),      b"U\xff"),
        (u32::from(u8::max_value()) + 1,  b"I\x01\x00"),
        (i16::max_value() as u32,         b"I\x7f\xff"),
        (i16::max_value() as u32 + 1,     b"l\x00\x00\x80\x00"),
        (i32::max_value() as u32,         b"l\x7f\xff\xff\xff"),
        (i32::max_value() as u32 + 1,     b"L\x00\x00\x00\x00\x80\x00\x00\x00"),
        (u32::max_value(),                b"L\x00\x00\x00\x00\xff\xff\xff\xff"),
    }
}

#[test]
fn serialize_u64() {
    test_cases! {
        (0u64,                            b"U\x00"),
        (u64::from(u8::max_value()),      b"U\xff"),
        (u64::from(u8::max_value()) + 1,  b"I\x01\x00"),
        (i16::max_value() as u64,         b"I\x7f\xff"),
        (i16::max_value() as u64 + 1,     b"l\x00\x00\x80\x00"),
        (i32::max_value() as u64,         b"l\x7f\xff\xff\xff"),
        (i32::max_value() as u64 + 1,     b"L\x00\x00\x00\x00\x80\x00\x00\x00"),
        (i64::max_value() as u64,         b"L\x7f\xff\xff\xff\xff\xff\xff\xff"),
        (i64::max_value() as u64 + 1,     b"HU\x139223372036854775808"),
        (u64::max_value(),                b"HU\x1418446744073709551615"),
    }
}

#[test]
fn serialize_f32() {
    use std::f32::consts;
    test_cases! {
        (0.0f32,     b"d\x00\x00\x00\x00"),
        (consts::PI, b"d\x40\x49\x0f\xdb"),
        (consts::E,  b"d\x40\x2d\xf8\x54"),
    }
}

#[test]
fn serialize_f64() {
    use std::f64::consts;
    test_cases! {
        (0.0f64,     b"D\x00\x00\x00\x00\x00\x00\x00\x00"),
        (consts::PI, b"D\x40\x09\x21\xfb\x54\x44\x2d\x18"),
        (consts::E,  b"D\x40\x05\xbf\x0a\x8b\x14\x57\x69"),
    }
}

#[test]
fn serialize_char() {
    test_cases! {
        ('A',  b"CA"),
        ('À',  b"U\xc0"),
        ('가', b"l\x00\x00\xac\x00"),
    }
}
