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
    ETermBinary,
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

/// Replacement for `std::convert::TryInto<T>` that doesn't require `Sized`.
pub trait TryToExternalBinary {
    fn try_to_external_binary(&self) -> Result<ETermBinary, Error>;
}

/// Replacement for `std::convert::Into<T>` that doesn't require `Sized`.
pub trait ToExternalBinary {
    fn to_external_binary(&self) -> ETermBinary;
}

impl<X> TryToExternalBinary for X where X: ToExternalBinary {
    fn try_to_external_binary(&self) -> Result<ETermBinary, Error> {
        Ok(ToExternalBinary::to_external_binary(self))
    }
}

impl ToExternalBinary for i8 {
    fn to_external_binary(&self) -> ETermBinary {
        (*self as i32).to_external_binary()
    }
}

impl ToExternalBinary for u8 {
    fn to_external_binary(&self) -> ETermBinary {
        let data: &[u8; 1] = &self.to_be_bytes();

        ETermBinary(vec![97u8, data[0]])
    }
}

impl ToExternalBinary for i16 {
    fn to_external_binary(&self) -> ETermBinary {
        (*self as i32).to_external_binary()
    }
}

impl ToExternalBinary for u16 {
    fn to_external_binary(&self) -> ETermBinary {
        (*self as i32).to_external_binary()
    }
}

impl ToExternalBinary for i32 {
    fn to_external_binary(&self) -> ETermBinary {
        if *self <= u8::max_value().into() && *self >= 0 {
            (*self as u8).to_external_binary()
        } else {
            let mut result = vec![98u8];
            result.extend(&self.to_be_bytes());

            ETermBinary(result)
        }
    }
}

impl ToExternalBinary for u32 {
    fn to_external_binary(&self) -> ETermBinary {
        (*self as i128).to_external_binary()
    }
}

impl ToExternalBinary for i64 {
    fn to_external_binary(&self) -> ETermBinary {
        (*self as i128).to_external_binary()
    }
}

impl ToExternalBinary for u64 {
    fn to_external_binary(&self) -> ETermBinary {
        (*self as i128).to_external_binary()
    }
}

impl ToExternalBinary for i128 {
    fn to_external_binary(&self) -> ETermBinary {
        if *self <= i32::max_value().into() && *self >= i32::min_value().into() {
            (*self as i32).to_external_binary()
        } else {
            let data: &[u8; 16] = &self.to_be_bytes();
            let bytes: u8 = ((128 - self.leading_zeros()) / 8) as u8;
            let mut result = vec![110u8, bytes];
            result.extend_from_slice(&data[..(bytes as usize)]);

            ETermBinary(result)
        }
    }
}

impl ToExternalBinary for u128 {
    fn to_external_binary(&self) -> ETermBinary {
        // The big number here is i128::max_value() as a `From<u128>` for i128 is not implemented
        if *self <= 170_141_183_460_469_231_731_687_303_715_884_105_727u128 {
            (*self as i128).to_external_binary()
        } else {
            let data: &[u8; 16] = &self.to_be_bytes();
            let mut result = vec![110u8, 0u8, 0u8];
            for e in data.iter() {
                if *e != 0u8 {
                    result.push(*e)
                }
            }

            result[1] = (result.len() - 3) as u8;

            ETermBinary(result)
        }
    }
}

impl ToExternalBinary for isize {
    fn to_external_binary(&self) -> ETermBinary {
        (*self as i128).to_external_binary()
    }
}

impl ToExternalBinary for usize {
    fn to_external_binary(&self) -> ETermBinary {
        (*self as u128).to_external_binary()
    }
}

impl ToExternalBinary for ENil {
    fn to_external_binary(&self) -> ETermBinary {
        ETermBinary(vec![106u8])
    }
}

impl<'a> TryToExternalBinary for EList<'a> {
    fn try_to_external_binary(&self) -> Result<ETermBinary, Error> {
        if self.0.is_empty() {
            Ok(ETermBinary((ENil {}).try_to_binary()?))
        } else {
            let len: [u8; 4] = (self.0.len() as i32).to_be_bytes();
            let mut result = vec![108, len[0], len[1], len[2], len[3]];
            for d in self.0.iter() {
                result.extend(d.try_to_binary()?);
            }
            result.extend((ENil {}).try_to_binary()?);
            Ok(ETermBinary(result))
        }
    }
}

impl<'a> TryToExternalBinary for ENonProperList<'a> {
    fn try_to_external_binary(&self) -> Result<ETermBinary, Error> {
        if self.data.is_empty() {
            Ok(ETermBinary(self.tail.try_to_binary()?))
        } else {
            let len: [u8; 4] = (self.data.len() as i32).to_be_bytes();
            let mut result: Vec<u8> = vec![108, len[0], len[1], len[2], len[3]];
            for d in self.data.iter() {
                result.extend(d.try_to_binary()?);
            }

            result.extend(self.tail.try_to_binary()?);

            Ok(ETermBinary(result))
        }
    }
}

impl TryToExternalBinary for EAtom {
    fn try_to_external_binary(&self) -> Result<ETermBinary, Error> {
        let byte_length = self.0.as_bytes().len();

        if byte_length <= u8::max_value().into() {
            let mut result = vec![119u8, byte_length as u8];
            result.extend(self.0.as_bytes());
            Ok(ETermBinary(result))
        } else if byte_length <= u16::max_value().into() {
            let len: [u8; 8] = byte_length.to_be_bytes();
            let mut result = vec![118u8, len[6], len[7]];
            result.extend(self.0.as_bytes());
            Ok(ETermBinary(result))
        } else {
            Err(Error::data(ErrorCode::ValueNotEncodable(Box::from((&self.0).as_str().to_owned()))))
        }
    }
}

impl TryToExternalBinary for f32 {
    fn try_to_external_binary(&self) -> Result<ETermBinary, Error> {
        if self.is_finite() {
            let bytes = self.to_be_bytes();
            Ok(ETermBinary(vec![70u8, 0, 0, 0, 0, bytes[0], bytes[1], bytes[2], bytes[3]]))
        } else {
            Err(Error::data(ErrorCode::ValueNotEncodable(Box::from(self.to_string()))))
        }
    }
}

impl TryToExternalBinary for f64 {
    fn try_to_external_binary(&self) -> Result<ETermBinary, Error> {
        if self.is_finite() {
            let bytes = self.to_be_bytes();
            Ok(ETermBinary(vec![70u8, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]))
        } else {
            Err(Error::data(ErrorCode::ValueNotEncodable(Box::from(self.to_string()))))
        }
    }
}

impl TryToExternalBinary for EExport {
    fn try_to_external_binary(&self) -> Result<ETermBinary, Error> {
        let mut result = vec![113u8];
        result.extend(ETerm::try_to_binary(&self.module)?);
        result.extend(ETerm::try_to_binary(&self.function)?);
        result.extend(ETerm::try_to_binary(&self.arity)?);

        Ok(ETermBinary(result))
    }
}

impl<'a> TryToExternalBinary for ETuple<'a> {
    fn try_to_external_binary(&self) -> Result<ETermBinary, Error> {
        let mut result: Vec<u8>;
        let len = (self.0.len() as u32).to_be_bytes();
        if self.0.len() <= u8::max_value().into() {
            result = vec![104u8, len[3]];
        } else {
            result = vec![105u8, len[0], len[1], len[2], len[3]];
        }
        for d in self.0.iter() {
            result.extend(d.try_to_binary()?);
        }

        Ok(ETermBinary(result))
    }
}

impl TryToExternalBinary for EString {
    fn try_to_external_binary(&self) -> Result<ETermBinary, Error> {
        let byte_length = self.0.as_bytes().len();

        if byte_length <= u16::max_value().into() {
            let len: [u8; 8] = byte_length.to_be_bytes();
            let mut result = vec![107u8, len[6], len[7]];
            result.extend(self.0.as_bytes());
            Ok(ETermBinary(result))
        } else {
            EList(&(self.0.as_bytes().iter().map(|x| x as &dyn ETerm).collect())).try_to_external_binary()
        }
    }
}

impl TryToExternalBinary for EPort {
    fn try_to_external_binary(&self) -> Result<ETermBinary, Error> {
        let mut result = vec![89u8];
        result.extend(self.node.try_to_binary()?);
        result.extend(self.id.to_be_bytes().iter());
        result.extend(self.creation.to_be_bytes().iter());

        Ok(ETermBinary(result))
    }
}

impl TryToExternalBinary for EPid {
    fn try_to_external_binary(&self) -> Result<ETermBinary, Error> {
        let mut result = vec![88u8];
        result.extend(self.node.try_to_binary()?);
        result.extend(self.id.to_be_bytes().iter());
        result.extend(self.serial.to_be_bytes().iter());
        result.extend(self.creation.to_be_bytes().iter());
        
        Ok(ETermBinary(result))
    }
}
impl<'a> TryToExternalBinary for EMap<'a> {
    fn try_to_external_binary(&self) -> Result<ETermBinary, Error> {
        let mut result = vec![116u8];
        result.extend((self.0.len() as u32).to_be_bytes().iter());

        for (k, v) in self.0 {
            result.extend(k.try_to_binary()?);
            result.extend(v.try_to_binary()?);
        };

        Ok(ETermBinary(result))
    }
}

impl<'a> ToExternalBinary for EBinary<'a> {
    fn to_external_binary(&self) -> ETermBinary {
        let mut result = vec![109u8];
        result.extend(self.0.len().to_be_bytes().iter());
        result.extend(self.0);

        ETermBinary(result)
    }
}

#[cfg(feature="bigint")]
impl TryToExternalBinary for BigInt {
    fn try_to_external_binary(&self) -> Result<ETermBinary, Error> {
        match self.to_i128() {
            Some(x) => return x.try_to_external_binary(),
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
            let mut result = vec![110u8, len, sign];
            result.extend(tmp.iter());
            return Ok(ETermBinary(result));
        };

        if tmp.len() <= (u32::max_value() as usize) {
            let len = (tmp.len() as u32).to_be_bytes();
            let mut result = vec![111u8, len[0], len[1], len[2], len[3], sign];
            result.extend(tmp.iter());
            return Ok(ETermBinary(result));
        };

        Err(Error::data(ErrorCode::ValueNotEncodable(Box::from("Integer is too large or too small to be encoded as an erlang term."))))
    }
}

#[cfg(feature="bigint")]
impl TryToExternalBinary for BigUint {
    fn try_to_external_binary(&self) -> Result<ETermBinary, Error> {
        match self.to_i128() {
            Some(x) => return x.try_to_external_binary(),
            _ => Option::<i32>::None
        };

        let tmp = self.to_bytes_be();

        if tmp.len() <= (u8::max_value() as usize) {
            let len = tmp.len() as u8;
            let mut result = vec![110u8, len, 0u8];
            result.extend(tmp.iter());
            return Ok(ETermBinary(result));
        };

        if tmp.len() <= (u32::max_value() as usize) {
            let len = (tmp.len() as u32).to_be_bytes();
            let mut result = vec![111u8, len[0], len[1], len[2], len[3], 0u8];
            result.extend(tmp.iter());
            return Ok(ETermBinary(result));
        };

        Err(Error::data(ErrorCode::ValueNotEncodable(Box::from("Integer is too large or too small to be encoded as an erlang term."))))
    }
}
