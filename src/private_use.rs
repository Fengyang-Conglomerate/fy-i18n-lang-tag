//! Pure private-use language tag module.
//!
//! Contains error definitions, borrowed views, and generic owned structures for pure private-use language tags.

pub mod error;
pub mod tag;
pub mod view;

pub use error::{ParseError, TagError};
pub use tag::PrivateUseTag;
pub use view::{PrivateUseTagView, is_singleton, is_subtag_bytes};
