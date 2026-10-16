//! Transformed content extension (`t` extension) syntax validation.
//!
//! According to RFC 6497 and Unicode UTS #35 (LDML):
//! Implements §2.2 production structure and §2.2(c) field separator deduplication,
//! without deep §2.5(b) business date subtag validation.
//! ```text
//! t-ext     = "t" (("-" lang *("-" field)) / 1*("-" field))
//! lang      = language ["-" script] ["-" region] *("-" variant)
//! field     = fsep 1*("-" 3*8alphanum)
//! fsep      = ALPHA DIGIT
//! alphanum  = ALPHA / DIGIT
//! ```
//!
//! Core rules:
//! 1. Singleton is `'t'` or `'T'`;
//! 2. Must contain at least one subtag;
//! 3. Supports two structural modes:
//!    - Mode A (with source language): starts with language subtag sequence `lang`, followed by zero or more `field`s;
//!    - Mode B (without source language): composed directly of one or more `field`s;
//! 4. Field separator `fsep` must be exactly 1 ASCII letter followed by 1 ASCII digit (e.g., `m0`, `t0`);
//! 5. Each `fsep` must be followed by **at least one** 3 to 8 character alphanumeric field value;
//! 6. **Duplicate field separators prohibited**: within a single `t` extension, duplicate `fsep`s are forbidden (case-insensitive, e.g., `m0` conflicts with `M0`).

use super::error::ParseError;
use super::variant;

/// Checks whether a single byte is a `t` extension singleton (`'t'` or `'T'`).
#[must_use]
#[inline]
pub const fn is_singleton(b: u8) -> bool {
    b == b't' || b == b'T'
}

/// Checks whether the given byte slice is a field separator `fsep` (exactly 1 letter + 1 digit).
#[must_use]
pub const fn is_fsep_bytes(bytes: &[u8]) -> bool {
    bytes.len() == 2 && bytes[0].is_ascii_alphabetic() && bytes[1].is_ascii_digit()
}

/// Checks whether the given string slice is a field separator `fsep`.
#[must_use]
#[inline]
pub const fn is_fsep(subtag: &str) -> bool {
    is_fsep_bytes(subtag.as_bytes())
}

/// Checks whether the given byte slice is a valid field value (3 to 8 ASCII alphanumerics).
#[must_use]
pub const fn is_field_value_bytes(bytes: &[u8]) -> bool {
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

/// Checks whether the given string slice is a valid field value.
#[must_use]
#[inline]
pub const fn is_field_value(subtag: &str) -> bool {
    is_field_value_bytes(subtag.as_bytes())
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

/// Checks whether the slice consists entirely of ASCII alphabetic characters.
const fn is_all_alphabetic(bytes: &[u8]) -> bool {
    let mut i = 0usize;
    while i < bytes.len() {
        if !bytes[i].is_ascii_alphabetic() {
            return false;
        }
        i += 1;
    }
    true
}

/// Checks whether the slice is a valid region: 2 letters or 3 digits.
const fn is_valid_region(bytes: &[u8]) -> bool {
    let len = bytes.len();
    if len == 2 {
        bytes[0].is_ascii_alphabetic() && bytes[1].is_ascii_alphabetic()
    } else if len == 3 {
        bytes[0].is_ascii_digit() && bytes[1].is_ascii_digit() && bytes[2].is_ascii_digit()
    } else {
        false
    }
}

/// Validates `t` extension internal subtags sequence (without `t-` prefix).
///
/// Strictly conforms to RFC 6497 and performs duplicate field separator checks.
///
/// # Errors
///
/// Returns a [`ParseError`] if syntax is invalid or duplicate field separators occur.
pub const fn validate_subtags_bytes(bytes: &[u8]) -> Result<(), ParseError> {
    if bytes.is_empty() {
        return Err(ParseError::EmptySubtag);
    }
    if bytes[bytes.len() - 1] == b'-' {
        return Err(ParseError::TrailingSeparator);
    }

    let mut pos = 0usize;
    let first_end = find_next_sep(bytes, pos);
    let first_len = first_end - pos;
    if first_len == 0 {
        return Err(ParseError::EmptySubtag);
    }
    let first_subtag = slice_range(bytes, pos, first_end);

    // Distinguish between source language (lang) or direct field sequence
    if is_fsep_bytes(first_subtag) {
        // Mode B (no source language): proceed directly to field sequence validation
        validate_fields(bytes, pos)
    } else {
        // Mode A (with source language): validate primary language (RFC 6497 allows 2-8 ASCII letters)
        if !(first_len >= 2 && first_len <= 8) || !is_all_alphabetic(first_subtag) {
            return Err(ParseError::InvalidTLanguage);
        }

        pos = first_end;
        if pos < bytes.len() {
            pos += 1; // Skip '-'
        }

        // Parse subsequent components of lang: script -> region -> variants*
        let mut phase = 0u8; // 0: script acceptable, 1: region acceptable, 2: only variant acceptable
        let mut variants_count = 0usize;
        let mut variants_start = 0usize;

        while pos < bytes.len() {
            let end = find_next_sep(bytes, pos);
            let len = end - pos;
            if len == 0 {
                return Err(ParseError::EmptySubtag);
            }
            let subtag = slice_range(bytes, pos, end);

            // If fsep is encountered, lang phase ends; transition to field sequence validation
            if is_fsep_bytes(subtag) {
                return validate_fields(bytes, pos);
            }

            if phase == 0 && len == 4 && is_all_alphabetic(subtag) {
                phase = 1;
            } else if phase <= 1 && is_valid_region(subtag) {
                phase = 2;
            } else if variant::is_valid_bytes(subtag) {
                phase = 2;
                if variants_count == 0 {
                    variants_start = pos;
                } else {
                    let packed = super::extensions::pack_variant(subtag);
                    let target_len = subtag.len();
                    let mut scan = variants_start;
                    while scan < pos {
                        let scan_end = find_next_sep(bytes, scan);
                        let prev_len = scan_end - scan;
                        if prev_len == target_len {
                            let prev = slice_range(bytes, scan, scan_end);
                            if super::extensions::pack_variant(prev) == packed {
                                return Err(ParseError::DuplicateVariant);
                            }
                        }
                        scan = if scan_end < bytes.len() {
                            scan_end + 1
                        } else {
                            scan_end
                        };
                    }
                }
                variants_count += 1;
            } else {
                return Err(ParseError::InvalidTLanguage);
            }

            pos = end;
            if pos < bytes.len() {
                pos += 1;
            }
        }

        Ok(())
    }
}

/// Parses and validates field sequence starting from `pos`.
const fn validate_fields(bytes: &[u8], mut pos: usize) -> Result<(), ParseError> {
    let mut current_field_values = 0usize;
    let mut in_field = false;

    while pos < bytes.len() {
        let end = find_next_sep(bytes, pos);
        let len = end - pos;
        if len == 0 {
            return Err(ParseError::EmptySubtag);
        }
        let subtag = slice_range(bytes, pos, end);

        if is_fsep_bytes(subtag) {
            // Preceding field must contain at least one value subtag
            if in_field && current_field_values == 0 {
                return Err(ParseError::InvalidTField);
            }

            // Check if current fsep duplicates an earlier fsep
            if let Err(err) = check_duplicate_fsep(bytes, pos, subtag) {
                return Err(err);
            }

            in_field = true;
            current_field_values = 0;
        } else {
            if is_field_value_bytes(subtag) {
                current_field_values += 1;
            } else if len > 8 {
                return Err(ParseError::SubtagTooLong);
            } else {
                return Err(ParseError::InvalidTField);
            }
        }

        pos = end;
        if pos < bytes.len() {
            pos += 1;
        }
    }

    // The last field must have at least one value
    if in_field && current_field_values == 0 {
        return Err(ParseError::InvalidTField);
    }

    Ok(())
}

/// Backtracks to check if current fsep duplicates an earlier fsep.
const fn check_duplicate_fsep(
    bytes: &[u8],
    current_fsep_pos: usize,
    current_fsep: &[u8],
) -> Result<(), ParseError> {
    let mut scan_pos = 0usize;

    while scan_pos < current_fsep_pos {
        let end = find_next_sep(bytes, scan_pos);
        let sub = slice_range(bytes, scan_pos, end);

        if is_fsep_bytes(sub) && bytes_eq_ignore_ascii_case(sub, current_fsep) {
            return Err(ParseError::DuplicateTField);
        }

        scan_pos = end;
        if scan_pos < current_fsep_pos {
            scan_pos += 1;
        }
    }

    Ok(())
}

/// Validates `t` extension internal subtags sequence string (without `t-` prefix).
#[inline]
pub const fn validate_subtags(subtags: &str) -> Result<(), ParseError> {
    validate_subtags_bytes(subtags.as_bytes())
}

/// Validates full `t` extension string (optionally containing leading `t-` or `T-`).
pub const fn validate(extension: &str) -> Result<(), ParseError> {
    let bytes = extension.as_bytes();
    if bytes.len() >= 2 && is_singleton(bytes[0]) && bytes[1] == b'-' {
        let (_, subtags) = bytes.split_at(2);
        validate_subtags_bytes(subtags)
    } else {
        validate_subtags_bytes(bytes)
    }
}

/// Read-only borrowed view of a transformed content extension (RFC 6497).
///
/// Wraps a well-formed `t` extension subtag sequence (without `t-` prefix), providing efficient source language and field lookups.
#[derive(Debug, Clone, Copy)]
pub struct TExtensionView<'a> {
    subtags: &'a str,
}

impl<'a> TExtensionView<'a> {
    /// Validates and parses a `t` extension string slice (optionally containing leading `t-` or `T-`).
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

    /// Constructs view directly from internal subtags sequence (without `t-` prefix).
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

    /// Returns the raw string slice of subtags (without `t-` prefix).
    #[must_use]
    #[inline]
    pub const fn as_str(&self) -> &'a str {
        self.subtags
    }

    /// Returns the source language tag slice if present.
    ///
    /// Mode A (e.g., `"t-it-m0-ungegn"`) returns `Some("it")`;
    /// Mode B (e.g., `"t-m0-ungegn"`, no source language, composed directly of fields) returns `None`.
    #[must_use]
    pub fn source_language(&self) -> Option<&'a str> {
        let bytes = self.subtags.as_bytes();
        let end = find_next_sep(bytes, 0);
        let sub = slice_range(bytes, 0, end);
        if is_fsep_bytes(sub) {
            return None;
        }

        // Source language might contain script/region/variants; scan until fsep or end
        let mut lang_end = end;
        let mut pos = end;
        if pos < bytes.len() {
            pos += 1;
        }
        while pos < bytes.len() {
            let next_end = find_next_sep(bytes, pos);
            let next_sub = slice_range(bytes, pos, next_end);
            if is_fsep_bytes(next_sub) {
                break;
            }
            lang_end = next_end;
            pos = next_end;
            if pos < bytes.len() {
                pos += 1;
            }
        }
        Some(&self.subtags[0..lang_end])
    }

    /// Checks whether the specified field separator (`fsep`) is present (case-insensitive).
    #[must_use]
    pub fn has_field(&self, fsep: &str) -> bool {
        for (f, _) in self.fields() {
            if f.eq_ignore_ascii_case(fsep) {
                return true;
            }
        }
        false
    }

    /// Retrieves the first field value for the given field separator (case-insensitive).
    #[must_use]
    pub fn field_value(&self, fsep: &str) -> Option<&'a str> {
        for (f, val) in self.fields() {
            if f.eq_ignore_ascii_case(fsep) {
                return Some(val);
            }
        }
        None
    }

    /// Returns an iterator over all field value subtags for the given field separator (case-insensitive).
    #[must_use]
    pub fn field_values(&self, target_fsep: &str) -> TFieldValues<'a> {
        let mut pos = 0usize;
        let bytes = self.subtags.as_bytes();
        let mut found_pos = None;
        while pos < bytes.len() {
            let end = find_next_sep(bytes, pos);
            let sub = slice_range(bytes, pos, end);
            if is_fsep_bytes(sub) {
                let fsep = &self.subtags[pos..end];
                let next_pos = if end < bytes.len() { end + 1 } else { end };
                if fsep.eq_ignore_ascii_case(target_fsep) {
                    found_pos = Some(next_pos);
                    break;
                }
            }
            pos = if end < bytes.len() { end + 1 } else { end };
        }
        TFieldValues {
            subtags: self.subtags,
            pos: found_pos.unwrap_or(bytes.len()),
        }
    }

    /// Returns an iterator over fields, yielding `(fsep: &'a str, first_val: &'a str)`.
    #[must_use]
    #[inline]
    pub fn fields(&self) -> TFields<'a> {
        TFields {
            subtags: self.subtags,
            pos: 0,
        }
    }
}

impl core::fmt::Display for TExtensionView<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::fmt::Write;
        f.write_str("t-")?;
        for b in self.subtags.bytes() {
            f.write_char(b.to_ascii_lowercase() as char)?;
        }
        Ok(())
    }
}

/// Iterator over `t` extension fields.
#[derive(Debug, Clone)]
pub struct TFields<'a> {
    subtags: &'a str,
    pos: usize,
}

impl<'a> Iterator for TFields<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let bytes = self.subtags.as_bytes();

        while self.pos < bytes.len() {
            let end = find_next_sep(bytes, self.pos);
            let sub = slice_range(bytes, self.pos, end);

            if is_fsep_bytes(sub) {
                let fsep = &self.subtags[self.pos..end];
                let next_pos = if end < bytes.len() { end + 1 } else { end };
                self.pos = next_pos;

                // Extract following field value
                if self.pos < bytes.len() {
                    let val_end = find_next_sep(bytes, self.pos);
                    let val = &self.subtags[self.pos..val_end];
                    self.pos = if val_end < bytes.len() {
                        val_end + 1
                    } else {
                        val_end
                    };
                    return Some((fsep, val));
                }
            }

            self.pos = if end < bytes.len() { end + 1 } else { end };
        }

        None
    }
}

impl PartialEq for TExtensionView<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.subtags.eq_ignore_ascii_case(other.subtags)
    }
}

impl Eq for TExtensionView<'_> {}

impl core::hash::Hash for TExtensionView<'_> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        for b in self.subtags.bytes() {
            state.write_u8(b.to_ascii_lowercase());
        }
        state.write_u8(0xFF);
    }
}

impl PartialOrd for TExtensionView<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TExtensionView<'_> {
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

impl<'a> TryFrom<&'a str> for TExtensionView<'a> {
    type Error = ParseError;

    #[inline]
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

/// Iterator over all field value subtags for a specific `fsep` in a `t` extension.
#[derive(Debug, Clone)]
pub struct TFieldValues<'a> {
    subtags: &'a str,
    pos: usize,
}

impl<'a> Iterator for TFieldValues<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let bytes = self.subtags.as_bytes();
        if self.pos < bytes.len() {
            let end = find_next_sep(bytes, self.pos);
            let sub = slice_range(bytes, self.pos, end);
            if is_fsep_bytes(sub) {
                // Next fsep encountered: current field values finished
                self.pos = bytes.len();
                return None;
            }
            if is_field_value_bytes(sub) {
                let val = &self.subtags[self.pos..end];
                self.pos = if end < bytes.len() { end + 1 } else { end };
                return Some(val);
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
        assert!(!bytes_eq_ignore_ascii_case(b"m0", b"m00"));
        let tv = TExtensionView::from_subtags_unchecked("m0");
        assert_eq!(tv.fields().count(), 0);
    }
}
