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
            let data: &[u8; 16] = &self.to_be_bytes();
            let bytes: u8 = ((128 - self.leading_zeros()) / 8) as u8;

            let mut amount = writer.write(&[super::SMALL_BIG_EXT, bytes])?;
            amount += writer.write(&data[..(bytes as usize)])?;

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
        match self.to_i128() {
            Some(x) => return x.try_to_writer(writer),
            _ => Option::<i32>::None
        };

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
            Err(Error::data(ErrorCode::ValueNotEncodable(Box::from("Integer is too large or too small to be encoded as an erlang term."))))
        }
    }
}

#[cfg(feature="bigint")]
impl TryToExternalBinary for BigUint {
    fn try_to_writer(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        match self.to_i128() {
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
            Err(Error::data(ErrorCode::ValueNotEncodable(Box::from("Integer is too large or too small to be encoded as an erlang term."))))
        }
    }
}

fn write(writer: &mut dyn Write, bytes: &[u8]) -> Result<usize, Error> {
    match writer.write(bytes) {
        Ok(written) => Ok(written),
        Err(e) => Err(Error::io(e)),
    }
}
