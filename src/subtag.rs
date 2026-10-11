//! Inline `Copy` subtags, normalized at construction.
//!
//! Atomic subtags use fixed-size inline storage without heap allocation and always
//! preserve valid, normalized values.

use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt::{Display, Formatter, Write as _};
use core::str::FromStr;

use crate::error::{BuildError, ParseError};

/// A language subtag: 2-3 or 5-8 ASCII letters; reserved four-letter forms are rejected.
/// Normalized to lowercase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Language {
    bytes: [u8; 8],
    len: u8,
}

/// A script subtag: exactly four ASCII letters, normalized to title case.
///
/// Stored inline in four bytes without an additional length field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Script {
    bytes: [u8; 4],
}

/// A region subtag: two ASCII letters or three ASCII digits, normalized to uppercase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    bytes: [u8; 3],
    len: u8,
}

/// A variant subtag: 5-8 ASCII alphanumerics, or four starting with a digit.
/// Normalized to lowercase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Variant {
    bytes: [u8; 8],
    len: u8,
}

/// A generic subtag: 1-8 ASCII alphanumerics.
///
/// Used in extensions and private-use sequences, normalized to lowercase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Subtag {
    bytes: [u8; 8],
    len: u8,
}

impl Language {
    /// Parses and normalizes a language subtag, including in const contexts.
    pub const fn parse(input: &str) -> Result<Self, ParseError> {
        if input.is_empty() {
            return Err(ParseError::Empty);
        }
        match Self::parse_bytes(input.as_bytes()) {
            Some(lang) => Ok(lang),
            None => Err(ParseError::InvalidLanguage { position: 0 }),
        }
    }

    pub(crate) const fn parse_bytes(part: &[u8]) -> Option<Self> {
        let len = part.len();
        if !((len >= 2 && len <= 3) || (len >= 5 && len <= 8)) {
            return None;
        }
        let mut bytes = [0u8; 8];
        let mut i = 0;
        while i < len {
            let b = part[i];
            if !b.is_ascii_alphabetic() {
                return None;
            }
            bytes[i] = b.to_ascii_lowercase();
            i += 1;
        }
        Some(Self {
            bytes,
            len: len as u8,
        })
    }

    /// Returns the normalized ASCII bytes.
    pub const fn as_bytes(&self) -> &[u8] {
        let len = self.len as usize;
        let (prefix, _) = self.bytes.split_at(len);
        prefix
    }

    /// Returns the normalized text.
    pub const fn as_str(&self) -> &str {
        match core::str::from_utf8(self.as_bytes()) {
            Ok(s) => s,
            Err(_) => panic!("language subtags contain only ASCII letters"),
        }
    }

    /// Returns the subtag length; subtags are nonempty by construction.
    #[allow(clippy::len_without_is_empty)]
    pub const fn len(&self) -> usize {
        self.len as usize
    }
}

impl Script {
    /// Parses and normalizes a script subtag, including in const contexts.
    pub const fn parse(input: &str) -> Result<Self, ParseError> {
        if input.is_empty() {
            return Err(ParseError::Empty);
        }
        match Self::parse_bytes(input.as_bytes()) {
            Some(script) => Ok(script),
            None => Err(ParseError::InvalidScript { position: 0 }),
        }
    }

    pub(crate) const fn parse_bytes(part: &[u8]) -> Option<Self> {
        if part.len() != 4 {
            return None;
        }
        let mut bytes = [0u8; 4];
        let mut i = 0;
        while i < 4 {
            let b = part[i];
            if !b.is_ascii_alphabetic() {
                return None;
            }
            bytes[i] = if i == 0 {
                b.to_ascii_uppercase()
            } else {
                b.to_ascii_lowercase()
            };
            i += 1;
        }
        Some(Self { bytes })
    }

    /// Returns the normalized ASCII bytes.
    pub const fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the normalized text.
    pub const fn as_str(&self) -> &str {
        match core::str::from_utf8(&self.bytes) {
            Ok(s) => s,
            Err(_) => panic!("script subtags contain only ASCII letters"),
        }
    }

    /// Returns the fixed length of four; scripts are nonempty by construction.
    #[allow(clippy::len_without_is_empty)]
    pub const fn len(&self) -> usize {
        4
    }
}

impl Region {
    /// Parses and normalizes a region subtag, including in const contexts.
    pub const fn parse(input: &str) -> Result<Self, ParseError> {
        if input.is_empty() {
            return Err(ParseError::Empty);
        }
        match Self::parse_bytes(input.as_bytes()) {
            Some(region) => Ok(region),
            None => Err(ParseError::InvalidRegion { position: 0 }),
        }
    }

    pub(crate) const fn parse_bytes(part: &[u8]) -> Option<Self> {
        let len = part.len();
        let mut bytes = [0u8; 3];
        if len == 2 {
            let b0 = part[0];
            let b1 = part[1];
            if !b0.is_ascii_alphabetic() || !b1.is_ascii_alphabetic() {
                return None;
            }
            bytes[0] = b0.to_ascii_uppercase();
            bytes[1] = b1.to_ascii_uppercase();
        } else if len == 3 {
            let b0 = part[0];
            let b1 = part[1];
            let b2 = part[2];
            if !b0.is_ascii_digit() || !b1.is_ascii_digit() || !b2.is_ascii_digit() {
                return None;
            }
            bytes[0] = b0;
            bytes[1] = b1;
            bytes[2] = b2;
        } else {
            return None;
        }
        Some(Self {
            bytes,
            len: len as u8,
        })
    }

    /// Returns the normalized ASCII bytes.
    pub const fn as_bytes(&self) -> &[u8] {
        let len = self.len as usize;
        let (prefix, _) = self.bytes.split_at(len);
        prefix
    }

    /// Returns the normalized text.
    pub const fn as_str(&self) -> &str {
        match core::str::from_utf8(self.as_bytes()) {
            Ok(s) => s,
            Err(_) => panic!("region subtags contain only ASCII letters or digits"),
        }
    }

    /// Returns the length (two or three); regions are nonempty by construction.
    #[allow(clippy::len_without_is_empty)]
    pub const fn len(&self) -> usize {
        self.len as usize
    }
}

impl Variant {
    /// Parses and normalizes a variant subtag, including in const contexts.
    pub const fn parse(input: &str) -> Result<Self, ParseError> {
        if input.is_empty() {
            return Err(ParseError::Empty);
        }
        match Self::parse_bytes(input.as_bytes()) {
            Some(variant) => Ok(variant),
            None => Err(ParseError::InvalidVariant { position: 0 }),
        }
    }

    pub(crate) const fn parse_bytes(part: &[u8]) -> Option<Self> {
        let len = part.len();
        let short_numeric = len == 4 && part[0].is_ascii_digit();
        if !((len >= 5 && len <= 8) || short_numeric) {
            return None;
        }
        let mut bytes = [0u8; 8];
        let mut i = 0;
        while i < len {
            let b = part[i];
            if !b.is_ascii_alphanumeric() {
                return None;
            }
            bytes[i] = b.to_ascii_lowercase();
            i += 1;
        }
        Some(Self {
            bytes,
            len: len as u8,
        })
    }

    /// Returns the normalized ASCII bytes.
    pub const fn as_bytes(&self) -> &[u8] {
        let len = self.len as usize;
        let (prefix, _) = self.bytes.split_at(len);
        prefix
    }

    /// Returns the normalized text.
    pub const fn as_str(&self) -> &str {
        match core::str::from_utf8(self.as_bytes()) {
            Ok(s) => s,
            Err(_) => panic!("variant subtags contain only ASCII alphanumerics"),
        }
    }

    /// Returns the subtag length; subtags are nonempty by construction.
    #[allow(clippy::len_without_is_empty)]
    pub const fn len(&self) -> usize {
        self.len as usize
    }
}

impl Subtag {
    /// Parses and normalizes a generic subtag, including in const contexts.
    pub const fn parse(input: &str) -> Result<Self, ParseError> {
        if input.is_empty() {
            return Err(ParseError::Empty);
        }
        let bytes = input.as_bytes();
        let len = bytes.len();
        if len > 8 {
            return Err(ParseError::SubtagTooLong {
                position: 0,
                length: len,
            });
        }
        let mut i = 0;
        while i < len {
            if !bytes[i].is_ascii_alphanumeric() {
                return Err(ParseError::InvalidCharacter { position: i });
            }
            i += 1;
        }
        match Self::parse_bytes(bytes) {
            Some(subtag) => Ok(subtag),
            None => unreachable!(),
        }
    }

    pub(crate) const fn parse_bytes(part: &[u8]) -> Option<Self> {
        let len = part.len();
        if len < 1 || len > 8 {
            return None;
        }
        let mut bytes = [0u8; 8];
        let mut i = 0;
        while i < len {
            let b = part[i];
            if !b.is_ascii_alphanumeric() {
                return None;
            }
            bytes[i] = b.to_ascii_lowercase();
            i += 1;
        }
        Some(Self {
            bytes,
            len: len as u8,
        })
    }

    /// Returns the normalized ASCII bytes.
    pub const fn as_bytes(&self) -> &[u8] {
        let len = self.len as usize;
        let (prefix, _) = self.bytes.split_at(len);
        prefix
    }

    /// Returns the normalized text.
    pub const fn as_str(&self) -> &str {
        match core::str::from_utf8(self.as_bytes()) {
            Ok(s) => s,
            Err(_) => panic!("generic subtags contain only ASCII alphanumerics"),
        }
    }

    /// Returns the subtag length; subtags are nonempty by construction.
    #[allow(clippy::len_without_is_empty)]
    pub const fn len(&self) -> usize {
        self.len as usize
    }
}

// Implement standard traits consistently for all atomic subtag types.
macro_rules! impl_subtag_traits {
    ($ty:ident) => {
        impl Display for $ty {
            fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl FromStr for $ty {
            type Err = ParseError;

            fn from_str(input: &str) -> Result<Self, Self::Err> {
                Self::parse(input)
            }
        }

        impl TryFrom<&str> for $ty {
            type Error = ParseError;

            fn try_from(input: &str) -> Result<Self, Self::Error> {
                Self::parse(input)
            }
        }

        impl AsRef<str> for $ty {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl core::hash::Hash for $ty {
            fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
                self.as_str().hash(state);
            }
        }

        impl PartialOrd for $ty {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $ty {
            fn cmp(&self, other: &Self) -> Ordering {
                self.as_str().cmp(other.as_str())
            }
        }
    };
}

impl_subtag_traits!(Language);
impl_subtag_traits!(Script);
impl_subtag_traits!(Region);
impl_subtag_traits!(Variant);
impl_subtag_traits!(Subtag);

/// An extension: a singleton followed by subtags of 2-8 characters each.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Extension {
    singleton: u8,
    subtags: Vec<Subtag>,
}

impl Extension {
    /// Constructs an extension.
    ///
    /// The singleton must match `[0-9a-wy-z]`, case-insensitively, and is lowercased;
    /// `x` is reserved for private use. The sequence must be nonempty, and each
    /// subtag must contain 2-8 characters. Subtag order is preserved.
    ///
    /// Validates the singleton first, returning [`BuildError::InvalidSingleton`]
    /// with the original character if invalid. Then returns
    /// [`BuildError::EmptyExtension`] for an empty sequence, or
    /// [`BuildError::InvalidExtensionSubtag`] with the first short subtag's index.
    pub fn new(singleton: char, subtags: Vec<Subtag>) -> Result<Self, BuildError> {
        let ascii_byte =
            u8::try_from(singleton).map_err(|_| BuildError::InvalidSingleton { singleton })?;
        let lower = ascii_byte.to_ascii_lowercase();
        if !lower.is_ascii_alphanumeric() || lower == b'x' {
            return Err(BuildError::InvalidSingleton { singleton });
        }
        if subtags.is_empty() {
            return Err(BuildError::EmptyExtension);
        }
        if let Some(index) = subtags.iter().position(|s| s.len() < 2) {
            return Err(BuildError::InvalidExtensionSubtag { index });
        }
        Ok(Self {
            singleton: lower,
            subtags,
        })
    }

    pub(crate) fn new_unchecked(singleton: u8, subtags: Vec<Subtag>) -> Self {
        Self { singleton, subtags }
    }

    /// Returns the singleton in `[0-9a-wy-z]`; `x` is reserved for private use.
    pub const fn singleton(&self) -> char {
        self.singleton as char
    }

    /// Returns the singleton's ASCII byte.
    pub const fn singleton_byte(&self) -> u8 {
        self.singleton
    }

    /// Returns lowercase extension subtags in their original relative order.
    pub fn subtags(&self) -> &[Subtag] {
        &self.subtags
    }
}

impl Display for Extension {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        formatter.write_char(self.singleton())?;
        for subtag in &self.subtags {
            write!(formatter, "-{subtag}")?;
        }
        Ok(())
    }
}

impl Extension {
    /// Parses an extension, such as `u-ca-buddhist`.
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        crate::tag::check_lexical_structure(input)?;

        let mut parts = input.split('-');
        let first = parts.next().unwrap();
        if first.len() != 1 {
            return Err(ParseError::InvalidExtension { position: 0 });
        }
        let singleton_byte = first.as_bytes()[0].to_ascii_lowercase();
        if !singleton_byte.is_ascii_alphanumeric() || singleton_byte == b'x' {
            return Err(ParseError::InvalidExtension { position: 0 });
        }

        let mut subtags = Vec::new();
        let mut pos = 2;
        for part in parts {
            if part.len() < 2 {
                return Err(ParseError::UnexpectedSubtag { position: pos });
            }
            let subtag = Subtag::parse_bytes(part.as_bytes())
                .ok_or(ParseError::UnexpectedSubtag { position: pos })?;
            subtags.push(subtag);
            pos += part.len() + 1;
        }

        if subtags.is_empty() {
            return Err(ParseError::InvalidExtension { position: 0 });
        }

        Ok(Self {
            singleton: singleton_byte,
            subtags,
        })
    }
}

impl FromStr for Extension {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::parse(input)
    }
}

impl TryFrom<&str> for Extension {
    type Error = ParseError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        Self::parse(input)
    }
}

impl PartialOrd for Extension {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Extension {
    fn cmp(&self, other: &Self) -> Ordering {
        self.singleton
            .cmp(&other.singleton)
            .then_with(|| self.subtags.cmp(&other.subtags))
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::{Extension, Language, Region, Script, Subtag, Variant};
    use core::fmt;
    use serde::de::{self, Visitor};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    macro_rules! impl_subtag_serde {
        ($ty:ident) => {
            impl Serialize for $ty {
                fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                    serializer.serialize_str(self.as_str())
                }
            }

            impl<'de> Deserialize<'de> for $ty {
                fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                    struct SubtagVisitor;

                    impl<'de> Visitor<'de> for SubtagVisitor {
                        type Value = $ty;

                        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                            formatter.write_str("a BCP 47 subtag string")
                        }

                        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                            $ty::parse(v).map_err(E::custom)
                        }
                    }

                    deserializer.deserialize_str(SubtagVisitor)
                }
            }
        };
    }

    impl_subtag_serde!(Language);
    impl_subtag_serde!(Script);
    impl_subtag_serde!(Region);
    impl_subtag_serde!(Variant);
    impl_subtag_serde!(Subtag);

    impl Serialize for Extension {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.collect_str(self)
        }
    }

    impl<'de> Deserialize<'de> for Extension {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct ExtensionVisitor;

            impl<'de> Visitor<'de> for ExtensionVisitor {
                type Value = Extension;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("a BCP 47 extension string")
                }

                fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                    Extension::parse(v).map_err(E::custom)
                }
            }

            deserializer.deserialize_str(ExtensionVisitor)
        }
    }
}
