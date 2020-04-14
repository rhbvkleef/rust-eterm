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
    TermTag,
};
use super::super::error::{ Error };

use std::io::Write;

/// Replacement for `std::convert::Into<T>` that doesn't require `Sized`.
pub trait ToExternalBinary {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error>;
}

impl ToExternalBinary for i8 {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        (*self as i32).to_writer(writer)
    }
}

impl ToExternalBinary for u8 {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        let data: &[u8; 1] = &self.to_be_bytes();

        Ok(writer.write(&[TermTag::SmallInteger as u8, data[0]])?)
    }
}

impl ToExternalBinary for i16 {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        (*self as i32).to_writer(writer)
    }
}

impl ToExternalBinary for u16 {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        (*self as i32).to_writer(writer)
    }
}

impl ToExternalBinary for i32 {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        if *self <= u8::max_value().into() && *self >= 0 {
            (*self as u8).to_writer(writer)
        } else {
            let mut amount = writer.write(&[TermTag::Integer as u8])?;
            amount += writer.write(&self.to_be_bytes())?;

            Ok(amount)
        }
    }
}

impl ToExternalBinary for u32 {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        (*self as i128).to_writer(writer)
    }
}

impl ToExternalBinary for i64 {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        (*self as i128).to_writer(writer)
    }
}

impl ToExternalBinary for u64 {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        (*self as i128).to_writer(writer)
    }
}

impl ToExternalBinary for i128 {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        if *self <= i32::max_value().into() && *self >= i32::min_value().into() {
            (*self as i32).to_writer(writer)
        } else {
            let mut abs = lossless_abs(*self);

            let bytes: u8 = (16 - (abs.leading_zeros() >> 3)) as u8;
            let sign: u8 = if *self >= 0 { 0 } else { 1 };

            let mut amount = writer.write(&[TermTag::SmallBig as u8, bytes, sign])?;

            while abs != 0 {
                amount += writer.write(&[(abs & 0xff) as u8])?;
                abs >>= 8;
            }

            Ok(amount)
        }
    }
}

impl ToExternalBinary for u128 {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        // The big number here is i128::max_value() as a `From<u128>` for i128 is not implemented
        if *self <= 170_141_183_460_469_231_731_687_303_715_884_105_727u128 {
            (*self as i128).to_writer(writer)
        } else {
            let mut tmp = *self;

            let bytes: u8 = (16 - (tmp.leading_zeros() >> 3)) as u8;
            let mut amount = writer.write(&[TermTag::SmallBig as u8, bytes, 0u8])?;

            while tmp != 0 {
                amount += writer.write(&[(tmp & 0xff) as u8])?;
                tmp >>= 8;
            }

            Ok(amount)
        }
    }
}

impl ToExternalBinary for isize {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        (*self as i128).to_writer(writer)
    }
}

impl ToExternalBinary for usize {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        (*self as u128).to_writer(writer)
    }
}

impl ToExternalBinary for ENil {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        Ok(writer.write(&[TermTag::Nil as u8])?)
    }
}

impl<'a> ToExternalBinary for EList {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        if self.0.is_empty() {
            ENil::default().to_writer(writer)
        } else {
            let len: [u8; 4] = (self.0.len() as i32).to_be_bytes();
            let mut amount = writer.write(&[TermTag::List as u8, len[0], len[1], len[2], len[3]])?;

            for d in self.0.iter() {
                amount += d.to_writer(writer)?;
            }

            amount += (ENil {}).to_writer(writer)?;
            
            Ok(amount)
        }
    }
}

impl<'a> ToExternalBinary for ENonProperList {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        let len: [u8; 4] = (self.data.len() as i32).to_be_bytes();
        let mut written = writer.write(&([TermTag::List as u8, len[0], len[1], len[2], len[3]]))?;

        for d in self.data.iter() {
            written += d.to_writer(writer)?;
        }

        written += self.tail.to_writer(writer)?;

        Ok(written)
    }
}

impl ToExternalBinary for EAtom {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        let byte_length = self.0.as_bytes().len();

        if byte_length <= u8::max_value().into() {
            let mut written = writer.write(&[TermTag::SmallAtomUtf8 as u8, byte_length as u8])?;
            written += writer.write(self.0.as_bytes())?;

            Ok(written)
        } else if byte_length <= u16::max_value().into() {
            let len: [u8; 8] = byte_length.to_be_bytes();
            let mut written = writer.write(&[TermTag::AtomUtf8 as u8, len[6], len[7]])?;
            written += writer.write(self.0.as_bytes())?;

            Ok(written)
        } else {
            Err(Error::Message(self.0.to_owned()))
        }
    }
}

impl ToExternalBinary for f32 {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        (*self as f64).to_writer(writer)
    }
}

impl ToExternalBinary for f64 {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        if self.is_finite() {
            let bytes = self.to_be_bytes();

            Ok(writer.write(&[TermTag::NewFloat as u8, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]])?)
        } else {
            Err(Error::Message(self.to_string()))
        }
    }
}

impl ToExternalBinary for EExport {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        let mut written = writer.write(&[TermTag::Export as u8])?;
        written += self.module.to_writer(writer)?;
        written += self.function.to_writer(writer)?;
        written += self.arity.to_writer(writer)?;

        Ok(written)
    }
}

impl ToExternalBinary for ETuple {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        let len = (self.0.len() as u32).to_be_bytes();
        let mut written = 0;

        if self.0.len() <= u8::max_value().into() {
            written += writer.write(&[TermTag::SmallTuple as u8, len[3]])?;
        } else {
            written += writer.write(&[TermTag::LargeTuple as u8, len[0], len[1], len[2], len[3]])?;
        }

        for d in self.0.iter() {
            written += d.to_writer(writer)?;
        }

        Ok(written)
    }
}

impl ToExternalBinary for EString {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        let byte_length = self.0.as_bytes().len();

        if byte_length <= u16::max_value().into() {
            let len: [u8; 8] = byte_length.to_be_bytes();
            let mut written = writer.write(&[TermTag::String as u8, len[6], len[7]])?;
            written += writer.write(self.0.as_bytes())?;

            Ok(written)
        } else {
            EList(self.0.as_bytes().iter().map(|x| Box::from(*x) as Box<dyn ETerm>).collect()).to_writer(writer)
        }
    }
}

impl ToExternalBinary for EPort {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        let mut written = writer.write(&[TermTag::NewPort as u8])?;
        written += self.node.to_writer(writer)?;
        written += writer.write(&self.id.to_be_bytes())?;
        written += writer.write(&self.creation.to_be_bytes())?;

        Ok(written)
    }
}

impl ToExternalBinary for EPid {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        let mut written = writer.write(&[TermTag::NewPid as u8])?;
        written += self.node.to_writer(writer)?;
        written += writer.write(&self.id.to_be_bytes())?;
        written += writer.write(&self.serial.to_be_bytes())?;
        written += writer.write(&self.creation.to_be_bytes())?;
        
        Ok(written)
    }
}

impl<'a> ToExternalBinary for EMap {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        let mut written = writer.write(&[TermTag::Map as u8])?;
        written += writer.write(&(self.0.len() as u32).to_be_bytes())?;

        for (k, v) in self.0.iter() {
            written += k.to_writer(writer)?;
            written += v.to_writer(writer)?;
        };

        Ok(written)
    }
}

impl ToExternalBinary for EBinary {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        let mut written = writer.write(&[TermTag::Binary as u8])?;
        written += writer.write(&self.0.len().to_be_bytes())?;
        written += writer.write(self.0.as_ref())?;

        Ok(written)
    }
}

#[cfg(feature="bigint")]
impl ToExternalBinary for BigInt {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        if let Some(x) = self.to_u128() {
            return x.to_writer(writer);
        }

        if let Some(x) = self.to_i128() {
            return x.to_writer(writer);
        }

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

        let abs = self.abs().to_biguint()
            .ok_or_else(|| Error::Message("Non-negative bigint is somehow not convertible to biguint.".to_string()))?;

        let len = abs.to_bytes_be().len();

        if len <= (u8::max_value() as usize) {
            let mut written = writer.write(&[TermTag::SmallBig as u8, len as u8, sign])?;
            written += writer.write(abs.to_bytes_le().as_ref())?;

            Ok(written)
        } else if len <= (u32::max_value() as usize) {
            let len_bytes = (len as u32).to_be_bytes();
            let mut written = writer.write(&[TermTag::LargeBig as u8, len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3], sign])?;
            written += writer.write(abs.to_bytes_le().as_ref())?;

            Ok(written)
        } else {
            Err(Error::Message("Integer size is outside of the possible ranges for an erlang term (-2^N..2^N-1 with N=(2^32)*8)".to_string()))
        }
    }
}

#[cfg(feature="bigint")]
impl ToExternalBinary for BigUint {
    fn to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        if let Some(x) = self.to_u128() {
            return x.to_writer(writer);
        }

        let tmp = self.to_bytes_be();

        if tmp.len() <= (u8::max_value() as usize) {
            let len = tmp.len() as u8;
            let mut written = writer.write(&[TermTag::SmallBig as u8, len, 0u8])?;
            written += writer.write(self.to_bytes_le().as_ref())?;

            Ok(written)
        } else if tmp.len() <= (u32::max_value() as usize) {
            let len = (tmp.len() as u32).to_be_bytes();
            let mut written = writer.write(&[TermTag::LargeBig as u8, len[0], len[1], len[2], len[3], 0u8])?;
            written += writer.write(self.to_bytes_le().as_ref())?;

            Ok(written)
        } else {
            Err(Error::Message("Integer size is outside of the possible ranges for an erlang term (-2^N..2^N-1 with N=(2^32)*8)".to_string()))
        }
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

    #[cfg(feature="bigint")]
    use num_bigint::{ BigInt, BigUint, Sign };

    struct Test {
        pub binary: Vec<u8>,
        pub is_negative: bool,
        pub number: u128,
    }

    macro_rules! test {
        ($bin:tt, false, $val:expr) => {
            Test::new(vec!$bin, false, $val as u128)
        };
        ($bin:tt, true, $val:expr) => {
            Test::new(vec!$bin, true, lossless_abs($val as i128))
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
                    (self.with_sign() as i8).to_external_binary(),
                (false, 8, false, n) if n <= (u8::max_value() as u128) =>
                    (self.number as u8).to_external_binary(),
                (true, 16, s, n) if n <= (i16::max_value() as u128) || (s && n <= lossless_abs(i16::min_value() as i128)) =>
                    (self.with_sign() as i16).to_external_binary(),
                (false, 16, false, n) if n <= (u16::max_value() as u128) =>
                    (self.number as u16).to_external_binary(),
                (true, 32, s, n) if n <= (i32::max_value() as u128) || (s && n <= lossless_abs(i32::min_value() as i128)) =>
                    (self.with_sign() as i32).to_external_binary(),
                (false, 32, false, n) if n <= (u32::max_value() as u128) =>
                    (self.number as u32).to_external_binary(),
                (true, 64, s, n) if n <= (i64::max_value() as u128) || (s && n <= lossless_abs(i64::min_value() as i128)) =>
                    (self.with_sign() as i64).to_external_binary(),
                (false, 64, false, n) if n <= (u64::max_value() as u128) =>
                    (self.number as u64).to_external_binary(),
                (true, 128, s, n) if n <= (i128::max_value() as u128) || (s && n <= lossless_abs(i128::min_value() as i128)) =>
                    (self.with_sign() as i128).to_external_binary(),
                (false, 128, false, n) if n <= u128::max_value() =>
                    self.number.to_external_binary(),
                #[cfg(feature="bigint")]
                (true, 0, s, n) =>
                    BigInt::from_biguint(if s {
                        Sign::Minus
                    } else {
                        Sign::Plus
                    }, BigUint::from(n)).to_external_binary(),
                #[cfg(feature="bigint")]
                (false, 0, false, n) =>
                    BigUint::from(n).to_external_binary(),
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
        ($name:ident) => (
            #[test]
            fn $name() {
                Test::run_all(stringify!($name).starts_with("i"), std::mem::size_of::<$name>() as u8);
            }
        );
        ($name:ident, $bits:expr) => (
            #[test]
            fn $name() {
                Test::run_all(stringify!($name).starts_with("i"), $bits);
            }
        );
    }

    maketest!(u8);
    maketest!(i8);
    maketest!(u16);
    maketest!(i16);
    maketest!(u32);
    maketest!(i32);
    maketest!(u64);
    maketest!(i64);
    maketest!(u128);
    maketest!(i128);
    maketest!(usize);
    maketest!(isize);
    maketest!(uint_big, 0);
    maketest!(iint_big, 0);

    #[test]
    #[cfg(feature="bigint")]
    fn bigint_small_upper_boundary() {
        let mut val = BigInt::from(0);
        for _ in 0..255 {
            val <<= 8;
            val |= BigInt::from(0xffu8);
        }
        assert_eq!(vec![
            0x6e, // SMALL_BIG_EXT
            0xff, // Len (0xff=255)
            0x00, // Sign (positive)
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
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, // Value
        ], val.to_external_binary().unwrap(), "(positive)");
        val *= -1;
        assert_eq!(vec![
            0x6e, // SMALL_BIG_EXT
            0xff, // Len (0xff=255)
            0x01, // Sign (negative)
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
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, // Value
        ], val.to_external_binary().unwrap(), "(negative)");
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
            0x6e, // SMALL_BIG_EXT
            0xff, // Len (0xff=255)
            0x00, // Sign (positive)
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
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, // Value
        ], val.to_external_binary().unwrap());
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
            0x6f, // LARGE_BIG_EXT
            0x00, 0x00, 0x01, 0x00, // Len (0x100=256)
            0x00, // Sign (positive)
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
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01, // Value
        ], val.to_external_binary().unwrap(), "(positive)");
        val *= -1;
        assert_eq!(vec![
            0x6f, // LARGE_BIG_EXT
            0x00, 0x00, 0x01, 0x00, // Len (0x100=256)
            0x01, // Sign (negative)
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
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01, // Value
        ], val.to_external_binary().unwrap(), "(negative)");
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
            0x6f, // LARGE_BIG_EXT
            0x00, 0x00, 0x01, 0x00, // Len (0x100=256)
            0x00, // Sign (positive)
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
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01, // Value
        ], val.to_external_binary().unwrap());
    }

    struct FloatTest {
        pub value: f64,
        pub binary: Vec<u8>,
    }

    #[test]
    // Disable the float_cmp error as we are very careful with comparing
    //  floats.
    //  We are not doing any arithmetic (so therefore also not any arithmetic
    //  that disturps the exactness of these floats), so comparing is safe.
    #[allow(clippy::float_cmp)]
    fn floats() {
        // Note: Clippy warns about excessive precision here.
        //  That warning is correct, but we are including the precision
        //  here as it is possible to represent the entire precision in
        //  decimal.
        //  I feel it is more correct to include full precision whenever
        //  possible.
        //  If it is impossible to represent the full precision of the number
        //  in decimal, we should truncate it to as short as possible so that
        //  it is precise enough to be an unambiguous representation of an f64.
        for t in &[
            // f32 and f64
            &FloatTest{value: 0.5, binary: vec![70, 63, 224, 0, 0, 0, 0, 0, 0]},
            &FloatTest{value: -1.66656506061553955078125, binary: vec![70, 191, 250, 170, 64, 32, 0, 0, 0]},
            &FloatTest{value: 0.999999940395355224609375, binary: vec![70, 63, 239, 255, 255, 224, 0, 0, 0]},

            // f64 only
            &FloatTest{value: 0.200000000000000011102230246252, binary: vec![70, 63, 201, 153, 153, 153, 153, 153, 154]}
        ] {
            assert_eq!(t.binary, t.value.to_external_binary().unwrap(), "(f64: {})", t.value);

            // This tests whether the f32 representation is exact.
            //  Only if it is, testing the value as f32 is meaningful.
            if ((t.value as f32) as f64) == t.value {
                assert_eq!(t.binary, (t.value as f32).to_external_binary().unwrap(), "(f32: {})", t.value);
            }
        }
    }
}
