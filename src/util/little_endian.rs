//! Encoding and decoding of little-endian format.
//!
//! This was created out of fustration with the `byteorder` crate, which I felt had a heavy API, so
//! I created this crate.

/// Read an integer from a buffer.
///
/// This writes `buf` through the methods in `T`'s implementation of `Decode`.
pub fn read<T: Decode>(buf: &[u8]) -> T {
    T::read_le(buf)
}

/// Write some integer into a buffer.
///
/// This writes `from` into `buf` through the methods in `T`'s implementation of `Encode`.
pub fn write<T: Encode>(buf: &mut [u8], from: T) {
    from.write_le(buf)
}

/// An encodable type.
pub trait Encode {
    /// Write an integer in little-endian format.
    ///
    /// This writes `self` into the first n bytes (depending on the size of `Self`) of `into` in
    /// little-endian format (least significant byte first).
    ///
    /// # Panics
    ///
    /// This will potentially panic if `into` is not large enough.
    fn write_le(self, into: &mut [u8]);
}

/// A decodable type.
pub trait Decode {
    /// Read an integer in little-endian format.
    ///
    /// This reads the first n bytes (depending on the size of `Self`) of `from` in little-endian
    /// (least significant byte first).
    ///
    /// # Panics
    ///
    /// This will potentially panic if `from` is not large enough.
    fn read_le(from: &[u8]) -> Self;
}

impl Decode for u8 {
    fn read_le(from: &[u8]) -> u8 {
        from[0]
    }
}
impl Encode for u8 {
    fn write_le(self, into: &mut [u8]) {
        into[0] = self;
    }
}

impl Decode for u16 {
    fn read_le(from: &[u8]) -> u16 {
        u16::from(from[0]) | (u16::from(from[1])) << 8
    }
}
impl Encode for u16 {
    fn write_le(self, into: &mut [u8]) {
        into[0] = self as u8;
        into[1] = (self >> 8) as u8;
    }
}

impl Decode for u32 {
    fn read_le(from: &[u8]) -> u32 {
        from[0] as u32 | (from[1] as u32) << 8 | (from[2] as u32) << 16 | (from[3] as u32) << 24
    }
}
impl Encode for u32 {
    fn write_le(self, into: &mut [u8]) {
        into[0] = self as u8;
        into[1] = (self >> 8) as u8;
        into[2] = (self >> 16) as u8;
        into[3] = (self >> 24) as u8;
    }
}

impl Decode for u64 {
    fn read_le(from: &[u8]) -> u64 {
        u64::from(from[0])
            | (u64::from(from[1])) << 8
            | (u64::from(from[2])) << 16
            | (u64::from(from[3])) << 24
            | (u64::from(from[4])) << 32
            | (u64::from(from[5])) << 40
            | (u64::from(from[6])) << 48
            | (u64::from(from[7])) << 56
    }
}
impl Encode for u64 {
    fn write_le(self, into: &mut [u8]) {
        into[0] = self as u8;
        into[1] = (self >> 8) as u8;
        into[2] = (self >> 16) as u8;
        into[3] = (self >> 24) as u8;
        into[4] = (self >> 32) as u8;
        into[5] = (self >> 40) as u8;
        into[6] = (self >> 48) as u8;
        into[7] = (self >> 56) as u8;
    }
}

impl Decode for u128 {
    fn read_le(from: &[u8]) -> u128 {
        u128::from(from[0])
            | (u128::from(from[1])) << 8
            | (u128::from(from[2])) << 16
            | (u128::from(from[3])) << 24
            | (u128::from(from[4])) << 32
            | (u128::from(from[5])) << 40
            | (u128::from(from[6])) << 48
            | (u128::from(from[7])) << 56
            | (u128::from(from[8])) << 64
            | (u128::from(from[9])) << 72
            | (u128::from(from[10])) << 80
            | (u128::from(from[11])) << 88
            | (u128::from(from[12])) << 96
            | (u128::from(from[13])) << 104
            | (u128::from(from[14])) << 112
            | (u128::from(from[15])) << 120
    }
}
impl Encode for u128 {
    fn write_le(self, into: &mut [u8]) {
        into[0] = self as u8;
        into[1] = (self >> 8) as u8;
        into[2] = (self >> 16) as u8;
        into[3] = (self >> 24) as u8;
        into[4] = (self >> 32) as u8;
        into[5] = (self >> 40) as u8;
        into[6] = (self >> 48) as u8;
        into[7] = (self >> 56) as u8;
        into[8] = (self >> 64) as u8;
        into[9] = (self >> 72) as u8;
        into[10] = (self >> 80) as u8;
        into[11] = (self >> 88) as u8;
        into[12] = (self >> 96) as u8;
        into[13] = (self >> 104) as u8;
        into[14] = (self >> 112) as u8;
        into[15] = (self >> 120) as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fmt, mem, ops};

    fn test_int<T>(n: T)
    where
        T: Encode
            + Decode
            + Copy
            + PartialEq
            + From<u8>
            + fmt::Debug
            + ops::BitAnd<T, Output = T>
            + ops::Shr<T, Output = T>,
    {
        let len = mem::size_of::<T>();
        let mut buf = [0; 32];
        write(&mut buf, n);

        for i in 0..len {
            assert_eq!(T::from(buf[i]), (n >> T::from(i as u8 * 8)) & T::from(0xFF));
        }

        assert_eq!(read::<T>(&buf), n);
    }

    #[test]
    fn u8() {
        test_int(12u8);
        test_int(1u8);
        test_int(0u8);
    }

    #[test]
    fn u16() {
        test_int::<u16>(0xAB);
        test_int::<u16>(0xBA);
        test_int::<u16>(0);
    }

    #[test]
    fn u32() {
        test_int::<u32>(0xABCD);
        test_int::<u32>(0xDCBA);
        test_int::<u32>(0);
    }

    #[test]
    fn u64() {
        test_int::<u64>(0xABCDEF0);
        test_int::<u64>(0x0FEDCBA);
        test_int::<u64>(0);
    }

    #[test]
    fn u128() {
        test_int::<u128>(0xABCDEF012345678);
        test_int::<u128>(0x876543210FEDCBA);
        test_int::<u128>(0);
    }
}
