#[macro_use]
extern crate lazy_static;

#[cfg(feature="bigint")]
extern crate num_bigint;

#[cfg(feature="bigint")]
extern crate num_traits;

extern crate regex;

pub mod error;
pub mod terms;
