// Copyright (c) 2013-2015 Sandstorm Development Group, Inc. and contributors
// Licensed under the MIT License:
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

//! # Cap'n Proto Runtime Library
//!
//! [Cap'n Proto](https://capnproto.org) is an extremely efficient protocol for
//! sharing data and capabilities.
//!
//! The Rust implementation is split into three separate crates.
//!
//! Code generation is handled by [capnpc-rust](https://github.com/dwrensha/capnpc-rust).
//!
//! The present crate is the runtime library required by that generated code. It is hosted on Github
//! [here](https://github.com/dwrensha/capnproto-rust).
//!
//! [capnp-rpc-rust](https://github.com/dwrensha/capnp-rpc-rust) is an implementation of a
//! distributed object-capability layer.

#![allow(raw_pointer_derive)]

extern crate byteorder;

#[cfg(any(feature="quickcheck", test))]
extern crate quickcheck;

#[cfg(feature = "rpc")]
extern crate gj;

pub mod any_pointer;
pub mod capability;
pub mod data;
pub mod data_list;
pub mod enum_list;
pub mod list_list;
pub mod message;
pub mod primitive_list;
pub mod private;
pub mod serialize;
pub mod serialize_packed;
pub mod struct_list;
pub mod text;
pub mod text_list;
pub mod traits;

mod util;

/// Eight bytes of memory with opaque interior.
///
/// This type is used to ensure that the data of a message is properly aligned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Word(u64);

impl Word {
    /// Does this, but faster:
    /// `::std::iter::repeat(Word(0)).take(length).collect()`
    pub fn allocate_zeroed_vec(length: usize) -> Vec<Word> {
        let mut result : Vec<Word> = Vec::with_capacity(length);
        unsafe {
            result.set_len(length);
            let p : *mut u8 = result.as_mut_ptr() as *mut u8;
            ::std::ptr::write_bytes(p, 0u8, length * ::std::mem::size_of::<Word>());
        }
        result
    }

    pub fn bytes_to_words<'a>(bytes: &'a [u8]) -> &'a [Word] {
        unsafe {
            ::std::slice::from_raw_parts(bytes.as_ptr() as *const Word, bytes.len() / 8)
        }
    }

    pub fn bytes_to_words_mut<'a>(bytes: &'a mut [u8]) -> &'a mut [Word] {
        unsafe {
            ::std::slice::from_raw_parts_mut(bytes.as_ptr() as *mut Word, bytes.len() / 8)
        }
    }

    pub fn words_to_bytes<'a>(words: &'a [Word]) -> &'a [u8] {
        unsafe {
            ::std::slice::from_raw_parts(words.as_ptr() as *const u8, words.len() * 8)
        }
    }

    pub fn words_to_bytes_mut<'a>(words: &'a mut [Word]) -> &'a mut [u8] {
        unsafe {
            ::std::slice::from_raw_parts_mut(words.as_mut_ptr() as *mut u8, words.len() * 8)
        }
    }

    #[cfg(test)]
    pub fn from(n: u64) -> Word {
        Word(n)
    }
}

#[cfg(any(feature="quickcheck", test))]
impl quickcheck::Arbitrary for Word {
    fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Word {
        Word(quickcheck::Arbitrary::arbitrary(g))
    }
}

/// Size of a message. Every generated struct has a method `.total_size()` that returns this.
#[derive(Clone, Copy, PartialEq)]
pub struct MessageSize {
    pub word_count: u64,

    /// Size of the capability table.
    pub cap_count: u32
}

impl MessageSize {
    pub fn plus_eq(&mut self, other : MessageSize) {
        self.word_count += other.word_count;
        self.cap_count += other.cap_count;
    }
}

/// An enum value or union discriminant that was not found among those defined in a schema.
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct NotInSchema(pub u16);

impl ::std::fmt::Display for NotInSchema {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        write!(fmt, "Enum value or union discriminant {} was not present in the schema.", self.0)
    }
}

impl ::std::error::Error for NotInSchema {
    fn description<'a>(&'a self) -> &'a str {
        "Enum value or union disriminant was not present in schema."
    }
}

/// Because messages are lazily validated, the return type of any method that reads a pointer field
/// must be wrapped in a Result.
pub type Result<T> = ::std::result::Result<T, Error>;

/// Describes an arbitrary error that prevented an operation from completing.
#[derive(Debug, Clone)]
pub struct Error {
    /// The type of the error. The purpose of this enum is not to describe the error itself, but
    /// rather to describe how the client might want to respond to the error.
    pub kind: ErrorKind,

    /// Human-readable failure description.
    pub reason: String,
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorKind {
    /// A generic problem occurred, and it is believed that if the operation were repeated without
    /// any change in the state of the world, the problem would occur again.
    ///
    /// A client might respond to this error by logging it for investigation by the developer and/or
    /// displaying it to the user.
    Failed,


    /// The request was rejected due to a temporary lack of resources.
    ///
    /// Examples include:
    ///  - There's not enough CPU time to keep up with incoming requests, so some are rejected.
    ///  - The server ran out of RAM or disk space during the request.
    ///  - The operation timed out (took significantly longer than it should have).
    ///
    /// A client might respond to this error by scheduling to retry the operation much later. The
    /// client should NOT retry again immediately since this would likely exacerbate the problem.
    Overloaded,

    /// The method failed because a connection to some necessary capability was lost.
    ///
    /// Examples include:
    /// - The client introduced the server to a third-party capability, the connection to that third
    ///   party was subsequently lost, and then the client requested that the server use the dead
    ///   capability for something.
    /// - The client previously requested that the server obtain a capability from some third party.
    ///   The server returned a capability to an object wrapping the third-party capability. Later,
    ///   the server's connection to the third party was lost.
    /// - The capability has been revoked. Revocation does not necessarily mean that the client is
    ///   no longer authorized to use the capability; it is often used simply as a way to force the
    ///   client to repeat the setup process, perhaps to efficiently move them to a new back-end or
    ///   get them to recognize some other change that has occurred.
    ///
    /// A client should normally respond to this error by releasing all capabilities it is currently
    /// holding related to the one it called and then re-creating them by restoring SturdyRefs and/or
    /// repeating the method calls used to create them originally. In other words, disconnect and
    /// start over. This should in turn cause the server to obtain a new copy of the capability that
    /// it lost, thus making everything work.
    ///
    /// If the client receives another `disconnencted` error in the process of rebuilding the
    /// capability and retrying the call, it should treat this as an `overloaded` error: the network
    /// is currently unreliable, possibly due to load or other temporary issues.
    Disconnected,

    /// The server doesn't implement the requested method. If there is some other method that the
    /// client could call (perhaps an older and/or slower interface), it should try that instead.
    /// Otherwise, this should be treated like `failed`.
    Unimplemented,
}

impl Error {
    pub fn new_decode_error(description: String) -> Error {
        Error { reason: description, kind: ErrorKind::Failed }
    }
}

impl ::std::convert::From<::std::io::Error> for Error {
    fn from(err: ::std::io::Error) -> Error {
        Error { reason: format!("{}", err), kind: ErrorKind::Failed }
    }
}

impl ::std::convert::From<NotInSchema> for Error {
    fn from(e: NotInSchema) -> Error {
        Error::new_decode_error(format!("Enum value or union discriminant {} was not present in schema.", e.0))
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        write!(fmt, "{:?}: {}", self.kind, self.reason)
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        &self.reason
    }
    fn cause(&self) -> Option<&::std::error::Error> {
        None
    }
}

#[cfg(feature = "rpc")]
impl ::gj::FulfillerDropped for Error {
    fn fulfiller_dropped() -> Error {
        Error {
            reason: "Promise fulfiller was dropped.".to_string(),
            kind: ErrorKind::Failed
        }
    }
}

/// Helper struct that allows `MessageBuilder::get_segments_for_output()` to avoid heap allocations
/// in the single-segment case.
pub enum OutputSegments<'a> {
    #[doc(hidden)]
    SingleSegment([&'a [Word]; 1]),

    #[doc(hidden)]
    MultiSegment(Vec<&'a [Word]>),
}

impl <'a> ::std::ops::Deref for OutputSegments<'a> {
    type Target = [&'a [Word]];
    fn deref<'b>(&'b self) -> &'b [&'a [Word]] {
        match self {
            &OutputSegments::SingleSegment(ref s) => {
                s
            }
            &OutputSegments::MultiSegment(ref v) => {
                &*v
            }
        }
    }
}
