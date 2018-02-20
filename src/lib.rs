extern crate byteorder;

#[allow(unused_imports)]
#[macro_use]
extern crate pod_io_derive;

#[doc(hidden)]
pub use pod_io_derive::*;

use std::io::{Read, Result};
pub use byteorder::ByteOrder;
use byteorder::ReadBytesExt;

pub trait Decode<R: Read, Arg=Nil> {
    fn decode<T: ByteOrder>(_r: &mut R, _p: Arg) -> Result<Self> where Self: std::marker::Sized;
}

pub struct Nil;

macro_rules! impl_decode {
    (byte => $r:ident, $fn:ident) => ($r.$fn());
    (bytes => $r:ident, $fn:ident) => ($r.$fn::<T>());
    ($i:ident, $ty:ty, $fn:ident) => (
        impl<'a, R: Read> Decode<R, Nil> for $ty {
            fn decode<T: ByteOrder>(r: &mut R, _p: Nil) -> Result<$ty> {
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
        impl<'a, R: Read> Decode<R, Nil> for [$ty; $len] {
            fn decode<T: ByteOrder>(r: &mut R, _p: Nil) -> Result<[$ty; $len]> {
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

    #[derive(Debug, PartialEq)]
    struct MyInt(u8);
    impl<R: Read> Decode<R, MyFn> for MyInt {
        fn decode<T: ByteOrder>(r: &mut R, p: MyFn) -> Result<MyInt> {
            let c = r.read_u8()?;
            Ok(MyInt(c + p()))
        }
    }

    type MyFn = fn() -> u8;

    #[derive(Decode, Debug, PartialEq)]
    #[Arg = "MyFn"]
    struct ABC {
        a: u8,
        #[BE]
        b: u16,
        c: i32,
        d: f32,
        e: [i32; 3],
        f: [f32; 3],
        #[Arg = "MyFn"]
        g: MyInt,
    }

    #[test]
    fn it_works() {
        use std::io::Cursor;
        let mut data = Cursor::new(&[1u8,
                                    0u8, 1u8,
                                    1u8, 0u8, 0u8, 0u8,
                                    0xC3, 0xF5, 0x48, 0x40,
                                    1u8, 0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8,
                                    0xC3, 0xF5, 0x48, 0x40, 0xC3, 0xF5, 0x48, 0x40, 0xC3, 0xF5, 0x48, 0x40,
                                    0x44][..]);
        assert_eq!(ABC::decode::<LE>(&mut data, || 0x55).unwrap(),
            ABC {
                a: 1, b: 1, c: 1, d: 3.14,
                e: [1, 1, 1],
                f: [3.14, 3.14, 3.14],
                g: MyInt(0x99),
            }
        );
    }
}
