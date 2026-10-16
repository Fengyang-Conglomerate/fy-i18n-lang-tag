//! Normal language tag module.
//!
//! Contains RFC 5646 compliant normal language tag borrowed views [`NormalTagView`],
//! owned generic structures [`NormalTag`], and syntax validation/matching modules:
//! - [`variant`]: Variant subtag syntax validation;
//! - [`extensions`]: Generic extension subtag syntax validation;
//! - [`u`]: Unicode locale extension (RFC 6067) syntax validation;
//! - [`t`]: Transformed content extension (RFC 6497) syntax validation;
//! - [`matching`]: Simple syntax-level language tag matching and structural fallback;
//! - [`crate::private_use`]: Private-use subtag syntax validation.

pub mod error;
pub mod extensions;
pub mod language;
pub mod matching;
pub mod region;
pub mod script;
pub mod t;
pub mod tag;
pub mod u;
pub mod variant;
pub mod view;

pub use error::{ParseError, TagError};
pub use extensions::{OtherExtensionView, OtherExtensionsIter};
pub use language::LanguageView;
pub use matching::{
    LanguageMatch, MatchQuality, MatchResult, StructuralFallbackIter, match_language,
    match_languages,
};
pub use region::RegionView;
pub use script::ScriptView;
pub use t::TExtensionView;
pub use tag::NormalTag;
pub use u::UExtensionView;
pub use view::NormalTagView;
