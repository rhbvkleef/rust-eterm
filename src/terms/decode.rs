#[cfg(feature="bigint")]
use num_bigint::{ BigInt, BigUint, Sign };

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

use std::any::Any;
use std::io::{ Read };
use std::convert::TryInto;
use std::str::FromStr;

pub struct DecodeOptions {
    read_string_ext_as_list: bool,
    try_read_list_ext_as_estring: bool,
}

impl Default for DecodeOptions {
    fn default() -> DecodeOptions {
        DecodeOptions {
            read_string_ext_as_list: false,
            try_read_list_ext_as_estring: true,
        }
    }
}

pub fn decode(reader: &mut dyn Read, options: &DecodeOptions) -> Result<Box<dyn ETerm>, Error> {
    let tag: TermTag = read_u8(reader)?
        .try_into()
        .map_err(|_| Error::Message("Unsupported term type".to_string()))?;

    match tag {
        TermTag::String => {
            if options.read_string_ext_as_list {
                let length = read_u16(reader)? as usize;
                Ok(Box::from(EString(read_string(reader, length)?)))
            } else {
                let length = read_u16(reader)? as usize;
                let mut entries: Vec<Box<dyn ETerm>> = vec![];

                for byte in read_vec(reader, length)? {
                    entries.push(Box::from(byte));
                }

                Ok(Box::from(EList(entries)))
            }
        },
        TermTag::List => {
            let len = read_u32(reader)? as usize;
            if options.try_read_list_ext_as_estring {
                let list: Vec<Box<dyn ETerm>> = read_terms(reader, len, options)?;
                let tail: Box<dyn ETerm> = decode(reader, options)?;

                let mut bytes: Vec<u8> = vec![];

                for e in &list {
                    let x: &dyn Any = e;

                    if let Some(byte) = (*x).downcast_ref::<u8>() {
                        bytes.push(*byte);
                    } else {
                        let y: &dyn Any = &tail;

                        return if (*y).downcast_ref::<ENil>().is_some() {
                            Ok(Box::from(EList(list)))
                        } else {
                            Ok(Box::from(ENonProperList { data: list, tail }))
                        };
                    }
                }

                Ok(Box::from(EString(read_string(&mut bytes.as_mut_slice().as_ref(), list.len())?)))
            } else {
                Ok(Box::from(EList(read_terms(reader, len, options)?)))
            }
        },
        TermTag::NewFloat => Ok(Box::from(read_f64(reader)?)),
        TermTag::Float => {
            match f64::from_str(read_string(reader, 31)?.as_ref()) {
                Ok(f) => Ok(Box::from(f)),
                Err(e) => Err(Error::Message(e.to_string())),
            }
        },
        TermTag::SmallInteger => Ok(Box::from(read_u8(reader)?)),
        TermTag::Integer => Ok(Box::from(read_i32(reader)?)),
        TermTag::Nil => Ok(Box::from(ENil::default())),
        TermTag::SmallBig => {
            let len = read_u8(reader)? as usize;
            let negative = read_u8(reader)? != 0;
            let value = read_vec(reader, len)?;
            num_vec_to_term(negative, value)
        },
        TermTag::LargeBig => {
            let len = read_u32(reader)? as usize;
            let negative = read_u8(reader)? != 0;
            let value = read_vec(reader, len)?;
            num_vec_to_term(negative, value)
        },
        TermTag::SmallTuple => {
            let len = read_u8(reader)? as usize;
            Ok(Box::from(ETuple(read_terms(reader, len, options)?)))
        },
        TermTag::LargeTuple => {
            let len = read_u32(reader)? as usize;
            Ok(Box::from(ETuple(read_terms(reader, len, options)?)))
        },
        TermTag::Map => {
            let len = read_u32(reader)? as usize;
            Ok(Box::from(read_map(reader, options, len)?))
        },
        TermTag::SmallAtom => {
            let len = read_u8(reader)? as usize;
            Ok(Box::from(EAtom(read_latin1(reader, len)?)))
        },
        TermTag::Atom => {
            let len = read_u16(reader)? as usize;
            Ok(Box::from(EAtom(read_latin1(reader, len)?)))
        },
        TermTag::SmallAtomUtf8 => {
            let len = read_u8(reader)? as usize;
            Ok(Box::from(EAtom(read_string(reader, len)?)))
        },
        TermTag::AtomUtf8 => {
            let len = read_u16(reader)? as usize;
            Ok(Box::from(EAtom(read_string(reader, len)?)))
        },
        TermTag::Binary => {
            let len = read_u32(reader)? as usize;
            Ok(Box::from(EBinary(read_vec(reader, len)?)))
        }
        _ => Err(Error::Message("Decoding of this term type not yet supported".to_string())),
    }
}

fn read_terms(reader: &mut dyn Read, length: usize, options: &DecodeOptions) -> Result<Vec<Box<dyn ETerm>>, Error> {
    let mut entries: Vec<Box<dyn ETerm>> = vec![];

    for _ in 0..length {
        entries.push(decode(reader, options)?)
    }

    Ok(entries)
}

macro_rules! read_type {
    ($fname:ident, $type:ty) => {
        fn $fname(reader: &mut dyn Read) -> Result<$type, Error> {
            let mut buf = [0; std::mem::size_of::<$type>()];
            reader.read_exact(&mut buf)?;
            Ok(<$type>::from_be_bytes(buf))
        }
    }
}

read_type!(read_u8, u8);
read_type!(read_u16, u16);
read_type!(read_u32, u32);
read_type!(read_i32, i32);
read_type!(read_f64, f64);

fn read_string(reader: &mut dyn Read, length: usize) -> Result<String, Error> {
    String::from_utf8(read_vec(reader, length)?)
        .map_err(|e| Error::Message(e.to_string()))
}

fn read_latin1(reader: &mut dyn Read, length: usize) -> Result<String, Error> {
    Ok(read_vec(reader, length)?.iter().map(|&c| c as char).collect())
}

fn read_vec(reader: &mut dyn Read, length: usize) -> Result<Vec<u8>, Error> {
    let mut buf = vec![0; length];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

fn read_map(reader: &mut dyn Read, options: &DecodeOptions, length: usize) -> Result<EMap, Error> {
    let mut result: Vec<(Box<dyn ETerm>, Box<dyn ETerm>)> = Vec::with_capacity(length);

    for _ in 0..length {
        result.push((
            decode(reader, options)?,
            decode(reader, options)?,
        ));
    }

    Ok(EMap(result))
}

fn num_vec_to_term(negative: bool, data: Vec<u8>) -> Result<Box<dyn ETerm>, Error> {
    match (negative, data.len()) {
        (_, 0) => Ok(Box::from(0u8)),
        (false, 1..=16) =>
            Ok(Box::from(u128::from_be_bytes(slice_16_from_vec(&data)))),
        (true, 1..=16) if u128::from_be_bytes(slice_16_from_vec(&data)) <= i128::max_value() as u128 =>
            Ok(Box::from(-i128::from_be_bytes(slice_16_from_vec(&data)))),
        (true, 16) if data[0] == 0x7fu8 => Ok(Box::from(i128::min_value())),
        #[cfg(feature="bigint")]
        (false, _) => Ok(Box::from(BigUint::from_bytes_le(&data))),
        #[cfg(feature="bigint")]
        (true, _) => Ok(Box::from(BigInt::from_bytes_le(Sign::Minus, &data))),
        #[cfg(not(feature="bigint"))]
        _ => Err(Error::Message("Attempting to deserialize a big integer, but the feature is not enabled.".to_string())),
    }
}

fn slice_16_from_vec(data: &[u8]) -> [u8; 16] {
    let mut slice = [0; 16];

    slice[16 - std::cmp::min(16, data.len())..16].clone_from_slice(&data[16 - std::cmp::min(16, data.len())..16]);

    slice
}
