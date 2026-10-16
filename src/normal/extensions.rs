//! Extension subtag (`extensions`) syntax validation.
//!
//! According to RFC 5646 §2.1:
//! ```text
//! extension = singleton 1*("-" (2*8alphanum))
//! singleton = DIGIT / %x41-57 / %x59-5A / %x61-77 / %x79-7A
//! ```
//! A singleton is any single ASCII letter or digit except `x` and `X`;
//! each extension section must be followed by at least one 2-to-8 character ASCII alphanumeric subtag.

use super::error::ParseError;

/// Checks whether a single byte is a valid extension singleton (single ASCII alphanumeric character except `x` and `X`).
#[must_use]
#[inline]
pub const fn is_singleton(b: u8) -> bool {
    b.is_ascii_alphanumeric() && b != b'x' && b != b'X'
}

/// Checks whether the given byte slice is a valid extension subtag (2 to 8 ASCII alphanumeric characters).
#[must_use]
pub const fn is_extension_subtag_bytes(bytes: &[u8]) -> bool {
    let len = bytes.len();
    if len < 2 || len > 8 {
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

/// Checks whether the string slice is a valid extension subtag.
#[must_use]
#[inline]
pub const fn is_extension_subtag(subtag: &str) -> bool {
    is_extension_subtag_bytes(subtag.as_bytes())
}

/// Validates the validity of a singleton character.
pub const fn validate_singleton(b: u8) -> Result<(), ParseError> {
    if is_singleton(b) {
        Ok(())
    } else {
        Err(ParseError::InvalidExtension)
    }
}

/// Validates the validity of a single extension internal subtag.
pub const fn validate_subtag(subtag: &str) -> Result<(), ParseError> {
    if subtag.is_empty() {
        return Err(ParseError::EmptySubtag);
    }
    let bytes = subtag.as_bytes();
    if bytes.len() < 2 {
        return Err(ParseError::InvalidExtension);
    }
    if bytes.len() > 8 {
        return Err(ParseError::SubtagTooLong);
    }
    let mut i = 0usize;
    while i < bytes.len() {
        if !bytes[i].is_ascii_alphanumeric() {
            return Err(ParseError::InvalidCharacter);
        }
        i += 1;
    }
    Ok(())
}

/// Read-only view of a generic single-letter extension (extensions other than `u`, `t`, `x`, e.g., `a-foo-bar`).
#[derive(Debug, Clone, Copy)]
pub struct OtherExtensionView<'a> {
    singleton: u8,
    subtags: &'a str,
}

impl PartialEq for OtherExtensionView<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.singleton.eq_ignore_ascii_case(&other.singleton)
            && self.subtags.eq_ignore_ascii_case(other.subtags)
    }
}

impl Eq for OtherExtensionView<'_> {}

impl<'a> OtherExtensionView<'a> {
    /// Validates and parses a single-letter extension string slice like `"a-foo-bar"`.
    pub const fn parse(s: &'a str) -> Result<Self, ParseError> {
        let bytes = s.as_bytes();
        if bytes.len() < 3 {
            return Err(ParseError::InvalidExtension);
        }
        let b = bytes[0];
        if !is_singleton(b)
            || super::u::is_singleton(b)
            || super::t::is_singleton(b)
            || bytes[1] != b'-'
        {
            return Err(ParseError::InvalidExtension);
        }
        let (_, rest) = bytes.split_at(2);
        if let Err(err) = validate_single_extension(b, rest) {
            return Err(err);
        }
        let (_, subtags) = s.split_at(2);
        Ok(Self {
            singleton: b,
            subtags,
        })
    }

    /// Internal unchecked constructor.
    #[must_use]
    #[inline]
    pub(crate) const fn new(singleton: u8, subtags: &'a str) -> Self {
        Self { singleton, subtags }
    }

    /// Returns the singleton character (e.g., `'a'`).
    #[must_use]
    #[inline]
    pub const fn singleton(&self) -> char {
        self.singleton as char
    }

    /// Returns the singleton ASCII byte (e.g., `b'a'`).
    #[must_use]
    #[inline]
    pub const fn singleton_byte(&self) -> u8 {
        self.singleton
    }

    /// Returns the pure extension subtag sequence string slice (**without** the singleton prefix and hyphen, e.g., `"foo-bar"`).
    #[must_use]
    #[inline]
    pub const fn subtags(&self) -> &'a str {
        self.subtags
    }

    /// Returns the pure extension subtag sequence string slice (equivalent to [`subtags`](Self::subtags)).
    #[must_use]
    #[inline]
    pub const fn as_str(&self) -> &'a str {
        self.subtags
    }
}

impl core::fmt::Display for OtherExtensionView<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::fmt::Write;
        f.write_char((self.singleton as char).to_ascii_lowercase())?;
        f.write_char('-')?;
        for b in self.subtags.as_bytes() {
            f.write_char(b.to_ascii_lowercase() as char)?;
        }
        Ok(())
    }
}

impl core::hash::Hash for OtherExtensionView<'_> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write_u8(self.singleton.to_ascii_lowercase());
        for b in self.subtags.bytes() {
            state.write_u8(b.to_ascii_lowercase());
        }
        state.write_u8(0xFF);
    }
}

impl PartialOrd for OtherExtensionView<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OtherExtensionView<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let ca = self.singleton.to_ascii_lowercase();
        let cb = other.singleton.to_ascii_lowercase();
        if ca != cb {
            return ca.cmp(&cb);
        }
        let a = self.subtags.as_bytes();
        let b = other.subtags.as_bytes();
        let min_len = if a.len() < b.len() { a.len() } else { b.len() };
        let mut i = 0;
        while i < min_len {
            let ba = a[i].to_ascii_lowercase();
            let bb = b[i].to_ascii_lowercase();
            if ba != bb {
                return ba.cmp(&bb);
            }
            i += 1;
        }
        a.len().cmp(&b.len())
    }
}

impl<'a> TryFrom<&'a str> for OtherExtensionView<'a> {
    type Error = ParseError;

    #[inline]
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

/// Generic extensions iterator, obtainable only from validated normal tags.
///
/// ```compile_fail
/// use fy_i18n_lang_tag::OtherExtensionsIter;
/// let _ = OtherExtensionsIter::new("a-invalid");
/// ```
#[derive(Debug, Clone)]
pub struct OtherExtensionsIter<'a> {
    raw: &'a str,
    pos: usize,
}

impl<'a> OtherExtensionsIter<'a> {
    /// Constructs a generic extensions iterator.
    #[must_use]
    #[inline]
    pub(crate) const fn new(raw: &'a str) -> Self {
        Self { raw, pos: 0 }
    }

    /// Finds the extension view matching the specified singleton (case-insensitive comparison).
    #[must_use]
    pub fn find_singleton(mut self, singleton: char) -> Option<OtherExtensionView<'a>> {
        if !singleton.is_ascii() {
            return None;
        }
        let b = singleton as u8;
        self.find(|ext| ext.singleton.eq_ignore_ascii_case(&b))
    }
}

impl<'a> Iterator for OtherExtensionsIter<'a> {
    type Item = OtherExtensionView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let bytes = self.raw.as_bytes();

        while self.pos < bytes.len() {
            let end = find_sep(bytes, self.pos);
            let len = end - self.pos;

            if len == 1 {
                let b = bytes[self.pos];
                // Private use extension is at the very end; terminate on x/X
                if b == b'x' || b == b'X' {
                    self.pos = bytes.len();
                    return None;
                }

                let val_start = if end < bytes.len() { end + 1 } else { end };

                // Find the end position of this singleton extension block (until next single-letter singleton or end)
                let mut cur = val_start;
                let mut val_end = bytes.len();
                while cur < bytes.len() {
                    let sub_end = find_sep(bytes, cur);
                    if sub_end - cur == 1 {
                        val_end = if cur > 0 { cur - 1 } else { 0 };
                        break;
                    }
                    cur = if sub_end < bytes.len() {
                        sub_end + 1
                    } else {
                        sub_end
                    };
                }

                if val_end < val_start {
                    val_end = val_start;
                }

                let next_pos = if val_end < bytes.len() {
                    val_end + 1
                } else {
                    bytes.len()
                };
                self.pos = next_pos;

                // If u or t extension, skip and continue searching for other extensions
                if b == b'u' || b == b'U' || b == b't' || b == b'T' {
                    continue;
                }

                if val_end == val_start {
                    continue;
                }

                let subtags = &self.raw[val_start..val_end];
                return Some(OtherExtensionView::new(b, subtags));
            }

            self.pos = if end < bytes.len() { end + 1 } else { end };
        }

        None
    }
}

const fn find_sep(bytes: &[u8], start: usize) -> usize {
    let mut i = start;
    while i < bytes.len() {
        if bytes[i] == b'-' {
            return i;
        }
        i += 1;
    }
    bytes.len()
}

/// Byte offset ranges of parsed components in an extensions sequence.
#[derive(Debug, Clone, Copy)]
pub struct ParsedExtRanges {
    /// Variant subtag sequence range (byte offsets).
    pub variants: Option<(usize, usize)>,
    /// u-extension sequence range.
    pub u: Option<(usize, usize)>,
    /// t-extension sequence range.
    pub t: Option<(usize, usize)>,
    /// Whether other single-letter extensions (besides u/t/x) are present.
    pub has_other: bool,
    /// Private-use subtag sequence range.
    pub private_use: Option<(usize, usize)>,
}

/// Validates and structurally partitions the extension sequence according to RFC 5646.
///
/// Strictly conforms to the RFC 5646 syntax production:
/// `*("-" variant) *("-" extension) ["-" privateuse]`
pub const fn parse_and_validate_extensions_sequence(
    bytes: &[u8],
) -> Result<ParsedExtRanges, ParseError> {
    if bytes.is_empty() {
        return Err(ParseError::EmptySubtag);
    }
    if bytes[bytes.len() - 1] == b'-' {
        return Err(ParseError::TrailingSeparator);
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Phase {
        Variant,
        Extension,
        PrivateUse,
    }

    let mut phase = Phase::Variant;

    let mut pos = 0usize;
    let mut current_singleton = 0u8;
    let mut current_ext_start = 0usize;

    let mut variants_count = 0usize;
    let mut variants_end = 0usize;

    let mut u_range = None;
    let mut t_range = None;
    let mut has_other = false;
    let mut pu_range = None;

    let mut variant_table = [0u64; 128];
    let mut table_entries = 0usize;

    while pos < bytes.len() {
        let sub_end = find_sep(bytes, pos);
        let sub_len = sub_end - pos;

        if sub_len == 0 {
            return Err(ParseError::EmptySubtag);
        }

        let subtag = slice_range(bytes, pos, sub_end);

        match phase {
            Phase::Variant => {
                // Variant phase: check if variant, singleton, or private use
                if sub_len == 1 {
                    let b = subtag[0];
                    if crate::private_use::is_singleton(b) {
                        phase = Phase::PrivateUse; // Enter private use phase
                        if sub_end >= bytes.len() {
                            return Err(ParseError::InvalidPrivateUse);
                        }
                        pu_range = Some((sub_end + 1, bytes.len()));
                    } else if is_singleton(b) {
                        phase = Phase::Extension; // Enter extension phase
                        current_singleton = b;
                        current_ext_start = sub_end + 1;
                    } else {
                        return Err(ParseError::InvalidCharacter);
                    }
                } else if super::variant::is_valid_bytes(subtag) {
                    // Valid variant subtag: duplicate check using fixed-size open addressing table and multi-round XOR-folding hash
                    let packed = pack_variant(subtag);
                    if table_entries < 128 {
                        // Double multiplication and right-shift XOR folding to disperse bits into lower 7 bits
                        let mut h =
                            (packed ^ 0xcbf2_9ce4_8422_2325).wrapping_mul(0x517c_c1b7_2722_0a95);
                        h ^= h >> 32;
                        h = h.wrapping_mul(0x9e37_79b9_7f4a_7c15);
                        h ^= h >> 28;
                        let mut slot = (h as usize) % 128;

                        let mut probes = 0usize;
                        let mut inserted = false;
                        while probes < 128 {
                            if variant_table[slot] == 0 {
                                variant_table[slot] = packed;
                                table_entries += 1;
                                inserted = true;
                                break;
                            } else if variant_table[slot] == packed {
                                return Err(ParseError::DuplicateVariant);
                            }
                            slot = (slot + 1) % 128;
                            probes += 1;
                        }
                        if !inserted {
                            if let Err(err) = check_duplicate_variant(bytes, pos, subtag, packed) {
                                return Err(err);
                            }
                        }
                    } else {
                        // Variant capacity reached 128: linear backtrack bypass
                        if let Err(err) = check_duplicate_variant(bytes, pos, subtag, packed) {
                            return Err(err);
                        }
                    }

                    variants_count += 1;
                    variants_end = sub_end;
                } else {
                    return Err(ParseError::InvalidVariant);
                }
            }
            Phase::Extension => {
                // Extension phase
                if sub_len == 1 {
                    // Encountered new singleton: validate previous extension segment
                    if pos == 0 || current_ext_start >= pos {
                        return Err(ParseError::InvalidExtension);
                    }
                    let ext_slice = slice_range(bytes, current_ext_start, pos - 1);
                    if let Err(err) = validate_single_extension(current_singleton, ext_slice) {
                        return Err(err);
                    }

                    if super::u::is_singleton(current_singleton) {
                        u_range = Some((current_ext_start, pos - 1));
                    } else if super::t::is_singleton(current_singleton) {
                        t_range = Some((current_ext_start, pos - 1));
                    } else {
                        has_other = true;
                    }

                    let b = subtag[0];
                    if crate::private_use::is_singleton(b) {
                        phase = Phase::PrivateUse; // Enter private use phase
                        if sub_end >= bytes.len() {
                            return Err(ParseError::InvalidPrivateUse);
                        }
                        pu_range = Some((sub_end + 1, bytes.len()));
                    } else if is_singleton(b) {
                        // Check for duplicate singleton
                        if check_duplicate_singleton(bytes, pos, b) {
                            return Err(ParseError::DuplicateSingleton);
                        }
                        current_singleton = b;
                        current_ext_start = sub_end + 1;
                    } else {
                        return Err(ParseError::InvalidCharacter);
                    }
                }
            }
            Phase::PrivateUse => {
                // Private-use phase: must continue to end
                if !crate::private_use::is_subtag_bytes(subtag) {
                    if sub_len > 8 {
                        return Err(ParseError::SubtagTooLong);
                    }
                    return Err(ParseError::InvalidPrivateUse);
                }
            }
        }

        pos = sub_end;
        if pos < bytes.len() {
            pos += 1; // Skip '-'
        }
    }

    // Validate the final extension segment if loop finishes in extension phase
    if matches!(phase, Phase::Extension) {
        if current_ext_start >= bytes.len() {
            return Err(ParseError::InvalidExtension);
        }
        let ext_slice = slice_range(bytes, current_ext_start, bytes.len());
        if let Err(err) = validate_single_extension(current_singleton, ext_slice) {
            return Err(err);
        }
        if super::u::is_singleton(current_singleton) {
            u_range = Some((current_ext_start, bytes.len()));
        } else if super::t::is_singleton(current_singleton) {
            t_range = Some((current_ext_start, bytes.len()));
        } else {
            has_other = true;
        }
    }

    let variants = if variants_count > 0 {
        Some((0, variants_end))
    } else {
        None
    };

    Ok(ParsedExtRanges {
        variants,
        u: u_range,
        t: t_range,
        has_other,
        private_use: pu_range,
    })
}

/// Dispatches and validates subtag sequence inside a single extension block according to singleton.
const fn validate_single_extension(singleton: u8, ext_subtags: &[u8]) -> Result<(), ParseError> {
    if ext_subtags.is_empty() {
        return Err(ParseError::EmptySubtag);
    }
    if ext_subtags[ext_subtags.len() - 1] == b'-' {
        return Err(ParseError::TrailingSeparator);
    }
    if super::u::is_singleton(singleton) {
        super::u::validate_subtags_bytes(ext_subtags)
    } else if super::t::is_singleton(singleton) {
        super::t::validate_subtags_bytes(ext_subtags)
    } else {
        // Generic extension rule: each subtag has 2-8 alphanumeric characters
        let mut p = 0usize;
        while p < ext_subtags.len() {
            let end = find_sep(ext_subtags, p);
            let len = end - p;
            let sub = slice_range(ext_subtags, p, end);
            if !is_extension_subtag_bytes(sub) {
                if len > 8 {
                    return Err(ParseError::SubtagTooLong);
                }
                return Err(ParseError::InvalidExtension);
            }
            p = end;
            if p < ext_subtags.len() {
                p += 1;
            }
        }
        Ok(())
    }
}

/// Checks whether the given singleton duplicates an earlier singleton in the sequence.
const fn check_duplicate_singleton(bytes: &[u8], current_pos: usize, singleton: u8) -> bool {
    let mut scan_pos = 0usize;
    while scan_pos < current_pos {
        let end = find_sep(bytes, scan_pos);
        if end - scan_pos == 1 {
            let b = bytes[scan_pos];
            if b.eq_ignore_ascii_case(&singleton) {
                return true;
            }
        }
        scan_pos = end;
        if scan_pos < current_pos {
            scan_pos += 1;
        }
    }
    false
}

/// Normalizes and packs a 4-8 byte variant subtag into a u64 integer (lowercase ASCII, zero-padded).
#[inline]
pub(crate) const fn pack_variant(subtag: &[u8]) -> u64 {
    let mut buf = [0u8; 8];
    let limit = if subtag.len() < 8 { subtag.len() } else { 8 };
    let mut i = 0usize;
    while i < limit {
        buf[i] = subtag[i].to_ascii_lowercase();
        i += 1;
    }
    u64::from_ne_bytes(buf)
}

/// Backtracks to check if current variant duplicates an earlier variant.
const fn check_duplicate_variant(
    bytes: &[u8],
    current_variant_pos: usize,
    current_variant: &[u8],
    current_packed: u64,
) -> Result<(), ParseError> {
    let mut scan_pos = 0usize;
    let target_len = current_variant.len();
    while scan_pos < current_variant_pos {
        let end = find_sep(bytes, scan_pos);
        let sub_len = end - scan_pos;
        if sub_len == target_len {
            let sub = slice_range(bytes, scan_pos, end);
            if pack_variant(sub) == current_packed {
                return Err(ParseError::DuplicateVariant);
            }
        }
        scan_pos = end;
        if scan_pos < current_variant_pos {
            scan_pos += 1;
        }
    }
    Ok(())
}

const fn slice_range(bytes: &[u8], start: usize, end: usize) -> &[u8] {
    let (_, tail) = bytes.split_at(start);
    let (target, _) = tail.split_at(end - start);
    target
}
