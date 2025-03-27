//! Tests for the derive implementations for the `Decode` and `Encode` traits.
#![cfg(all(feature = "derive", feature = "alloc"))]

use ssh_encoding::{Decode, Encode, Error};

#[derive(Debug, PartialEq, Decode, Encode)]
struct MostTypes<T>
where
    T: Encode + Decode<Error = Error>,
{
    a: u8,
    b: u32,
    c: u64,
    d: usize,
    e: bool,
    f: [u8; 7],
    g: String,
    h: Vec<u8>,
    i: T,
}

#[derive(Debug, PartialEq, Encode)]
struct Reference<'a>(&'a [u8]);

#[derive(Debug, PartialEq)]
struct CustomError;

impl From<ssh_encoding::Error> for CustomError {
    fn from(_: ssh_encoding::Error) -> Self {
        CustomError
    }
}

#[derive(Debug, PartialEq, Decode, Encode)]
#[ssh(decode_error(CustomError))]
struct CustomErrorType(u8, u32);

#[derive(Debug, PartialEq, Decode, Encode)]
#[ssh(length_prefixed)]
struct LengthPrefixed {
    #[ssh(length_prefixed)]
    a: u32,
    b: String,
}

#[test]
fn derive_encode_decode_roundtrip_most_types() {
    #[rustfmt::skip]
    let data = [
        42,
        0xDE, 0xAD, 0xBE, 0xEF,
        0xCA, 0xFE, 0xBA, 0xBE, 0xFA, 0xCE, 0xFE, 0xED,
        0x00, 0x00, 0xAB, 0xCD,
        0x01,
        b'e', b'x', b'a', b'm', b'p', b'l', b'e',
        0x00, 0x00, 0x00, 0x05, b'h', b'e', b'l', b'l', b'o',
        0x00, 0x00, 0x00, 0x05, b'w', b'o', b'r', b'l', b'd',
        0x20,
    ];
    let expected = MostTypes {
        a: 42,
        b: 0xDEAD_BEEF,
        c: 0xCAFE_BABE_FACE_FEED,
        d: 0xABCD,
        e: true,
        f: *b"example",
        g: "hello".to_string(),
        h: b"world".to_vec(),
        i: 0x20u8,
    };
    assert_eq!(&data, expected.encode_vec().unwrap().as_slice());
    let most_types = MostTypes::<u8>::decode(&mut &data[..]).unwrap();
    assert_eq!(most_types, expected);
}

#[test]
fn derive_encode_reference() {
    let data = b"\x00\x00\x00\x07example";
    let expected = Reference(&data[4..]);
    assert_eq!(data, expected.encode_vec().unwrap().as_slice());
}

#[test]
fn derive_decode_custom_error() {
    let data = [42, 0xDE, 0xAD, 0xBE, 0xEF];
    let expected = CustomErrorType(42, 0xDEAD_BEEF);
    assert_eq!(expected, CustomErrorType::decode(&mut &data[..]).unwrap());
    assert_eq!(
        CustomError,
        CustomErrorType::decode(&mut &data[1..]).unwrap_err()
    );
}

#[test]
fn derive_encode_decode_roundtrip_length_prefixed() {
    #[rustfmt::skip]
    let data = [
        0x00, 0x00, 0x00, 0x11,
        0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x2A,
        0x00, 0x00, 0x00, 0x05, b'h', b'e', b'l', b'l', b'o',
    ];
    let expected = LengthPrefixed {
        a: 42,
        b: "hello".to_string(),
    };
    assert_eq!(&data, expected.encode_vec().unwrap().as_slice());
    let length_prefixed = LengthPrefixed::decode(&mut &data[..]).unwrap();
    assert_eq!(length_prefixed, expected);
}
