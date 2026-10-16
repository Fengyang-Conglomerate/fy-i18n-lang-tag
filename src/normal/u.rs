//! Unicode locale extension (`u` extension) syntax validation.
//!
//! According to RFC 6067 and Unicode UTS #35 (LDML):
//! ```text
//! u-ext     = "u" *("-" attribute) *("-" keyword)
//! attribute = 3*8alphanum
//! keyword   = key *("-" type)
//! key       = 2alphanum
//! type      = 3*8alphanum
//! ```
//!
//! Core rules:
//! 1. Singleton is `'u'` or `'U'`;
//! 2. Must contain at least one subtag;
//! 3. All `attribute`s (3-8 ASCII alphanumerics) must precede all `keyword`s;
//! 4. Each `keyword` starts with a 2-character alphanumeric `key`, followed by zero or more 3-8 character `type`s;
//! 5. **Duplicate keys prohibited**: within a single `u` extension, duplicate keys are forbidden (case-insensitive, e.g., `ca` conflicts with `CA`);
//! 6. All subtags must be ASCII alphanumeric.

use super::error::ParseError;

/// Checks whether a single byte is a `u` extension singleton (`'u'` or `'U'`).
#[must_use]
#[inline]
pub const fn is_singleton(b: u8) -> bool {
    b == b'u' || b == b'U'
}

/// Checks whether the given byte slice is a valid `key` (exactly 2 ASCII alphanumerics).
#[must_use]
pub const fn is_key_bytes(bytes: &[u8]) -> bool {
    bytes.len() == 2 && bytes[0].is_ascii_alphanumeric() && bytes[1].is_ascii_alphanumeric()
}

/// Checks whether the given string slice is a valid `key`.
#[must_use]
#[inline]
pub const fn is_key(subtag: &str) -> bool {
    is_key_bytes(subtag.as_bytes())
}

/// Checks whether the given byte slice is a valid `attribute` or `type` (3 to 8 ASCII alphanumerics).
#[must_use]
pub const fn is_alphanum_3_8_bytes(bytes: &[u8]) -> bool {
    let len = bytes.len();
    if len < 3 || len > 8 {
        return false;
    }
    let mut i = 0usize;
    while i < len {
        if !bytes[i].is_ascii_alphanumeric() {
            return false;
        }
        i += 1;
    }
    true
}

/// Checks whether the given string slice is a valid `attribute` (3 to 8 ASCII alphanumerics).
#[must_use]
#[inline]
pub const fn is_attribute(subtag: &str) -> bool {
    is_alphanum_3_8_bytes(subtag.as_bytes())
}

/// Checks whether the given string slice is a valid `type` (3 to 8 ASCII alphanumerics).
#[must_use]
#[inline]
pub const fn is_type(subtag: &str) -> bool {
    is_alphanum_3_8_bytes(subtag.as_bytes())
}

/// Compares two byte slices for equality ignoring ASCII case.
const fn bytes_eq_ignore_ascii_case(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0usize;
    while i < a.len() {
        if !a[i].eq_ignore_ascii_case(&b[i]) {
            return false;
        }
        i += 1;
    }
    true
}

/// Finds the next `-` hyphen or end of slice.
const fn find_next_sep(bytes: &[u8], start: usize) -> usize {
    let mut i = start;
    while i < bytes.len() {
        if bytes[i] == b'-' {
            return i;
        }
        i += 1;
    }
    bytes.len()
}

/// Safely extracts a sub-range from a byte slice.
const fn slice_range(bytes: &[u8], start: usize, end: usize) -> &[u8] {
    let (_, tail) = bytes.split_at(start);
    let (target, _) = tail.split_at(end - start);
    target
}

/// Validates `u` extension internal subtags sequence (without `u-` prefix).
///
/// Strictly conforms to RFC 6067 and performs duplicate key checks.
///
/// # Errors
///
/// Returns a [`ParseError`] if syntax is invalid or duplicate keys occur.
pub const fn validate_subtags_bytes(bytes: &[u8]) -> Result<(), ParseError> {
    if bytes.is_empty() {
        return Err(ParseError::EmptySubtag);
    }
    if bytes[bytes.len() - 1] == b'-' {
        return Err(ParseError::TrailingSeparator);
    }

    // Phase indicator:
    // false = AttributePhase
    // true  = KeywordPhase
    let mut in_keyword_phase = false;
    let mut pos = 0usize;

    while pos < bytes.len() {
        let sub_end = find_next_sep(bytes, pos);
        let sub_len = sub_end - pos;

        if sub_len == 0 {
            return Err(ParseError::EmptySubtag);
        }

        let subtag = slice_range(bytes, pos, sub_end);

        // Character validity check
        let mut i = 0usize;
        while i < subtag.len() {
            if !subtag[i].is_ascii_alphanumeric() {
                return Err(ParseError::InvalidCharacter);
            }
            i += 1;
        }

        if !in_keyword_phase {
            // Attribute phase: accepts 3-8 char attribute or 2 char key
            if sub_len == 2 {
                // First key encountered, enter keyword phase
                in_keyword_phase = true;
            } else if sub_len >= 3 && sub_len <= 8 {
                // Valid attribute, stay in attribute phase
            } else if sub_len > 8 {
                return Err(ParseError::SubtagTooLong);
            } else {
                return Err(ParseError::InvalidUAttribute);
            }
        } else {
            // Keyword phase: allows 2 char key or 3-8 char type
            if sub_len == 2 {
                // New key: check for duplicates against previously encountered keys
                if let Err(err) = check_duplicate_key(bytes, pos, subtag) {
                    return Err(err);
                }
            } else if sub_len >= 3 && sub_len <= 8 {
                // Valid type belonging to previous key
            } else if sub_len > 8 {
                return Err(ParseError::SubtagTooLong);
            } else {
                return Err(ParseError::InvalidUType);
            }
        }

        pos = sub_end;
        if pos < bytes.len() {
            pos += 1; // Skip '-'
        }
    }

    Ok(())
}

/// Backtracks to check if current key duplicates an earlier key.
const fn check_duplicate_key(
    bytes: &[u8],
    current_key_pos: usize,
    current_key: &[u8],
) -> Result<(), ParseError> {
    let mut scan_pos = 0usize;

    while scan_pos < current_key_pos {
        let end = find_next_sep(bytes, scan_pos);
        let len = end - scan_pos;

        if len == 2 {
            let prev_key = slice_range(bytes, scan_pos, end);
            if bytes_eq_ignore_ascii_case(prev_key, current_key) {
                return Err(ParseError::DuplicateUKey);
            }
        }

        scan_pos = end;
        if scan_pos < current_key_pos {
            scan_pos += 1;
        }
    }

    Ok(())
}

/// Validates `u` extension internal subtags sequence string (without `u-` prefix).
#[inline]
pub const fn validate_subtags(subtags: &str) -> Result<(), ParseError> {
    validate_subtags_bytes(subtags.as_bytes())
}

/// Validates full `u` extension string (optionally containing leading `u-` or `U-`).
pub const fn validate(extension: &str) -> Result<(), ParseError> {
    let bytes = extension.as_bytes();
    if bytes.len() >= 2 && is_singleton(bytes[0]) && bytes[1] == b'-' {
        let (_, subtags) = bytes.split_at(2);
        validate_subtags_bytes(subtags)
    } else {
        validate_subtags_bytes(bytes)
    }
}

/// Read-only borrowed view of a Unicode locale extension (RFC 6067).
///
/// Wraps a well-formed `u` extension subtag sequence (without `u-` prefix), providing efficient attribute and keyword lookups.
#[derive(Debug, Clone, Copy)]
pub struct UExtensionView<'a> {
    subtags: &'a str,
}

impl<'a> UExtensionView<'a> {
    /// Validates and parses a `u` extension string slice (optionally containing leading `u-` or `U-`).
    pub const fn parse(s: &'a str) -> Result<Self, ParseError> {
        let bytes = s.as_bytes();
        if bytes.len() >= 2 && is_singleton(bytes[0]) && bytes[1] == b'-' {
            let (_, subtags_bytes) = bytes.split_at(2);
            if let Err(err) = validate_subtags_bytes(subtags_bytes) {
                return Err(err);
            }
            let (_, subtags_str) = s.split_at(2);
            Ok(Self {
                subtags: subtags_str,
            })
        } else {
            if let Err(err) = validate_subtags_bytes(bytes) {
                return Err(err);
            }
            Ok(Self { subtags: s })
        }
    }

    /// Constructs view directly from internal subtags sequence (without `u-` prefix).
    pub const fn from_subtags(subtags: &'a str) -> Result<Self, ParseError> {
        if let Err(err) = validate_subtags(subtags) {
            return Err(err);
        }
        Ok(Self { subtags })
    }

    /// Internal unchecked constructor.
    #[must_use]
    #[inline]
    pub(crate) const fn from_subtags_unchecked(subtags: &'a str) -> Self {
        Self { subtags }
    }

    /// Returns the raw string slice of subtags (without `u-` prefix).
    #[must_use]
    #[inline]
    pub const fn as_str(&self) -> &'a str {
        self.subtags
    }

    /// Checks whether the specified attribute is present (case-insensitive).
    #[must_use]
    pub fn has_attribute(&self, target_attr: &str) -> bool {
        for attr in self.attributes() {
            if attr.eq_ignore_ascii_case(target_attr) {
                return true;
            }
        }
        false
    }

    /// Checks whether the specified key is present (case-insensitive).
    #[must_use]
    pub fn has_key(&self, target_key: &str) -> bool {
        for (k, _) in self.keywords() {
            if k.eq_ignore_ascii_case(target_key) {
                return true;
            }
        }
        false
    }

    /// Retrieves the first type for a given key if present (case-insensitive).
    #[must_use]
    pub fn keyword_type(&self, target_key: &str) -> Option<&'a str> {
        for (k, t) in self.keywords() {
            if k.eq_ignore_ascii_case(target_key) {
                return t;
            }
        }
        None
    }

    /// Returns an iterator over all type subtags for a given key (case-insensitive).
    #[must_use]
    pub fn keyword_types(&self, target_key: &str) -> UKwTypes<'a> {
        let mut pos = 0usize;
        let bytes = self.subtags.as_bytes();
        let mut found_pos = None;
        while pos < bytes.len() {
            let key_end = find_next_sep(bytes, pos);
            let key_len = key_end - pos;
            if key_len == 2 {
                let key = &self.subtags[pos..key_end];
                let next_pos = if key_end < bytes.len() {
                    key_end + 1
                } else {
                    key_end
                };
                if key.eq_ignore_ascii_case(target_key) {
                    found_pos = Some(next_pos);
                    break;
                }
            }
            pos = if key_end < bytes.len() {
                key_end + 1
            } else {
                key_end
            };
        }
        UKwTypes {
            subtags: self.subtags,
            pos: found_pos.unwrap_or(bytes.len()),
        }
    }

    /// Returns an iterator over attributes.
    #[must_use]
    #[inline]
    pub fn attributes(&self) -> UAttributes<'a> {
        UAttributes {
            subtags: self.subtags,
            pos: 0,
        }
    }

    /// Returns an iterator over keywords.
    #[must_use]
    #[inline]
    pub fn keywords(&self) -> UKeywords<'a> {
        UKeywords {
            subtags: self.subtags,
            pos: 0,
        }
    }
}

impl core::fmt::Display for UExtensionView<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::fmt::Write;
        f.write_str("u-")?;
        for b in self.subtags.bytes() {
            f.write_char(b.to_ascii_lowercase() as char)?;
        }
        Ok(())
    }
}

/// Iterator over `u` extension attribute subtags.
#[derive(Debug, Clone)]
pub struct UAttributes<'a> {
    subtags: &'a str,
    pos: usize,
}

impl<'a> Iterator for UAttributes<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let bytes = self.subtags.as_bytes();
        if self.pos < bytes.len() {
            let end = find_next_sep(bytes, self.pos);
            let len = end - self.pos;
            if len == 2 {
                // First 2-character key encountered: attribute phase finished
                self.pos = bytes.len();
                return None;
            }
            let attr = &self.subtags[self.pos..end];
            self.pos = if end < bytes.len() { end + 1 } else { end };
            return Some(attr);
        }
        None
    }
}

/// Iterator over `u` extension keywords, yielding `(key, first_type)`.
#[derive(Debug, Clone)]
pub struct UKeywords<'a> {
    subtags: &'a str,
    pos: usize,
}

impl<'a> Iterator for UKeywords<'a> {
    type Item = (&'a str, Option<&'a str>);

    fn next(&mut self) -> Option<Self::Item> {
        let bytes = self.subtags.as_bytes();

        // 1. Find next key (2-character subtag)
        while self.pos < bytes.len() {
            let key_end = find_next_sep(bytes, self.pos);
            let key_len = key_end - self.pos;

            if key_len == 2 {
                let key = &self.subtags[self.pos..key_end];
                let next_pos = if key_end < bytes.len() {
                    key_end + 1
                } else {
                    key_end
                };
                self.pos = next_pos;

                // Check if followed by first type (3-8 characters)
                if self.pos < bytes.len() {
                    let type_end = find_next_sep(bytes, self.pos);
                    let type_len = type_end - self.pos;
                    if (3..=8).contains(&type_len) {
                        let type_str = &self.subtags[self.pos..type_end];
                        self.pos = if type_end < bytes.len() {
                            type_end + 1
                        } else {
                            type_end
                        };
                        return Some((key, Some(type_str)));
                    }
                }
                return Some((key, None));
            }

            self.pos = if key_end < bytes.len() {
                key_end + 1
            } else {
                key_end
            };
        }

        None
    }
}

impl PartialEq for UExtensionView<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.subtags.eq_ignore_ascii_case(other.subtags)
    }
}

impl Eq for UExtensionView<'_> {}

impl core::hash::Hash for UExtensionView<'_> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        for b in self.subtags.bytes() {
            state.write_u8(b.to_ascii_lowercase());
        }
        state.write_u8(0xFF);
    }
}

impl PartialOrd for UExtensionView<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UExtensionView<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let a = self.subtags.as_bytes();
        let b = other.subtags.as_bytes();
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

impl<'a> TryFrom<&'a str> for UExtensionView<'a> {
    type Error = ParseError;

    #[inline]
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

/// Iterator over all type subtags for a specific key in a `u` extension.
#[derive(Debug, Clone)]
pub struct UKwTypes<'a> {
    subtags: &'a str,
    pos: usize,
}

impl<'a> Iterator for UKwTypes<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let bytes = self.subtags.as_bytes();
        if self.pos < bytes.len() {
            let end = find_next_sep(bytes, self.pos);
            let len = end - self.pos;
            if len == 2 {
                // Next 2-character key encountered: types for current key finished
                self.pos = bytes.len();
                return None;
            }
            if (3..=8).contains(&len) {
                let t = &self.subtags[self.pos..end];
                self.pos = if end < bytes.len() { end + 1 } else { end };
                return Some(t);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_helpers() {
        assert!(!bytes_eq_ignore_ascii_case(b"ca", b"caa"));
    }
}
