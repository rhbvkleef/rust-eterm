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
//! * [`ATOM_UTF8_EXT`], [`SMALL_ATOM_UTF8_EXT`] (for [`EAtom`])
//! * [`STRING_EXT`] (for [`EString`])
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
//! * [`ETF_VERSION`]
//! * [`DIST_HDR_NORMAL`]
//! * [`DIST_HDR_FRAG_START`]
//! * [`DIST_HDR_FRAG_CONT`]
//! * [`DIST_HDR_COMPRESSED`]
//! 
//! [`ETF_VERSION`]: constant.ETF_VERSION.html
//! [`DIST_HDR_NORMAL`]: constant.DIST_HDR_NORMAL.html
//! [`DIST_HDR_FRAG_START`]: constant.DIST_HDR_FRAG_START.html
//! [`DIST_HDR_FRAG_CONT`]: constant.DIST_HDR_FRAG_CONT.html
//! [`DIST_HDR_COMPRESSED`]: constant.DIST_HDR_COMPRESSED.html
//! [`ATOM_CACHE_REF`]: constant.ATOM_CACHE_REF.html
//! [`SMALL_INTEGER_EXT`]: constant.SMALL_INTEGER_EXT.html
//! [`INTEGER_EXT`]: constant.INTEGER_EXT.html
//! [`FLOAT_EXT`]: constant.FLOAT_EXT.html
//! [`PORT_EXT`]: constant.PORT_EXT.html
//! [`NEW_PORT_EXT`]: constant.NEW_PORT_EXT.html
//! [`PID_EXT`]: constant.PID_EXT.html
//! [`NEW_PID_EXT`]: constant.NEW_PID_EXT.html
//! [`SMALL_TUPLE_EXT`]: constant.SMALL_TUPLE_EXT.html
//! [`LARGE_TUPLE_EXT`]: constant.LARGE_TUPLE_EXT.html
//! [`MAP_EXT`]: constant.MAP_EXT.html
//! [`NIL_EXT`]: constant.NIL_EXT.html
//! [`STRING_EXT`]: constant.STRING_EXT.html
//! [`LIST_EXT`]: constant.LIST_EXT.html
//! [`BINARY_EXT`]: constant.BINARY_EXT.html
//! [`SMALL_BIG_EXT`]: constant.SMALL_BIG_EXT.html
//! [`LARGE_BIG_EXT`]: constant.LARGE_BIG_EXT.html
//! [`REFERENCE_EXT`]: constant.REFERENCE_EXT.html
//! [`NEW_REFERENCE_EXT`]: constant.NEW_REFERENCE_EXT.html
//! [`NEWER_REFERENCE_EXT`]: constant.NEWER_REFERENCE_EXT.html
//! [`FUN_EXT`]: constant.FUN_EXT.html
//! [`NEW_FUN_EXT`]: constant.NEW_FUN_EXT.html
//! [`EXPORT_EXT`]: constant.EXPORT_EXT.html
//! [`BIT_BINARY_EXT`]: constant.BIT_BINARY_EXT.html
//! [`NEW_FLOAT_EXT`]: constant.NEW_FLOAT_EXT.html
//! [`ATOM_UTF8_EXT`]: constant.ATOM_UTF8_EXT.html
//! [`SMALL_ATOM_UTF8_EXT`]: constant.SMALL_ATOM_UTF8_EXT.html
//! [`ATOM_EXT`]: constant.ATOM_EXT.html
//! [`SMALL_ATOM_EXT`]: constant.SMALL_ATOM_EXT.html
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

mod encode;
pub mod decode;

use std::fmt;
use std::io::Write;
use std::any::Any;
use std::convert::TryFrom;

use regex::Regex;

use super::error::Error;

/// This is the code of the start of a message.
/// This message will contain an encoded tag or a distribution header.
pub const ETF_VERSION:         u8 =131;

pub enum DistHeaderTag {
    /// The tag for a normal unfragmented and uncompressed distribution header.
    /// 
    /// ### Binary representation
    /// 
    /// | 1 byte | 1 byte                  | `NumberOfAtomCacheRefs/2+1 \| 0` bytes | `N \| 0` bytes    |
    /// | ------ | ----------------------- | -------------------------------------- | ----------------- |
    /// | `68`   | `NumberOfAtomCacheRefs` | `Flags`                                | `AtomCacheRefs`   |
    /// 
    /// * `NumberOfAtomCacheRefs` is the amount of atom cache references in
    ///   this message.
    /// * `Flags` is a list of 4-byte values containig flags in the following
    ///    format:
    ///   
    ///   | 1 bit           | 3 bits         |
    ///   | --------------- | -------------- |
    ///   | `NewCacheEntry` | `SegmentIndex` |
    ///   
    ///   * `NewCacheEntry` describes whether the atom is new in the cache.
    ///   * `SegmentIndex` describes in which segment the atom is located.
    ///   
    ///   and after the flags for each of the references, one entry of this is
    ///   sent:
    ///   
    ///   | 3 bits | 1 bit       |
    ///   | ------ | ----------- |
    ///   | Unused | `LongAtoms` |
    ///   
    ///   * `LongAtoms` states whether 1 or 2 bytes are used for the atom
    ///     values in this distribution header.
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
    ///   * `Length` is either 1 byte when `LongAtoms` is 0, and 2 bytes
    ///     otherwise.
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
    /// (optionally) [atom references][`AtomCacheRef`] to this header.
    /// 
    /// [`AtomCacheRef`]: enum.TermTag.html#variant.AtomCacheRef
    Normal = 68,

    /// The tag for a header stating that the message is fragmented.
    /// 
    /// Note: This header must contain the entire atom cache.
    /// 
    /// ### Binary representation
    /// 
    /// | 1 byte | 8 bytes      | 8 bytes      | 1 byte                  | `NumberOfAtomCacheRefs/2+1 \| 0` bytes | `N \| 0` bytes  |
    /// | ------ | ------------ | ------------ | ----------------------- | -------------------------------------- | --------------- |
    /// | `69`   | `SequenceId` | `FragmentId` | `NumberOfAtomCacheRefs` | `Flags`                                | `AtomCacheRefs` |
    /// 
    /// * `SequenceId` uniquely identifies the message that this fragment is
    ///   part of.
    /// * `FragmentId` is a number that decreases with 1 for each fragment, and
    ///   at N, where N is the number of fragments (so the last fragment has
    ///   `FragmentId` of `1`).
    /// * `NumberOfAtomCacheRefs`, `Flags`, and `AtomCacheRefs` act just like
    ///   in [`Normal`].
    /// 
    /// Some data MAY then be sent after this header, and each packet after
    /// this MUST only contain data of the term that is being sent.
    /// 
    /// [`Normal`]: #variant.normal
    Fragmented = 69,

    /// The tag denoting a follow-up fragment of apreviously fragment message
    /// (either another fragment, or the [first fragment][`Fragmented`]).
    /// 
    /// ### Binary representation
    /// 
    /// | 1 byte | 8 bytes      | 8 bytes      |
    /// | ------ | ------------ | ------------ |
    /// | `70`   | `SequenceId` | `FragmentId` |
    /// 
    /// * `SequenceId`, just like in [`Fragmented`] uniquely identifies
    ///   this message, and must be the same as in the associated
    ///   `DIST_HDR_FRAG_CONT`'s.
    /// * `FragmentId`, just like in [`Fragmented`] is a number that
    ///   decreases with 1 for each fragment, with the last fragment having a
    ///   `FragmentId` of 1.
    /// 
    /// [`Fragmented`]: #variant.fragmented
    Fragment = 70,

    /// The tag denoting a compressed value (either a distribution header or an
    /// encoded atom).
    /// 
    /// ### Binary representation
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
    Compressed = 80,
}

#[repr(u8)]
pub enum TermTag {
    /// Refers to the atom with AtomCacheReferenceIndex in the
    /// [distribution header].
    ///
    /// ### Binary representation
    ///
    /// | 1 byte | 1 byte                    |
    /// | ------ | ------------------------- |
    /// | `82`   | `AtomCacheReferenceIndex` |
    /// 
    /// ### String representation
    /// 
    /// This string is represented just like an [`atom`].
    /// The value of this atom is looked up in the [distribution header].
    /// 
    /// [`atom`]: #variant.Atom
    /// [distribution header]: enum.DistHeaderTag.html#variant.Normal
    AtomCacheRef = 82,

    /// Unsigned 8-bit integer.
    ///
    /// ### Binary representation
    ///
    /// | 1 byte | 1 byte |
    /// | ------ | ------ |
    /// | `97`   | `u8`   |
    /// 
    /// ### String representation
    /// 
    /// Represent the value of this 2's complement 8-bit signed integer as
    /// either:
    /// 
    /// * Simple decimal
    /// * An ASCII character Z in the form `$Z`
    /// * Any other base B in 2..36 in the form `B#N`
    /// 
    /// The recommendation is to always output like a simple decimal, as it is
    /// the simplest and most portable.
    SmallInteger = 97,

    /// Signed 32-bit integer.
    ///
    /// ### Binary layout
    ///
    /// | 1 byte | 4 bytes |
    /// | ------ | ------- |
    /// | `98`   | `i32`   |
    /// 
    /// ### String representation
    /// 
    /// The representations of this 2's complement 32-bit signed integer are:
    /// 
    /// * Simple decimal
    /// * An ASCII character Z in the form `$Z`
    /// * Any other base B in 2..36 in the form `B#N`
    /// 
    /// The recommendation is to always output like a simple decimal, as it is
    /// the simplest and most portable.
    Integer = 98,

    ///  A finite float (i.e. not inf, -inf or NaN) is stored in string format.
    ///
    /// This term is used in minor version 0 of the external format;
    /// it has been superseded by [`NewFloat`].
    ///
    /// ### Binary representation
    ///
    /// | 1 byte | 31 bytes        |
    /// | ------ | --------------- |
    /// | `99`   | Float as string |
    ///
    /// The format used in sprintf to format the float is `%.20e` (there are
    /// more bytes allocated than necessary).
    /// 
    /// ### String representation
    /// 
    /// The parsable values should be a decimal integer with either an exponent
    /// or a trailing dot with 1 or more numbers behind it.
    /// An optional base can also be specified in front of it, separating the
    /// value and base with an octothorpe (`#`).
    /// 
    /// The recommendation is to output with the format being `%.20e` as it is
    /// guaranteed not to lose precision.
    ///
    /// [`NewFloat`]: #variant.NewFloat
    Float = 99,

    /// Same as [`NewPort`] except the Creation field is only one byte and
    /// only two bits are significant, the rest are to be 0.
    ///
    /// ### Binary representation
    ///
    /// | 1 byte | N bytes | 4 bytes | 1 byte     |
    /// | ------ | ------- | ------- | ---------- |
    /// | `102`  | `Node`  | `ID`    | `Creation` |
    ///
    /// * `Node` is the name of the originating node, encoded using
    ///   [`AtomUtf8`], [`SmallAtomUtf8`] or [`AtomCacheRef`].
    /// * `ID` is a 32-bit big endian unsigned integer.
    ///   Only 15 bits are significant; the rest are to be 0.
    /// * `Creation` is a 8-bit unsigned integer.
    ///   All ports originating from the same node incarnation must have
    ///   identical Creation values.
    ///   This makes it possible to separate identifiers from old (crashed)
    ///   nodes from a new one.
    ///   The value zero should be avoided for normal operations as it is used
    ///   as a wild card for debug purpose
    ///   (like a pid returned by [`erlang:list_to_pid/1`]).
    /// 
    /// ### String representation
    /// 
    /// There is not actually a good way to represent a port as a string
    /// because the `Node` value may not be known.
    /// A representation of `#Port<0.{id}.{creation}>` can be used as a means
    /// to visualize the value of a port regardless.
    ///
    /// [`erlang:lst_to_pid/1`]: http://erlang.org/doc/man/erlang.html#list_to_pid-1
    /// [`AtomUtf8`]: #variant.AtomUtf8
    /// [`SmallAtomUtf8`]: #variant.SmallAtomUtf8
    /// [`AtomCacheRef`]: #variant.AtomCacheRef
    /// [`NewPort`]: #variant.NewPort
    Port = 102,

    /// Encodes a port identifier (obtained from [`erlang:open_port/2`]).
    ///
    /// Introduced in OTP 19, but only to be decoded and echoed back.
    /// Not encoded for local ports.
    /// Planned to supersede PORT_EXT in OTP 23 when
    /// [DFLAG_BIG_CREATON](dflags) becomes mandatory.
    ///
    /// # Binary representation
    ///
    /// | 1 byte | N bytes | 4 bytes | 4 bytes    |
    /// | ------ | ------- | ------- | ---------- |
    /// | `89`   | `Node`  | `ID`    | `Creation` |
    ///
    /// * `Node` is the name of the originating node, encoded using
    ///   [`AtomUtf8`], [`SmallAtomUtf8`] or [`AtomCacheRef`].
    /// * `ID` is a 32-bit big endian unsigned integer.
    ///   Only 15 bits are significant; the rest are to be 0.
    /// * `Creation` is a 32-bit big endian unsigned integer.
    ///   All ports originating from the same node incarnation must have
    ///   identical Creation values.
    ///   This makes it possible to separate identifiers from old (crashed)
    ///   nodes from a new one.
    ///   The value zero should be avoided for normal operations as it is used
    ///   as a wild card for debug purpose
    ///   (like a pid returned by [`erlang:list_to_pid/1`]).
    ///
    /// # String representation
    /// 
    /// There is not actually a good way to represent a port as a string
    /// because the `Node` value may not be known.
    /// A representation of #Port<0.{id}.{creation}> can be used as a means to
    /// visualize the value of a port regardless.
    /// 
    /// [`erlang:lst_to_pid/1`]: http://erlang.org/doc/man/erlang.html#list_to_pid-1
    /// [`AtomUtf8`]: #variant.AtomUtf8
    /// [`SmallAtomUtf8`]: #variant.SmallAtomUtf8
    /// [`AtomCacheRef`]: #variant.AtomCacheRef
    /// [`erlang:open_port/2`]: http://erlang.org/doc/man/erlang.html#open_port-2
    /// [dflags]: http://erlang.org/doc/apps/erts/erl_dist_protocol.html#dflags
    NewPort = 89,

    /// Same as [`NewPid`] except the Creation field is only one byte and only
    /// two bits are significant, the rest are to be 0.
    ///
    /// # Binary representation
    ///
    /// | 1 byte | N bytes | 4 bytes | 4 bytes  | 1 byte     |
    /// | ------ | ------- | ------- | -------- | ---------- |
    /// | `103`  | `Node`  | `ID`    | `Serial` | `Creation` |
    ///
    /// * `Node` is the name of the originating node, encoded using
    ///   [`AtomUtf8`], [`SmallAtomUtf8`] or [`AtomCacheRef`].
    /// * `ID` is a 32-bit big endian unsigned integer.
    ///   Only 15 bits are significant; the rest are to be 0.
    /// * `Serial` is a 32-bit big endian unsigned integer.
    ///   Only 13 bits are significant; the rest are to be 0.
    /// * `Creation` is a 8-bit unsigned integer.
    ///   All identifiers originating from the same node incarnation must have
    ///   identical Creation values.
    ///   This makes it possible to separate identifiers from old (crashed)
    ///   nodes from a new one.
    ///   The value zero should be avoided for normal operations as it is used
    ///   as a wild card for debug purpose
    ///   (like a pid returned by [`erlang:list_to_pid/1`]).
    /// 
    /// # String representation
    /// 
    /// There is not actually a good way to represent a pid as a string because
    /// the `Node` value may not be known.
    /// A representation of `<0.{id}.{creation}>` can be used as a means to
    /// visualize the value of a pid regardless.
    ///
    /// [`erlang:lst_to_pid/1`]: http://erlang.org/doc/man/erlang.html#list_to_pid-1
    /// [`AtomUtf8`]: #variant.AtomUtf8
    /// [`SmallAtomUtf8`]: #variant.SmallAtomUtf8
    /// [`AtomCacheRef`]: #variant.AtomCacheRef
    /// [`NewPid`]: #variant.NewPid
    Pid = 103,

    ///  Encodes an Erlang process identifier object.
    ///
    /// Introduced in OTP 19, but only to be decoded and echoed back.
    /// Not encoded for local processes.
    /// Planned to supersede [`Pid`] in OTP 23 when [DFLAG_BIG_CREATON](dflags)
    /// becomes mandatory.
    ///
    /// # Binary representation
    ///
    /// | 1 byte | N bytes | 4 bytes | 4 bytes  | 4 bytes    |
    /// | ------ | ------- | ------- | -------- | ---------- |
    /// | `88`   | `Node`  | `ID`    | `Serial` | `Creation` |
    ///
    /// * `Node` is the name of the originating node, encoded using
    ///   [`AtomUtf8`], [`SmallAtomUtf8`] or [`AtomCacheRef`].
    /// * `ID` is a 32-bit big endian unsigned integer.
    ///   Only 15 bits are significant; the rest are to be 0.
    /// * `Serial` is a 32-bit big endian unsigned integer.
    ///   Only 13 bits are significant; the rest are to be 0.
    /// * `Creation` is a 32-bit big endian unsigned integer.
    ///   All identifiers originating from the same node incarnation must have
    ///   identical Creation values.
    ///   This makes it possible to separate identifiers from old (crashed)
    ///   nodes from a new one.
    ///   The value zero should be avoided for normal operations as it is used
    ///   as a wild card for debug purpose
    ///   (like a pid returned by [`erlang:list_to_pid/1`]).
    /// 
    /// # String representation
    /// 
    /// There is not actually a good way to represent a pid as a string because
    /// the `Node` value may not be known.
    /// A representation of `<0.{id}.{creation}>` can be used as a means to
    /// visualize the value of a pid regardless.
    /// 
    /// [`erlang:lst_to_pid/1`]: http://erlang.org/doc/man/erlang.html#list_to_pid-1
    /// [`AtomUtf8`]: #variant.AtomUtf8
    /// [`SmallAtomUtf8`]: #variant.SmallAtomUtf8
    /// [`AtomCacheRef`]: #variant.AtomCacheRef
    /// [`Pid`]: #variant.Pid
    /// [dflags]: http://erlang.org/doc/apps/erts/erl_dist_protocol.html#dflags
    NewPid = 88,

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
    /// 
    /// # String representation
    /// 
    /// The string representation is simply a comma-separated list of
    /// stringified terms completely wrapped with curly braces.
    SmallTuple = 104,

    /// Same as [`SmallTuple`] except that Arity is an unsigned 4 byte
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
    /// # String representation
    /// 
    /// The string representation is simply a comma-separated list of
    /// stringified terms completely wrapped with curly braces.
    ///
    /// [`SmallTuple`]: #variant.SmallTuple
    LargeTuple = 105,

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
    /// 
    /// # String representation
    /// 
    /// A map is represented like this:
    /// 
    /// `#{ key0 => value0 (, keyN => valueN )* }`
    Map = 116,

    /// The representation for an empty list, that is, the Erlang syntax `[]`.
    ///
    /// # Binary representation
    ///
    /// | 1 byte |
    /// | ------ |
    /// | `106`  |
    /// 
    /// # String representation
    /// 
    /// `[]`.
    Nil = 106,

    /// String does not have a corresponding Erlang representation, but is an
    /// optimization for sending lists of bytes (integer in the range 0-255)
    /// more efficiently over the distribution.
    ///
    /// As field Length is an unsigned 2 byte integer (big-endian),
    /// implementations must ensure that lists longer than 65535 elements are
    /// encoded as [`List`].
    ///
    /// # Binary representation
    ///
    /// | 1 byte | 2 bytes  | `Length` bytes |
    /// | ------ | -------- | -------------- |
    /// | 107    | `Length` | `Characters`   |
    ///
    /// The characters are UTF-8 encoded characters.
    /// There are a maximum of 65535 bytes, however many characters that turns
    /// out to be.
    /// It is *not* a maximum of 65535 characters!
    /// 
    /// # String representation
    /// 
    /// Place the string between double quotes, and apply the following escape
    /// sequences:
    /// 
    /// | Actual character            | Escape sequence |
    /// | --------------------------- | --------------- |
    /// | backspace                   | `\b`            |
    /// | delete                      | `\d`            |
    /// | escape                      | `\e`            |
    /// | form feed                   | `\f`            |
    /// | newline                     | `\n`            |
    /// | carriage return             | `\r`            |
    /// | tab                         | `\t`            |
    /// | vertical tab                | `\v`            |
    /// | double qoute (")            | `\"`            |
    ///
    /// [`List`]: #variant.List
    String = 107,

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
    /// * `Tail` is the final tail of the list; it is [`Nil`] for a proper list,
    ///   but can be any type if the list is improper (for example, `[a|b]`).
    /// 
    /// # String representation
    /// 
    /// Place a comma-separated list of stringified terms between block-quotes.
    ///
    /// [`Nil`]: #variant.Nil
    List = 108,

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
    /// 
    /// # String representation
    /// 
    /// The output format is simply to convert each byte to a decimal number
    /// and print them as a comma-separated list between a pair of double angle
    /// brackets (`<<`, `>>`).
    /// 
    /// For reading, the values should be any of:
    /// 
    /// * A number according to [`Integer`]
    /// 
    /// [`Integer`]: #variant.Integer
    Binary = 109,

    /// Integer representation of an integer N where `-2^256 < N < 2^256`.
    /// 
    /// # Binary representation
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
    /// 
    /// # String representation
    /// 
    /// The representations of this 2's complement 32-bit signed integer are:
    /// 
    /// * Simple decimal
    /// * An ASCII character Z in the form `$Z`
    /// * Any other base B in 2..36 in the form `B#N`
    /// 
    /// The recommendation is to always output like a simple decimal, as it is
    /// the simplest and most portable.
    SmallBig = 110,

    /// Integer representation of an integer N where `-2^(2^32) < N < 2^(2^32)`.
    ///
    /// # Binary representation
    /// 
    /// | 1 byte | 4 bytes | 1 byte | `Len` bytes        |
    /// | ------ | ------- | ------ | ------------------ |
    /// | `111`  | `Len`   | `Sign` | `d0`..`d(Len - 1)` |
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
    ///
    /// # String representation
    ///
    /// The representations of this 2's complement 32-bit signed integer are:
    ///
    /// * Simple decimal
    /// * An ASCII character Z in the form `$Z`
    /// * Any other base B in 2..36 in the form `B#N`
    ///
    /// The recommendation is to always output like a simple decimal, as it is
    /// the simplest and most portable.
    LargeBig = 111,

    /// Deprecated method for encoding a reference term.
    ///
    /// # Binary representation
    ///
    /// | 1 byte | `N` bytes | 4 bytes | 1 byte     |
    /// | ------ | --------- | ------- | ---------- |
    /// | `101`  | `Node`    | `ID`    | `Creation` |
    ///
    /// * `Node` is the name of the originating node, encoded using
    ///   [`AtomUtf8`], [`SmallAtomUtf8`] or [`AtomCacheRef`].
    /// * `Creation` is a 8-bit unsigned integer.
    ///   All references originating from the same node incarnation must have
    ///   identical Creation values.
    ///   This makes it possible to separate references from old (crashed) nodes
    ///   from a new one.
    ///   The value zero should be avoided for normal operations as it is used
    ///   as a wild card for debug purpose
    ///   (like a pid returned by [`erlang:list_to_pid/1`]).
    /// * `ID` is a node-unique number that describes this reference uniquely.
    ///
    /// [`erlang:lst_to_pid/1`]: http://erlang.org/doc/man/erlang.html#list_to_pid-1
    /// [`AtomUtf8`]: #variant.AtomUtf8
    /// [`SmallAtomUtf8`]: #variant.SmallAtomUtf8
    /// [`AtomCacheRef`]: #variant.AtomCacheRef
    Reference = 101,

    /// Deprecated method for encoding a reference term.
    ///
    /// # Binary representation
    ///
    /// | 1 byte | 2 bytes | `N` bytes | 1 byte     | `Len * 4` bytes |
    /// | ------ | ------- | --------- | ---------- | --------------- |
    /// | `101`  | `Len`   | `Node`    | `Creation` | `ID`            |
    ///
    /// * `Node` is the name of the originating node, encoded using
    ///   [`AtomUtf8`], [`SmallAtomUtf8`] or [`AtomCacheRef`].
    /// * `Creation` is a 8-bit unsigned integer.
    ///   All references originating from the same node incarnation must have
    ///   identical Creation values.
    ///   This makes it possible to separate references from old (crashed) nodes
    ///   from a new one.
    ///   The value zero should be avoided for normal operations as it is used
    ///   as a wild card for debug purpose
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
    /// [`AtomUtf8`]: #variant.AtomUtf8
    /// [`SmallAtomUtf8`]: #variant.SmallAtomUtf8
    /// [`AtomCacheRef`]: #variant.AtomCacheRef
    NewReference = 114,

    /// Encodes a reference term.
    ///
    /// # Binary representation
    ///
    /// | 1 byte | 2 bytes | `N` bytes | 4 bytes    | `Len * 4` bytes |
    /// | ------ | ------- | --------- | ---------- | --------------- |
    /// | `90`   | `Len`   | `Node`    | `Creation` | `ID`            |
    ///
    /// * `Node` is the name of the originating node, encoded using
    ///   [`AtomUtf8`], [`SmallAtomUtf8`] or [`AtomCacheRef`].
    /// * `Creation` is a 32-bit big-endian unsigned integer.
    ///   All references originating from the same node incarnation must have
    ///   identical Creation values.
    ///   This makes it possible to separate references from old (crashed) nodes
    ///   from a new one.
    ///   The value zero should be avoided for normal operations as it is used
    ///   as a wild card for debug purpose
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
    /// [`AtomUtf8`]: #variant.AtomUtf8
    /// [`SmallAtomUtf8`]: #variant.SmallAtomUtf8
    /// [`AtomCacheRef`]: #variant.AtomCacheRef
    NewerReference = 90,

    /// Old encoding of internal functions:
    /// `fun F/A and fun(Arg1,..) -> ... end`.
    ///
    /// # Binary representation
    ///
    /// | 1 byte | 4 bytes   | N bytes | N' bytes | N'' bytes | N'''  bytes | N'''' bytes    |
    /// | ------ | --------- | ------- | -------- | --------- | ----------- | -------------- |
    /// | `117`  | `NumFree` | `Pid`   | `Module` | `Index`   | `Uniq`      | `Free vars...` |
    ///
    /// * `NumFree` is a 32-bit big-endian unsigned integer is the number of
    ///   free variables (`Free vars`).
    /// * `Pid` is a process identifier as in [`Pid`].
    ///   Represents the process in which the fun was created.
    /// * `Module`, encoded as an atom using [`AtomUtf8`], [`SmallAtomUtf8`], or
    ///   [`AtomCacheRef`], is the module that the fun is implemented in.
    /// * `Index`, encoded as an integer using [`SmallInteger`] or
    ///   [`Integer`] is typically a small index into the module's fun table.
    /// * `Uniq`, encoded as an integer using [`SmallInteger`] or
    ///   [`Integer`] is the hash value of the parse for the fun.
    /// * `Free vars` are `NumFree` amount of terms, each one encoded according
    ///   to its type.
    ///
    /// [`AtomUtf8`]: #variant.AtomUtf8
    /// [`SmallAtomUtf8`]: #variant.SmallAtomUtf8
    /// [`AtomCacheRef`]: #variant.AtomCacheRef
    /// [`SmallInteger`]: #variant.SmallInteger
    /// [`Integer`]: #variant.Integer
    /// [`Pid`]: #variant.Pid
    Fun = 117,

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
    /// * `NumFree` is the number of free variables, stored as a 32-bit big
    ///   endian unsigned integer.
    /// * `Module`, encoded as an atom using [`AtomUtf8`],
    ///   [`SmallAtomUtf8`], or [`AtomCacheRef`], is the module that the fun is
    ///   implemented in.
    /// * `OldIndex`, encoded as an integer encoded using [`SmallInteger`] or
    ///   [`Integer`], is typically a small index into the module's fun table.
    /// * `OldUniq`, encoded as an integer using using [`SmallInteger`] or
    ///   [`Integer`], is the hash value of the parse tree for the fun.
    /// * `Pid`, encoded using [`Pid`], represnts the process in which the fun
    ///   was created.
    /// * `Free vars` are `NumFree` amount of terms, each one encoded according
    ///   to its type.
    ///
    /// [`AtomUtf8`]: #variant.AtomUtf8
    /// [`SmallAtomUtf8`]: #variant.SmallAtomUtf8
    /// [`AtomCacheRef`]: #variant.AtomCacheRef
    /// [`SmallInteger`]: #variant.SmallInteger
    /// [`Integer`]: #variant.Integer
    /// [`Pid`]: #variant.Pid
    NewFun = 112,

    /// Encodes functions of the shape `fun M:F/A`.
    ///
    /// # Binary representation
    ///
    /// | 1 byte | N bytes  | N' bytes   | N'' bytes |
    /// | ------ | -------- | ---------- | --------- |
    /// | `113`  | `Module` | `Function` | `Arity`   |
    ///
    /// * `Module`, encoded using [`AtomUtf8`], [`SmallAtomUtf8`], or
    ///   [`AtomCacheRef`], is the module that exports this function.
    /// * `Function`, encoded using [`AtomUtf8`], [`SmallAtomUtf8`], or
    ///   [`AtomCacheRef`], is the name of this function.
    /// * `Arity`, encoded using [`SmallInteger`], is the arity of this
    ///   function.
    ///
    /// [`AtomUtf8`]: #variant.AtomUtf8
    /// [`SmallAtomUtf8`]: #variant.SmallAtomUtf8
    /// [`AtomCacheRef`]: #variant.AtomCacheRef
    /// [`SmallInteger`]: #variant.SmallInteger
    Export = 113,

    /// This term represents a bitstring whose length in bits does not have to
    /// be a multiple of 8.
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
    BitBinary = 77,

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
    NewFloat = 70,

    /// This term represents an atom.
    ///
    /// # Binary representation
    ///
    /// | 1 byte | 2 bytes | `Len` bytes |
    /// | ------ | ------- | ----------- |
    /// | `118`  | `Len`   | `AtomName`  |
    ///
    /// * `Len`, represented as a 16-bit big-endian unsigned integer, is the
    ///   amount of bytes the atom name takes.
    /// * `AtomName` is an unescaped UTF8 string of `Len` bytes long
    ///   representing the name of this atom.
    AtomUtf8 = 118,

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
    /// * `Len`, represented as an 8-bit unsigned integer, is the amount of
    ///   bytes the atom name takes.
    /// * `AtomName` is an unescaped UTF8 string of `Len` bytes long
    ///   representing the name of this atom.
    SmallAtomUtf8 = 119,

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
    Atom = 100,

    /// This represents a LATIN-1 atom, but should not be encoded anymore.
    ///
    /// # Binary representation
    ///
    /// | 1 byte |  byte | `Len` bytes |
    /// | ------ | ----- | ----------- |
    /// | `118`  | `Len` | `AtomName`  |
    ///
    /// * `Len`, represented as an 8-bit unsigned integer, is the amount of
    ///   bytes the atom name takes.
    /// * `AtomName` is an unescaped LATIN-1 string of `Len` bytes long
    ///   representing the name of this atom.
    SmallAtom = 115,
}

impl TryFrom<u8> for TermTag {
    type Error = ();

    #[allow(clippy::cognitive_complexity)]
    fn try_from(from: u8) -> Result<TermTag, ()> {
        match from {
            x if x == TermTag::AtomCacheRef as u8 => Ok(TermTag::AtomCacheRef),
            x if x == TermTag::SmallInteger as u8 => Ok(TermTag::SmallInteger),
            x if x == TermTag::Integer as u8 => Ok(TermTag::Integer),
            x if x == TermTag::Float as u8 => Ok(TermTag::Float),
            x if x == TermTag::Port as u8 => Ok(TermTag::Port),
            x if x == TermTag::NewPort as u8 => Ok(TermTag::NewPort),
            x if x == TermTag::Pid as u8 => Ok(TermTag::Pid),
            x if x == TermTag::NewPid as u8 => Ok(TermTag::NewPid),
            x if x == TermTag::SmallTuple as u8 => Ok(TermTag::SmallTuple),
            x if x == TermTag::LargeTuple as u8 => Ok(TermTag::LargeTuple),
            x if x == TermTag::Map as u8 => Ok(TermTag::Map),
            x if x == TermTag::Nil as u8 => Ok(TermTag::Nil),
            x if x == TermTag::String as u8 => Ok(TermTag::String),
            x if x == TermTag::List as u8 => Ok(TermTag::List),
            x if x == TermTag::Binary as u8 => Ok(TermTag::Binary),
            x if x == TermTag::SmallBig as u8 => Ok(TermTag::SmallBig),
            x if x == TermTag::LargeBig as u8 => Ok(TermTag::LargeBig),
            x if x == TermTag::Reference as u8 => Ok(TermTag::Reference),
            x if x == TermTag::NewReference as u8 => Ok(TermTag::NewReference),
            x if x == TermTag::NewerReference as u8 => Ok(TermTag::NewerReference),
            x if x == TermTag::Fun as u8 => Ok(TermTag::Fun),
            x if x == TermTag::NewFun as u8 => Ok(TermTag::NewFun),
            x if x == TermTag::Export as u8 => Ok(TermTag::Export),
            x if x == TermTag::BitBinary as u8 => Ok(TermTag::BitBinary),
            x if x == TermTag::NewFloat as u8 => Ok(TermTag::NewFloat),
            x if x == TermTag::AtomUtf8 as u8 => Ok(TermTag::AtomUtf8),
            x if x == TermTag::SmallAtomUtf8 as u8 => Ok(TermTag::SmallAtomUtf8),
            x if x == TermTag::Atom as u8 => Ok(TermTag::Atom),
            x if x == TermTag::SmallAtom as u8 => Ok(TermTag::SmallAtom),
            _ => Err(()),
        }
    }
}

/// A type that can be converted to an Erlang Binary Term format and two valid
/// Erlang String Term representations.
pub trait ETerm: encode::ToExternalBinary + fmt::Display + Any {
    fn write_to(&self, writer: &mut dyn Write) -> Result<usize, Error> {
        self.to_writer(writer)
    }

    fn to_external_binary(&self) -> Result<Vec<u8>, Error> {
        let mut result = Vec::new();

        self.to_writer(&mut result)?;

        Ok(result)
    }
}

impl<T> ETerm for T where T: encode::ToExternalBinary + fmt::Display + Any {}

/// Represents an Erlang `NIL_EXT` term.
#[derive(Default)]
pub struct ENil;

impl fmt::Display for ENil {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[]")
    }
}

/// Represents a proper `LIST_EXT` term with a `nil` tail.
pub struct EList(Vec<Box<dyn ETerm>>);

impl<'a> fmt::Display for EList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = "[".to_string();
        for d in self.0.iter() {
            s.push_str(format!("{}", *d).as_ref());
        }
        s.push(']');

        f.write_str(s.as_ref())
    }
}

/// Describes a `LIST_EXT` term with a possible non-`nil` tail.
pub struct ENonProperList {
    data: Vec<Box<dyn ETerm>>,
    tail: Box<dyn ETerm>,
}

impl fmt::Display for ENonProperList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = "[".to_string();
        for d in self.data.iter() {
            s.push_str(format!("{}", d).as_ref());
        }
        s.push('|');
        s.push_str(format!("{}", self.tail).as_ref());
        s.push(']');

        f.write_str(s.as_ref())
    }
}

/// Describes an `ATOM_UTF8_EXT` term and a `SMALL_ATOM_UTF8_EXT` term.
///
/// TODO: Make sure the string representation of atoms that need to be quoted
///  is implemented.
pub struct EAtom(String);

impl fmt::Display for EAtom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        lazy_static! {
            static ref RX_SIMPLE_ATOM_REPR: Regex =
                Regex::new("[a-z@][0-9a-zA-Z_@]*").unwrap();
        }

        if RX_SIMPLE_ATOM_REPR.is_match(self.0.as_ref()) {
            // It is not necessary to escape the atom, so don't.
            write!(f, "{}", self.0)
        } else {
            // It is necessary to escape the atom.
            write!(f, "'{}'", escape_string(&self.0))
        }
    }
}

/// Describes an Erlang `EXPORT_EXT` term.
pub struct EExport {
    module: EAtom,
    function: EAtom,
    arity: u8,
}

impl fmt::Display for EExport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = "{".to_string();
        result.push_str(format!("{}", self.module).as_ref());
        result.push(',');
        result.push_str(format!("{}", self.function).as_ref());
        result.push(',');
        result.push_str(self.arity.to_string().as_ref());
        result.push('}');

        f.write_str(result.as_ref())
    }
}

/// Represents a `LARGE_TUPLE_EXT` or a `SMALL_TUPLE_EXT` term.
pub struct ETuple(Vec<Box<dyn ETerm>>);

impl fmt::Display for ETuple {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = "{".to_string();
        let mut parts: Vec<String> = Vec::new();
        for d in self.0.iter() {
            parts.push(format!("{}", d))
        }
        result.push_str(parts.join(",").as_ref());
        result.push('}');
        
        f.write_str(result.as_ref())
    }
}

/// Describes an `STRING_EXT` term.
///
/// TODO: Test proper quoting of string.
pub struct EString(String);

impl fmt::Display for EString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\"{}\"", escape_string(&self.0))
    }
}

/// Describes an Erlang Port
pub struct EPort {
    node: EAtom,
    id: u32,
    creation: u32,
}

impl fmt::Display for EPort {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#Port<{}.{}>", self.node, self.id)
    }
}

/// Describes an Erlang PID.
pub struct EPid {
    node: EAtom,
    id: u32,
    serial: u32,
    creation: u32,
}

impl fmt::Display for EPid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}.{}.{}>", self.node, self.id, self.serial)
    }
}

/// Describes an Erlang Map
pub struct EMap(Vec<(Box<dyn ETerm>, Box<dyn ETerm>)>);

impl fmt::Display for EMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = "#{".to_string();
        let mut parts: Vec<String> = Vec::new();
        for (k, v) in self.0.iter() {
            parts.push(format!("{}=>{}", k, v));
        }
        result.push_str(parts.join(",").as_ref());
        result.push('}');
        
        f.write_str(result.as_ref())
    }
}

/// Describes an Erlang Binary
struct EBinary(Vec<u8>);

impl fmt::Display for EBinary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = "<<".to_string();
        let mut parts = Vec::new();
        for byte in self.0.iter() {
            parts.push(byte.to_string());
        }

        result.push_str(parts.join(",").as_ref());
        result.push_str(">>");

        f.write_str(result.as_ref())
    }
}

fn escape_string(s: &str) -> String {
    let mut result = String::new();
    s.escape_default();
    for c in s.chars() {
        result.push_str(
            match c {
                '\\' => "\\\\".to_string(),
                '\x01'..='\x07' => format!("\\x{}", to_hex(c, 2)),
                '\x08' => "\\b".to_string(),
                '\t' => "\\t".to_string(),
                '\n' => "\\n".to_string(),
                '\x0b' => "\\v".to_string(),
                '\x0c' => "\\f".to_string(),
                '\r' => "\\r".to_string(),
                '\x0e'..='\x1a' => format!("\\x{}", to_hex(c, 2)),
                '\x1b' => "\\e".to_string(),
                '\x1c'..='\x1f' => format!("\\x{}", to_hex(c, 2)),
                '\x20'..='\x7e' => c.to_string(),
                '\x7f' => "\\d".to_string(),
                _ => format!("\\x{{{}}}", to_hex(c, 1)), // Convert to the shortest hex sequence possible
            }.as_ref()
        );
    }
    result
}

fn to_hex(c: char, len: usize) -> String {
    let mut tmp = c as u32;
    let mut result = String::new();

    while tmp != 0 {
        let val = tmp % 16;
        result.push(
            match val {
                0..=9 => (val + 48) as u8 as char,
                10..=15 => ((val - 10 + 65) as u8 as char),
                _ => '\0',
            }
        );
        tmp /= 16;
    }

    // String::len() is fine here as it only contains single-byte characters.
    while result.len() < len {
        result.push('0');
    }

    result.chars().rev().collect()
}
