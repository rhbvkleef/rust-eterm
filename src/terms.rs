//! Currently implemented term types:
//! * ATOM_UTF8_EXT, SMALL_ATOM_UTF8_EXT (for EAtom(String))
//! * INTEGER_EXT, SMALL_INTEGER_EXT (for {f,u}{8,16,32,64,128,size})
//! * FLOAT_EXT (for f32, f64)
//! * NIL_EXT (for ENil)
//! * LIST_EXT (for EList, ENonProperList)
//! * EXPORT_EXT (for EExport)
//!
//! Not yet implemented term types:
//! * ATOM_CACHE_REF
//! * PORT_EXT
//! * NEW_PORT_EXT
//! * PID_EXT
//! * NEW_PID_EXT
//! * SMALL_TUPLE_EXT
//! * LARGE_TUPLE_EXT
//! * MAP_EXT
//! * STRING_EXT
//! * BINARY_EXT
//! * SMALL_BIG_EXT
//! * LARGE_BIG_EXT
//! * REFERENCE_EXT (deprecated)
//! * NEW_REFERENCE_EXT
//! * NEWER_REFERENCE_EXT
//! * FUN_EXT
//! * NEW_FUN_EXT
//! * BIT_BINARY_EXT
//! * ATOM_EXT (deprecated)
//! * SMALL_ATOM_EXT (deprecated)

use super::error::{Error, ErrorCode};

macro_rules! into_etermstr_from_tostr {
    ($structure:ty) => (
        impl<'a> TryTo<ETermString> for $structure {
            fn try_to(&self) -> Result<ETermString, Error> {
                Ok(ETermString((&self.to_string()).to_owned()))
            }
        }
        impl<'a> TryTo<EPrettyTermString> for $structure {
            fn try_to(&self) -> Result<EPrettyTermString, Error> {
                Ok(EPrettyTermString(self.try_to()?))
            }
        }
    )
}

pub trait TryTo<T> {
    fn try_to(&self) -> Result<T, Error>;
}

pub trait To<T> {
    fn to(&self) -> T;
}

impl<X, T> TryTo<T> for X where X: To<T> {
    fn try_to(&self) -> Result<T, Error> {
        Ok(To::<T>::to(self))
    }
}

#[derive(Clone)]
pub struct EBinary(Vec<u8>);

impl To<Vec<u8>> for EBinary {
    fn to(&self) -> Vec<u8> {
        self.0.to_owned()
    }
}

#[derive(Clone)]
pub struct ETermString(String);

impl To<String> for ETermString {
    fn to(&self) -> String {
        self.0.to_owned()
    }
}

#[derive(Clone)]
pub struct EPrettyTermString(ETermString);

impl To<ETermString> for EPrettyTermString {
    fn to(&self) -> ETermString {
        self.0.to_owned()
    }
}

impl To<String> for EPrettyTermString {
    fn to(&self) -> String {
        To::<String>::to(&self.0)
    }
}

pub trait ETerm: TryTo<EBinary> + TryTo<ETermString> + TryTo<EPrettyTermString> {
    fn try_to_term_string(&self) -> Result<String, Error> {
        Ok(To::to(&TryTo::<ETermString>::try_to(self)?))
    }

    fn try_to_binary(&self) -> Result<Vec<u8>, Error> {
        Ok(To::to(&TryTo::<EBinary>::try_to(self)?))
    }
}

impl<T> ETerm for T where T: TryTo<EBinary> + TryTo<ETermString> + TryTo<EPrettyTermString> {}

impl TryTo<EBinary> for i8 {
    fn try_to(&self) -> Result<EBinary, Error> {
        let data: &[u8; 1] = &self.to_be_bytes();
        Ok(EBinary(vec![97u8, data[0]]))
    }
}

into_etermstr_from_tostr!(i8);

impl TryTo<EBinary> for u8 {
    fn try_to(&self) -> Result<EBinary, Error> {
        (*self as i32).try_to()
    }
}

into_etermstr_from_tostr!(u8);

impl TryTo<EBinary> for i16 {
    fn try_to(&self) -> Result<EBinary, Error> {
        (*self as i32).try_to()
    }
}

into_etermstr_from_tostr!(i16);

impl TryTo<EBinary> for u16 {
    fn try_to(&self) -> Result<EBinary, Error> {
        (*self as i32).try_to()
    }
}

into_etermstr_from_tostr!(u16);

impl TryTo<EBinary> for i32 {
    fn try_to(&self) -> Result<EBinary, Error> {
        if *self <= i8::max_value().into() && *self >= i8::min_value().into() {
            (*self as i8).try_to()
        } else {
            let data: &[u8; 4] = &self.to_be_bytes();
            Ok(EBinary(concat(&[98u8], data)))
        }
    }
}

into_etermstr_from_tostr!(i32);

impl TryTo<EBinary> for u32 {
    fn try_to(&self) -> Result<EBinary, Error> {
        (*self as i128).try_to()
    }
}

into_etermstr_from_tostr!(u32);

impl TryTo<EBinary> for i64 {
    fn try_to(&self) -> Result<EBinary, Error> {
        (*self as i128).try_to()
    }
}

into_etermstr_from_tostr!(i64);

impl TryTo<EBinary> for u64 {
    fn try_to(&self) -> Result<EBinary, Error> {
        (*self as i128).try_to()
    }
}

into_etermstr_from_tostr!(u64);

impl TryTo<EBinary> for i128 {
    fn try_to(&self) -> Result<EBinary, Error> {
        if *self <= i32::max_value().into() && *self >= i32::min_value().into() {
            (*self as i32).try_to()
        } else {
            let data: &[u8; 16] = &self.to_be_bytes();
            let bytes: u8 = ((128 - self.leading_zeros()) / 8) as u8;
            Ok(EBinary(concat(&[110u8, bytes], &data[..(bytes as usize)])))
        }
    }
}

into_etermstr_from_tostr!(i128);

impl TryTo<EBinary> for u128 {
    fn try_to(&self) -> Result<EBinary, Error> {
        // The big number here is i128::max_value() as a `From<u128>` for i128 is not implemented
        if *self <= 170141183460469231731687303715884105727u128 {
            (*self as i128).try_to()
        } else {
            let data: &[u8; 16] = &self.to_be_bytes();
            Ok(EBinary(concat(&[110u8, 17u8, 0], data)))
        }
    }
}

into_etermstr_from_tostr!(u128);

impl TryTo<EBinary> for isize {
    fn try_to(&self) -> Result<EBinary, Error> {
        (*self as i128).try_to()
    }
}

into_etermstr_from_tostr!(isize);

impl TryTo<EBinary> for usize {
    fn try_to(&self) -> Result<EBinary, Error> {
        (*self as u128).try_to()
    }
}

into_etermstr_from_tostr!(usize);

pub struct ENil;

impl ToString for ENil {
    fn to_string(&self) -> String {
        "[]".to_string()
    }
}

impl TryTo<EBinary> for ENil {
    fn try_to(&self) -> Result<EBinary, Error> {
        Ok(EBinary(vec![106u8]))
    }
}

into_etermstr_from_tostr!(ENil);

pub struct EList<'a> {
    data: &'a Vec<&'a dyn ETerm>
}

impl<'a> From<&'a Vec<&'a dyn ETerm>> for EList<'a> {
    fn from(data: &'a Vec<&'a dyn ETerm<>>) -> Self {
        EList {
            data
        }
    }
}

impl<'a> ToString for EList<'a> {
    fn to_string(&self) -> String {
        let mut s = "[".to_string();
        for d in self.data.iter() {
            s.push_str(d.try_to_term_string().unwrap().as_ref());
        }
        s.push(']');
        s
    }
}

impl<'a> TryTo<EBinary> for EList<'a> {
    fn try_to(&self) -> Result<EBinary, Error> {
        let len: [u8; 4] = (self.data.len() as i32).to_be_bytes();
        let mut result = vec![108, len[0], len[1], len[2], len[3]];
        for d in self.data.iter() {
            result.extend(d.try_to_binary()?);
        }
        result.extend((ENil {}).try_to_binary()?);
        Ok(EBinary(result))
    }
}

into_etermstr_from_tostr!(EList<'a>);

pub struct ENonProperList<'a> {
    data: &'a Vec<&'a dyn ETerm>,
    tail: &'a dyn ETerm,
}

impl<'a> ToString for ENonProperList<'a> {
    fn to_string(&self) -> String {
        let mut s = "[".to_string();
        for d in self.data.iter() {
            s.push_str(d.try_to_term_string().unwrap().as_ref());
        }
        s.push('|');
        s.push_str(self.tail.try_to_term_string().unwrap().as_ref());
        s.push(']');
        s
    }
}

impl<'a> TryTo<EBinary> for ENonProperList<'a> {
    fn try_to(&self) -> Result<EBinary, Error> {
        let len: [u8; 4] = (self.data.len() as i32).to_be_bytes();
        let mut result: Vec<u8> = vec![108, len[0], len[1], len[2], len[3]];
        for d in self.data.iter() {
            result.extend(d.try_to_binary()?);
        }

        result.extend(self.tail.try_to_binary()?);

        Ok(EBinary(result))
    }
}

into_etermstr_from_tostr!(ENonProperList<'a>);

pub struct EAtom(String);

impl ToString for EAtom {
    fn to_string(&self) -> String {
        (&self.0).to_owned()
    }
}

impl TryTo<EBinary> for EAtom {
    fn try_to(&self) -> Result<EBinary, Error> {
        if self.0.len() <= u8::max_value().into() {
            Ok(EBinary(concat(&[119u8, self.0.len() as u8], self.0.as_bytes())))
        } else if self.0.len() <= 65535 {
            let len: [u8; 8] = self.0.len().to_be_bytes();
            Ok(EBinary(concat(&[118, len[6], len[7]], self.0.as_bytes())))
        } else {
            Err(Error::data(ErrorCode::ValueNotEncodable(Box::from((&self.0).as_str().to_owned()))))
        }
    }
}

into_etermstr_from_tostr!(EAtom);

impl TryTo<EBinary> for f32 {
    fn try_to(&self) -> Result<EBinary, Error> {
        if self.is_finite() {
            let bytes = self.to_be_bytes();
            Ok(EBinary(vec![70u8, 0, 0, 0, 0, bytes[0], bytes[1], bytes[2], bytes[3]]))
        } else {
            Err(Error::data(ErrorCode::ValueNotEncodable(Box::from(self.to_string().to_owned()))))
        }
    }
}

into_etermstr_from_tostr!(f32);

impl TryTo<EBinary> for f64 {
    fn try_to(&self) -> Result<EBinary, Error> {
        if self.is_finite() {
            let bytes = self.to_be_bytes();
            Ok(EBinary(vec![70u8, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]))
        } else {
            Err(Error::data(ErrorCode::ValueNotEncodable(Box::from(self.to_string().to_owned()))))
        }
    }
}

into_etermstr_from_tostr!(f64);

pub struct EExport {
    module: EAtom,
    function: EAtom,
    arity: u8,
}

impl ToString for EExport {
    fn to_string(&self) -> String {
        let mut result = "{".to_string();
        result.push_str(self.module.to_string().as_ref());
        result.push(',');
        result.push_str(self.function.to_string().as_ref());
        result.push(',');
        result.push_str(self.arity.to_string().as_ref());
        result.push('}');
        result
    }
}

impl TryTo<EBinary> for EExport {
    fn try_to(&self) -> Result<EBinary, Error> {
        let mut result = vec![113u8];
        result.extend(ETerm::try_to_binary(&self.module)?);
        result.extend(ETerm::try_to_binary(&self.function)?);
        result.extend(ETerm::try_to_binary(&self.arity)?);
        Ok(EBinary(result))
    }
}

fn concat<T>(p1: &[T], p2: &[T]) -> Vec<T> where T: Clone {
    let mut concat = p1.to_vec();
    concat.extend(p2.iter().cloned());
    concat
}
