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
//! [Cap'n Proto](http://kentonv.github.io/capnproto/) is an extremely efficient protocol for
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

#![feature(alloc, io, core, unsafe_destructor)]
#![allow(raw_pointer_derive)]

#![crate_name="capnp"]
#![crate_type = "lib"]

// reexports
pub use blob::{text, data};
pub use common::{MessageSize, Word};
pub use list::{primitive_list, enum_list, struct_list, text_list, data_list, list_list};
pub use message::{MessageBuilder, BuilderOptions, MessageReader, ReaderOptions};
pub use message::MallocMessageBuilder;
pub use serialize::OwnedSpaceMessageReader;

pub mod any_pointer;
pub mod arena;
mod blob;
pub mod capability;
mod common;
mod endian;
pub mod private;
pub mod io;
pub mod layout;
pub mod list;
mod mask;
pub mod message;
pub mod serialize;
pub mod serialize_packed;
pub mod traits;


#[cfg(test)]
pub mod layout_test;
#[cfg(test)]
pub mod serialize_packed_test;




