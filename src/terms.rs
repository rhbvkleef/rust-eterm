//! ## Partially implemented term types (into binary, String):
//! * [`ATOM_UTF8_EXT`], [`SMALL_ATOM_UTF8_EXT`] (for [`EAtom`], quoted atoms
//!   are not yet supported)
//! * [`STRING_EXT`] (for [`EString`], Strings that require escaping are not
//!   yet supported)
//!
//! ## Currently implemented term types (into binary, String):
//! * [`INTEGER_EXT`], [`SMALL_INTEGER_EXT`] (for `{i,u}{8,16,32,64,128,size}`)
//! * [`FLOAT_EXT`] (for `f32`, `f64`)
//! * [`NIL_EXT`] (for [`ENil`])
//! * [`LIST_EXT`] (for [`EList`], [`ENonProperList`])
//! * [`EXPORT_EXT`] (for [`EExport`])
//! * [`LARGE_TUPLE_EXT`], [`SMALL_TUPLE_EXT`] (for [`ETuple`])
//! * [`MAP_EXT`] (for [`EMap`])
//! * [`BINARY_EXT`] (for [`EBinary`])
//! * [`SMALL_BIG_EXT`] (for `BigInt`, `BigUint`, `u32`, `i64`, `u64`, `i128`,
//!   `u128`)
//! * [`LARGE_BIG_EXT`] (for `BigInt`, `BigUint`)
//! * [`NEW_PORT_EXT`] (for [`EPort`])
//! * [`NEW_PID_EXT`] (for [`EPid`])
//!
//! ## Currently implemented term types (from binary, String)
//! None
//!
//! ## Currently implemented term types (both from binary, String and into binary, String)
//! None
//!
//! ## Not yet implemented term/value types:
//! * [`NEWER_REFERENCE_EXT`]
//! * [`BIT_BINARY_EXT`]
//! * [`ATOM_CACHE_REF`] (distribution header will probably not be supported, or not soon at least)
//! * [`FUN_EXT`] (seems unnecessary for now, maybe in the future?)
//! * [`NEW_FUN_EXT`] (seems unnecessary for now, maybe in the future?)
//! * [`REFERENCE_EXT`] (deprecated, decoding support will be added)
//! * [`NEW_REFERENCE_EXT`] (decoding support will be added)
//! * [`ATOM_EXT`] (deprecated, decoding support will be added)
//! * [`SMALL_ATOM_EXT`] (deprecated, decoding support will be added)
//! * [`PORT_EXT`] (decoding support will be added)
//! * [`PID_EXT`] (decoding support will be added)
//! * [`DIST_HDR`], including:
//!   * [`DIST_HDR_NORMAL`]
//!   * [`DIST_HDR_FRAGMENTED`]
//!   * [`DIST_HDR_FRAGMENT`]
//!   * [`DIST_HDR_COMPRESSED`]
//! 
//! [`DIST_HDR`]: static.DIST_HDR.html
//! [`DIST_HDR_NORMAL`]: static.DIST_HDR_NORMAL.html
//! [`DIST_HDR_FRAGMENTED`]: static.DIST_HDR_FRAGMENTED.html
//! [`DIST_HDR_FRAGMENT`]: static.DIST_HDR_FRAGMENT.html
//! [`DIST_HDR_COMPRESSED`]: static.DIST_HDR_COMPRESSED.html
//! [`ATOM_CACHE_REF`]: static.ATOM_CACHE_REF.html
//! [`SMALL_INTEGER_EXT`]: static.SMALL_INTEGER_EXT.html
//! [`INTEGER_EXT`]: static.INTEGER_EXT.html
//! [`FLOAT_EXT`]: static.FLOAT_EXT.html
//! [`PORT_EXT`]: static.PORT_EXT.html
//! [`NEW_PORT_EXT`]: static.NEW_PORT_EXT.html
//! [`PID_EXT`]: static.PID_EXT.html
//! [`NEW_PID_EXT`]: static.NEW_PID_EXT.html
//! [`SMALL_TUPLE_EXT`]: static.SMALL_TUPLE_EXT.html
//! [`LARGE_TUPLE_EXT`]: static.LARGE_TUPLE_EXT.html
//! [`MAP_EXT`]: static.MAP_EXT.html
//! [`NIL_EXT`]: static.NIL_EXT.html
//! [`STRING_EXT`]: static.STRING_EXT.html
//! [`LIST_EXT`]: static.LIST_EXT.html
//! [`BINARY_EXT`]: static.BINARY_EXT.html
//! [`SMALL_BIG_EXT`]: static.SMALL_BIG_EXT.html
//! [`LARGE_BIG_EXT`]: static.LARGE_BIG_EXT.html
//! [`REFERENCE_EXT`]: static.REFERENCE_EXT.html
//! [`NEW_REFERENCE_EXT`]: static.NEW_REFERENCE_EXT.html
//! [`NEWER_REFERENCE_EXT`]: static.NEWER_REFERENCE_EXT.html
//! [`FUN_EXT`]: static.FUN_EXT.html
//! [`NEW_FUN_EXT`]: static.NEW_FUN_EXT.html
//! [`EXPORT_EXT`]: static.EXPORT_EXT.html
//! [`BIT_BINARY_EXT`]: static.BIT_BINARY_EXT.html
//! [`NEW_FLOAT_EXT`]: static.NEW_FLOAT_EXT.html
//! [`ATOM_UTF8_EXT`]: static.ATOM_UTF8_EXT.html
//! [`SMALL_ATOM_UTF8_EXT`]: static.SMALL_ATOM_UTF8_EXT.html
//! [`ATOM_EXT`]: static.ATOM_EXT.html
//! [`SMALL_ATOM_EXT`]: static.SMALL_ATOM_EXT.html
//! 
//! [`ETermBinary`]: struct.ETermBinary.html
//! [`ETermString`]: struct.ETermString.html
//! [`ETermPrettyString`]: struct.ETermPrettyString.html
//! [`ENil`]: struct.ENil.html
//! [`EList`]: struct.EList.html
//! [`ENonProperList`]: struct.ENonProperList.html
//! [`EAtom`]: struct.EAtom.html
//! [`EExport`]: struct.EExport.html
//! [`ETuple`]: struct.ETuple.html
//! [`EString`]: struct.EString.html
//! [`EPort`]: struct.EPort.html
//! [`EPid`]: struct.EPid.html
//! [`EMap`]: struct.EMap.html
//! [`EBinary`]: struct.EBinary.html
//! 
//! [`ETerm`]: trait.ETerm.html
//! [`To`]: trait.To.html
//! [`TryTo`]: trait.TryTo.html

use super::error::{Error, ErrorCode};

#[cfg(feature="bigint")]
use {
    num_bigint::{ BigInt, BigUint, Sign },
    num_traits::{ sign::Signed, cast::ToPrimitive },
};

macro_rules! into_etermstr_from_tostr {
    ($structure:ty) => (
        impl<'a> To<ETermString> for $structure {
            fn to(&self) -> ETermString {
                ETermString((&self.to_string()).to_owned())
            }
        }
        impl<'a> TryTo<ETermPrettyString> for $structure {
            fn try_to(&self) -> Result<ETermPrettyString, Error> {
                Ok(ETermPrettyString(self.try_to()?))
            }
        }
    )
}

/// This is the code of the start of a message
/// 
/// The distribution header can be of multiple variants:
/// * [`DIST_HDR_NORMAL`]
/// * [`DIST_HDR_FRAGMENTED`]
/// * [`DIST_HDR_FRAGMENT`]
/// * [`DIST_HDR_COMPRESSED`]
/// * An encoded atom
/// 
/// [`DIST_HDR_NORMAL`]: static.DIST_HDR_NORMAL.html
/// [`DIST_HDR_FRAGMENTED`]: static.DIST_HDR_FRAGMENTED.html
/// [`DIST_HDR_FRAGMENT`]: static.DIST_HDR_FRAGMENT.html
/// [`DIST_HDR_COMPRESSED`]: static.DIST_HDR_COMPRESSED.html
pub static DIST_HDR:            u8 =131;

/// The tag for a normal unfragmented and uncompressed distribution header.
/// 
/// # Binary representation
/// 
/// | 1 byte | 1 byte                  | `NumberOfAtomCacheRefs/2+1 \| 0` bytes | `N \| 0` bytes    |
/// | ------ | ----------------------- | -------------------------------------- | ----------------- |
/// | `68`   | `NumberOfAtomCacheRefs` | `Flags`                                | `AtomCacheRefs`   |
/// 
/// * `NumberOfAtomCacheRefs` is the amount of atom cache references in this
///   message.
/// * `Flags` is a list of 4-byte values containig flags in the following format:
///   
///   | 1 bit           | 3 bits         |
///   | --------------- | -------------- |
///   | `NewCacheEntry` | `SegmentIndex` |
///   
///   * `NewCacheEntry` describes whether the atom is new in the cache.
///   * `SegmentIndex` describes in which segment the atom is located.
///   
///   and after the flags for each of the references, one entry of this is sent:
///   
///   | 3 bits | 1 bit       |
///   | ------ | ----------- |
///   | Unused | `LongAtoms` |
///   
///   * `LongAtoms` states whether 1 or 2 bytes are used for the atom values
///     in this distribution header.
/// * `AtomCacheRefs` are:
///   
///   When for this atom, `NewCacheEntry` is 1:
///   
///   | 1 byte                 | 1\|2 bytes | `Length` bytes |
///   | ---------------------- | ---------- | -------------- |
///   | `InternalSegmentIndex` | `Length`   | `AtomText`     |
///   
///   * `InternalSegmentIndex`, together with `SegmentIndex` in the flags
///     entry completely define the location of the atom in the atom cache.
///   * `Length` is either 1 byte when `LongAtoms` is 0, and 2 bytes otherwise.
///     It describes how many bytes long the `AtomText` is.
///   * `AtomText` is the actual name of the atom.
///   
///   Or when for this atom, `NewCacheEntry` is 0:
///   
///   | 1 byte                 |
///   | ---------------------- |
///   | `InternalSegmentIndex` |
///   
///   * `InternalSegmentIndex`, together with `SegmentIndex` in the flags
///     entry completely define the location of the atom in the atom cache.
/// 
/// This header is then trailed with a term_to_binary-encoded term with
/// (optionally) [atom references][`ATOM_CACHE_REF`] to this header.
/// 
/// [`ATOM_CACHE_REF`]: static.ATOM_CACHE_REF.html
pub static DIST_HDR_NORMAL:     u8 = 68;

/// The tag for a header stating that the message is fragmented.
/// 
/// Note: This header must contain the entire atom cache.
/// 
/// # Binary representation
/// 
/// | 1 byte | 8 bytes      | 8 bytes      | 1 byte                  | `NumberOfAtomCacheRefs/2+1 \| 0` bytes | `N \| 0` bytes    |
/// | ------ | ------------ | ------------ | ----------------------- | -------------------------------------- | ----------------- |
/// | `69`   | `SequenceId` | `FragmentId` | `NumberOfAtomCacheRefs` | `Flags`                                | `AtomCacheRefs`   |
/// 
/// * `SequenceId` uniquely identifies the message that this fragment is part
///   of.
/// * `FragmentId` is a number that decreases with 1 for each fragment, and
///   at N, where N is the number of fragments (so the last fragment has
///   `FragmentId` of `1`).
/// * `NumberOfAtomCacheRefs`, `Flags`, and `AtomCacheRefs` act just like in
///   [`DIST_HDR_NORMAL`].
/// 
/// Some data MAY then be sent after this header, and each packet after this
/// MUST only contain data of the term that is being sent.
/// 
/// [`DIST_HDR_NORMAL`]: static.DIST_HDR_NORMAL.html
pub static DIST_HDR_FRAGMENTED: u8 = 69;

/// The tag denoting a follow-up fragment of apreviously fragment message
/// (either another fragment, or the [first fragment][`DIST_HDR_FRAGMENTED`]).
/// 
/// # Binary representation
/// 
/// | 1 byte | 8 bytes      | 8 bytes      |
/// | ------ | ------------ | ------------ |
/// | `70`   | `SequenceId` | `FragmentId` |
/// 
/// * `SequenceId`, just like in [`DIST_HDR_FRAGMENTED`] uniquely identifies
///   this message, and must be the same as in the associated
///   [`DIST_HDR_FRAGMENT`].
/// * `FragmentId`, just like in [`DIST_HDR_FRAGMENTED`] is a number that
///   decreases with 1 for each fragment, with the last fragment having a
///   `FragmentId` of 1.
/// 
/// [`DIST_HDR_FRAGMENTED`]: static.DIST_HDR_FRAGMENTED.html
pub static DIST_HDR_FRAGMENT:   u8 = 70;

/// The tag denoting a compressed value (either a distribution header or an
/// encoded atom).
/// 
/// # Binary representation
/// 
/// | 1 byte | 4 bytes            | N                    |
/// | ------ | ------------------ | -------------------- |
/// | `80`   | `UncompressedSize` | `ZLibCompressedData` |
/// 
/// * `UncompressedSize` is the complete size of the uncompressed
///   `ZLibCompressedData` (including the tag).
/// * `ZLibCompressedData` compresses data of the shape:
///   
///   | 1 byte | N bytes |
///   | ------ | ------- |
///   | `Tag`  | `Data`  |
pub static DIST_HDR_COMPRESSED: u8 = 80;

/// Refers to the atom with AtomCacheReferenceIndex in the
/// [distribution header].
///
/// # Binary representation
///
/// | 1 byte | 1 byte                    |
/// | ------ | ------------------------- |
/// | `82`   | `AtomCacheReferenceIndex` |
/// 
/// # String representation
/// 
/// This string is represented just like an [`atom`].
/// The value of this atom is looked up in the [distribution header].
/// 
/// [`atom`]: static.ATOM_EXT.html
/// [distribution header]: static.DIST_HDR.html
pub static ATOM_CACHE_REF:      u8 =  82;

/// Unsigned 8-bit integer.
///
/// # Binary representation
///
/// | 1 byte | 1 byte |
/// | ------ | ------ |
/// | `97`   | `u8`   |
/// 
/// # String representation
/// 
/// Represent the value of this 2's complement 8-bit signed integer as either:
/// 
/// * Simple decimal
/// * An ASCII character Z in the form `$Z`
/// * Any other base B in 2..36 in the form `B#N`
/// 
/// The recommendation is to always output like a simple decimal, as it is the
/// simplest and most portable.
pub static SMALL_INTEGER_EXT:   u8 =  97;

/// Signed 32-bit integer.
///
/// # Binary layout
///
/// | 1 byte | 4 bytes |
/// | ------ | ------- |
/// | `98`   | `i32`   |
/// 
/// # String representation
/// 
/// The representations of this 2's complement 32-bit signed integer are:
/// 
/// * Simple decimal
/// * An ASCII character Z in the form `$Z`
/// * Any other base B in 2..36 in the form `B#N`
/// 
/// The recommendation is to always output like a simple decimal, as it is the
/// simplest and most portable.
pub static INTEGER_EXT:         u8 =  98;

///  A finite float (i.e. not inf, -inf or NaN) is stored in string format.
///
/// This term is used in minor version 0 of the external format;
/// it has been superseded by [`NEW_FLOAT_EXT`].
///
/// # Binary representation
///
/// | 1 byte | 31 bytes        |
/// | ------ | --------------- |
/// | `99`   | Float as string |
///
/// The format used in sprintf to format the float is `%.20e` (there are more
/// bytes allocated than necessary).
/// 
/// # String representation
/// 
/// The parsable values should be a decimal integer with either an exponent
/// or a trailing dot with 1 or more numbers behind it.
/// An optional base can also be specified in front of it, separating the
/// value and base with an octothorpe (`#`).
/// 
/// The recommendation is to output with the format being `%.20e` as it is
/// guaranteed not to lose precision.
///
/// [`NEW_FLOAT_EXT`]: static.NEW_FLOAT_EXT.html
pub static FLOAT_EXT:           u8 =  99;

/// Same as [`NEW_PORT_EXT`] except the Creation field is only one byte and
/// only two bits are significant, the rest are to be 0.
///
/// # Binary representation
///
/// | 1 byte | N bytes | 4 bytes | 1 byte     |
/// | ------ | ------- | ------- | ---------- |
/// | `102`  | `Node`  | `ID`    | `Creation` |
///
/// * `Node` is the name of the originating node, encoded using
///   [`ATOM_UTF8_EXT`], [`SMALL_ATOM_UTF8_EXT`] or [`ATOM_CACHE_REF`].
/// * `ID` is a 32-bit big endian unsigned integer.
///   Only 15 bits are significant; the rest are to be 0.
/// * `Creation` is a 8-bit unsigned integer.
///   All ports originating from the same node incarnation must have
///   identical Creation values.
///   This makes it possible to separate identifiers from old (crashed) nodes
///   from a new one.
///   The value zero should be avoided for normal operations as it is used as
///   a wild card for debug purpose
///   (like a pid returned by [`erlang:list_to_pid/1`]).
///
/// [`erlang:lst_to_pid/1`]: http://erlang.org/doc/man/erlang.html#list_to_pid-1
/// [`ATOM_UTF8_EXT`]: static.ATOM_UTF8_EXT.html
/// [`SMALL_ATOM_UTF8_EXT`]: static.SMALL_ATOM_UTF8_EXT.html
/// [`ATOM_CACHE_REF`]: static.ATOM_CACHE_REF.html
/// [`NEW_PORT_EXT`]: static.NEW_PORT_EXT.html
pub static PORT_EXT:            u8 = 102;

/// Encodes a port identifier (obtained from [`erlang:open_port/2`]).
///
/// Introduced in OTP 19, but only to be decoded and echoed back.
/// Not encoded for local ports.
/// Planned to supersede PORT_EXT in OTP 23 when [DFLAG_BIG_CREATON](dflags)
/// becomes mandatory.
///
/// # Binary representation
///
/// | 1 byte | N bytes | 4 bytes | 4 bytes    |
/// | ------ | ------- | ------- | ---------- |
/// | `89`   | `Node`  | `ID`    | `Creation` |
///
/// * `Node` is the name of the originating node, encoded using
///   [`ATOM_UTF8_EXT`], [`SMALL_ATOM_UTF8_EXT`] or [`ATOM_CACHE_REF`].
/// * `ID` is a 32-bit big endian unsigned integer.
///   Only 15 bits are significant; the rest are to be 0.
/// * `Creation` is a 32-bit big endian unsigned integer.
///   All ports originating from the same node incarnation must have
///   identical Creation values.
///   This makes it possible to separate identifiers from old (crashed) nodes
///   from a new one.
///   The value zero should be avoided for normal operations as it is used as
///   a wild card for debug purpose
///   (like a pid returned by [`erlang:list_to_pid/1`]).
///
/// [`erlang:lst_to_pid/1`]: http://erlang.org/doc/man/erlang.html#list_to_pid-1
/// [`ATOM_UTF8_EXT`]: static.ATOM_UTF8_EXT.html
/// [`SMALL_ATOM_UTF8_EXT`]: static.SMALL_ATOM_UTF8_EXT.html
/// [`ATOM_CACHE_REF`]: static.ATOM_CACHE_REF.html
/// [`erlang:open_port/2`]: http://erlang.org/doc/man/erlang.html#open_port-2
/// [dflags]: http://erlang.org/doc/apps/erts/erl_dist_protocol.html#dflags
pub static NEW_PORT_EXT:        u8 =  89;

/// Same as [`NEW_PID_EXT`] except the Creation field is only one byte and only
/// two bits are significant, the rest are to be 0.
///
/// # Binary representation
///
/// | 1 byte | N bytes | 4 bytes | 4 bytes  | 1 byte     |
/// | ------ | ------- | ------- | -------- | ---------- |
/// | `103`  | `Node`  | `ID`    | `Serial` | `Creation` |
///
/// * `Node` is the name of the originating node, encoded using
///   [`ATOM_UTF8_EXT`], [`SMALL_ATOM_UTF8_EXT`] or [`ATOM_CACHE_REF`].
/// * `ID` is a 32-bit big endian unsigned integer.
///   Only 15 bits are significant; the rest are to be 0.
/// * `Serial` is a 32-bit big endian unsigned integer.
///   Only 13 bits are significant; the rest are to be 0.
/// * `Creation` is a 8-bit unsigned integer.
///   All identifiers originating from the same node incarnation must have
///   identical Creation values.
///   This makes it possible to separate identifiers from old (crashed) nodes
///   from a new one.
///   The value zero should be avoided for normal operations as it is used as
///   a wild card for debug purpose
///   (like a pid returned by [`erlang:list_to_pid/1`]).
///
/// [`erlang:lst_to_pid/1`]: http://erlang.org/doc/man/erlang.html#list_to_pid-1
/// [`ATOM_UTF8_EXT`]: static.ATOM_UTF8_EXT.html
/// [`SMALL_ATOM_UTF8_EXT`]: static.SMALL_ATOM_UTF8_EXT.html
/// [`ATOM_CACHE_REF`]: static.ATOM_CACHE_REF.html
/// [`NEW_PID_EXT`]: static.NEW_PID_EXT.html
pub static PID_EXT:             u8 = 103;

///  Encodes an Erlang process identifier object.
///
/// Introduced in OTP 19, but only to be decoded and echoed back.
/// Not encoded for local processes.
/// Planned to supersede PID_EXT in OTP 23 when [DFLAG_BIG_CREATON](dflags)
/// becomes mandatory.
///
/// # Binary representation
///
/// | 1 byte | N bytes | 4 bytes | 4 bytes  | 4 bytes    |
/// | ------ | ------- | ------- | -------- | ---------- |
/// | `88`   | `Node`  | `ID`    | `Serial` | `Creation` |
///
/// * `Node` is the name of the originating node, encoded using
///   [`ATOM_UTF8_EXT`], [`SMALL_ATOM_UTF8_EXT`] or [`ATOM_CACHE_REF`].
/// * `ID` is a 32-bit big endian unsigned integer.
///   Only 15 bits are significant; the rest are to be 0.
/// * `Serial` is a 32-bit big endian unsigned integer.
///   Only 13 bits are significant; the rest are to be 0.
/// * `Creation` is a 32-bit big endian unsigned integer.
///   All identifiers originating from the same node incarnation must have
///   identical Creation values.
///   This makes it possible to separate identifiers from old (crashed) nodes
///   from a new one.
///   The value zero should be avoided for normal operations as it is used as
///   a wild card for debug purpose
///   (like a pid returned by [`erlang:list_to_pid/1`]).
///
/// [`erlang:lst_to_pid/1`]: http://erlang.org/doc/man/erlang.html#list_to_pid-1
/// [`ATOM_UTF8_EXT`]: static.ATOM_UTF8_EXT.html
/// [`SMALL_ATOM_UTF8_EXT`]: static.SMALL_ATOM_UTF8_EXT.html
/// [`ATOM_CACHE_REF`]: static.ATOM_CACHE_REF.html
/// [dflags]: http://erlang.org/doc/apps/erts/erl_dist_protocol.html#dflags
pub static NEW_PID_EXT:         u8 =  88;

/// Encodes a tuple.
///
/// # Binary representation
///
/// | 1 byte | 1 byte  | N bytes    |
/// | ------ | ------- | ---------- |
/// | `104`  | `Arity` | `Elements` |
///
/// * `Arity` is an unsigned 8-bit integer.
/// * `Elements` are each encoded terms.
///   There are `Arity` amount of elements.
pub static SMALL_TUPLE_EXT:     u8 = 104;

/// Same as [`SMALL_TUPLE_EXT`] except that Arity is an unsigned 4 byte
/// integer.
///
/// # Binary representation
///
/// | 1 byte | 4 bytes | N bytes    |
/// | ------ | ------- | ---------- |
/// | `105`  | `Arity` | `Elements` |
///
/// * `Arity` is an unsigned 32-bit big-endian integer.
/// * `Elements` are each encoded terms.
///   There are `Arity` amount of elements.
///
/// [`SMALL_TUPLE_EXT`]: static.SMALL_TUPLE_EXT.html
pub static LARGE_TUPLE_EXT:     u8 = 105;

/// Encodes a map.
///
/// # Binary representation
///
/// | 1 byte | 4 bytes | N bytes |
/// | ------ | ------- | ------- |
/// | `116`  | `Arity` | `Pairs` |
///
/// * `Arity` is the amount of key-value pairs in this map.
/// * `Pairs` are each two encoded terms.
///   There are `Arity` amount of pairs.
///   There are no duplicate keys.
pub static MAP_EXT:             u8 = 116;

/// The representation for an empty list, that is, the Erlang syntax `[]`.
///
/// # Binary representation
///
/// | 1 byte |
/// | ------ |
/// | `106`  |
pub static NIL_EXT:             u8 = 106;

/// String does not have a corresponding Erlang representation, but is an
/// optimization for sending lists of bytes (integer in the range 0-255) more
/// efficiently over the distribution.
///
/// As field Length is an unsigned 2 byte integer (big-endian), implementations
/// must ensure that lists longer than 65535 elements are encoded as
/// [`LIST_EXT`].
///
/// # Binary representation
///
/// | 1 byte | 2 bytes  | `Length` bytes |
/// | ------ | -------- | -------------- |
/// | 107    | `Length` | `Characters`   |
///
/// The characters are UTF-8 encoded characters.
/// There are a maximum of 65535 bytes, however many characters that turns out
/// to be.
/// It is *not* a maximum of 65535 characters!
///
/// [`LIST_EXT`]: static.LIST_EXT.html
pub static STRING_EXT:          u8 = 107;

/// The representation for a non-empty list.
///
/// # Binary representation
///
/// | 1 byte | 4 bytes  | N bytes    | M bytes |
/// | ------ | -------- | ---------- | ------- |
/// | 108    | `Length` | `Elements` | `Tail`  |
///
/// * `Length` is a 32-bit big-endian unsigned number that is the number of
///   elements that follows in section `Elements`.
/// * `Elements` are each encoded terms.
/// * `Tail` is the final tail of the list; it is [`NIL_EXT`] for a proper list,
///   but can be any type if the list is improper (for example, `[a|b]`).
///
/// [`NIL_EXT`]: static.NIL_EXT.html
pub static LIST_EXT:            u8 = 108;

/// Binaries are byte-arrays.
/// They are represented as bitstrings or binaries in the Erlang language.
///
/// # Binary representation
///
/// | 1 byte | 4 bytes | `Len` bytes |
/// | ------ | ------- | ----------- |
/// | `109`  | `Len`   | `Data`      |
///
/// * The Len length field is an unsigned 4 byte integer (big-endian).
pub static BINARY_EXT:          u8 = 109;

/// Integer representation of an integer N where `-2^256 < N < 2^256`.
/// 
/// # Binary format
/// 
/// | 1 byte | 1 byte | 1 byte | `Len` bytes        |
/// | ------ | ------ | ------ | ------------------ |
/// | `110`  | `Len`  | `Sign` | `d0`..`d(Len - 1)` |
/// 
/// Bignums are stored in unary form with a Sign byte, that is, 0 if the
/// bignum is positive and 1 if it is negative.
/// The digits are stored with the least significant byte stored first.
/// To calculate the integer, the following formula can be used:
///
/// ```text
/// B = 256
/// (d0*B^0 + d1*B^1 + d2*B^2 + ... d(N-1)*B^(n-1))
/// ```
pub static SMALL_BIG_EXT:       u8 = 110;

/// Integer representation of an integer N where `-2^(2^32) < N < 2^(2^32)`.
/// 
/// # Binary representation
/// 
/// | 1 byte | 4 bytes | 1 byte | `Len` bytes        |
/// | ------ | ------- | ------ | ------------------ |
/// | `110`  | `Len`   | `Sign` | `d0`..`d(Len - 1)` |
/// 
/// Bignums are stored in unary form with a Sign byte, that is, 0 if the
/// bignum is positive and 1 if it is negative.
/// The digits are stored with the least significant byte stored first.
/// To calculate the integer, the following formula can be used:
///
/// ```text
/// B = 256
/// (d0*B^0 + d1*B^1 + d2*B^2 + ... d(N-1)*B^(n-1))
/// ```
pub static LARGE_BIG_EXT:       u8 = 111;

/// Deprecated method for encoding a reference term.
/// 
/// # Binary representation
/// 
/// | 1 byte | `N` bytes | 4 bytes | 1 byte     |
/// | ------ | --------- | ------- | ---------- |
/// | `101`  | `Node`    | `ID`    | `Creation` |
/// 
/// * `Node` is the name of the originating node, encoded using
///   [`ATOM_UTF8_EXT`], [`SMALL_ATOM_UTF8_EXT`] or [`ATOM_CACHE_REF`].
/// * `Creation` is a 8-bit unsigned integer.
///   All references originating from the same node incarnation must have
///   identical Creation values.
///   This makes it possible to separate references from old (crashed) nodes
///   from a new one.
///   The value zero should be avoided for normal operations as it is used as
///   a wild card for debug purpose
///   (like a pid returned by [`erlang:list_to_pid/1`]).
/// * `ID` is a node-unique number that describes this reference uniquely.
/// 
/// [`erlang:lst_to_pid/1`]: http://erlang.org/doc/man/erlang.html#list_to_pid-1
/// [`ATOM_UTF8_EXT`]: static.ATOM_UTF8_EXT.html
/// [`SMALL_ATOM_UTF8_EXT`]: static.SMALL_ATOM_UTF8_EXT.html
/// [`ATOM_CACHE_REF`]: static.ATOM_CACHE_REF.html
pub static REFERENCE_EXT:       u8 = 101;

/// Deprecated method for encoding a reference term.
/// 
/// # Binary representation
/// 
/// | 1 byte | 2 bytes | `N` bytes | 1 byte     | `Len * 4` bytes |
/// | ------ | ------- | --------- | ---------- | --------------- |
/// | `101`  | `Len`   | `Node`    | `Creation` | `ID`            |
/// 
/// * `Node` is the name of the originating node, encoded using
///   [`ATOM_UTF8_EXT`], [`SMALL_ATOM_UTF8_EXT`] or [`ATOM_CACHE_REF`].
/// * `Creation` is a 8-bit unsigned integer.
///   All references originating from the same node incarnation must have
///   identical Creation values.
///   This makes it possible to separate references from old (crashed) nodes
///   from a new one.
///   The value zero should be avoided for normal operations as it is used as
///   a wild card for debug purpose
///   (like a pid returned by [`erlang:list_to_pid/1`]).
/// * `Len` describes how many 32-bit values are contained within `ID`.
/// * `ID` is a series of 32-bit unsigned integers.
///   The erlang docs state:
///   
///   > A sequence of `Len` big-endian unsigned integers (4 bytes each, so
///   > `N' = 4 * Len`), but is to be regarded as uninterpreted data.
///   
///   I am having trouble interpreting this statement, so the assumption is
///   that only the first is interpreted, and only the 18 least-significant
///   bits of that value are interpreted, and that the rest are to be 0.
/// 
/// [`erlang:lst_to_pid/1`]: http://erlang.org/doc/man/erlang.html#list_to_pid-1
/// [`ATOM_UTF8_EXT`]: static.ATOM_UTF8_EXT.html
/// [`SMALL_ATOM_UTF8_EXT`]: static.SMALL_ATOM_UTF8_EXT.html
/// [`ATOM_CACHE_REF`]: static.ATOM_CACHE_REF.html
pub static NEW_REFERENCE_EXT:   u8 = 114;

/// Encodes a reference term.
/// 
/// # Binary representation
/// 
/// | 1 byte | 2 bytes | `N` bytes | 4 bytes    | `Len * 4` bytes |
/// | ------ | ------- | --------- | ---------- | --------------- |
/// | `90`   | `Len`   | `Node`    | `Creation` | `ID`            |
/// 
/// * `Node` is the name of the originating node, encoded using
///   [`ATOM_UTF8_EXT`], [`SMALL_ATOM_UTF8_EXT`] or [`ATOM_CACHE_REF`].
/// * `Creation` is a 32-bit big-endian unsigned integer.
///   All references originating from the same node incarnation must have
///   identical Creation values.
///   This makes it possible to separate references from old (crashed) nodes
///   from a new one.
///   The value zero should be avoided for normal operations as it is used as
///   a wild card for debug purpose
///   (like a pid returned by [`erlang:list_to_pid/1`]).
/// * `Len` describes how many 32-bit values are contained within `ID`.
/// * `ID` is a series of 32-bit unsigned integers.
///   The erlang docs state:
///   
///   > A sequence of `Len` big-endian unsigned integers (4 bytes each, so
///   > `N' = 4 * Len`), but is to be regarded as uninterpreted data.
///   
///   I am having trouble interpreting this statement, so the assumption is
///   that only the first is interpreted, and only the 18 least-significant
///   bits of that value are interpreted, and that the rest are to be 0.
/// 
/// [`erlang:lst_to_pid/1`]: http://erlang.org/doc/man/erlang.html#list_to_pid-1
/// [`ATOM_UTF8_EXT`]: static.ATOM_UTF8_EXT.html
/// [`SMALL_ATOM_UTF8_EXT`]: static.SMALL_ATOM_UTF8_EXT.html
/// [`ATOM_CACHE_REF`]: static.ATOM_CACHE_REF.html
pub static NEWER_REFERENCE_EXT: u8 =  90;

/// Old encoding of internal functions: `fun F/A and fun(Arg1,..) -> ... end`.
/// 
/// # Binary representation
/// 
/// | 1 byte | 4 bytes   | N bytes | N' bytes | N'' bytes | N'''  bytes | N'''' bytes    |
/// | ------ | --------- | ------- | -------- | --------- | ----------- | -------------- |
/// | `117`  | `NumFree` | `Pid`   | `Module` | `Index`   | `Uniq`      | `Free vars...` |
/// 
/// * `NumFree` is a 32-bit big-endian unsigned integer is the number of
///   free variables (`Free vars`).
/// * `Pid` is a process identifier as in [`PID_EXT`].
///   Represents the process in which the fun was created.
/// * `Module`, encoded as an atom using [`ATOM_UTF8_EXT`],
///   [`SMALL_ATOM_UTF8_EXT`], or [`ATOM_CACHE_REF`], is the module that the
///   fun is implemented in.
/// * `Index`, encoded as an integer using [`SMALL_INTEGER_EXT`] or
///   [`INTEGER_EXT`] is typically a small index into the module's fun table.
/// * `Uniq`, encoded as an integer using [`SMALL_INTEGER_EXT`] or
///   [`INTEGER_EXT`] is the hash value of the parse for the fun.
/// * `Free vars` are `NumFree` amount of terms, each one encoded according to
///   its type.
/// 
/// [`ATOM_UTF8_EXT`]: static.ATOM_UTF8_EXT.html
/// [`SMALL_ATOM_UTF8_EXT`]: static.SMALL_ATOM_UTF8_EXT.html
/// [`ATOM_CACHE_REF`]: static.ATOM_CACHE_REF.html
/// [`SMALL_INTEGER_EXT`]: static.SMALL_INTEGER_EXT.html
/// [`INTEGER_EXT`]: static.INTEGER_EXT.html
/// [`PID_EXT`]: static.PID_EXT.html
pub static FUN_EXT:             u8 = 117;

/// Encoding of internal functions: `fun F/A and fun(Arg1,..) -> ... end`.
/// 
/// # Binary representation
/// 
/// | 1 byte | 4 bytes | 1 byte   | 16 bytes | 4 bytes  | 4 bytes    | N bytes   | N' bytes    | N'' bytes  | N''' bytes | N'''' bytes    |
/// | ------ | ------- | -------- | -------- | -------- | ---------- | --------- | ----------- | ---------- | ---------- | -------------- |
/// | `112`  | `Size`  | `Arity`  | `Uniq`   | `Index`  | `NumFree`  | `Module`  | `OldIndex`  | `OldUniq`  | `Pid`      | `Free vars...` | 
///
/// * `Size` is the total number of bytes, including field Size.
/// * `Arity` is the arity of the function implementing the fun, as an 8-bit
///   unsigned number.
/// * `Uniq` is a 16 bytes MD5 of the significant parts of the Beam file.
/// * `Index` is a 32-bit big endian unsigned index number.
///   Each fun within a module has an unique index.
/// * `NumFree` is the number of free variables, stored as a 32-bit big endian
///   unsigned integer.
/// * `Module`, encoded as an atom using [`ATOM_UTF8_EXT`],
///   [`SMALL_ATOM_UTF8_EXT`], or [`ATOM_CACHE_REF`], is the module that the
///   fun is implemented in.
/// * `OldIndex`, encoded as an integer encoded using [`SMALL_INTEGER_EXT`] or
///   [`INTEGER_EXT`], is typically a small index into the module's fun table.
/// * `OldUniq`, encoded as an integer using using [`SMALL_INTEGER_EXT`] or
///   [`INTEGER_EXT`], is the hash value of the parse tree for the fun.
/// * `Pid`, encoded using [`PID_EXT`], represnts the process in which the fun
///   was created.
/// * `Free vars` are `NumFree` amount of terms, each one encoded according to
///   its type.
/// 
/// [`ATOM_UTF8_EXT`]: static.ATOM_UTF8_EXT.html
/// [`SMALL_ATOM_UTF8_EXT`]: static.SMALL_ATOM_UTF8_EXT.html
/// [`ATOM_CACHE_REF`]: static.ATOM_CACHE_REF.html
/// [`SMALL_INTEGER_EXT`]: static.SMALL_INTEGER_EXT.html
/// [`INTEGER_EXT`]: static.INTEGER_EXT.html
/// [`PID_EXT`]: static.PID_EXT.html
pub static NEW_FUN_EXT:         u8 = 112;

/// Encodes functions of the shape `fun M:F/A`.
/// 
/// # Binary representation
/// 
/// | 1 byte | N bytes  | N' bytes   | N'' bytes |
/// | ------ | -------- | ---------- | --------- |
/// | `113`  | `Module` | `Function` | `Arity`   |
/// 
/// * `Module`, encoded using [`ATOM_UTF8_EXT`], [`SMALL_ATOM_UTF8_EXT`], or
///   [`ATOM_CACHE_REF`], is the module that exports this function.
/// * `Function`, encoded using [`ATOM_UTF8_EXT`], [`SMALL_ATOM_UTF8_EXT`], or
///   [`ATOM_CACHE_REF`], is the name of this function.
/// * `Arity`, encoded using [`SMALL_INTEGER_EXT`], is the arity of this
///   function.
/// 
/// [`ATOM_UTF8_EXT`]: static.ATOM_UTF8_EXT.html
/// [`SMALL_ATOM_UTF8_EXT`]: static.SMALL_ATOM_UTF8_EXT.html
/// [`ATOM_CACHE_REF`]: static.ATOM_CACHE_REF.html
/// [`SMALL_INTEGER_EXT`]: static.SMALL_INTEGER_EXT.html
pub static EXPORT_EXT:          u8 = 113;

/// This term represents a bitstring whose length in bits does not have to be
/// a multiple of 8.
/// 
/// # Binary representation
/// 
/// | 1 byte | 4 bytes | 1 byte | `Len` bytes |
/// | ------ | ------- | ------ | ----------- |
/// | `77`   | `Len`   | `Bits` | `Data`      |
/// 
/// * `Len`, encoded as a 32-bit big endian unsigned integer, is the amount
///   of `Data` bytes.
/// * `Bits`, encoded as an 8-bit number N, where 1 >= N >= 8, is the amount
///   of bits that are used in the last data byte, counting from most
///   significant to least significant.
/// * `Data` is the actual binary data in this bitstring.
pub static BIT_BINARY_EXT:      u8 =  77;

/// This term represents a float
/// 
/// # Binary representation
/// 
/// | 1 byte | 8 bytes |
/// | ------ | ------- |
/// | `70`   | `Float` |
/// 
/// * `Float` is an 32-bit IEEE floating point number stored in big-endian
///   format.
pub static NEW_FLOAT_EXT:       u8 =  70;

/// This term represents an atom.
/// 
/// # Binary representation
/// 
/// | 1 byte | 2 bytes | `Len` bytes |
/// | ------ | ------- | ----------- |
/// | `118`  | `Len`   | `AtomName`  |
/// 
/// * `Len`, represented as a 16-bit big-endian unsigned integer, is the amount
///   of bytes the atom name takes.
/// * `AtomName` is an unescaped UTF8 string of `Len` bytes long representing
///   the name of this atom.
pub static ATOM_UTF8_EXT:       u8 = 118;

/// This term represents an atom that takes up at most 255 bytes
/// (which may be less than 255 characters as UTF-8 can have multi-byte
/// characters).
/// 
/// # Binary representation
/// 
/// | 1 byte | 1 byte | `Len` bytes |
/// | ------ | ------ | ----------- |
/// | `118`  | `Len`  | `AtomName`  |
/// 
/// * `Len`, represented as an 8-bit unsigned integer, is the amount of bytes
///   the atom name takes.
/// * `AtomName` is an unescaped UTF8 string of `Len` bytes long representing
///   the name of this atom.
pub static SMALL_ATOM_UTF8_EXT: u8 = 119;

/// This represents a LATIN-1 atom, but should not be encoded anymore.
/// 
/// # Binary representation
/// 
/// | 1 byte | 2 bytes | `Len` bytes |
/// | ------ | ------- | ----------- |
/// | `118`  | `Len`   | `AtomName`  |
/// 
/// * `Len`, represented as an 16-bit big endian unsigned integer, is the
///   amount of bytes the atom name takes.
/// * `AtomName` is an unescaped LATIN-1 string of `Len` bytes long
///   representing the name of this atom.
pub static ATOM_EXT:            u8 = 100;

/// This represents a LATIN-1 atom, but should not be encoded anymore.
/// 
/// # Binary representation
/// 
/// | 1 byte |  byte | `Len` bytes |
/// | ------ | ----- | ----------- |
/// | `118`  | `Len` | `AtomName`  |
/// 
/// * `Len`, represented as an 8-bit unsigned integer, is the amount of bytes
///   the atom name takes.
/// * `AtomName` is an unescaped LATIN-1 string of `Len` bytes long
///   representing the name of this atom.
pub static SMALL_ATOM_EXT:      u8 = 115;

/// Replacement for `std::convert::TryInto<T>` that doesn't require `Sized`.
pub trait TryTo<T> {
    fn try_to(&self) -> Result<T, Error>;
}

/// Replacement for `std::convert::Into<T>` that doesn't require `Sized`.
pub trait To<T> {
    fn to(&self) -> T;
}

impl<X, T> TryTo<T> for X where X: To<T> {
    fn try_to(&self) -> Result<T, Error> {
        Ok(To::<T>::to(self))
    }
}

/// Binary representation for an `ETerm`.
#[derive(Clone)]
pub struct ETermBinary(Vec<u8>);

impl To<Vec<u8>> for ETermBinary {
    fn to(&self) -> Vec<u8> {
        self.0.to_owned()
    }
}

/// Ugly String representation for an `ETerm`.
#[derive(Clone)]
pub struct ETermString(String);

impl To<String> for ETermString {
    fn to(&self) -> String {
        self.0.to_owned()
    }
}

/// Pretty String representation for an `Eterm`.
#[derive(Clone)]
pub struct ETermPrettyString(ETermString);

impl To<ETermString> for ETermPrettyString {
    fn to(&self) -> ETermString {
        self.0.to_owned()
    }
}

impl To<String> for ETermPrettyString {
    fn to(&self) -> String {
        To::<String>::to(&self.0)
    }
}

/// A type that can be converted to an Erlang Binary Term format and two valid
/// Erlang String Term representations.
pub trait ETerm: TryTo<ETermBinary> + TryTo<ETermString> + TryTo<ETermPrettyString> {
    fn try_to_term_string(&self) -> Result<String, Error> {
        Ok(To::to(&TryTo::<ETermString>::try_to(self)?))
    }

    fn try_to_term_pretty_string(&self) -> Result<String, Error> {
        Ok(To::to(&TryTo::<ETermPrettyString>::try_to(self)?))
    }

    fn try_to_binary(&self) -> Result<Vec<u8>, Error> {
        Ok(To::to(&TryTo::<ETermBinary>::try_to(self)?))
    }
}

impl<T> ETerm for T where T: TryTo<ETermBinary> + TryTo<ETermString> + TryTo<ETermPrettyString> {}

impl TryTo<ETermBinary> for i8 {
    fn try_to(&self) -> Result<ETermBinary, Error> {
        if *self >= 0i8 {
            (*self as u8).try_to()
        } else {
            (*self as i32).try_to()
        }
    }
}

into_etermstr_from_tostr!(i8);

impl To<ETermBinary> for u8 {
    fn to(&self) -> ETermBinary {
        let data: &[u8; 1] = &self.to_be_bytes();
        ETermBinary(vec![97u8, data[0]])
    }
}

into_etermstr_from_tostr!(u8);

impl TryTo<ETermBinary> for i16 {
    fn try_to(&self) -> Result<ETermBinary, Error> {
        (*self as i32).try_to()
    }
}

into_etermstr_from_tostr!(i16);

impl TryTo<ETermBinary> for u16 {
    fn try_to(&self) -> Result<ETermBinary, Error> {
        (*self as i32).try_to()
    }
}

into_etermstr_from_tostr!(u16);

impl TryTo<ETermBinary> for i32 {
    fn try_to(&self) -> Result<ETermBinary, Error> {
        if *self <= i8::max_value().into() && *self >= i8::min_value().into() {
            (*self as i8).try_to()
        } else {
            let data: &[u8; 4] = &self.to_be_bytes();
            Ok(ETermBinary(concat(&[98u8], data)))
        }
    }
}

into_etermstr_from_tostr!(i32);

impl TryTo<ETermBinary> for u32 {
    fn try_to(&self) -> Result<ETermBinary, Error> {
        (*self as i128).try_to()
    }
}

into_etermstr_from_tostr!(u32);

impl TryTo<ETermBinary> for i64 {
    fn try_to(&self) -> Result<ETermBinary, Error> {
        (*self as i128).try_to()
    }
}

into_etermstr_from_tostr!(i64);

impl TryTo<ETermBinary> for u64 {
    fn try_to(&self) -> Result<ETermBinary, Error> {
        (*self as i128).try_to()
    }
}

into_etermstr_from_tostr!(u64);

impl TryTo<ETermBinary> for i128 {
    fn try_to(&self) -> Result<ETermBinary, Error> {
        if *self <= i32::max_value().into() && *self >= i32::min_value().into() {
            (*self as i32).try_to()
        } else {
            let data: &[u8; 16] = &self.to_be_bytes();
            let bytes: u8 = ((128 - self.leading_zeros()) / 8) as u8;
            Ok(ETermBinary(concat(&[110u8, bytes], &data[..(bytes as usize)])))
        }
    }
}

into_etermstr_from_tostr!(i128);

impl TryTo<ETermBinary> for u128 {
    fn try_to(&self) -> Result<ETermBinary, Error> {
        // The big number here is i128::max_value() as a `From<u128>` for i128 is not implemented
        if *self <= 170_141_183_460_469_231_731_687_303_715_884_105_727u128 {
            (*self as i128).try_to()
        } else {
            let data: &[u8; 16] = &self.to_be_bytes();
            Ok(ETermBinary(concat(&[110u8, 17u8, 0], data)))
        }
    }
}

into_etermstr_from_tostr!(u128);

impl TryTo<ETermBinary> for isize {
    fn try_to(&self) -> Result<ETermBinary, Error> {
        (*self as i128).try_to()
    }
}

into_etermstr_from_tostr!(isize);

impl TryTo<ETermBinary> for usize {
    fn try_to(&self) -> Result<ETermBinary, Error> {
        (*self as u128).try_to()
    }
}

into_etermstr_from_tostr!(usize);

/// Represents an Erlang `NIL_EXT` term.
pub struct ENil;

impl ToString for ENil {
    fn to_string(&self) -> String {
        "[]".to_string()
    }
}

impl To<ETermBinary> for ENil {
    fn to(&self) -> ETermBinary {
        ETermBinary(vec![106u8])
    }
}

into_etermstr_from_tostr!(ENil);

/// Represents a proper `LIST_EXT` term with a `nil` tail.
pub struct EList<'a>(&'a Vec<&'a dyn ETerm>);

impl<'a> ToString for EList<'a> {
    fn to_string(&self) -> String {
        let mut s = "[".to_string();
        for d in self.0.iter() {
            s.push_str(d.try_to_term_string().unwrap().as_ref());
        }
        s.push(']');
        s
    }
}

impl<'a> TryTo<ETermBinary> for EList<'a> {
    fn try_to(&self) -> Result<ETermBinary, Error> {
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

into_etermstr_from_tostr!(EList<'a>);

/// Describes a `LIST_EXT` term with a possible non-`nil` tail.
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

impl<'a> TryTo<ETermBinary> for ENonProperList<'a> {
    fn try_to(&self) -> Result<ETermBinary, Error> {
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

into_etermstr_from_tostr!(ENonProperList<'a>);

/// Describes an `ATOM_UTF8_EXT` term and a `SMALL_ATOM_UTF8_EXT` term.
///
/// TODO: Make sure the string representation of atoms that need to be quoted
///  is implemented.
pub struct EAtom(String);

impl ToString for EAtom {
    fn to_string(&self) -> String {
        (&self.0).to_owned()
    }
}

impl TryTo<ETermBinary> for EAtom {
    fn try_to(&self) -> Result<ETermBinary, Error> {
        if self.0.len() <= u8::max_value().into() {
            Ok(ETermBinary(concat(&[119u8, self.0.len() as u8], self.0.as_bytes())))
        } else if self.0.len() <= 65535 {
            let len: [u8; 8] = self.0.len().to_be_bytes();
            Ok(ETermBinary(concat(&[118, len[6], len[7]], self.0.as_bytes())))
        } else {
            Err(Error::data(ErrorCode::ValueNotEncodable(Box::from((&self.0).as_str().to_owned()))))
        }
    }
}

into_etermstr_from_tostr!(EAtom);

impl TryTo<ETermBinary> for f32 {
    fn try_to(&self) -> Result<ETermBinary, Error> {
        if self.is_finite() {
            let bytes = self.to_be_bytes();
            Ok(ETermBinary(vec![70u8, 0, 0, 0, 0, bytes[0], bytes[1], bytes[2], bytes[3]]))
        } else {
            Err(Error::data(ErrorCode::ValueNotEncodable(Box::from(self.to_string()))))
        }
    }
}

into_etermstr_from_tostr!(f32);

impl TryTo<ETermBinary> for f64 {
    fn try_to(&self) -> Result<ETermBinary, Error> {
        if self.is_finite() {
            let bytes = self.to_be_bytes();
            Ok(ETermBinary(vec![70u8, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]))
        } else {
            Err(Error::data(ErrorCode::ValueNotEncodable(Box::from(self.to_string()))))
        }
    }
}

into_etermstr_from_tostr!(f64);

/// Describes an Erlang `EXPORT_EXT` term.
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

impl TryTo<ETermBinary> for EExport {
    fn try_to(&self) -> Result<ETermBinary, Error> {
        let mut result = vec![113u8];
        result.extend(ETerm::try_to_binary(&self.module)?);
        result.extend(ETerm::try_to_binary(&self.function)?);
        result.extend(ETerm::try_to_binary(&self.arity)?);
        Ok(ETermBinary(result))
    }
}

into_etermstr_from_tostr!(EExport);

/// Represents a `LARGE_TUPLE_EXT` or a `SMALL_TUPLE_EXT` term with a `nil`
/// tail.
pub struct ETuple<'a>(&'a Vec<&'a dyn ETerm>);

impl<'a> ToString for ETuple<'a> {
    fn to_string(&self) -> String {
        let mut result = "{".to_string();
        let mut parts: Vec<String> = Vec::new();
        for d in self.0.iter() {
            parts.push(d.try_to_term_string().unwrap())
        }
        result.push_str(parts.join(",").as_ref());
        result.push('}');
        result
    }
}

impl<'a> TryTo<ETermBinary> for ETuple<'a> {
    fn try_to(&self) -> Result<ETermBinary, Error> {
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

into_etermstr_from_tostr!(ETuple<'a>);

/// Describes an `ATOM_UTF8_EXT` term and a `SMALL_ATOM_UTF8_EXT` term.
///
/// TODO: Make sure the string representation of atoms that need to be quoted
///  is implemented.
pub struct EString(String);

impl ToString for EString {
    fn to_string(&self) -> String {
        let mut result = "\"".to_string();
        result.push_str((&self.0).as_ref());
        result.push('"');
        result
    }
}

impl TryTo<ETermBinary> for EString {
    fn try_to(&self) -> Result<ETermBinary, Error> {
        if self.0.len() <= u16::max_value().into() {
            let len: [u8; 8] = self.0.len().to_be_bytes();
            Ok(ETermBinary(concat(&[107, len[6], len[7]], self.0.as_bytes())))
        } else {
            EList(&(self.0.as_bytes().iter().map(|x| x as &dyn ETerm).collect())).try_to()
        }
    }
}

into_etermstr_from_tostr!(EString);

/// Describes an Erlang Port
pub struct EPort {
    node: EAtom,
    id: u32,
    creation: u32,
}

impl ToString for EPort {
    fn to_string(&self) -> String {
        let mut result = "#Port<".to_string();
        result.push_str(self.node.to_string().as_ref());
        result.push('.');
        result.push_str(self.id.to_string().as_ref());
        result.push('>');
        result
    }
}

impl TryTo<ETermBinary> for EPort {
    fn try_to(&self) -> Result<ETermBinary, Error> {
        let mut result = vec![89u8];
        result.extend(self.node.try_to_binary()?);
        result.extend(self.id.to_be_bytes().iter());
        result.extend(self.creation.to_be_bytes().iter());
        Ok(ETermBinary(result))
    }
}

into_etermstr_from_tostr!(EPort);

/// Describes an Erlang PID.
pub struct EPid {
    node: EAtom,
    id: u32,
    serial: u32,
    creation: u32,
}

impl ToString for EPid {
    fn to_string(&self) -> String {
        let mut result = "<".to_string();
        result.push_str(self.node.to_string().as_ref());
        result.push('.');
        result.push_str(self.id.to_string().as_ref());
        result.push('.');
        result.push_str(self.serial.to_string().as_ref());
        result.push('>');
        result
    }
}

impl TryTo<ETermBinary> for EPid {
    fn try_to(&self) -> Result<ETermBinary, Error> {
        let mut result = vec![88u8];
        result.extend(self.node.try_to_binary()?);
        result.extend(self.id.to_be_bytes().iter());
        result.extend(self.serial.to_be_bytes().iter());
        result.extend(self.creation.to_be_bytes().iter());
        Ok(ETermBinary(result))
    }
}

into_etermstr_from_tostr!(EPid);

/// Describes an Erlang Map
pub struct EMap<'a>(&'a Vec<(&'a dyn ETerm, &'a dyn ETerm)>);

impl<'a> ToString for EMap<'a> {
    fn to_string(&self) -> String {
        let mut result = "#{".to_string();
        let mut parts: Vec<String> = Vec::new();
        for d in self.0.iter() {
            let (k, v) = *d;
            let mut entry = k.try_to_term_string().unwrap();
            entry.push_str("=>");
            entry.push_str(v.try_to_term_string().unwrap().as_ref());
            parts.push(entry);
        }
        result.push_str(parts.join(",").as_ref());
        result.push('}');
        result
    }
}

impl<'a> TryTo<ETermBinary> for EMap<'a> {
    fn try_to(&self) -> Result<ETermBinary, Error> {
        let mut result = vec![116u8];
        result.extend((self.0.len() as u32).to_be_bytes().iter());

        for (k, v) in self.0 {
            result.extend(k.try_to_binary()?);
            result.extend(v.try_to_binary()?);
        };

        Ok(ETermBinary(result))
    }
}

into_etermstr_from_tostr!(EMap<'a>);

/// Describes an Erlang Binary
struct EBinary<'a>(&'a Vec<u8>);

impl<'a> ToString for EBinary<'a> {
    fn to_string(&self) -> String {
        let mut result = "<<".to_string();
        let mut parts = Vec::new();
        for byte in self.0 {
            parts.push(byte.to_string());
        }

        result.push_str(parts.join(",").as_ref());
        result.push_str(">>");

        result
    }
}

impl<'a> TryTo<ETermBinary> for EBinary<'a> {
    fn try_to(&self) -> Result<ETermBinary, Error> {
        let mut result = vec![109u8];
        result.extend(self.0.len().to_be_bytes().iter());
        result.extend(self.0);
        Ok(ETermBinary(result))
    }
}

into_etermstr_from_tostr!(EBinary<'a>);

fn concat<T>(p1: &[T], p2: &[T]) -> Vec<T> where T: Clone {
    let mut concat = p1.to_vec();
    concat.extend(p2.iter().cloned());
    concat
}

#[cfg(feature="bigint")]
impl TryTo<ETermBinary> for BigInt {
    fn try_to(&self) -> Result<ETermBinary, Error> {
        match self.to_i128() {
            Some(x) => return x.try_to(),
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
into_etermstr_from_tostr!(BigInt);

#[cfg(feature="bigint")]
impl TryTo<ETermBinary> for BigUint {
    fn try_to(&self) -> Result<ETermBinary, Error> {
        match self.to_i128() {
            Some(x) => return x.try_to(),
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

#[cfg(feature="bigint")]
into_etermstr_from_tostr!(BigUint);
