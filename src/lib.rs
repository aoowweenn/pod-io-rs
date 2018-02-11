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

macro_rules! impl_decode {
    (byte => $r:ident, $fn:ident) => ($r.$fn());
    (bytes => $r:ident, $fn:ident) => ($r.$fn::<T>());
    ($i:ident, $ty:ty, $fn:ident) => (
        impl Decode for $ty {
            type Output = $ty;
            fn decode<T: ByteOrder, R: Read>(r: &mut R) -> Result<$ty> {
                impl_decode!($i => r, $fn)
            }
        }
    );
}

macro_rules! impl_decode_array {
    (byte => $r:ident, $ty:ty, $len:expr, $fn:ident) => ({
        let mut buf: [u8; $len] = [0; $len];
        $r.$fn(&mut buf)?;
        unsafe { std::mem::transmute::<[u8; $len], [$ty; $len]>(buf) }
    });
    (bytes => $r:ident, $ty:ty, $len:expr, $fn:ident) => ({
        let mut buf: [$ty; $len] = [0; $len];
        $r.$fn::<T>(&mut buf)?;
        buf
    });
    (floats => $r:ident, $ty:ty, $len:expr, $fn:ident) => ({
        let mut buf: [$ty; $len] = [0.0; $len];
        $r.$fn::<T>(&mut buf)?;
        buf
    });
    ($i:ident, $ty:ty, $len:expr, $fn:ident) => (
        impl Decode for [$ty; $len] {
            type Output = [$ty; $len];
            fn decode<T: ByteOrder, R: Read>(r: &mut R) -> Result<[$ty; $len]> {
                Ok(impl_decode_array!($i => r, $ty, $len, $fn))
            }
        }
    );
    ($i:ident, $ty:ty, $fn:ident) => (
        impl_decode_array!($i, $ty, 2, $fn);
        impl_decode_array!($i, $ty, 3, $fn);
        impl_decode_array!($i, $ty, 4, $fn);
    );
}

impl_decode!(byte, u8, read_u8);
impl_decode!(byte, i8, read_i8);
impl_decode!(bytes, u16, read_u16);
impl_decode!(bytes, i16, read_i16);
impl_decode!(bytes, u32, read_u32);
impl_decode!(bytes, i32, read_i32);
impl_decode!(bytes, u64, read_u64);
impl_decode!(bytes, i64, read_i64);
impl_decode!(bytes, f32, read_f32);
impl_decode!(bytes, f64, read_f64);
impl_decode_array!(byte, u8, read_exact);
impl_decode_array!(byte, i8, read_exact);
impl_decode_array!(bytes, u16, read_u16_into);
impl_decode_array!(bytes, i16, read_i16_into);
impl_decode_array!(bytes, u32, read_u32_into);
impl_decode_array!(bytes, i32, read_i32_into);
impl_decode_array!(bytes, u64, read_u64_into);
impl_decode_array!(bytes, i64, read_i64_into);
impl_decode_array!(floats, f32, read_f32_into);
impl_decode_array!(floats, f64, read_f64_into);

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
