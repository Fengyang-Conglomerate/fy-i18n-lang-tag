//! Language subtag (`language`) view and syntax validation.
//!
//! According to RFC 5646 §2.1:
//! ```text
//! language = 2*3ALPHA / 4ALPHA / 5*8ALPHA
//! ```
//! A primary language subtag must consist of 2 to 8 ASCII letters.

use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::Deref;

use super::error::ParseError;

/// Checks whether a given byte slice matches primary language subtag syntax (2-8 ASCII letters, covering RFC 5646 4ALPHA).
#[must_use]
pub const fn is_valid_bytes(bytes: &[u8]) -> bool {
    let len = bytes.len();
    if len < 2 || len > 8 {
        return false;
    }
    let mut i = 0usize;
    while i < len {
        if !bytes[i].is_ascii_alphabetic() {
            return false;
        }
        i += 1;
    }
    true
}

/// Checks whether a given string slice matches primary language subtag syntax.
#[must_use]
#[inline]
pub const fn is_valid(subtag: &str) -> bool {
    is_valid_bytes(subtag.as_bytes())
}

/// Validates whether a subtag matches primary language syntax, returning [`ParseError`] on failure.
pub const fn validate(subtag: &str) -> Result<(), ParseError> {
    if subtag.is_empty() {
        return Err(ParseError::Empty);
    }
    if is_valid(subtag) {
        Ok(())
    } else {
        Err(ParseError::InvalidLanguage)
    }
}

/// A read-only borrowed view of a primary language subtag.
///
/// Wraps a validated primary language subtag slice (2 to 8 ASCII letters).
/// Guarantees validity at the type-system level so constructing or deriving tags from it requires no re-validation.
#[derive(Debug, Clone, Copy, Eq)]
pub struct LanguageView<'a> {
    subtag: &'a str,
}

impl<'a> LanguageView<'a> {
    /// Validates and parses a string slice into a primary language subtag view.
    ///
    /// Callable in `const` compile-time contexts.
    pub const fn parse(s: &'a str) -> Result<Self, ParseError> {
        let bytes = s.as_bytes();
        if bytes.is_empty() {
            return Err(ParseError::Empty);
        }
        if is_valid_bytes(bytes) {
            Ok(Self { subtag: s })
        } else {
            Err(ParseError::InvalidLanguage)
        }
    }

    /// Internal trusted constructor: creates a view with zero overhead when the slice is known to be valid.
    #[must_use]
    #[inline]
    pub(crate) const fn from_str_unchecked(subtag: &'a str) -> Self {
        Self { subtag }
    }

    /// Returns the underlying string slice.
    #[must_use]
    #[inline]
    pub const fn as_str(&self) -> &'a str {
        self.subtag
    }

    /// Returns the underlying byte slice.
    #[must_use]
    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.subtag.as_bytes()
    }

    /// Returns the byte length of the subtag.
    #[must_use]
    #[inline]
    pub const fn len(&self) -> usize {
        self.subtag.len()
    }

    /// Returns `true` if the subtag is empty.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.subtag.is_empty()
    }
}

impl Deref for LanguageView<'_> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.subtag
    }
}

impl AsRef<str> for LanguageView<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.subtag
    }
}

impl fmt::Display for LanguageView<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.subtag)
    }
}

impl PartialEq for LanguageView<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.subtag.eq_ignore_ascii_case(other.subtag)
    }
}

impl PartialOrd for LanguageView<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LanguageView<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let a = self.subtag.as_bytes();
        let b = other.subtag.as_bytes();
        let min_len = if a.len() < b.len() { a.len() } else { b.len() };
        let mut i = 0;
        while i < min_len {
            let ca = a[i].to_ascii_lowercase();
            let cb = b[i].to_ascii_lowercase();
            if ca != cb {
                return ca.cmp(&cb);
            }
            i += 1;
        }
        a.len().cmp(&b.len())
    }
}

impl Hash for LanguageView<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for b in self.subtag.as_bytes() {
            state.write_u8(b.to_ascii_lowercase());
        }
        state.write_u8(0xFF);
    }
}

impl<'a> TryFrom<&'a str> for LanguageView<'a> {
    type Error = ParseError;

    #[inline]
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}
