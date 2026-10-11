//! Errors returned by parsing and component-based construction.

use core::fmt::{Display, Formatter};

/// An error encountered while parsing a tag or subtag from a string.
///
/// Parsing positions are zero-based byte offsets into the input. They point to the
/// offending subtag, except [`ParseError::InvalidCharacter`], which points to the
/// character itself. A trailing empty subtag starts at the input's length.
/// Component-based constructors return [`BuildError`] instead.
///
/// Tag and extension parsers scan lexical constraints from left to right before
/// checking grammar. Invalid characters are detected when visited; empty and
/// overlong subtags are detected at a separator or the end of input. The first
/// detected lexical error wins. Grammar errors depend on the entry point and
/// the slots still available, not only on the offending subtag's shape.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseError {
    /// The input is empty.
    Empty,
    /// A character is not allowed in this input. Tags accept ASCII letters, digits,
    /// and hyphen separators; an individual [`Subtag`](crate::Subtag) accepts only
    /// ASCII letters and digits.
    InvalidCharacter {
        /// The byte offset of the invalid character.
        position: usize,
    },
    /// An empty subtag caused by consecutive, leading, or trailing hyphens.
    EmptySubtag {
        /// The byte offset of the empty subtag.
        position: usize,
    },
    /// A subtag exceeds eight bytes, the maximum length of a supported ASCII subtag.
    SubtagTooLong {
        /// The byte offset of the subtag.
        position: usize,
        /// The length of the subtag in bytes.
        length: usize,
    },
    /// An invalid language: expected 2-3 or 5-8 ASCII letters.
    /// Also returned for private-use-only input to [`NormalTag::parse`](crate::NormalTag::parse),
    /// or a non-`x` prefix passed to [`PrivateUseTag::parse`](crate::PrivateUseTag::parse).
    InvalidLanguage {
        /// The byte offset of the language subtag.
        position: usize,
    },
    /// An invalid script subtag: expected exactly four ASCII letters.
    InvalidScript {
        /// The byte offset of the subtag.
        position: usize,
    },
    /// An invalid region subtag: expected two ASCII letters or three ASCII digits.
    InvalidRegion {
        /// The byte offset of the subtag.
        position: usize,
    },
    /// An invalid variant: expected 5-8 ASCII alphanumerics, or four starting with a digit.
    InvalidVariant {
        /// The byte offset of the variant subtag.
        position: usize,
    },
    /// A variant occurs more than once, compared case-insensitively.
    DuplicateVariant {
        /// The byte offset of the duplicate variant.
        position: usize,
    },
    /// An invalid or duplicate extension singleton, or a missing or invalid payload.
    /// The singleton must be an ASCII letter or digit other than `x`
    /// (case-insensitive), followed by at least one subtag of 2-8 ASCII alphanumerics.
    InvalidExtension {
        /// The byte offset of the singleton.
        position: usize,
    },
    /// A private-use sequence is empty: the `x` prefix requires at least one subtag.
    InvalidPrivateUse {
        /// The byte offset of the `x` prefix.
        position: usize,
    },
    /// A private-use-only parser requires an `x` prefix (case-insensitive).
    MissingPrivateUsePrefix {
        /// The byte offset of the first subtag, which is not `x`.
        position: usize,
    },
    /// A subtag has no supported role, such as an extlang-shaped subtag or a leftover
    /// after all applicable grammar slots have closed.
    UnexpectedSubtag {
        /// The byte offset of the subtag.
        position: usize,
    },
}

impl ParseError {
    /// Returns the error's byte offset, or `0` for empty input.
    pub const fn position(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::InvalidCharacter { position }
            | Self::EmptySubtag { position }
            | Self::SubtagTooLong { position, .. }
            | Self::InvalidLanguage { position }
            | Self::InvalidScript { position }
            | Self::InvalidRegion { position }
            | Self::InvalidVariant { position }
            | Self::DuplicateVariant { position }
            | Self::InvalidExtension { position }
            | Self::InvalidPrivateUse { position }
            | Self::MissingPrivateUsePrefix { position }
            | Self::UnexpectedSubtag { position } => *position,
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Empty => formatter.write_str("input is empty"),
            Self::InvalidCharacter { position } => {
                write!(formatter, "invalid character at byte {position}")
            }
            Self::EmptySubtag { position } => {
                write!(
                    formatter,
                    "empty subtag at byte {position}: consecutive, leading, or trailing hyphens"
                )
            }
            Self::SubtagTooLong { position, length } => {
                write!(
                    formatter,
                    "subtag at byte {position} is {length} bytes long; maximum is 8"
                )
            }
            Self::InvalidLanguage { position } => {
                write!(
                    formatter,
                    "invalid language subtag at byte {position}: expected 2-3 or 5-8 ASCII letters"
                )
            }
            Self::InvalidScript { position } => {
                write!(
                    formatter,
                    "invalid script subtag at byte {position}: expected 4 ASCII letters"
                )
            }
            Self::InvalidRegion { position } => {
                write!(
                    formatter,
                    "invalid region subtag at byte {position}: expected 2 ASCII letters or 3 ASCII digits"
                )
            }
            Self::InvalidVariant { position } => {
                write!(
                    formatter,
                    "invalid variant subtag at byte {position}: expected 5-8 ASCII alphanumerics or 4 starting with a digit"
                )
            }
            Self::DuplicateVariant { position } => {
                write!(formatter, "duplicate variant subtag at byte {position}")
            }
            Self::InvalidExtension { position } => {
                write!(
                    formatter,
                    "invalid extension at byte {position}: invalid or duplicate singleton, or missing or invalid subtags"
                )
            }
            Self::InvalidPrivateUse { position } => {
                write!(
                    formatter,
                    "empty private-use sequence at byte {position}: expected at least one subtag"
                )
            }
            Self::MissingPrivateUsePrefix { position } => write!(
                formatter,
                "missing private-use prefix at byte {position}: expected 'x'"
            ),
            Self::UnexpectedSubtag { position } => {
                write!(
                    formatter,
                    "unexpected subtag at byte {position}: not allowed here by the supported syntax"
                )
            }
        }
    }
}

impl core::error::Error for ParseError {}

/// An error returned by component-based construction, such as
/// [`NormalTag::from_parts`](crate::NormalTag::from_parts).
///
/// Unlike [`ParseError`], these errors describe component values or sequence
/// indices, not byte offsets into a source string.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BuildError {
    /// The extension singleton is not an ASCII letter other than `x`/`X`, or a digit.
    InvalidSingleton {
        /// The original, unnormalized character.
        singleton: char,
    },
    /// The extension subtag sequence is empty.
    EmptyExtension,
    /// An extension subtag contains fewer than two characters.
    InvalidExtensionSubtag {
        /// The zero-based index in the supplied subtag sequence.
        index: usize,
    },
    /// The private-use subtag sequence is empty.
    EmptyPrivateUse,
    /// The variant sequence contains a duplicate.
    DuplicateVariant,
    /// The extension sequence contains a duplicate singleton.
    DuplicateExtensionSingleton {
        /// The duplicate singleton character.
        singleton: char,
    },
}

impl Display for BuildError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidSingleton { singleton } => write!(
                formatter,
                "cannot build extension: invalid singleton {singleton:?}"
            ),
            Self::EmptyExtension => {
                formatter.write_str("cannot build extension: empty subtag sequence")
            }
            Self::InvalidExtensionSubtag { index } => write!(
                formatter,
                "cannot build extension: subtag at index {index} must contain at least 2 characters"
            ),
            Self::EmptyPrivateUse => {
                formatter.write_str("cannot build private-use tag: empty subtag sequence")
            }
            Self::DuplicateVariant => {
                formatter.write_str("cannot build tag: duplicate variant subtag")
            }
            Self::DuplicateExtensionSingleton { singleton } => {
                write!(
                    formatter,
                    "cannot build tag: duplicate extension singleton '{singleton}'"
                )
            }
        }
    }
}

impl core::error::Error for BuildError {}
