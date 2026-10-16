//! Script subtag view and syntax validation.
//!
//! According to RFC 5646 §2.1:
//! ```text
//! script = 4ALPHA
//! ```
//! A script subtag must consist of exactly 4 ASCII letters (e.g., `"Hans"`, `"Latn"`).

use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::Deref;

use super::error::ParseError;

/// Checks whether the given byte slice conforms to script subtag syntax rules.
#[must_use]
pub const fn is_valid_bytes(bytes: &[u8]) -> bool {
    if bytes.len() != 4 {
        return false;
    }
    let mut i = 0usize;
    while i < 4 {
        if !bytes[i].is_ascii_alphabetic() {
            return false;
        }
        i += 1;
    }
    true
}

/// Checks whether the given string slice conforms to script subtag syntax rules.
#[must_use]
#[inline]
pub const fn is_valid(subtag: &str) -> bool {
    is_valid_bytes(subtag.as_bytes())
}

/// Validates whether the subtag conforms to script syntax rules, returning a [`ParseError`] on failure.
pub const fn validate(subtag: &str) -> Result<(), ParseError> {
    if subtag.is_empty() {
        return Err(ParseError::Empty);
    }
    if is_valid(subtag) {
        Ok(())
    } else {
        Err(ParseError::InvalidScript)
    }
}

/// Read-only borrowed view for a script subtag.
///
/// Wraps a well-formed script subtag slice (4 ASCII letters).
#[derive(Debug, Clone, Copy, Eq)]
pub struct ScriptView<'a> {
    subtag: &'a str,
}

impl<'a> ScriptView<'a> {
    /// Validates and parses a string slice into a script subtag view.
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
            Err(ParseError::InvalidScript)
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

    /// Returns the length in bytes (fixed at 4).
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

impl Deref for ScriptView<'_> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.subtag
    }
}

impl AsRef<str> for ScriptView<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.subtag
    }
}

impl fmt::Display for ScriptView<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.subtag)
    }
}

impl PartialEq for ScriptView<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.subtag.eq_ignore_ascii_case(other.subtag)
    }
}

impl PartialOrd for ScriptView<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScriptView<'_> {
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

impl Hash for ScriptView<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for b in self.subtag.as_bytes() {
            state.write_u8(b.to_ascii_lowercase());
        }
        state.write_u8(0xFF);
    }
}

impl<'a> TryFrom<&'a str> for ScriptView<'a> {
    type Error = ParseError;

    #[inline]
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}
