//! Error types for private-use language tag (`PrivateUseTag`) parsing and construction.

use crate::storage::CapacityError;
use core::fmt;

/// An error indicating that a private-use tag failed well-formedness parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParseError {
    /// Input is empty.
    Empty,
    /// Invalid prefix: private-use tags must begin with `x-` or `X-`.
    InvalidPrefix,
    /// Empty subtag encountered (e.g. consecutive hyphens `--`).
    EmptySubtag,
    /// Trailing separator hyphen encountered.
    TrailingSeparator,
    /// Subtag length exceeds the 8-character limit.
    SubtagTooLong,
    /// Invalid character encountered: subtags must consist of ASCII alphanumerics.
    InvalidCharacter,
}

impl ParseError {
    /// Returns a human-readable description of the error.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Empty => "input is empty",
            Self::InvalidPrefix => "invalid prefix: private-use tag must start with 'x-' or 'X-'",
            Self::EmptySubtag => "empty subtag: consecutive hyphens encountered",
            Self::TrailingSeparator => "trailing separator hyphen",
            Self::SubtagTooLong => "subtag exceeds 8-character maximum length",
            Self::InvalidCharacter => "invalid character: non-alphanumeric character encountered",
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl core::error::Error for ParseError {}

/// An aggregate error returned when constructing or parsing an owned private-use tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TagError {
    /// Syntax well-formedness validation failed.
    Parse(ParseError),
    /// Storage backend capacity was insufficient.
    Capacity(CapacityError),
}

impl From<ParseError> for TagError {
    #[inline]
    fn from(err: ParseError) -> Self {
        Self::Parse(err)
    }
}

impl From<CapacityError> for TagError {
    #[inline]
    fn from(err: CapacityError) -> Self {
        Self::Capacity(err)
    }
}

impl fmt::Display for TagError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(err) => write!(f, "parse error: {err}"),
            Self::Capacity(err) => write!(f, "capacity error: {err}"),
        }
    }
}

impl core::error::Error for TagError {}
