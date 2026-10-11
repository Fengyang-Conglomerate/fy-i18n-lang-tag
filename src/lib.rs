#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

extern crate alloc;

mod error;
mod subtag;
mod tag;

pub use error::{BuildError, ParseError};
pub use subtag::{Extension, Language, Region, Script, Subtag, Variant};
pub use tag::{NormalTag, PrivateUseTag, Tag};
