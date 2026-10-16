//! Region subtag (`region`) view and syntax validation.
//!
//! According to RFC 5646 §2.1:
//! ```text
//! region = 2ALPHA / 3DIGIT
//! ```
//! A region subtag must be 2 ASCII letters (ISO 3166-1 alpha-2 code, e.g., `"CN"`, `"US"`)
//! or exactly 3 ASCII digits (UN M.49 region code, e.g., `"419"`, `"001"`).

use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::Deref;

use super::error::ParseError;

/// Checks whether the given byte slice conforms to region subtag syntax rules.
#[must_use]
pub const fn is_valid_bytes(bytes: &[u8]) -> bool {
    let len = bytes.len();
    if len == 2 {
        bytes[0].is_ascii_alphabetic() && bytes[1].is_ascii_alphabetic()
    } else if len == 3 {
        bytes[0].is_ascii_digit() && bytes[1].is_ascii_digit() && bytes[2].is_ascii_digit()
    } else {
        false
    }
}

/// Checks whether the given string slice conforms to region subtag syntax rules.
#[must_use]
#[inline]
pub const fn is_valid(subtag: &str) -> bool {
    is_valid_bytes(subtag.as_bytes())
}

/// Validates whether the subtag conforms to region syntax rules, returning a [`ParseError`] on failure.
pub const fn validate(subtag: &str) -> Result<(), ParseError> {
    if subtag.is_empty() {
        return Err(ParseError::Empty);
    }
    if is_valid(subtag) {
        Ok(())
    } else {
        Err(ParseError::InvalidRegion)
    }
}

/// Read-only borrowed view for a region subtag.
///
/// Wraps a well-formed region subtag slice (2 letters or 3 digits).
#[derive(Debug, Clone, Copy, Eq)]
pub struct RegionView<'a> {
    subtag: &'a str,
}

impl<'a> RegionView<'a> {
    /// Validates and parses a string slice into a region subtag view.
    ///
    /// Callable in `const` contexts.
    pub const fn parse(s: &'a str) -> Result<Self, ParseError> {
        let bytes = s.as_bytes();
        if bytes.is_empty() {
            return Err(ParseError::Empty);
        }
        if is_valid_bytes(bytes) {
            Ok(Self { subtag: s })
        } else {
            Err(ParseError::InvalidRegion)
        }
    }

    /// Internal unchecked constructor for subtag slices known to be valid.
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

    /// Returns the subtag length (2 or 3).
    #[must_use]
    #[inline]
    pub const fn len(&self) -> usize {
        self.subtag.len()
    }

    /// Returns whether the subtag is empty (always returns `false`).
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        false
    }
}

impl Deref for RegionView<'_> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.subtag
    }
}

impl AsRef<str> for RegionView<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.subtag
    }
}

impl fmt::Display for RegionView<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.subtag)
    }
}

impl PartialEq for RegionView<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.subtag.eq_ignore_ascii_case(other.subtag)
    }
}

impl PartialOrd for RegionView<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RegionView<'_> {
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

impl Hash for RegionView<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for b in self.subtag.as_bytes() {
            state.write_u8(b.to_ascii_lowercase());
        }
        state.write_u8(0xFF);
    }
}

impl<'a> TryFrom<&'a str> for RegionView<'a> {
    type Error = ParseError;

    #[inline]
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}
