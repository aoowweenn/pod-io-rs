extern crate byteorder;

#[allow(unused_imports)]
#[macro_use]
extern crate pod_io_derive;

#[doc(hidden)]
pub use pod_io_derive::*;

use std::io::{Read, Result};
use byteorder::{ByteOrder, ReadBytesExt};

trait Decode {
    type Output;
    fn decode<T: ByteOrder, R: Read>(r: &mut R) -> Result<Self::Output>;
}

impl Decode for u8 {
    type Output = u8;
    fn decode<T: ByteOrder, R: Read>(r: &mut R) -> Result<u8> {
        r.read_u8()
    }
}

impl Decode for i8 {
    type Output = i8;
    fn decode<T: ByteOrder, R: Read>(r: &mut R) -> Result<i8> {
        r.read_i8()
    }
}

macro_rules! impl_decode {
    ($ty:ty, $fn:ident) => (
        impl Decode for $ty {
            type Output = $ty;
            fn decode<T: ByteOrder, R: Read>(r: &mut R) -> Result<$ty> {
                r.$fn::<T>()
            }
        }
    );
}

macro_rules! impl_decode_array {
    ($ty:ty, $len:expr, $fn:ident) => (
        impl Decode for [$ty; $len] {
            type Output = [$ty; $len];
            fn decode<T: ByteOrder, R: Read>(r: &mut R) -> Result<[$ty; $len]> {
                let mut buf: [$ty; $len] = [0 as $ty; $len];
                r.$fn::<T>(&mut buf)?;
                Ok(buf)
            }
        }
    );
    ($ty:ty, $fn:ident) => (
        impl_decode_array!($ty, 2, $fn);
        impl_decode_array!($ty, 3, $fn);
        impl_decode_array!($ty, 4, $fn);
    );
}

impl_decode!(u16, read_u16);
impl_decode!(i16, read_i16);
impl_decode!(u32, read_u32);
impl_decode!(i32, read_i32);
impl_decode!(u64, read_u64);
impl_decode!(i64, read_i64);
impl_decode!(f32, read_f32);
impl_decode!(f64, read_f64);
impl_decode_array!(u16, read_u16_into);
impl_decode_array!(i16, read_i16_into);
impl_decode_array!(u32, read_u32_into);
impl_decode_array!(i32, read_i32_into);
impl_decode_array!(u64, read_u64_into);
impl_decode_array!(i64, read_i64_into);
impl_decode_array!(f32, read_f32_into);
impl_decode_array!(f64, read_f64_into);

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::{BE, LE};

    #[derive(Decode, Debug, PartialEq)]
    struct ABC {
        a: u8,
        #[BE]
        b: u16,
        c: i32,
        d: f32,
        e: [i32; 3],
        f: [f32; 3],
    }

    #[test]
    fn it_works() {
        use std::io::Cursor;
        let mut data = Cursor::new(&[1u8,
                                    0u8, 1u8,
                                    1u8, 0u8, 0u8, 0u8,
                                    0xC3, 0xF5, 0x48, 0x40,
                                    1u8, 0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8,
                                    0xC3, 0xF5, 0x48, 0x40, 0xC3, 0xF5, 0x48, 0x40, 0xC3, 0xF5, 0x48, 0x40][..]);
        assert_eq!(ABC::decode::<LE, _>(&mut data).unwrap(),
            ABC {
                a: 1, b: 1, c: 1, d: 3.14,
                e: [1, 1, 1],
                f: [3.14, 3.14, 3.14],
            }
        );
    }
}
