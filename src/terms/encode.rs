#[cfg(feature="bigint")]
extern crate num_bigint;

#[cfg(feature="bigint")]
extern crate num_traits;

#[cfg(feature="bigint")]
use {
    num_bigint::{ BigInt, BigUint, Sign },
    num_traits::{ sign::Signed, cast::ToPrimitive },
};

use super::{
    EList,
    ENil,
    ENonProperList,
    EAtom,
    EExport,
    ETerm,
    ETuple,
    EString,
    EPort,
    EPid,
    EMap,
    EBinary,
};
use super::super::error::{ Error, ErrorCode };

use std::io::{ Write, Error as IOError };

/// Replacement for `std::convert::TryInto<T>` that doesn't require `Sized`.
pub trait TryToExternalBinary {
    fn try_to_vec(&self) -> Result<Vec<u8>, Error> {
        let mut result: Vec<u8> = Vec::new();
        self.try_to_writer(&mut result)?;

        Ok(result)
    }

    fn try_to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error>;
}

/// Replacement for `std::convert::Into<T>` that doesn't require `Sized`.
pub trait ToExternalBinary {
    fn to_vec(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        self.to_writer(&mut result).unwrap();

        result
    }

    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, IOError>;
}

impl<X> TryToExternalBinary for X where X: ToExternalBinary {
    fn try_to_vec(&self) -> Result<Vec<u8>, Error> {
        Ok(ToExternalBinary::to_vec(self))
    }

    fn try_to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        match self.to_writer(writer) {
            Ok(size) => Ok(size),
            Err(e) => Err(Error::io(e)),
        }
    }
}

impl ToExternalBinary for i8 {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, IOError> {
        (*self as i32).to_writer(writer)
    }
}

impl ToExternalBinary for u8 {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, IOError> {
        let data: &[u8; 1] = &self.to_be_bytes();

        writer.write(&[super::SMALL_INTEGER_EXT, data[0]])
    }
}

impl ToExternalBinary for i16 {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, IOError> {
        (*self as i32).to_writer(writer)
    }
}

impl ToExternalBinary for u16 {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, IOError> {
        (*self as i32).to_writer(writer)
    }
}

impl ToExternalBinary for i32 {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, IOError> {
        if *self <= u8::max_value().into() && *self >= 0 {
            (*self as u8).to_writer(writer)
        } else {
            let mut amount = writer.write(&[super::INTEGER_EXT])?;
            amount += writer.write(&self.to_be_bytes())?;

            Ok(amount)
        }
    }
}

impl ToExternalBinary for u32 {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, IOError> {
        (*self as i128).to_writer(writer)
    }
}

impl ToExternalBinary for i64 {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, IOError> {
        (*self as i128).to_writer(writer)
    }
}

impl ToExternalBinary for u64 {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, IOError> {
        (*self as i128).to_writer(writer)
    }
}

impl ToExternalBinary for i128 {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, IOError> {
        if *self <= i32::max_value().into() && *self >= i32::min_value().into() {
            (*self as i32).to_writer(writer)
        } else {
            let mut abs = lossless_abs(*self);

            let bytes: u8 = (16 - (abs.leading_zeros() >> 3)) as u8;
            let sign: u8 = if *self >= 0 { 0 } else { 1 };

            let mut amount = writer.write(&[super::SMALL_BIG_EXT, bytes, sign])?;

            while abs != 0 {
                amount += writer.write(&[(abs & 0xff) as u8])?;
                abs >>= 8;
            }

            Ok(amount)
        }
    }
}

impl ToExternalBinary for u128 {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, IOError> {
        // The big number here is i128::max_value() as a `From<u128>` for i128 is not implemented
        if *self <= 170_141_183_460_469_231_731_687_303_715_884_105_727u128 {
            (*self as i128).to_writer(writer)
        } else {
            let data: &[u8; 16] = &self.to_be_bytes();
            let mut len = writer.write(&[
                super::SMALL_BIG_EXT,
                data.iter().filter(|n| **n != 0).count() as u8,
                0u8
            ])?;

            for e in data.iter() {
                if *e != 0u8 {
                    len += writer.write(&[*e])?;
                }
            }

            Ok(len)
        }
    }
}

impl ToExternalBinary for isize {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, IOError> {
        (*self as i128).to_writer(writer)
    }
}

impl ToExternalBinary for usize {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, IOError> {
        (*self as u128).to_writer(writer)
    }
}

impl ToExternalBinary for ENil {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, IOError> {
        writer.write(&[super::NIL_EXT])
    }
}

impl<'a> TryToExternalBinary for EList<'a> {
    fn try_to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        if self.0.is_empty() {
            (ENil {}).try_to_writer(writer)
        } else {
            let len: [u8; 4] = (self.0.len() as i32).to_be_bytes();
            let mut amount = write(writer, &[super::LIST_EXT, len[0], len[1], len[2], len[3]])?;

            for d in self.0.iter() {
                amount += d.try_to_writer(writer)?;
            }

            amount += (ENil {}).try_to_writer(writer)?;
            
            Ok(amount)
        }
    }
}

impl<'a> TryToExternalBinary for ENonProperList<'a> {
    fn try_to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        let len: [u8; 4] = (self.data.len() as i32).to_be_bytes();
        let mut written = write(writer, &([super::LIST_EXT, len[0], len[1], len[2], len[3]]))?;

        for d in self.data.iter() {
            written += d.try_to_writer(writer)?;
        }

        written += self.tail.try_to_writer(writer)?;

        Ok(written)
    }
}

impl TryToExternalBinary for EAtom {
    fn try_to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        let byte_length = self.0.as_bytes().len();

        if byte_length <= u8::max_value().into() {
            let mut written = write(writer, &[super::SMALL_ATOM_UTF8_EXT, byte_length as u8])?;
            written += write(writer, self.0.as_bytes())?;

            Ok(written)
        } else if byte_length <= u16::max_value().into() {
            let len: [u8; 8] = byte_length.to_be_bytes();
            let mut written = write(writer, &[super::ATOM_UTF8_EXT, len[6], len[7]])?;
            written += write(writer, self.0.as_bytes())?;

            Ok(written)
        } else {
            Err(Error::data(ErrorCode::ValueNotEncodable(Box::from((&self.0).as_str().to_owned()))))
        }
    }
}

impl TryToExternalBinary for f32 {
    fn try_to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        if self.is_finite() {
            let bytes = self.to_be_bytes();

            write(writer, &[super::NEW_FLOAT_EXT, 0, 0, 0, 0, bytes[0], bytes[1], bytes[2], bytes[3]])
        } else {
            Err(Error::data(ErrorCode::ValueNotEncodable(Box::from(self.to_string()))))
        }
    }
}

impl TryToExternalBinary for f64 {
    fn try_to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        if self.is_finite() {
            let bytes = self.to_be_bytes();

            write(writer, &[super::NEW_FLOAT_EXT, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]])
        } else {
            Err(Error::data(ErrorCode::ValueNotEncodable(Box::from(self.to_string()))))
        }
    }
}

impl TryToExternalBinary for EExport {
    fn try_to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        let mut written = write(writer, &[super::EXPORT_EXT])?;
        written += self.module.try_to_writer(writer)?;
        written += self.function.try_to_writer(writer)?;
        written += self.arity.try_to_writer(writer)?;

        Ok(written)
    }
}

impl<'a> TryToExternalBinary for ETuple<'a> {
    fn try_to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        let len = (self.0.len() as u32).to_be_bytes();
        let mut written = 0;

        if self.0.len() <= u8::max_value().into() {
            written += write(writer, &[super::SMALL_TUPLE_EXT, len[3]])?;
        } else {
            written += write(writer, &[super::LARGE_TUPLE_EXT, len[0], len[1], len[2], len[3]])?;
        }

        for d in self.0.iter() {
            written += d.try_to_writer(writer)?;
        }

        Ok(written)
    }
}

impl TryToExternalBinary for EString {
    fn try_to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        let byte_length = self.0.as_bytes().len();

        if byte_length <= u16::max_value().into() {
            let len: [u8; 8] = byte_length.to_be_bytes();
            let mut written = write(writer, &[super::STRING_EXT, len[6], len[7]])?;
            written += write(writer, self.0.as_bytes())?;

            Ok(written)
        } else {
            EList(&(self.0.as_bytes().iter().map(|x| x as &dyn ETerm).collect())).try_to_writer(writer)
        }
    }
}

impl TryToExternalBinary for EPort {
    fn try_to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        let mut written = write(writer, &[super::NEW_PORT_EXT])?;
        written += self.node.try_to_writer(writer)?;
        written += write(writer, &self.id.to_be_bytes())?;
        written += write(writer, &self.creation.to_be_bytes())?;

        Ok(written)
    }
}

impl TryToExternalBinary for EPid {
    fn try_to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        let mut written = write(writer, &[super::NEW_PID_EXT])?;
        written += self.node.try_to_writer(writer)?;
        written += write(writer, &self.id.to_be_bytes())?;
        written += write(writer, &self.serial.to_be_bytes())?;
        written += write(writer, &self.creation.to_be_bytes())?;
        
        Ok(written)
    }
}
impl<'a> TryToExternalBinary for EMap<'a> {
    fn try_to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        let mut written = write(writer, &[super::MAP_EXT])?;
        written += write(writer, &(self.0.len() as u32).to_be_bytes())?;

        for (k, v) in self.0 {
            written += k.try_to_writer(writer)?;
            written += v.try_to_writer(writer)?;
        };

        Ok(written)
    }
}

impl<'a> ToExternalBinary for EBinary<'a> {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, IOError> {
        let mut written = writer.write(&[super::BINARY_EXT])?;
        written += writer.write(&self.0.len().to_be_bytes())?;
        written += writer.write(self.0)?;

        Ok(written)
    }
}

#[cfg(feature="bigint")]
impl TryToExternalBinary for BigInt {
    fn try_to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        match self.to_u128() {
            Some(x) => return x.try_to_writer(writer),
            _ => Option::<i32>::None
        };

        match self.to_i128() {
            Some(x) => return x.try_to_writer(writer),
            _ => Option::<i32>::None
        };

        // Failing to delegate to u128 and i128 already establishes that
        // either a SMALL_BIG_EXT or a LARGE_BIG_EXT is required to encode
        // the value, so we don't attempt to fit it into smaller datatypes
        // anymore.
        // The range of SMALL_BIG_EXT is -2^2048..(2^2048)-1, so we still need
        // to check whether we fit into a SMALL_BIG_EXT, as 2048 > 128.

        let sign: u8 = match self.sign() {
            Sign::Minus => 1,
            Sign::NoSign => 0,
            Sign::Plus => 0
        };

        let tmp = match self.abs().to_biguint() {
            Option::Some(x) => x,
            Option::None => return Err(Error::data(ErrorCode::ValueNotEncodable(Box::from("Non-negative bigint is somehow not convertible to biguint."))))
        }.to_bytes_be();

        if tmp.len() <= (u8::max_value() as usize) {
            let len = tmp.len() as u8;
            let mut written = write(writer, &[super::SMALL_BIG_EXT, len, sign])?;
            written += write(writer, &tmp)?;

            Ok(written)
        } else if tmp.len() <= (u32::max_value() as usize) {
            let len = (tmp.len() as u32).to_be_bytes();
            let mut written = write(writer, &[super::LARGE_BIG_EXT, len[0], len[1], len[2], len[3], sign])?;
            written += write(writer, &tmp)?;

            Ok(written)
        } else {
            Err(Error::data(ErrorCode::ValueNotEncodable(Box::from("Integer size is outside of the possible ranges for an erlang term (-2^N..2^N-1 with N=(2^32)*8)"))))
        }
    }
}

#[cfg(feature="bigint")]
impl TryToExternalBinary for BigUint {
    fn try_to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        match self.to_u128() {
            Some(x) => return x.try_to_writer(writer),
            _ => Option::<i32>::None
        };

        let tmp = self.to_bytes_be();

        if tmp.len() <= (u8::max_value() as usize) {
            let len = tmp.len() as u8;
            let mut written = write(writer, &[super::SMALL_BIG_EXT, len, 0u8])?;
            written += write(writer, &tmp)?;

            Ok(written)
        } else if tmp.len() <= (u32::max_value() as usize) {
            let len = (tmp.len() as u32).to_be_bytes();
            let mut written = write(writer, &[super::LARGE_BIG_EXT, len[0], len[1], len[2], len[3], 0u8])?;
            written += write(writer, &tmp)?;

            Ok(written)
        } else {
            Err(Error::data(ErrorCode::ValueNotEncodable(Box::from("Integer size is outside of the possible ranges for an erlang term (-2^N..2^N-1 with N=(2^32)*8)"))))
        }
    }
}

fn write(writer: &mut dyn Write, bytes: &[u8]) -> Result<usize, Error> {
    match writer.write(bytes) {
        Ok(written) => Ok(written),
        Err(e) => Err(Error::io(e)),
    }
}

fn lossless_abs(num: i128) -> u128 {
    if num >= 0 {
        num as u128
    } else if num == i128::min_value() {
        0x80_00_00_00_00_00_00_00_00_00_00_00_00_00_00_00u128
    } else {
        -num as u128
    }
}

#[cfg(test)]
mod tests {
    use super::ETerm;
    use super::lossless_abs;
    use num_bigint::{ BigInt, BigUint };

    struct Test {
        binary: Vec<u8>,
        is_negative: bool,
        number: u128,
    }

    macro_rules! test {
        ($bin:tt, false, $val:expr) => {
            &Test::new(vec!$bin, false, $val as u128)
        };
        ($bin:tt, true, $val:expr) => {
            &Test::new(vec!$bin, true, lossless_abs($val as i128))
        };
    }

    impl Test {
        fn new(binary: Vec<u8>, is_negative: bool, number: u128) -> Test {
            Test {
                binary,
                is_negative,
                number,
            }
        }

        fn run_all(signed: bool, bits: u8) {
            for test in &[
                test!([97, 0], false, 0),

                // i8 and u8 type boundaries
                test!([98, 255, 255, 255, 128], true, i8::min_value()),
                test!([97, 127], false, i8::max_value()),
                test!([97, 255], false, u8::max_value()),

                // i16 and u16 type boundaries
                test!([98, 255,255, 128, 0], true, i16::min_value()),
                test!([98, 0, 0, 127, 255], false, i16::max_value()),
                test!([98, 0, 0, 255, 255], false, u16::max_value()),

                // i32 and u32 type boundaries
                test!([98, 128, 0, 0, 0], true, i32::min_value()),
                test!([98, 127, 255, 255, 255], false, i32::max_value()),
                test!([110, 4, 0, 255, 255, 255, 255], false, u32::max_value()),

                // i64 and u64 type boundaries
                test!([110, 8, 1, 0, 0, 0, 0, 0, 0, 0, 128], true, i64::min_value()),
                test!([110, 8, 0, 255, 255, 255, 255, 255, 255, 255, 127], false, i64::max_value()),
                test!([110, 8, 0, 255, 255, 255, 255, 255, 255, 255, 255], false, u64::max_value()),

                // i128 and u128 type boundaries
                test!([110, 16, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128], true, i128::min_value()),
                test!([110, 16, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 127], false, i128::max_value()),
                test!([110, 16, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255], false ,u128::max_value()),
            ] {

                test.run(signed, bits);
            }
        }

        fn run(&self, signed: bool, bits: u8) {
            let bin = match (signed, bits, self.is_negative, self.number) {
                (true, 8, s, n) if n <= (i8::max_value() as u128) || (s && n <= lossless_abs(i8::min_value() as i128)) =>
                    (self.with_sign() as i8).try_to_external_binary(),
                (false, 8, false, n) if n <= (u8::max_value() as u128) =>
                    (self.number as u8).try_to_external_binary(),
                (true, 16, s, n) if n <= (i16::max_value() as u128) || (s && n <= lossless_abs(i16::min_value() as i128)) =>
                    (self.with_sign() as i16).try_to_external_binary(),
                (false, 16, false, n) if n <= (u16::max_value() as u128) =>
                    (self.number as u16).try_to_external_binary(),
                (true, 32, s, n) if n <= (i32::max_value() as u128) || (s && n <= lossless_abs(i32::min_value() as i128)) =>
                    (self.with_sign() as i32).try_to_external_binary(),
                (false, 32, false, n) if n <= (u32::max_value() as u128) =>
                    (self.number as u32).try_to_external_binary(),
                (true, 64, s, n) if n <= (i64::max_value() as u128) || (s && n <= lossless_abs(i64::min_value() as i128)) =>
                    (self.with_sign() as i64).try_to_external_binary(),
                (false, 64, false, n) if n <= (u64::max_value() as u128) =>
                    (self.number as u64).try_to_external_binary(),
                (true, 128, s, n) if n <= (i128::max_value() as u128) || (s && n <= lossless_abs(i128::min_value() as i128)) =>
                    (self.with_sign() as i128).try_to_external_binary(),
                (false, 128, false, n) if n <= u128::max_value() =>
                    self.number.try_to_external_binary(),
                _ => return,
            }.unwrap();

            assert_eq!(self.binary, bin, "Call mode: {{signed: {}, bit_width: {}}}, Test mode: {{negative: {}, number: {}}}", signed, bits, self.is_negative, self.number);
        }

        fn with_sign(&self) -> i128 {
            if self.is_negative && self.number == lossless_abs(i128::min_value()) {
                i128::min_value()
            } else if self.is_negative {
                -(self.number as i128)
            } else {
                self.number as i128
            }
        }
    }

    macro_rules! maketest {
        ($name:ident, unsigned, $bits:expr) => (
            #[test]
            fn $name() {
                Test::run_all(false, $bits);
            }
        );
        ($name:ident, signed, $bits:expr) => (
            #[test]
            fn $name() {
                Test::run_all(true, $bits);
            }
        );
    }

    maketest!(u8, unsigned, 8);
    maketest!(i8, signed, 8);
    maketest!(u16, unsigned, 16);
    maketest!(i16, signed, 16);
    maketest!(u32, unsigned, 32);
    maketest!(i32, signed, 32);
    maketest!(u64, unsigned, 64);
    maketest!(i64, signed, 64);
    maketest!(u128, unsigned, 128);
    maketest!(i128, signed, 128);

    #[test]
    #[cfg(feature="bigint")]
    fn bigint_small_upper_boundary() {
        let mut val = BigInt::from(0);
        for _ in 0..255 {
            val <<= 8;
            val |= BigInt::from(0xffu8);
        }
        assert_eq!(vec![
            0x6e, 0xff, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff
        ], val.try_to_external_binary().unwrap());
        val *= -1;
        assert_eq!(vec![
            0x6e, 0xff, 0x01, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff
        ], val.try_to_external_binary().unwrap());
    }

    #[test]
    #[cfg(feature="bigint")]
    fn biguint_small_upper_boundary() {
        let mut val = BigUint::from(0u8);
        for _ in 0..255 {
            val <<= 8;
            val |= BigUint::from(0xffu8);
        }
        assert_eq!(vec![
            0x6e, 0xff, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff
        ], val.try_to_external_binary().unwrap());
    }

    #[test]
    #[cfg(feature="bigint")]
    fn bigint_big_lower_boundary() {
        let mut val = BigInt::from(0);
        for _ in 0..255 {
            val <<= 8;
            val |= BigInt::from(0xffu8);
        }
        val += 1;
        assert_eq!(vec![
            0x6f, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01
        ], val.try_to_external_binary().unwrap());
        val *= -1;
        assert_eq!(vec![
            0x6f, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01
        ], val.try_to_external_binary().unwrap());
    }

    #[test]
    #[cfg(feature="bigint")]
    fn biguint_big_lower_boundary() {
        let mut val = BigUint::from(0u8);
        for _ in 0..255 {
            val <<= 8;
            val |= BigUint::from(0xffu8);
        }
        val += 1u8;
        assert_eq!(vec![
            0x6f, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01
        ], val.try_to_external_binary().unwrap());
    }
}
