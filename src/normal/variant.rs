//! Variant subtag (`variant`) syntax validation.
//!
//! According to RFC 5646 §2.1:
//! ```text
//! variant = 5*8alphanum / (DIGIT 3alphanum)
//! ```
//! A variant subtag must meet one of the following two formats:
//! 1. 5 to 8 ASCII letters or digits (e.g., `"fonipa"`, `"nedis"`);
//! 2. Exactly 4 characters, where the first character must be an ASCII digit (e.g., `"1996"`, `"1901"`).

use super::error::ParseError;

/// Checks whether the given byte slice conforms to variant subtag syntax rules.
#[must_use]
pub const fn is_valid_bytes(bytes: &[u8]) -> bool {
    let len = bytes.len();
    if len >= 5 && len <= 8 {
        let mut i = 0usize;
        while i < len {
            if !bytes[i].is_ascii_alphanumeric() {
                return false;
            }
            i += 1;
        }
        true
    } else if len == 4 {
        if !bytes[0].is_ascii_digit() {
            return false;
        }
        let mut i = 1usize;
        while i < 4 {
            if !bytes[i].is_ascii_alphanumeric() {
                return false;
            }
            i += 1;
        }
        true
    } else {
        false
    }
}

/// Checks whether the given string slice conforms to variant subtag syntax rules.
#[must_use]
#[inline]
pub const fn is_valid(subtag: &str) -> bool {
    is_valid_bytes(subtag.as_bytes())
}

/// Validates whether the subtag conforms to variant syntax rules, returning a [`ParseError`] on failure.
pub const fn validate(subtag: &str) -> Result<(), ParseError> {
    if subtag.is_empty() {
        return Err(ParseError::EmptySubtag);
    }
    if is_valid(subtag) {
        Ok(())
    } else {
        Err(ParseError::InvalidVariant)
    }
}
