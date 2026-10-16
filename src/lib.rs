//! # fy-i18n-lang-tag
//!
//! Lightweight BCP 47 language tag parsing and manipulation library.
//!
//! - 100% Safe Rust by default, with optional `unsafe` feature for maximum throughput;
//! - Core `no_std` support, with optional `alloc`;
//! - Zero-allocation borrowed views ([`NormalTagView`], [`PrivateUseTagView`]);
//! - Owned structures feature compact 15-byte core storage and pre-indexed extension boundaries, projecting views without re-parsing;
//! - Fixed-rule core matching and structural fallback by component stripping;
//! - Optional `serde` serialization/deserialization support.

#![no_std]
#![cfg_attr(not(feature = "unsafe"), forbid(unsafe_code))]
#![cfg_attr(feature = "unsafe", deny(unsafe_op_in_unsafe_fn))]
#![deny(missing_docs)]

pub mod normal;
pub mod private_use;
pub mod storage;

mod tag_bytes;

#[cfg(feature = "serde")]
mod serde;

pub use normal::{
    LanguageMatch, LanguageView, MatchQuality, MatchResult, NormalTag, NormalTagView,
    OtherExtensionView, OtherExtensionsIter, ParseError, RegionView, ScriptView,
    StructuralFallbackIter, TExtensionView, TagError, UExtensionView, match_language,
    match_languages,
};
pub use private_use::{PrivateUseTag, PrivateUseTagView};
pub use storage::{CapacityError, Fixed, Storage};

#[cfg(feature = "alloc")]
pub use storage::Grow;
