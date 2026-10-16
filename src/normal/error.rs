//! Error types for normal language tag (`NormalTag`) parsing and construction.

use crate::storage::CapacityError;
use core::fmt;

/// An error indicating that a normal language tag failed well-formedness parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParseError {
    /// Input is empty.
    Empty,
    /// Invalid primary language subtag: expected 2-8 ASCII letters (covering RFC 5646 4ALPHA rule).
    InvalidLanguage,
    /// Invalid script subtag: expected exactly 4 ASCII letters.
    InvalidScript,
    /// Invalid region subtag: expected 2 ASCII letters or 3 ASCII digits.
    InvalidRegion,
    /// Invalid variant subtag: expected 5-8 ASCII alphanumerics or 4 starting with a digit.
    InvalidVariant,
    /// Duplicate variant subtag encountered.
    DuplicateVariant,
    /// Invalid extension: singleton must be an alphanumeric other than `x`/`X`, followed by 2-8 character subtags.
    InvalidExtension,
    /// Duplicate extension singleton encountered.
    DuplicateSingleton,
    /// Invalid private-use subtag: must start with `x`/`X` followed by at least one 1-8 character subtag.
    InvalidPrivateUse,
    /// Empty subtag encountered (e.g. consecutive hyphens `--`).
    EmptySubtag,
    /// Trailing separator hyphen encountered.
    TrailingSeparator,
    /// Subtag length exceeds the 8-character limit.
    SubtagTooLong,
    /// Invalid character encountered: subtags must consist of ASCII alphanumerics.
    InvalidCharacter,
    /// Duplicate key in `u` extension (forbidden by strict well-formedness validation).
    DuplicateUKey,
    /// Invalid `u` extension type: expected 3-8 ASCII alphanumerics.
    InvalidUType,
    /// Invalid `u` extension attribute: expected 3-8 ASCII alphanumerics appearing before keywords.
    InvalidUAttribute,
    /// Invalid `t` extension field: expected a field separator (ALPHA DIGIT) followed by 3-8 character subtags.
    InvalidTField,
    /// Duplicate field separator in `t` extension.
    DuplicateTField,
    /// Invalid source language tag in `t` extension.
    InvalidTLanguage,
}

impl ParseError {
    /// Returns a human-readable description of the error.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Empty => "input is empty",
            Self::InvalidLanguage => "invalid primary language: expected 2 to 8 ASCII letters",
            Self::InvalidScript => "invalid script subtag: expected 4 ASCII letters",
            Self::InvalidRegion => {
                "invalid region subtag: expected 2 ASCII letters or 3 ASCII digits"
            }
            Self::InvalidVariant => {
                "invalid variant subtag: expected 5-8 alphanumerics or 4 starting with a digit"
            }
            Self::DuplicateVariant => "duplicate variant subtag",
            Self::InvalidExtension => {
                "invalid extension: singleton must be non-x alphanumeric followed by 2-8 character subtags"
            }
            Self::DuplicateSingleton => "duplicate extension singleton",
            Self::InvalidPrivateUse => {
                "invalid private-use field: expected 'x-' followed by 1-8 character subtags"
            }
            Self::EmptySubtag => "empty subtag: consecutive hyphens encountered",
            Self::TrailingSeparator => "trailing separator hyphen",
            Self::SubtagTooLong => "subtag exceeds 8-character maximum length",
            Self::InvalidCharacter => "invalid character: non-alphanumeric character encountered",
            Self::DuplicateUKey => "duplicate key in u extension",
            Self::InvalidUType => "invalid u extension type: expected 3-8 ASCII alphanumerics",
            Self::InvalidUAttribute => {
                "invalid u extension attribute: expected 3-8 ASCII alphanumerics appearing before keywords"
            }
            Self::InvalidTField => {
                "invalid t extension field: expected alphanumeric separator followed by at least one 3-8 character subtag"
            }
            Self::DuplicateTField => "duplicate field separator in t extension",
            Self::InvalidTLanguage => "invalid source language tag in t extension",
        }
    }
}

impl From<crate::private_use::ParseError> for ParseError {
    #[inline]
    fn from(err: crate::private_use::ParseError) -> Self {
        match err {
            crate::private_use::ParseError::Empty => Self::Empty,
            crate::private_use::ParseError::InvalidPrefix => Self::InvalidPrivateUse,
            crate::private_use::ParseError::EmptySubtag => Self::EmptySubtag,
            crate::private_use::ParseError::TrailingSeparator => Self::TrailingSeparator,
            crate::private_use::ParseError::SubtagTooLong => Self::SubtagTooLong,
            crate::private_use::ParseError::InvalidCharacter => Self::InvalidCharacter,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl core::error::Error for ParseError {}

/// An aggregate error returned when constructing or parsing an owned normal language tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TagError {
    /// Syntax well-formedness validation failed.
    Parse(ParseError),
    /// Storage backend capacity was insufficient for extensions.
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
            Self::Capacity(err) => write!(f, "extension capacity error: {err}"),
        }
    }
}

impl core::error::Error for TagError {}
